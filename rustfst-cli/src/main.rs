use std::process;

use clap::{App, Arg, SubCommand};
use failure::format_err;
use log::error;

use crate::cmds::arcsort::ArcsortAlgorithm;
use crate::cmds::connect::ConnectAlgorithm;
use crate::cmds::invert::InvertAlgorithm;
use crate::cmds::minimize::MinimizeAlgorithm;
use crate::cmds::project::ProjectFstAlgorithm;
use crate::cmds::reverse::ReverseAlgorithm;
use crate::cmds::topsort::TopsortAlgorithm;
use crate::pretty_errors::ExitFailure;
use crate::unary_fst_algorithm::UnaryFstAlgorithm;

pub mod cmds;
pub mod pretty_errors;
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

    // Arcsort
    let arcsort_cmd = SubCommand::with_name("arcsort")
        .about("Arcsort algorithm.")
        .arg(
            Arg::with_name("sort_type")
                .help("Comparison method.")
                .long("sort_type")
                .takes_value(true)
                .possible_values(&["ilabel", "olabel"])
                .default_value("ilabel"),
        );
    app = app.subcommand(one_in_one_out_options(arcsort_cmd));

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

    let matches = app.get_matches();

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");

    env_logger::Builder::from_env(env)
        .default_format_timestamp_nanos(true)
        .init();

    if let Err(e) = handle(matches) {
        error!("{:?}", e);
        process::exit(1)
    }
}

/// Handles the command-line input.
fn handle(matches: clap::ArgMatches) -> Result<(), ExitFailure> {
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
        ("arcsort", Some(m)) => ArcsortAlgorithm::new(
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
        )
}
