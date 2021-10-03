use criterion::{black_box, criterion_group, criterion_main, Criterion};
use screech::basic::{Oscillator, Track};
use screech::core::{BasicTracker, Primary};
use screech::traits::Source;

fn primary(osc_count: usize, track_count: usize) -> Vec<f32> {
    let tracker = BasicTracker::<20000>::new();
    let mut primary = Primary::<960>::with_tracker(Box::new(tracker), 48_000);
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

    let mut sources: Vec<&mut dyn Source> = vec![];
    let mut tracks: Vec<&mut dyn Source> =
        tracks.iter_mut().map(|t| t as &mut dyn Source).collect();
    let mut oscs: Vec<&mut dyn Source> = oscs.iter_mut().map(|o| o as &mut dyn Source).collect();

    sources.append(&mut tracks);
    sources.append(&mut oscs);

    primary.sample(sources).unwrap().to_vec()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sampling oscillators across multiple tracks", |b| {
        b.iter(|| {
            primary(
                black_box(10), // # oscillators
                black_box(10), // times # tracks
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
