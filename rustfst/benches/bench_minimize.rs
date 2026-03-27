use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rustfst::algorithms::{minimize, minimize_with_config, MinimizeConfig};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{MutableFst, SerializableFst};
use rustfst::semirings::TropicalWeight;

/// Build a linear-chain acceptor with `n` states and deterministic transitions.
fn build_linear_fst(n: usize) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::<TropicalWeight>::new();
    for _ in 0..n {
        fst.add_state();
    }
    fst.set_start(0).unwrap();
    for i in 0..(n - 1) {
        fst.emplace_tr(i as u32, (i % 5 + 1) as u32, (i % 5 + 1) as u32, 0.0, (i + 1) as u32)
            .unwrap();
    }
    fst.set_final((n - 1) as u32, TropicalWeight::from(0.0))
        .unwrap();
    fst
}

/// Build an FST shaped like a tree that converges back, creating many
/// equivalent states suitable for minimization.
fn build_diamond_fst(depth: usize, width: usize) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::<TropicalWeight>::new();
    let start = fst.add_state();
    fst.set_start(start).unwrap();

    let mut prev_layer = vec![start];
    for d in 0..depth {
        let mut next_layer = Vec::new();
        for _ in 0..width {
            next_layer.push(fst.add_state());
        }
        for &src in &prev_layer {
            for (j, &dst) in next_layer.iter().enumerate() {
                let label = (d * width + j) % 10 + 1;
                fst.emplace_tr(src, label as u32, label as u32, 0.0, dst)
                    .unwrap();
            }
        }
        prev_layer = next_layer;
    }

    let final_state = fst.add_state();
    fst.set_final(final_state, TropicalWeight::from(0.0))
        .unwrap();
    for &src in &prev_layer {
        fst.emplace_tr(src, 1, 1, 0.0, final_state).unwrap();
    }

    fst
}

/// Build an FST from the issue #158 test case (realistic workload).
fn build_issue_158_fst() -> VectorFst<TropicalWeight> {
    let text_fst = r#"0	5	101	101	0
0	4	100	100	0
0	3	99	99	0
0	2	98	98	0
0	1	97	97	0
1	10	101	101	0
1	9	100	100	0
1	8	99	99	0
1	7	98	98	0
1	6	97	97	0
2	11	101	101	0
2	10	100	100	0
2	9	99	99	0
2	8	98	98	0
2	7	97	97	0
3	11	100	100	0
3	10	99	99	0
3	9	98	98	0
3	8	97	97	0
4	11	99	99	0
4	10	98	98	0
4	9	97	97	0
5	11	98	98	0
5	10	97	97	0
6	15	101	101	0
6	14	100	100	0
6	13	99	99	0
6	12	98	98	0
7	16	101	101	0
7	15	100	100	0
7	14	99	99	0
7	13	98	98	0
7	12	97	97	0
8	16	100	100	0
8	15	99	99	0
8	14	98	98	0
8	13	97	97	0
9	16	99	99	0
9	15	98	98	0
9	14	97	97	0
10	16	98	98	0
10	15	97	97	0
11	16	97	97	0
12	17	101	101	0
13	17	100	100	0
14	17	99	99	0
15	17	98	98	0
16	17	97	97	0
17	18	32	32	0
18	0
    "#;
    VectorFst::from_text_string(text_fst).unwrap()
}

fn bench_minimize(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimize");

    // Linear FSTs of increasing size
    for &n in &[50, 200, 1000] {
        group.bench_with_input(BenchmarkId::new("linear", n), &n, |b, &n| {
            b.iter_batched(
                || build_linear_fst(n),
                |mut fst| {
                    let config = MinimizeConfig::default().with_allow_nondet(true);
                    minimize_with_config(&mut fst, config).unwrap();
                    fst
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    // Diamond FSTs (many equivalent states)
    for &(depth, width) in &[(5, 4), (8, 4), (5, 8)] {
        group.bench_with_input(
            BenchmarkId::new("diamond", format!("d{}_w{}", depth, width)),
            &(depth, width),
            |b, &(depth, width)| {
                b.iter_batched(
                    || build_diamond_fst(depth, width),
                    |mut fst| {
                        let config = MinimizeConfig::default().with_allow_nondet(true);
                        minimize_with_config(&mut fst, config).unwrap();
                        fst
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    // Real-world FST from issue #158
    group.bench_function("issue_158", |b| {
        b.iter_batched(
            build_issue_158_fst,
            |mut fst| {
                minimize(&mut fst).unwrap();
                fst
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_minimize);
criterion_main!(benches);
