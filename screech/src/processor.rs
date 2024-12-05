use crate::{Module, Patchbay};

#[derive(PartialEq)]
enum Mode {
    A,
    B,
}

/// Processor for [Module]s.
///
/// Keeps track of the dependencies between modules and runs the [`Module::process`] fn
/// for each module in the correct order.
///
/// For circular connections the order is undetermined and the previous sample might be read
pub struct Processor<const SAMPLE_RATE: usize, const MODULES: usize, M: Module<SAMPLE_RATE>> {
    pub modules: [Option<M>; MODULES],
    pub module_ids: [Option<usize>; MODULES],
    pub order_set: bool,
    mode: Mode,
}

impl<const SAMPLE_RATE: usize, const MODULES: usize, M: Module<SAMPLE_RATE>>
    Processor<SAMPLE_RATE, MODULES, M>
{
    /// Instantiates a new processor given a set of modules.
    pub fn new(modules: [Option<M>; MODULES]) -> Self {
        let module_ids = core::array::from_fn(|i| modules[i].as_ref().map(|_| i));

        Processor {
            modules,
            module_ids,
            order_set: false,
            mode: Mode::A,
        }
    }

    /// Instantiates a new empty processor.
    pub fn empty() -> Self {
        Processor {
            modules: core::array::from_fn(|_| None),
            module_ids: [None; MODULES],
            order_set: false,
            mode: Mode::A,
        }
    }

    pub fn mode_a(&mut self) {
        self.mode = Mode::A;
    }

    pub fn mode_b(&mut self) {
        self.mode = Mode::B;
    }

    /// Take all modules from the processor leaving it empty.
    ///
    /// ```
    /// use screech::Processor;
    /// use screech::modules::Dummy;
    ///
    /// const SAMPLE_RATE: usize = 48000;
    /// const MODULES: usize = 4;
    ///
    /// let mut processor: Processor<SAMPLE_RATE, MODULES, Dummy> = Processor::new([None, None, None, None]);
    ///
    /// processor.insert_module(Dummy);
    /// processor.insert_module(Dummy);
    ///
    /// assert_eq!(processor.take_modules(), [Some(Dummy), Some(Dummy), None, None]);
    /// assert_eq!(processor.take_modules(), [None, None, None, None]);
    /// ```
    pub fn take_modules(&mut self) -> [Option<M>; MODULES] {
        let mut modules = core::array::from_fn(|_| None);

        for i in 0..MODULES {
            modules[i] = self.modules[i].take();
        }

        self.module_ids = [None; MODULES];

        modules
    }

    /// Get a reference to a module at a given index.
    ///
    /// ```
    /// use screech::{Patchbay, Processor};
    /// use screech::modules::Dummy;
    ///
    /// let mut processor: Processor<48_000, 256, Dummy> = Processor::new([Some(Dummy); 256]);
    /// assert!(processor.get_module(128) == Some(&Dummy));
    /// ```
    pub fn get_module(&self, index: usize) -> Option<&M> {
        self.module_ids[index].and_then(|i| self.modules[i].as_ref())
    }

    /// Get a mutable reference to a module at a given index.
    ///
    /// ```
    /// use screech::{Patchbay, Processor};
    /// use screech::modules::Dummy;
    ///
    /// let mut processor: Processor<48_000, 256, Dummy> = Processor::new([Some(Dummy); 256]);
    /// assert!(processor.get_module_mut(64) == Some(&mut Dummy));
    /// ```
    pub fn get_module_mut(&mut self, index: usize) -> Option<&mut M> {
        self.module_ids[index].and_then(move |i| self.modules[i].as_mut())
    }

    /// Insert a module
    ///
    /// ```
    /// use screech::{Module, Patchbay, Processor};
    /// use screech::modules::Oscillator;
    /// use screech_macro::modularize;
    ///
    /// const SAMPLE_RATE: usize = 48_000;
    /// const MODULES: usize = 256;
    /// const PATCHES: usize = 256;
    ///
    /// const EMPTY: Option<Oscillator> = None;
    /// let mut patchbay: Patchbay<PATCHES> = Patchbay::new();
    /// let mut processor: Processor<SAMPLE_RATE, MODULES, Oscillator> = Processor::new([EMPTY; MODULES]);
    /// let mut osc = Oscillator::new(patchbay.point().unwrap());
    ///
    /// osc.set_frequency(440.0);
    ///
    /// let id = processor.insert_module(osc).unwrap();
    ///
    /// match processor.get_module(id) {
    ///     Some(o) => assert_eq!(o.get_frequency(), 440.0),
    ///     _ => panic!("expected `Oscillator` module type"),
    /// }
    /// ```
    pub fn insert_module(&mut self, module: M) -> Option<usize> {
        // @TODO: convert to Result type?
        for i in 0..MODULES {
            if self.module_ids[i].is_none() {
                for m in 0..MODULES {
                    if self.modules[m].is_none() {
                        self.modules[m] = Some(module);
                        self.module_ids[i] = Some(m);

                        // Bust the cache
                        self.order_set = false;

                        return Some(i);
                    }
                }

                // Mismatch between available `modules` and `module_ids`
                return None;
            }
        }

        None
    }

    /// Replace a module at a given index.
    ///
    /// ```
    /// use screech::{Module, Patchbay, Processor};
    /// use screech::modules::{Vca, Oscillator};
    /// use screech_macro::modularize;
    ///
    /// const SAMPLE_RATE: usize = 48_000;
    /// const MODULES: usize = 256;
    /// const PATCHES: usize = 256;
    ///
    /// #[modularize]
    /// enum Modules {
    ///    Oscillator(Oscillator),
    ///    Vca(Vca),
    /// }
    ///
    /// const EMPTY: Option<Modules> = None;
    ///
    /// let mut patchbay: Patchbay<PATCHES> = Patchbay::new();
    /// let mut processor: Processor<SAMPLE_RATE, MODULES, Modules> = Processor::new([EMPTY; MODULES]);
    /// let mut osc = Oscillator::new(patchbay.point().unwrap());
    ///
    /// osc.set_frequency(440.0);
    ///
    /// processor.replace_module(Modules::Oscillator(osc), 192);
    ///
    /// match processor.get_module(192) {
    ///     Some(Modules::Oscillator(o)) => assert_eq!(o.get_frequency(), 440.0),
    ///     _ => panic!("expected `Oscillator` module type"),
    /// }
    /// ```
    pub fn replace_module(&mut self, module: M, index: usize) {
        // Bust the cache
        self.order_set = false;

        match self.module_ids[index] {
            Some(i) => self.modules[i] = Some(module),
            None => {
                for i in 0..MODULES {
                    if self.modules[i].is_none() {
                        self.modules[i] = Some(module);
                        self.module_ids[index] = Some(i);
                        break;
                    }
                }
            }
        }
    }
    /// Callback to process modules, usually called from a loop to process the entire buffer.
    ///
    /// ```
    /// use screech::{Patchbay, Processor};
    /// use screech::modules::Oscillator;
    ///
    /// const BUFFER_SIZE: usize = 256;
    /// const SAMPLE_RATE: usize = 48_000;
    ///
    /// let mut patchbay: Patchbay<8> = Patchbay::new();
    /// let osc = Oscillator::new(patchbay.point().unwrap());
    /// let mut processor: Processor<SAMPLE_RATE, 1, Oscillator> = Processor::new([Some(osc)]);
    ///
    /// for _ in 0..BUFFER_SIZE {
    ///   processor.process_modules(&mut patchbay);
    /// }
    /// ```
    ///
    /// Internally calls `order_modules` if no order has been determined yet,
    /// to avoid the initial performance hit you can call `order_modules` manually.
    pub fn process_modules<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        if !self.order_set {
            self.order_and_process_modules(patchbay);
        } else {
            for i in 0..MODULES {
                match self.modules[i].as_mut() {
                    Some(m) => m.process(patchbay),
                    None => break,
                }
            }
        }
    }

    fn order_and_process_modules<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        patchbay.clear_marks();

        let mut new_index = 0;
        let mut new_order: [Option<usize>; MODULES] = [None; MODULES];
        let mut processed = [false; MODULES];

        loop {
            let mut updated_modules = 0;

            for index in 0..MODULES {
                match (
                    processed[index],
                    self.module_ids[index].and_then(|id| self.modules[id].as_mut()),
                ) {
                    // If it has not been processed already and contains a module
                    (false, Some(m)) => {
                        if m.is_ready(patchbay) {
                            // Process the module so the outputs are set.
                            m.process(patchbay);
                            // Mark as already processed
                            processed[index] = true;
                            // Put it in cache processing order
                            new_order[index] = Some(new_index);
                            new_index += 1;
                            // Tell the loop something has changed, so keep going
                            updated_modules += 1;
                        }
                    }
                    _ => (),
                }
            }

            if updated_modules == 0 {
                break;
            }
        }

        // Process and sort the remaining non ready modules
        for index in 0..MODULES {
            match (
                processed[index],
                self.module_ids[index].and_then(|id| self.modules[id].as_mut()),
            ) {
                (false, Some(m)) => {
                    // Process the module so the outputs are set.
                    m.process(patchbay);
                    // Put it in cache processing order
                    new_order[index] = Some(new_index);
                    new_index += 1;
                }
                _ => (),
            }
        }

        let mut modules_cache: [Option<M>; MODULES] = core::array::from_fn(|_| None);

        // Reorder the modules
        for index in 0..MODULES {
            if let Some(old_id) = self.module_ids[index] {
                let new_id = new_order[index].unwrap_or(old_id);
                modules_cache[new_id] = self.modules[old_id].take();
                self.module_ids[index] = Some(new_id);
            }
        }

        // Swap the modules
        self.modules = modules_cache;

        self.order_set = true;
    }

    pub fn clear_cache(&mut self) {
        self.order_set = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::Dummy;
    use crate::{PatchPoint, Patchbay, Signal};
    use screech_macro::modularize;

    const SAMPLE_RATE: usize = 48_000;

    struct Constant {
        value: f32,
        output: PatchPoint,
    }

    impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Constant {
        fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
            patchbay.set(&mut self.output, self.value);
        }
    }

    struct Divide {
        value: f32,
        input: Signal,
        output: PatchPoint,
    }

    impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Divide {
        fn is_ready<const P: usize>(&self, patchbay: &Patchbay<P>) -> bool {
            patchbay.check(self.input)
        }

        fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
            patchbay.set(&mut self.output, patchbay.get(self.input) / self.value);
        }
    }

    struct Add {
        x: Signal,
        y: Signal,
        output: PatchPoint,
    }

    impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Add {
        fn is_ready<const P: usize>(&self, patchbay: &Patchbay<P>) -> bool {
            patchbay.check(self.x) && patchbay.check(self.y)
        }

        fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
            patchbay.set(
                &mut self.output,
                patchbay.get(self.x) + patchbay.get(self.y),
            );
        }
    }

    #[modularize]
    enum Modules {
        Constant(Constant),
        Divide(Divide),
        Add(Add),
    }

    #[test]
    fn process_should_allow_adding_modules() {
        let mut processor: Processor<SAMPLE_RATE, 4, Dummy> =
            Processor::new([None, None, None, None]);

        processor.insert_module(Dummy);
        processor.insert_module(Dummy);

        assert_eq!(
            processor.take_modules(),
            [Some(Dummy), Some(Dummy), None, None]
        );

        processor.insert_module(Dummy);
        processor.insert_module(Dummy);
        processor.insert_module(Dummy);

        assert_eq!(
            processor.take_modules(),
            [Some(Dummy), Some(Dummy), Some(Dummy), None]
        );
    }

    #[test]
    fn process_should_allow_replacing_modules() {
        let mut processor: Processor<SAMPLE_RATE, 4, Dummy> =
            Processor::new([None, None, None, None]);

        processor.replace_module(Dummy, 2);

        assert_eq!(processor.take_modules(), [Some(Dummy), None, None, None]);
    }

    #[test]
    fn process_should_allow_getting_modules() {
        let mut processor: Processor<SAMPLE_RATE, 4, Dummy> =
            Processor::new([None, None, None, Some(Dummy)]);

        let id = processor.insert_module(Dummy).unwrap();
        processor.replace_module(Dummy, 2);

        assert_eq!(processor.get_module(id), Some(&Dummy));
        assert_eq!(processor.get_module_mut(2), Some(&mut Dummy));
        assert_eq!(processor.get_module_mut(3), Some(&mut Dummy));
        assert_eq!(
            processor.take_modules(),
            [Some(Dummy), Some(Dummy), None, Some(Dummy)]
        );
    }

    #[test]
    fn process_should_run_process_on_modules() {
        let mut patchbay: Patchbay<1> = Patchbay::new();
        let output = patchbay.point().unwrap();
        let signal = output.signal();
        let mut processor: Processor<SAMPLE_RATE, 1, _> =
            Processor::new([Some(Modules::Constant(Constant { value: 0.8, output }))]);

        processor.process_modules(&mut patchbay);
        assert_eq!(patchbay.get(signal), 0.8);
    }

    #[test]
    fn process_should_run_modules_in_the_correct_order() {
        let mut patchbay: Patchbay<32> = Patchbay::new();

        let constant = Constant {
            value: 0.8,
            output: patchbay.point().unwrap(),
        };
        let divide1 = Divide {
            value: 4.0,
            input: constant.output.signal(),
            output: patchbay.point().unwrap(),
        };
        let divide2 = Divide {
            value: 2.0,
            input: divide1.output.signal(),
            output: patchbay.point().unwrap(),
        };

        let output = divide2.output.signal();

        let mut processor: Processor<SAMPLE_RATE, 3, _> = Processor::new([
            Some(Modules::Divide(divide2)),
            Some(Modules::Divide(divide1)),
            Some(Modules::Constant(constant)),
        ]);

        processor.process_modules(&mut patchbay);

        assert_eq!(patchbay.get(output), 0.1);
    }

    #[test]
    fn process_should_allow_circular_connections() {
        let mut patchbay: Patchbay<3> = Patchbay::new();

        let add_output = patchbay.point().unwrap();
        let output = add_output.signal();

        let constant = Constant {
            value: 0.8,
            output: patchbay.point().unwrap(),
        };
        let divide = Divide {
            value: 2.0,
            input: output,
            output: patchbay.point().unwrap(),
        };
        let add = Add {
            x: constant.output.signal(),
            y: divide.output.signal(),
            output: add_output,
        };

        let mut processor: Processor<SAMPLE_RATE, 3, _> = Processor::new([
            Some(Modules::Add(add)),
            Some(Modules::Constant(constant)),
            Some(Modules::Divide(divide)),
        ]);

        processor.process_modules(&mut patchbay);
        assert_eq!(patchbay.get(output), 0.8);

        processor.process_modules(&mut patchbay);
        assert_eq!(patchbay.get(output), 1.2);
    }
}
