use crate::module::Module;
use crate::patchbay::Patchbay;

/// Processor for [Module]s.
///
/// Keeps track of the dependencies between modules and runs the [`Module::process`] fn
/// for each module in the correct order.
///
/// For circular connections the order is undetermined and the previous sample might be read
pub struct Processor<const MODULES: usize, M> {
    pub modules: [M; MODULES],
    order: [usize; MODULES],
    order_set: bool,
}

impl<const MODULES: usize, M: Module> Processor<MODULES, M> {
    /// Instantiates a new processor given a set of modules.
    ///
    /// Modules are expected to implement the [`Module`] trait,
    pub fn new(modules: [M; MODULES]) -> Self {
        Processor {
            modules,
            order: [0; MODULES],
            order_set: false,
        }
    }

    /// Creates the order in which to run [`Module::process`] on modules based on their [`Patchbay`] connections.
    ///
    /// **Note:** to determine the order it executes the process function on the modules which might cause state changes
    pub fn order_modules<const POINTS: usize>(&mut self, patchbay: &mut Patchbay<POINTS>) {
        patchbay.set_marks();

        let mut index = 0;
        let mut processed = [false; MODULES];

        loop {
            let mut number_of_oks = 0;

            for (i, module) in self.modules.iter_mut().enumerate() {
                if processed[i] {
                    continue;
                }

                match module.process(patchbay) {
                    Ok(_) => {
                        // Mark as already processed
                        processed[i] = true;

                        // Put it in cache processing order
                        self.order[index] = i;
                        index += 1;

                        // Tell the loop something has changed, so keep going
                        number_of_oks += 1;
                    }
                    _ => (),
                }
            }

            if number_of_oks == 0 {
                break;
            }
        }

        // Add unprocessed to the cache order
        for (i, p) in processed.iter().enumerate() {
            if !p {
                self.order[index] = i;
                index += 1;
            }
        }

        self.order_set = true;
        patchbay.clear_marks();
    }

    /// Callback to process modules, usually called from a loop to process the entire buffer.
    ///
    /// ```
    /// for _ in 0..BUFFER_SIZE {
    ///   processor.process_modules(&mut patchbay);
    /// }
    /// ```
    ///
    /// Internally calls `order_modules` if no order has been determined yet,
    /// to avoid the initial performance hit you can call `order_modules` manually.
    pub fn process_modules<const POINTS: usize>(&mut self, patchbay: &mut Patchbay<POINTS>) {
        if !self.order_set {
            self.order_modules(patchbay);
        }

        for &index in self.order.iter() {
            let _ = self.modules[index].process(patchbay);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patchbay::{PatchError, PatchPoint, PatchPointValue, Patchbay};
    use crate::sample::Sample;
    use screech_macro::modularize;

    struct Fixed {
        value: Sample,
        output: PatchPoint,
    }

    impl Fixed {
        fn process<const POINTS: usize>(
            &mut self,
            patchbay: &mut Patchbay<POINTS>,
        ) -> Result<(), PatchError> {
            patchbay.set_sample(&mut self.output, self.value);
            Ok(())
        }
    }

    struct Divide {
        value: Sample,
        input: PatchPointValue,
        output: PatchPoint,
    }

    impl Divide {
        fn process<const POINTS: usize>(
            &mut self,
            patchbay: &mut Patchbay<POINTS>,
        ) -> Result<(), PatchError> {
            patchbay.set_sample(
                &mut self.output,
                patchbay.get_sample(self.input)? / self.value,
            );
            Ok(())
        }
    }

    struct Mix {
        inputs: [PatchPointValue; 2],
        output: PatchPoint,
    }

    impl Mix {
        fn process<const POINTS: usize>(
            &mut self,
            patchbay: &mut Patchbay<POINTS>,
        ) -> Result<(), PatchError> {
            let mut result = 0.0;

            for s in self.inputs {
                result += patchbay.get_sample(s)?;
            }

            patchbay.set_sample(&mut self.output, result);
            Ok(())
        }
    }

    #[modularize]
    enum Modules {
        Fixed(Fixed),
        Divide(Divide),
        Mix(Mix),
    }

    #[test]
    fn process_should_run_process_on_modules() {
        let mut patchbay: Patchbay<32> = Patchbay::new();
        let fixed_point = patchbay.get_point();
        let final_output = fixed_point.value();
        let mut processor: Processor<1, _> = Processor::new([Modules::Fixed(Fixed {
            value: 0.8,
            output: fixed_point,
        })]);

        processor.process_modules(&mut patchbay);
        assert_eq!(patchbay.get_sample(final_output), Ok(0.8));
    }

    #[test]
    fn process_should_run_modules_in_the_correct_order() {
        let mut patchbay: Patchbay<32> = Patchbay::new();
        let divide2_point = patchbay.get_point();
        let divide1_point = patchbay.get_point();
        let fixed_point = patchbay.get_point();
        let final_output = divide2_point.value();

        let divide2 = Divide {
            value: 2.0,
            input: divide1_point.value(),
            output: divide2_point,
        };
        let divide1 = Divide {
            value: 4.0,
            input: fixed_point.value(),
            output: divide1_point,
        };
        let fixed = Fixed {
            value: 0.8,
            output: fixed_point,
        };

        let mut processor: Processor<3, _> = Processor::new([
            Modules::Divide(divide1),
            Modules::Divide(divide2),
            Modules::Fixed(fixed),
        ]);

        processor.process_modules(&mut patchbay);

        let result = patchbay.get_sample(final_output);

        assert_eq!(result, Ok(0.1));
    }

    #[test]
    fn process_should_allow_circular_connections() {
        let mut patchbay: Patchbay<32> = Patchbay::new();
        let mix_point = patchbay.get_point();
        let divide_point = patchbay.get_point();
        let fixed_point = patchbay.get_point();
        let divide_value = divide_point.value();
        let final_output = mix_point.value();

        let divide = Divide {
            value: 2.0,
            input: mix_point.value(),
            output: divide_point,
        };
        let mix = Mix {
            inputs: [fixed_point.value(), divide_value],
            output: mix_point,
        };
        let fixed = Fixed {
            value: 0.8,
            output: fixed_point,
        };

        let mut processor: Processor<3, _> = Processor::new([
            Modules::Mix(mix),
            Modules::Fixed(fixed),
            Modules::Divide(divide),
        ]);

        processor.process_modules(&mut patchbay);
        let result = patchbay.get_sample(final_output);
        assert_eq!(result, Ok(0.8));

        processor.process_modules(&mut patchbay);
        let result = patchbay.get_sample(final_output);
        assert_eq!(result, Ok(1.2));
    }
}
