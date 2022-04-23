extern crate screech;

use screech::traits::{Source, Tracker};
use screech::{Input, Output, Screech, ScreechError};

struct FixedSignal {
    pub id: usize,
    pub output: Output,
    pub signal: f32,
}

impl FixedSignal {
    fn new(screech: &mut Screech, signal: f32) -> Self {
        let id = screech.create_source_id();

        FixedSignal {
            id,
            output: screech.init_output(&id, "output"),
            signal,
        }
    }
}

impl Source for FixedSignal {
    fn sample(&mut self, sources: &mut dyn Tracker, _sample_rate: usize) {
        let signal = sources.get_mut_output(&self.output).unwrap();

        for s in signal.samples.iter_mut() {
            *s = self.signal;
        }
    }

    fn get_source_id(&self) -> &usize {
        &self.id
    }
}

struct Mix {
    pub id: usize,
    pub input: Input,
    pub output: Output,
}

impl Mix {
    fn new(screech: &mut Screech) -> Self {
        let id = screech.create_source_id();

        Mix {
            id,
            input: screech.init_input(&id, "input"),
            output: screech.init_output(&id, "output"),
        }
    }
}

impl Source for Mix {
    fn sample(&mut self, tracker: &mut dyn Tracker, _sample_rate: usize) {
        for i in 0..*tracker.get_buffer_size() {
            let mut signal = 0.0;

            for input in tracker.get_input(&self.input).unwrap().into_iter() {
                if let Some(s) = tracker.get_output(&input).and_then(|o| o.samples.get(i)) {
                    signal += s;
                }
            }

            let output = tracker.get_mut_output(&self.output).unwrap();
            output.samples[i] = signal;
        }
    }

    fn get_source_id(&self) -> &usize {
        &self.id
    }
}

fn main() -> Result<(), ScreechError> {
    let buffer_size = 4;
    let sample_rate = 48_000;
    let mut screech = Screech::new(buffer_size, sample_rate);

    // setup sources
    let mut signal_a = FixedSignal::new(&mut screech, 0.1);
    let mut signal_b = FixedSignal::new(&mut screech, 0.2);
    let mut signal_c = FixedSignal::new(&mut screech, 0.3);
    let mut signal_d = FixedSignal::new(&mut screech, 0.4);
    let mut mix_a = Mix::new(&mut screech);
    let mut mix_b = Mix::new(&mut screech);

    // create new output buffer
    screech.create_main_out("mono_out");

    // setup connections
    screech.connect_signal(&signal_a.output, &mix_a.input);
    screech.connect_signal(&signal_b.output, &mix_a.input);
    screech.connect_signal(&signal_c.output, &mix_b.input);
    screech.connect_signal(&signal_d.output, &mix_b.input);
    screech.connect_signal_to_main_out(&mix_a.output, "mono_out");
    screech.connect_signal_to_main_out(&mix_b.output, "mono_out");

    let mut sources = vec![
        &mut signal_a as &mut dyn Source,
        &mut signal_b as &mut dyn Source,
        &mut signal_c as &mut dyn Source,
        &mut signal_d as &mut dyn Source,
        &mut mix_a as &mut dyn Source,
        &mut mix_b as &mut dyn Source,
    ];

    screech.sample(&mut sources)?;
    assert_eq!(
        screech.get_main_out("mono_out").unwrap().samples,
        [1.0, 1.0, 1.0, 1.0]
    );

    Ok(())
}
