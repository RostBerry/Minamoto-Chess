use std::time::Duration;
use minamoto_chess::{config, fen_api::FenApi};

use minamoto_chess_core::{board::Board, move_generation::{attack_calculator::AttackCalculator, move_gen}};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("minamoto_move_generation");
    
    // Shared resources
    let mut board = Board::from_fen(config::BENCHMARK_FEN);
    let mut legal_moves = Vec::with_capacity(218);

    group.bench_function(BenchmarkId::new("legal_movegen_benchmark", ""), |b| {
        b.iter(|| {
            let attack_calc = AttackCalculator::new(&board);
            std::hint::black_box(move_gen::generate_moves(&mut legal_moves, &mut board, &attack_calc));
            legal_moves.clear();
        });
    });

    group.bench_function(BenchmarkId::new("att_calc_benchmark", ""), |b| {
        b.iter(|| {
            std::hint::black_box(AttackCalculator::new(&board));
        });
    });

    let attack_calc = AttackCalculator::new(&board);

    group.bench_function(BenchmarkId::new("move_gen_benchmark", ""), |b| {
        b.iter(|| {
            std::hint::black_box(move_gen::generate_moves(&mut legal_moves, &mut board, &attack_calc));
            legal_moves.clear();
        });
    });

    legal_moves.clear();
    move_gen::generate_moves(&mut legal_moves, &mut board, &attack_calc);
    let mut filtered_moves = Vec::with_capacity(legal_moves.len());

    group.bench_function(BenchmarkId::new("move_gen_quiescence_benchmark", ""), |b| {
        b.iter(|| {
            std::hint::black_box(move_gen::filter_loud_moves(&mut legal_moves, &mut filtered_moves, &attack_calc, &board));
            legal_moves.clear();
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(1000).measurement_time(Duration::from_secs(5));
    targets = benchmark_comparison
}
criterion_main!(benches);