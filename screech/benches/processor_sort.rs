use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use screech::modules::{Dummy, Mix, Oscillator};
use screech::{Module, Patchbay, Processor};
use screech_macro::modularize;

const BUFFER_SIZE: usize = 64;
const MODULES: usize = 2048;
const OSCILLATORS: usize = 16;
const MIXERS: usize = 64;
const POINTS: usize = 4096;
const SAMPLE_RATE: usize = 48000;

#[modularize]
enum Modules {
    Mix(Mix),
    Oscillator(Oscillator),
    Dummy(Dummy),
}

pub fn bench(c: &mut Criterion) {
    const DUMMY: Option<Modules> = None;
    let mut processor: Processor<SAMPLE_RATE, MODULES, Modules> = Processor::new([DUMMY; MODULES]);
    let mut patchbay: Patchbay<POINTS> = Patchbay::new();

    for m in 0..MIXERS {
        let mut mix = Mix::new(patchbay.point().unwrap());
        for o in 0..OSCILLATORS {
            let mut osc = Oscillator::new(patchbay.point().unwrap());
            let index = MODULES - ((m + 1) * OSCILLATORS) + o;

            osc.output_sine()
                .set_frequency(m as f32 * 10.0 + o as f32 * 4.0);

            mix.add_input(osc.output(), o);
            processor.replace_module(Modules::Oscillator(osc), index);
        }

        processor.replace_module(Modules::Mix(mix), m);
    }

    processor.process_modules(&mut patchbay);

    let mut group = c.benchmark_group("Processor");

    group.bench_function("sort", |b| {
        b.iter(|| process(black_box(&mut processor), black_box(&mut patchbay)))
    });

    group.finish();
}

fn process<
    const MODULES: usize,
    const POINTS: usize,
    const SAMPLE_RATE: usize,
    M: Module<SAMPLE_RATE>,
>(
    processor: &mut Processor<SAMPLE_RATE, MODULES, M>,
    patchbay: &mut Patchbay<POINTS>,
) {
    processor.clear_cache();
    processor.process_modules(patchbay);
}

criterion_group!(benches, bench);
criterion_main!(benches);
