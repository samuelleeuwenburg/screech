use criterion::{criterion_group, criterion_main, Criterion};
use screech::{Module, PatchPoint, Patchbay, Processor, Signal};
use std::hint::black_box;

const BUFFER_SIZE: usize = 64;
const SAMPLE_RATE: usize = 48_000;

fn expensive(x: f32, y: f32) -> f32 {
    let a = (x.sin() * y.cos()).powf(3.0);
    let b = (x.log10() + y.exp()).sqrt();
    let c = (x.tan() * y.tan()).abs().ln();
    a + b - c
}

struct Calculation {
    value: f32,
    input: Signal,
    output: PatchPoint,
}

impl<const SAMPLE_RATE: usize> Module<SAMPLE_RATE> for Calculation {
    fn is_ready<const P: usize>(&self, patchbay: &Patchbay<P>) -> bool {
        patchbay.check(self.input)
    }

    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) {
        patchbay.set(
            &mut self.output,
            expensive(patchbay.get(self.input), self.value),
        );
    }
}

fn screech_process_buffer<
    const SAMPLE_RATE: usize,
    const P: usize,
    const MODULES: usize,
    M: Module<SAMPLE_RATE>,
>(
    seed: f32,
    patchbay: &mut Patchbay<P>,
    processor: &mut Processor<SAMPLE_RATE, MODULES, M>,
    input_point: &mut PatchPoint,
    output: Signal,
) -> [f32; BUFFER_SIZE] {
    let mut buffer = [seed; BUFFER_SIZE];

    for i in 0..BUFFER_SIZE {
        patchbay.set(input_point, buffer[i]);
        processor.process_modules(patchbay);
        buffer[i] = patchbay.get(output);
    }

    buffer
}

fn direct_process_buffer(seed: f32) -> [f32; BUFFER_SIZE] {
    let mut buffer = [seed; BUFFER_SIZE];

    for i in 0..BUFFER_SIZE {
        let calc1 = expensive(buffer[i], 1.0);
        let calc2 = expensive(calc1, 2.0);
        let calc3 = expensive(calc2, 3.0);
        let calc4 = expensive(calc3, 4.0);
        let calc5 = expensive(calc4, 5.0);
        let calc6 = expensive(calc5, 6.0);
        buffer[i] = calc6;
    }

    buffer
}

fn bench(c: &mut Criterion) {
    let mut patchbay: Patchbay<8> = Patchbay::new();
    let mut input_point = patchbay.point().unwrap();

    let calc1 = Calculation {
        value: 1.0,
        input: input_point.signal(),
        output: patchbay.point().unwrap(),
    };
    let calc2 = Calculation {
        value: 2.0,
        input: calc1.output.signal(),
        output: patchbay.point().unwrap(),
    };
    let calc3 = Calculation {
        value: 3.0,
        input: calc2.output.signal(),
        output: patchbay.point().unwrap(),
    };
    let calc4 = Calculation {
        value: 4.0,
        input: calc3.output.signal(),
        output: patchbay.point().unwrap(),
    };
    let calc5 = Calculation {
        value: 5.0,
        input: calc4.output.signal(),
        output: patchbay.point().unwrap(),
    };
    let calc6 = Calculation {
        value: 6.0,
        input: calc5.output.signal(),
        output: patchbay.point().unwrap(),
    };

    let output = calc6.output.signal();

    let mut processor: Processor<SAMPLE_RATE, 6, _> = Processor::new([
        Some(calc1),
        Some(calc2),
        Some(calc3),
        Some(calc4),
        Some(calc5),
        Some(calc6),
    ]);

    processor.process_modules(&mut patchbay);

    let mut group = c.benchmark_group("Comparison of screech with direct calculations");

    group.bench_function("screech", |b| {
        b.iter(|| {
            screech_process_buffer(
                black_box(1.0),
                black_box(&mut patchbay),
                black_box(&mut processor),
                black_box(&mut input_point),
                black_box(output),
            )
        })
    });

    group.bench_function("direct", |b| {
        b.iter(|| direct_process_buffer(black_box(1.0)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
