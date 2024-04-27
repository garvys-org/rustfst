use std::process;

use anyhow::{format_err, Result};
use clap::{App, Arg, ArgAction, SubCommand};
use log::error;

use crate::binary_fst_algorithm::BinaryFstAlgorithm;
use crate::cmds::compose::ComposeAlgorithm;
use crate::cmds::connect::ConnectAlgorithm;
use crate::cmds::determinize::DeterminizeAlgorithm;
use crate::cmds::invert::InvertAlgorithm;
use crate::cmds::map::MapAlgorithm;
use crate::cmds::minimize::MinimizeAlgorithm;
use crate::cmds::optimize::OptimizeAlgorithm;
use crate::cmds::project::ProjectFstAlgorithm;
use crate::cmds::push::PushAlgorithm;
use crate::cmds::reverse::ReverseAlgorithm;
use crate::cmds::rm_final_epsilon::RmFinalEpsilonAlgorithm;
use crate::cmds::shortest_path::ShortestPathAlgorithm;
use crate::cmds::topsort::TopsortAlgorithm;
use crate::cmds::tr_sort::TrsortAlgorithm;
use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub mod binary_fst_algorithm;
pub mod cmds;
pub mod unary_fst_algorithm;

fn main() {
    let mut app = App::new("rustfst")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .about("Rustfst CLI");

    // Determinize
    let determinize_cmd = SubCommand::with_name("determinize")
        .about("Determinize algorithm.")
        .arg(
            Arg::with_name("det_type")
                .long("det_type")
                .possible_values(["functional", "nonfunctional", "disambiguate"])
                .takes_value(true)
                .default_value("functional")
                .action(ArgAction::Set),
        );
    app = app.subcommand(one_in_one_out_options(determinize_cmd));

    // Minimization
    let minimize_cmd = SubCommand::with_name("minimize")
        .about("Minimization algorithm.")
        .arg(
            Arg::with_name("allow-nondet")
                .help("Minimize non-deterministic FSTs ?")
                .long("allow-nondet")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(minimize_cmd));

    // Connect
    let connect_cmd = SubCommand::with_name("connect").about("Connect algorithm.");
    app = app.subcommand(one_in_one_out_options(connect_cmd));

    // Trsort
    let tr_sort_cmd = SubCommand::with_name("tr_sort")
        .about("Trsort algorithm.")
        .alias("arcsort")
        .arg(
            Arg::with_name("sort_type")
                .help("Comparison method.")
                .long("sort_type")
                .takes_value(true)
                .possible_values(["ilabel", "olabel"])
                .default_value("ilabel")
                .action(ArgAction::Set),
        );
    app = app.subcommand(one_in_one_out_options(tr_sort_cmd));

    // Project
    let project_cmd = SubCommand::with_name("project")
        .about("Project algorithm.")
        .arg(
            Arg::with_name("project-output")
                .help("Project output (vs. input)")
                .long("project-output")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(project_cmd));

    // Invert
    let invert_cmd = SubCommand::with_name("invert").about("Invert algorithm.");
    app = app.subcommand(one_in_one_out_options(invert_cmd));

    // Topsort
    let topsort_cmd = SubCommand::with_name("topsort").about("Topsort algorithm.");
    app = app.subcommand(one_in_one_out_options(topsort_cmd));

    // Optimize
    let optimize_cmd = SubCommand::with_name("optimize").about("Optimize algorithm.");
    app = app.subcommand(one_in_one_out_options(optimize_cmd));

    // Reverse
    let reverse_cmd = SubCommand::with_name("reverse").about("Reverse algorithm.");
    app = app.subcommand(one_in_one_out_options(reverse_cmd));

    // Map
    let map_cmd = SubCommand::with_name("map")
        .about("Applies an operation to each tr of an FST.")
        .arg(
            Arg::with_name("map_type")
                .long("map_type")
                .possible_values([
                    "arc_sum",
                    "arc_unique",
                    "tr_sum",
                    "tr_unique",
                    "identity",
                    "input_epsilon",
                    "invert",
                    "output_epsilon",
                    "plus",
                    "quantize",
                    "rmweight",
                    "times",
                ])
                .takes_value(true)
                .default_value("identity")
                .help("Map operation.")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::with_name("weight")
                .long("weight")
                .takes_value(true)
                .required_ifs(&[("map_type", "plus"), ("map_type", "times")])
                .action(ArgAction::Set),
        );
    app = app.subcommand(one_in_one_out_options(map_cmd));

    // Shortest Path
    let shortest_path_cmd = SubCommand::with_name("shortestpath")
        .about("Shortest Path algorithm.")
        .arg(
            Arg::with_name("nshortest")
                .long("nshortest")
                .takes_value(true)
                .default_value("1")
                .help("Return N-shortest paths")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::with_name("unique")
                .long("unique")
                .help("Return unique strings")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(shortest_path_cmd));

    // Rm Final Epsilon
    let rm_final_epsilon_cmd =
        SubCommand::with_name("rmfinalepsilon").about("RmFinalEpsilon algorithm.");
    app = app.subcommand(one_in_one_out_options(rm_final_epsilon_cmd));

    // Push
    let push_cmd = SubCommand::with_name("push")
        .about("Push Weights/Labels algorithm")
        .arg(
            Arg::with_name("to_final")
                .long("to_final")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::with_name("push_weights")
                .long("push_weights")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::with_name("push_labels")
                .long("push_labels")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::with_name("remove_total_weight")
                .long("remove_total_weight")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::with_name("remove_common_affix")
                .long("remove_common_affix")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(push_cmd));

    // Compose
    let compose_cmd = SubCommand::with_name("compose")
        .about("Compose algorithm")
        .arg(
            Arg::with_name("compose_type")
                .long("compose_type")
                .possible_values(["default", "lookahead"])
                .takes_value(true)
                .default_value("default")
                .action(ArgAction::Set),
        );
    app = app.subcommand(two_in_one_out_options(compose_cmd));

    let matches = app.get_matches();

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");

    env_logger::Builder::from_env(env)
        .format_timestamp_nanos()
        .init();

    if let Err(e) = handle(matches) {
        error!("{:?}", e);
        process::exit(exitcode::OK)
    }
}

