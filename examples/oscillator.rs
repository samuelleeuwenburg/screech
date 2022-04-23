extern crate screech;

use screech::traits::{Source, Tracker};
use screech::{Output, Screech, ScreechError};

struct Oscillator {
    pub id: usize,
    pub output: Output,
    voltage: f32,
    frequency: f32,
}

impl Oscillator {
    fn new(screech: &mut Screech) -> Self {
        let id = screech.create_source_id();

        Oscillator {
            id,
            output: screech.init_output(&id, "output"),
            voltage: 0.0,
            frequency: 2.0,
        }
    }
}

impl Source for Oscillator {
    fn sample(&mut self, tracker: &mut dyn Tracker, sample_rate: usize) {
        let signal = tracker.get_mut_output(&self.output).unwrap();
        let increase_per_sample = 2.0 / sample_rate as f32 * self.frequency;

        for s in signal.samples.iter_mut() {
            *s = self.voltage;
            self.voltage += increase_per_sample;

            if self.voltage >= 1.0 {
                self.voltage = -1.0;
            }
        }
    }

    fn get_source_id(&self) -> &usize {
        &self.id
    }
}

fn main() -> Result<(), ScreechError> {
    let buffer_size = 5;
    let sample_rate = 8;
    let mut screech = Screech::new(buffer_size, sample_rate);

    // init oscillator
    let mut osc = Oscillator::new(&mut screech);

    // setup new output buffer
    screech.create_main_out("mono_out");

    // connect oscillator output to output buffer
    screech.connect_signal_to_main_out(&osc.output, "mono_out");

    let mut sources = vec![&mut osc as &mut dyn Source];

    screech.sample(&mut sources)?;
    assert_eq!(
        screech.get_main_out("mono_out").unwrap().samples,
        [0.0, 0.5, -1.0, -0.5, 0.0]
    );

    screech.sample(&mut sources)?;
    assert_eq!(
        screech.get_main_out("mono_out").unwrap().samples,
        [0.5, -1.0, -0.5, 0.0, 0.5]
    );

    screech.sample(&mut sources)?;
    assert_eq!(
        screech.get_main_out("mono_out").unwrap().samples,
        [-1.0, -0.5, 0.0, 0.5, -1.0]
    );

    Ok(())
}
