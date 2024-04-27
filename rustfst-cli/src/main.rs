use std::process;

use anyhow::{format_err, Result};
use clap::parser::ValueSource;
use clap::{Arg, ArgAction, Command};
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
    let mut app = Command::new("rustfst")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .about("Rustfst CLI");

    // Determinize
    let determinize_cmd = Command::new("determinize")
        .about("Determinize algorithm.")
        .arg(
            Arg::new("det_type")
                .long("det_type")
                .value_parser(["functional", "nonfunctional", "disambiguate"])
                .default_value("functional")
                .action(ArgAction::Set),
        );
    app = app.subcommand(one_in_one_out_options(determinize_cmd));

    // Minimization
    let minimize_cmd = Command::new("minimize")
        .about("Minimization algorithm.")
        .arg(
            Arg::new("allow-nondet")
                .help("Minimize non-deterministic FSTs ?")
                .long("allow-nondet")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(minimize_cmd));

    // Connect
    let connect_cmd = Command::new("connect").about("Connect algorithm.");
    app = app.subcommand(one_in_one_out_options(connect_cmd));

    // Trsort
    let tr_sort_cmd = Command::new("tr_sort")
        .about("Trsort algorithm.")
        .alias("arcsort")
        .arg(
            Arg::new("sort_type")
                .help("Comparison method.")
                .long("sort_type")
                .value_parser(["ilabel", "olabel"])
                .default_value("ilabel")
                .action(ArgAction::Set),
        );
    app = app.subcommand(one_in_one_out_options(tr_sort_cmd));

    // Project
    let project_cmd = Command::new("project").about("Project algorithm.").arg(
        Arg::new("project-output")
            .help("Project output (vs. input)")
            .long("project-output")
            .action(ArgAction::SetTrue),
    );
    app = app.subcommand(one_in_one_out_options(project_cmd));

    // Invert
    let invert_cmd = Command::new("invert").about("Invert algorithm.");
    app = app.subcommand(one_in_one_out_options(invert_cmd));

    // Topsort
    let topsort_cmd = Command::new("topsort").about("Topsort algorithm.");
    app = app.subcommand(one_in_one_out_options(topsort_cmd));

    // Optimize
    let optimize_cmd = Command::new("optimize").about("Optimize algorithm.");
    app = app.subcommand(one_in_one_out_options(optimize_cmd));

    // Reverse
    let reverse_cmd = Command::new("reverse").about("Reverse algorithm.");
    app = app.subcommand(one_in_one_out_options(reverse_cmd));

    // Map
    let map_cmd = Command::new("map")
        .about("Applies an operation to each tr of an FST.")
        .arg(
            Arg::new("map_type")
                .long("map_type")
                .value_parser([
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
                .default_value("identity")
                .help("Map operation.")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("weight")
                .long("weight")
                .required_if_eq_any([("map_type", "plus"), ("map_type", "times")])
                .action(ArgAction::Set),
        );
    app = app.subcommand(one_in_one_out_options(map_cmd));

    // Shortest Path
    let shortest_path_cmd = Command::new("shortestpath")
        .about("Shortest Path algorithm.")
        .arg(
            Arg::new("nshortest")
                .long("nshortest")
                .default_value("1")
                .help("Return N-shortest paths")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("unique")
                .long("unique")
                .help("Return unique strings")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(shortest_path_cmd));

    // Rm Final Epsilon
    let rm_final_epsilon_cmd = Command::new("rmfinalepsilon").about("RmFinalEpsilon algorithm.");
    app = app.subcommand(one_in_one_out_options(rm_final_epsilon_cmd));

    // Push
    let push_cmd = Command::new("push")
        .about("Push Weights/Labels algorithm")
        .arg(
            Arg::new("to_final")
                .long("to_final")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("push_weights")
                .long("push_weights")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("push_labels")
                .long("push_labels")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("remove_total_weight")
                .long("remove_total_weight")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("remove_common_affix")
                .long("remove_common_affix")
                .action(ArgAction::SetTrue),
        );
    app = app.subcommand(one_in_one_out_options(push_cmd));

    // Compose
    let compose_cmd = Command::new("compose").about("Compose algorithm").arg(
        Arg::new("compose_type")
            .long("compose_type")
            .value_parser(["default", "lookahead"])
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
            m.get_one::<String>("in.fst").unwrap(),
            m.value_source("allow-nondet") == Some(ValueSource::CommandLine),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("determinize", m)) => DeterminizeAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
            m.get_one::<String>("det_type").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("connect", m)) => ConnectAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("tr_sort", m)) => TrsortAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("sort_type").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("optimize", m)) => OptimizeAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("project", m)) => ProjectFstAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.value_source("project-output") == Some(ValueSource::CommandLine),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("invert", m)) => InvertAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("topsort", m)) => TopsortAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("reverse", m)) => ReverseAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("map", m)) => MapAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("map_type").unwrap(),
            m.get_one::<String>("weight").map(|s| s.as_str()),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("shortestpath", m)) => ShortestPathAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.value_source("unique") == Some(ValueSource::CommandLine),
            m.get_one::<String>("nshortest").unwrap().parse().unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("rmfinalepsilon", m)) => RmFinalEpsilonAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        Some(("push", m)) => PushAlgorithm::new(
            m.get_one::<String>("in.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
            m.value_source("to_final") == Some(ValueSource::CommandLine),
            m.value_source("push_weights") == Some(ValueSource::CommandLine),
            m.value_source("push_labels") == Some(ValueSource::CommandLine),
            m.value_source("remove_total_weight") == Some(ValueSource::CommandLine),
            m.value_source("remove_common_affix") == Some(ValueSource::CommandLine),
        )
        .run_cli_or_bench(m),
        Some(("compose", m)) => ComposeAlgorithm::new(
            m.get_one::<String>("in_1.fst").unwrap(),
            m.get_one::<String>("in_2.fst").unwrap(),
            m.get_one::<String>("out.fst").unwrap(),
            m.get_one::<String>("compose_type").unwrap(),
        )
        .run_cli_or_bench(m),
        Some((s, _)) => Err(format_err!("Unknown subcommand {}.", s)),
        None => Err(format_err!("Unknown None")),
    }
}