/// Handles the command-line input.
fn handle(matches: clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("minimize", m)) => MinimizeAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.is_present("allow-nondet"),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("determinize", m)) => DeterminizeAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
            m.value_of("det_type").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("connect", m)) => ConnectAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("tr_sort", m)) => TrsortAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("sort_type").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("optimize", m)) => OptimizeAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("project", m)) => ProjectFstAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.is_present("project-output"),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("invert", m)) => InvertAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("topsort", m)) => TopsortAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("reverse", m)) => ReverseAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("map", m)) => MapAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("map_type").unwrap(),
            m.value_of("weight"),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("shortestpath", m)) => ShortestPathAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.is_present("unique"),
            m.value_of("nshortest").unwrap().parse().unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("rmfinalepsilon", m)) => RmFinalEpsilonAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("push", m)) => PushAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
            m.is_present("to_final"),
            m.is_present("push_weights"),
            m.is_present("push_labels"),
            m.is_present("remove_total_weight"),
            m.is_present("remove_common_affix"),
        )
        .run_cli_or_bench(m),
        Some(("compose", m)) => ComposeAlgorithm::new(
            m.value_of("in_1.fst").unwrap(),
            m.value_of("in_2.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
            m.value_of("compose_type").unwrap(),
        )
        .run_cli_or_bench(m),
        Some((s, _)) => Err(format_err!("Unknown subcommand {}.", s)),
        None => Err(format_err!("Unknown None")),
    }
}

fn one_in_one_out_options(command: clap::App) -> clap::App {
    command
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(
            Arg::with_name("in.fst")
                .help("Path to input fst file.")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::with_name("out.fst")
                .help("Path to output fst file.")
                .required(true)
                .action(ArgAction::Set),
        ).arg(
            Arg::with_name("bench")
                .long("bench")
                .help("Whether to run multiple times the algorithm in order to have a reliable time measurement.")
                .action(ArgAction::SetTrue)
        ).arg(
            Arg::with_name("n_iters")
                .long("n_iters")
                .default_value("10")
                .help("Number of iterations to run for the benchmark.")
                .action(ArgAction::Set)
        ).arg(
            Arg::with_name("n_warm_ups")
                .long("n_warm_ups")
                .default_value("3")
                .help("Number of warm ups run before the actual benchmark.")
                .action(ArgAction::Set)
        ).arg(
        Arg::with_name("export-markdown")
            .long("export-markdown")
            .takes_value(true)
            .action(ArgAction::Set)
    )
}

fn two_in_one_out_options(command: clap::App) -> clap::App {
    command
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(
            Arg::with_name("in_1.fst")
                .help("Path to the first input fst file.")
                .required(true)
                .action(ArgAction::Set),

        )
        .arg(
            Arg::with_name("in_2.fst")
                .help("Path to the second input fst file.")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::with_name("out.fst")
                .help("Path to output fst file.")
                .required(true)
                .action(ArgAction::Set),
        ).arg(
        Arg::with_name("bench")
            .long("bench")
            .help("Whether to run multiple times the algorithm in order to have a reliable time measurement.")
            .action(ArgAction::Set)
    ).arg(
        Arg::with_name("n_iters")
            .long("n_iters")
            .default_value("10")
            .help("Number of iterations to run for the benchmark.")
            .action(ArgAction::Set)
    ).arg(
        Arg::with_name("n_warm_ups")
            .long("n_warm_ups")
            .default_value("3")
            .help("Number of warm ups run before the actual benchmark.")
            .action(ArgAction::Set)
    ).arg(
        Arg::with_name("export-markdown")
            .long("export-markdown")
            .takes_value(true)
            .action(ArgAction::Set)
    )
}
