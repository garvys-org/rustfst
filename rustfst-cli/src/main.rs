use std::process;

use anyhow::{format_err, Result};
use clap::{App, Arg, SubCommand};
use log::error;

use crate::cmds::connect::ConnectAlgorithm;
use crate::cmds::invert::InvertAlgorithm;
use crate::cmds::map::MapAlgorithm;
use crate::cmds::minimize::MinimizeAlgorithm;
use crate::cmds::project::ProjectFstAlgorithm;
use crate::cmds::push::PushAlgorithm;
use crate::cmds::reverse::ReverseAlgorithm;
use crate::cmds::rm_final_epsilon::RmFinalEpsilonAlgorithm;
use crate::cmds::shortest_path::ShortestPathAlgorithm;
use crate::cmds::topsort::TopsortAlgorithm;
use crate::cmds::tr_sort::TrsortAlgorithm;
use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub mod cmds;
pub mod unary_fst_algorithm;

fn main() {
    let mut app = App::new("rustfst")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .about("Rustfst CLI");

    // Minimization
    let minimize_cmd = SubCommand::with_name("minimize")
        .about("Minimization algorithm.")
        .arg(
            Arg::with_name("allow_nondet")
                .help("Minimize non-deterministic FSTs ?")
                .long("allow_nondet"),
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
                .possible_values(&["ilabel", "olabel"])
                .default_value("ilabel"),
        );
    app = app.subcommand(one_in_one_out_options(tr_sort_cmd));

    // Project
    let project_cmd = SubCommand::with_name("project")
        .about("Project algorithm.")
        .arg(
            Arg::with_name("project_output")
                .help("Project output (vs. input)")
                .long("project_output"),
        );
    app = app.subcommand(one_in_one_out_options(project_cmd));

    // Invert
    let invert_cmd = SubCommand::with_name("invert").about("Invert algorithm.");
    app = app.subcommand(one_in_one_out_options(invert_cmd));

    // Topsort
    let topsort_cmd = SubCommand::with_name("topsort").about("Topsort algorithm.");
    app = app.subcommand(one_in_one_out_options(topsort_cmd));

    // Reverse
    let reverse_cmd = SubCommand::with_name("reverse").about("Reverse algorithm.");
    app = app.subcommand(one_in_one_out_options(reverse_cmd));

    // Map
    let map_cmd = SubCommand::with_name("map")
        .about("Applies an operation to each tr of an FST.")
        .arg(
            Arg::with_name("map_type")
                .long("map_type")
                .possible_values(&[
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
                .help("Map operation."),
        )
        .arg(
            Arg::with_name("weight")
                .long("weight")
                .takes_value(true)
                .required_ifs(&[("map_type", "plus"), ("map_type", "times")]),
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
                .help("Return N-shortest paths"),
        )
        .arg(
            Arg::with_name("unique")
                .long("unique")
                .help("Return unique strings"),
        );
    app = app.subcommand(one_in_one_out_options(shortest_path_cmd));

    // Rm Final Epsilon
    let rm_final_epsilon_cmd =
        SubCommand::with_name("rmfinalepsilon").about("RmFinalEpsilon algorithm.");
    app = app.subcommand(one_in_one_out_options(rm_final_epsilon_cmd));

    // Push
    let push_cmd = SubCommand::with_name("push")
        .about("Push Weights/Labels algorithm")
        .arg(Arg::with_name("to_final").long("to_final"))
        .arg(Arg::with_name("push_weights").long("push_weights"))
        .arg(Arg::with_name("push_labels").long("push_labels"))
        .arg(Arg::with_name("remove_total_weight").long("remove_total_weight"))
        .arg(Arg::with_name("remove_common_affix").long("remove_common_affix"));
    app = app.subcommand(one_in_one_out_options(push_cmd));

    let matches = app.get_matches();

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");

    env_logger::Builder::from_env(env)
        .default_format_timestamp_nanos(true)
        .init();

    if let Err(e) = handle(matches) {
        error!("{:?}", e);
        process::exit(exitcode::OK)
    }
}

/// Handles the command-line input.
fn handle(matches: clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("minimize", Some(m)) => MinimizeAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.is_present("allow_nondet"),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("connect", Some(m)) => ConnectAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("tr_sort", Some(m)) => TrsortAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("sort_type").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("project", Some(m)) => ProjectFstAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.is_present("project_output"),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("invert", Some(m)) => InvertAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("topsort", Some(m)) => TopsortAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("reverse", Some(m)) => ReverseAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("map", Some(m)) => MapAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("map_type").unwrap(),
            m.value_of("weight"),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("shortestpath", Some(m)) => ShortestPathAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.is_present("unique"),
            m.value_of("nshortest").unwrap().parse().unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("rmfinalepsilon", Some(m)) => RmFinalEpsilonAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        )
        .run_cli_or_bench(m),
        ("push", Some(m)) => PushAlgorithm::new(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
            m.is_present("to_final"),
            m.is_present("push_weights"),
            m.is_present("push_labels"),
            m.is_present("remove_total_weight"),
            m.is_present("remove_common_affix"),
        )
        .run_cli_or_bench(m),
        (s, _) => Err(format_err!("Unknown subcommand {}.", s)),
    }
    .map_err(|e| e.into())
}

fn one_in_one_out_options<'a, 'b>(command: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    command
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(
            Arg::with_name("in.fst")
                .help("Path to input fst file.")
                .required(true),
        )
        .arg(
            Arg::with_name("out.fst")
                .help("Path to output fst file.")
                .required(true),
        ).arg(
            Arg::with_name("bench")
                .long("bench")
                .help("Whether to run multiple times the algorithm in order to have a reliable time measurement.")
        ).arg(
            Arg::with_name("n_iters")
                .long("n_iters")
                .default_value("10")
                .help("Number of iterations to run for the benchmark.")
        ).arg(
            Arg::with_name("n_warm_ups")
                .long("n_warm_ups")
                .default_value("3")
                .help("Number of warm ups run before the actual benchmark.")
        ).arg(
        Arg::with_name("export-markdown")
            .long("export-markdown")
            .takes_value(true)
    )
}
