use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rustfst::algorithms::{minimize, minimize_with_config, MinimizeConfig};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{MutableFst, SerializableFst};
use rustfst::semirings::TropicalWeight;

/// N identical chains from start to a shared final state.
/// All states at the same depth across chains are equivalent,
/// so `refine` must sort a large partition class and merge them into 1 group.
/// This stresses: sort on many-equal elements + partition merging.
fn build_parallel_chains(num_chains: usize, chain_len: usize) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::<TropicalWeight>::new();
    let start = fst.add_state();
    fst.set_start(start).unwrap();
    let final_state = fst.add_state();
    fst.set_final(final_state, TropicalWeight::from(0.0))
        .unwrap();

    for _ in 0..num_chains {
        let mut prev = start;
        for d in 0..chain_len {
            let s = fst.add_state();
            let label = (d % 5 + 1) as u32;
            fst.emplace_tr(prev, label, label, 0.0, s).unwrap();
            prev = s;
        }
        fst.emplace_tr(prev, 1, 1, 0.0, final_state).unwrap();
    }
    fst
}

/// Trie-like FST where each branch has unique labels.
/// States at the same depth are all DISTINCT (different outgoing labels),
/// so `refine` must sort a large partition class and split it into N groups.
/// This stresses: sort on all-distinct elements + many class allocations.
fn build_branching_trie(branching: usize, depth: usize) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::<TropicalWeight>::new();
    let start = fst.add_state();
    fst.set_start(start).unwrap();

    let mut current_layer = vec![start];
    for d in 0..depth {
        let mut next_layer = Vec::new();
        for (i, &src) in current_layer.iter().enumerate() {
            for b in 0..branching {
                let dst = fst.add_state();
                // Unique label per (parent_index, branch) so siblings are distinct
                let label = (i * branching + b + d * 1000 + 1) as u32;
                fst.emplace_tr(src, label, label, 0.0, dst).unwrap();
                next_layer.push(dst);
            }
        }
        current_layer = next_layer;
    }

    // All leaves go to a shared final state
    let final_state = fst.add_state();
    fst.set_final(final_state, TropicalWeight::from(0.0))
        .unwrap();
    for &leaf in &current_layer {
        fst.emplace_tr(leaf, 1, 1, 0.0, final_state).unwrap();
    }
    fst
}

/// Mix of equivalent and distinct states at each depth level.
/// `group_size` chains share the same labels (equivalent),
/// `num_groups` groups have different labels (distinct).
/// This stresses: sort with mixed equality + realistic group splitting.
fn build_mixed_equivalence(
    num_groups: usize,
    group_size: usize,
    depth: usize,
) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::<TropicalWeight>::new();
    let start = fst.add_state();
    fst.set_start(start).unwrap();
    let final_state = fst.add_state();
    fst.set_final(final_state, TropicalWeight::from(0.0))
        .unwrap();

    for g in 0..num_groups {
        for _ in 0..group_size {
            let mut prev = start;
            for d in 0..depth {
                let s = fst.add_state();
                // Same label within a group, different across groups
                let label = (g * depth + d + 1) as u32;
                fst.emplace_tr(prev, label, label, 0.0, s).unwrap();
                prev = s;
            }
            fst.emplace_tr(prev, 1, 1, 0.0, final_state).unwrap();
        }
    }
    fst
}

/// Cyclic FST: a ring of states with forward and cross-edges.
/// Tests the Hopcroft-style cyclic minimization path (not the acyclic refine).
fn build_cyclic_fst(n: usize) -> VectorFst<TropicalWeight> {
    let mut fst = VectorFst::<TropicalWeight>::new();
    for _ in 0..n {
        fst.add_state();
    }
    fst.set_start(0).unwrap();
    fst.set_final((n - 1) as u32, TropicalWeight::from(0.0))
        .unwrap();
    for i in 0..n {
        let src = i as u32;
        let next = ((i + 1) % n) as u32;
        let label = (i % 5 + 1) as u32;
        fst.emplace_tr(src, label, label, 0.0, next).unwrap();
        // Cross-edge to create non-trivial SCCs
        if i + 2 < n {
            let cross_label = (i % 3 + 6) as u32;
            let cross_dst = (i + 2) as u32;
            fst.emplace_tr(src, cross_label, cross_label, 0.0, cross_dst)
                .unwrap();
        }
    }
    fst
}

/// Real-world FST from issue #158.
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

    // === Acyclic path: parallel equivalent chains ===
    // Exercises: sort on equal elements, partition merging
    for &(chains, len) in &[(10, 10), (50, 10), (100, 5), (20, 20)] {
        let total = chains * len;
        group.bench_with_input(
            BenchmarkId::new(
                "parallel_chains",
                format!("{}chains_x_{}deep_{}states", chains, len, total),
            ),
            &(chains, len),
            |b, &(chains, len)| {
                b.iter_batched(
                    || build_parallel_chains(chains, len),
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

    // === Acyclic path: branching trie with all-distinct states ===
    // Exercises: sort on distinct elements, many class allocations
    for &(branching, depth) in &[(3usize, 4usize), (2, 7), (4, 3)] {
        let approx_states: usize = (0..=depth).map(|d| branching.pow(d as u32)).sum();
        group.bench_with_input(
            BenchmarkId::new(
                "branching_trie",
                format!("b{}_d{}_~{}states", branching, depth, approx_states),
            ),
            &(branching, depth),
            |b, &(branching, depth)| {
                b.iter_batched(
                    || build_branching_trie(branching, depth),
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

    // === Acyclic path: mixed equivalent + distinct states ===
    // Exercises: realistic refine with both merging and splitting
    for &(groups, size, depth) in &[(5, 10, 8), (10, 10, 5), (20, 5, 5)] {
        let total = groups * size * depth;
        group.bench_with_input(
            BenchmarkId::new(
                "mixed_equiv",
                format!("{}g_x_{}copies_x_{}deep_{}states", groups, size, depth, total),
            ),
            &(groups, size, depth),
            |b, &(groups, size, depth)| {
                b.iter_batched(
                    || build_mixed_equivalence(groups, size, depth),
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

    // === Cyclic minimization path (Hopcroft) ===
    for &n in &[50, 200, 500] {
        group.bench_with_input(BenchmarkId::new("cyclic", n), &n, |b, &n| {
            b.iter_batched(
                || build_cyclic_fst(n),
                |mut fst| {
                    let config = MinimizeConfig::default().with_allow_nondet(true);
                    minimize_with_config(&mut fst, config).unwrap();
                    fst
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    // === Real-world regression test ===
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
