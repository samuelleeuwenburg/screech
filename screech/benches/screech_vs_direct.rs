use criterion::{criterion_group, criterion_main, Criterion};
use screech::module::Module;
use screech::patchbay::{PatchError, PatchPoint, PatchPointValue, Patchbay};
use screech::processor::Processor;
use screech::sample::Sample;
use std::hint::black_box;

const BUFFER_SIZE: usize = 64;

fn expensive(x: Sample, y: Sample) -> Sample {
    let a = (x.sin() * y.cos()).powf(3.0);
    let b = (x.log10() + y.exp()).sqrt();
    let c = (x.tan() * y.tan()).abs().ln();
    a + b - c
}

struct Calculation {
    value: Sample,
    input: PatchPointValue,
    output: PatchPoint,
}

impl Module for Calculation {
    fn process<const P: usize>(&mut self, patchbay: &mut Patchbay<P>) -> Result<(), PatchError> {
        patchbay.set_sample(
            &mut self.output,
            expensive(patchbay.get_sample(self.input)?, self.value),
        );
        Ok(())
    }
}

fn screech_process_buffer<const POINTS: usize, const MODULES: usize, M: Module>(
    seed: Sample,
    patchbay: &mut Patchbay<POINTS>,
    processor: &mut Processor<MODULES, M>,
    input_point: &mut PatchPoint,
    output: PatchPointValue,
) -> [Sample; BUFFER_SIZE] {
    let mut buffer = [seed; BUFFER_SIZE];

    for i in 0..BUFFER_SIZE {
        patchbay.set_sample(input_point, buffer[i]);
        processor.process_modules(patchbay);
        buffer[i] = patchbay.get_sample(output).unwrap();
    }

    buffer
}

fn direct_process_buffer(seed: Sample) -> [Sample; BUFFER_SIZE] {
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

fn screech_benchmark(c: &mut Criterion) {
    let mut patchbay: Patchbay<8> = Patchbay::new();
    let mut input_point = patchbay.get_point();
    let point1 = patchbay.get_point();
    let point2 = patchbay.get_point();
    let point3 = patchbay.get_point();
    let point4 = patchbay.get_point();
    let point5 = patchbay.get_point();
    let point6 = patchbay.get_point();
    let output = point6.value();

    let calc6 = Calculation {
        value: 6.0,
        input: point5.value(),
        output: point6,
    };
    let calc5 = Calculation {
        value: 5.0,
        input: point4.value(),
        output: point5,
    };
    let calc4 = Calculation {
        value: 4.0,
        input: point3.value(),
        output: point4,
    };
    let calc3 = Calculation {
        value: 3.0,
        input: point2.value(),
        output: point3,
    };
    let calc2 = Calculation {
        value: 2.0,
        input: point1.value(),
        output: point2,
    };
    let calc1 = Calculation {
        value: 1.0,
        input: input_point.value(),
        output: point1,
    };

    let mut processor: Processor<6, _> = Processor::new([calc1, calc2, calc3, calc4, calc5, calc6]);

    processor.order_modules(&mut patchbay);

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

criterion_group!(benches, screech_benchmark);
criterion_main!(benches);
