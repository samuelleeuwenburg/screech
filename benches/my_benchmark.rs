use criterion::{black_box, criterion_group, criterion_main, Criterion};
use screech::oscillator::Oscillator;
use screech::primary::Primary;
use screech::track::Track;
use screech::traits::Source;

fn primary(osc_count: usize, track_count: usize, buffer_count: usize, ms: usize) {
    let sample_rate = 48_000;
    let buffer_size = (sample_rate / 1000) * ms; // 10ms

    let mut primary = Primary::new(buffer_size, sample_rate);

    let mut tracks: Vec<Track> = vec![];
    let mut oscs: Vec<Oscillator> = vec![];

    for _ in 0..track_count {
        let mut track = Track::new(&mut primary);
        primary.add_monitor(&track);

        for o in 0..osc_count {
            let mut osc = Oscillator::new(&mut primary);
            osc.frequency = (o * 100) as f32;
            track.add_input(&osc);
            oscs.push(osc);
        }

        tracks.push(track);
    }

    for _ in 0..buffer_count {
        let mut sources: Vec<&mut dyn Source> = vec![];
        let mut tracks: Vec<&mut dyn Source> =
            tracks.iter_mut().map(|t| t as &mut dyn Source).collect();
        let mut oscs: Vec<&mut dyn Source> =
            oscs.iter_mut().map(|o| o as &mut dyn Source).collect();

        sources.append(&mut tracks);
        sources.append(&mut oscs);

        primary.sample(sources).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("semi complex example of screech usage", |b| {
        b.iter(|| {
            primary(
                black_box(100), // # oscillators
                black_box(10),  // times # tracks
                black_box(1),   // # buffers
                black_box(100), // times # buffer size in ms
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);