fn one_in_one_out_options(command: Command) -> Command {
    command
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(
            Arg::new("in.fst")
                .help("Path to input fst file.")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("out.fst")
                .help("Path to output fst file.")
                .required(true)
                .action(ArgAction::Set),
        ).arg(
            Arg::new("bench")
                .long("bench")
                .help("Whether to run multiple times the algorithm in order to have a reliable time measurement.")
                .action(ArgAction::SetTrue)
        ).arg(
            Arg::new("n_iters")
                .long("n_iters")
                .default_value("10")
                .help("Number of iterations to run for the benchmark.")
                .action(ArgAction::Set)
        ).arg(
            Arg::new("n_warm_ups")
                .long("n_warm_ups")
                .default_value("3")
                .help("Number of warm ups run before the actual benchmark.")
                .action(ArgAction::Set)
        ).arg(
        Arg::new("export-markdown")
            .long("export-markdown")
            .action(ArgAction::Set)
    )
}

fn two_in_one_out_options(command: Command) -> clap::Command {
    command
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(
            Arg::new("in_1.fst")
                .help("Path to the first input fst file.")
                .required(true)
                .action(ArgAction::Set),

        )
        .arg(
            Arg::new("in_2.fst")
                .help("Path to the second input fst file.")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("out.fst")
                .help("Path to output fst file.")
                .required(true)
                .action(ArgAction::Set),
        ).arg(
        Arg::new("bench")
            .long("bench")
            .help("Whether to run multiple times the algorithm in order to have a reliable time measurement.")
            .action(ArgAction::Set)
    ).arg(
        Arg::new("n_iters")
            .long("n_iters")
            .default_value("10")
            .help("Number of iterations to run for the benchmark.")
            .action(ArgAction::Set)
    ).arg(
        Arg::new("n_warm_ups")
            .long("n_warm_ups")
            .default_value("3")
            .help("Number of warm ups run before the actual benchmark.")
            .action(ArgAction::Set)
    ).arg(
        Arg::new("export-markdown")
            .long("export-markdown")
            .action(ArgAction::Set)
    )
}
