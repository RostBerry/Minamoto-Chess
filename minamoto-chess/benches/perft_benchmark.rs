use std::time::Duration;

use minamoto_chess_core::{board::Board};
use minamoto_chess::{config, fen_api::FenApi};
use minamoto_chess::perft;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn benchmark_comparison (c: &mut Criterion) {
    let mut group = c.benchmark_group("minamoto_perft");
    
    // Shared resources
    let mut board = Board::from_fen(config::BENCHMARK_FEN);

    group.bench_function(BenchmarkId::new("perft_benchmark_depth_4", ""), |b| {
        b.iter(|| {
            std::hint::black_box(perft::run_perft(4, &mut board));
        });
    });

    group.finish();
}
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(230).measurement_time(Duration::from_secs(10));
    targets = benchmark_comparison
}

criterion_main!(benches);