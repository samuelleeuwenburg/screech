extern crate screech;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use screech::traits::{Source, Tracker};
use screech::{Output, Screech};

struct Oscillator {
    pub id: usize,
    pub output: Output,
    voltage: f32,
    frequency: f32,
}

impl Oscillator {
    fn new(screech: &mut Screech, frequency: f32) -> Self {
        let id = screech.create_source_id();

        Oscillator {
            id,
            frequency,
            output: screech.init_output(&id, "output"),
            voltage: 0.0,
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

fn speed_100_4096(c: &mut Criterion) {
    let buffer_size = 4096;
    let sample_rate = 48_000;
    let mut screech = Screech::new(buffer_size, sample_rate);
    let mut oscillators = vec![];

    for i in 0..100 {
        let osc = Oscillator::new(&mut screech, (i as f32 + 1.0) * 10.0);
        screech.create_main_out("mono_out");
        screech.connect_signal_to_main_out(&osc.output, "mono_out");
        oscillators.push(osc);
    }

    let mut sources: Vec<&mut dyn Source> = oscillators
        .iter_mut()
        .map(|o| o as &mut dyn Source)
        .collect();

    c.bench_function(
        "sampling speed 100 oscillators @ 48000hz 4096 samples",
        |b| {
            b.iter(|| {
                screech.sample(black_box(&mut sources)).unwrap();
            })
        },
    );
}

fn speed_100_512(c: &mut Criterion) {
    let buffer_size = 512;
    let sample_rate = 48_000;
    let mut screech = Screech::new(buffer_size, sample_rate);
    let mut oscillators = vec![];

    for i in 0..100 {
        let osc = Oscillator::new(&mut screech, (i as f32 + 1.0) * 10.0);
        screech.create_main_out("mono_out");
        screech.connect_signal_to_main_out(&osc.output, "mono_out");
        oscillators.push(osc);
    }

    let mut sources: Vec<&mut dyn Source> = oscillators
        .iter_mut()
        .map(|o| o as &mut dyn Source)
        .collect();

    c.bench_function(
        "sampling speed 100 oscillators @ 48000hz 512 samples",
        |b| {
            b.iter(|| {
                screech.sample(black_box(&mut sources)).unwrap();
            })
        },
    );
}

fn speed_100_64(c: &mut Criterion) {
    let buffer_size = 64;
    let sample_rate = 48_000;
    let mut screech = Screech::new(buffer_size, sample_rate);
    let mut oscillators = vec![];

    for i in 0..100 {
        let osc = Oscillator::new(&mut screech, (i as f32 + 1.0) * 10.0);
        screech.create_main_out("mono_out");
        screech.connect_signal_to_main_out(&osc.output, "mono_out");
        oscillators.push(osc);
    }

    let mut sources: Vec<&mut dyn Source> = oscillators
        .iter_mut()
        .map(|o| o as &mut dyn Source)
        .collect();

    c.bench_function("sampling speed 100 oscillators @ 48000hz 64 samples", |b| {
        b.iter(|| {
            screech.sample(black_box(&mut sources)).unwrap();
        })
    });
}

fn speed_100_1(c: &mut Criterion) {
    let buffer_size = 1;
    let sample_rate = 48_000;
    let mut screech = Screech::new(buffer_size, sample_rate);
    let mut oscillators = vec![];

    for i in 0..100 {
        let osc = Oscillator::new(&mut screech, (i as f32 + 1.0) * 10.0);
        screech.create_main_out("mono_out");
        screech.connect_signal_to_main_out(&osc.output, "mono_out");
        oscillators.push(osc);
    }

    let mut sources: Vec<&mut dyn Source> = oscillators
        .iter_mut()
        .map(|o| o as &mut dyn Source)
        .collect();

    c.bench_function("sampling speed 100 oscillators @ 48000hz 1 samples", |b| {
        b.iter(|| {
            screech.sample(black_box(&mut sources)).unwrap();
        })
    });
}

criterion_group!(
    benches,
    speed_100_4096,
    speed_100_512,
    speed_100_64,
    speed_100_1
);
criterion_main!(benches);
