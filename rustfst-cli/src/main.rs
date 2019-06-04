use std::process;

use clap::{App, Arg, SubCommand};
use failure::format_err;
use log::error;

use crate::pretty_errors::ExitFailure;

mod arcsort;
mod connect;
mod minimize;
mod pretty_errors;

fn main() {
    //    env_logxger::init();
    let mut app = App::new("rustfst")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .about("Rustfst CLI");

    // Minimization
    let minimize_cmd = SubCommand::with_name("minimize")
        .about("Minimization algorithm.")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(
            Arg::with_name("in.fst")
                .help("Path to input fst file.")
                .required(true),
        )
        .arg(
            Arg::with_name("allow_nondet")
                .help("Minimize non-deterministic FSTs ?")
                .long("allow_nondet"),
        )
        .arg(
            Arg::with_name("out.fst")
                .help("Path to output fst file.")
                .required(true),
        );
    app = app.subcommand(minimize_cmd);

    // Connect
    let connect_cmd = SubCommand::with_name("connect")
        .about("Connect algorithm.")
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
        );
    app = app.subcommand(connect_cmd);

    // Arcsort
    let arcsort_cmd = SubCommand::with_name("arcsort")
        .about("Arcsort algorithm.")
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
        )
        .arg(
            Arg::with_name("sort_type")
                .help("Comparison method.")
                .long("sort_type")
                .takes_value(true)
                .possible_values(&["ilabel", "olabel"])
                .default_value("ilabel"),
        );
    app = app.subcommand(arcsort_cmd);

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
        ("minimize", Some(m)) => crate::minimize::minimize_cli(
            m.value_of("in.fst").unwrap(),
            m.is_present("allow_nondet"),
            m.value_of("out.fst").unwrap(),
        ),
        ("connect", Some(m)) => crate::connect::connect_cli(
            m.value_of("in.fst").unwrap(),
            m.value_of("out.fst").unwrap(),
        ),
        ("arcsort", Some(m)) => crate::arcsort::arcsort_cli(
            m.value_of("in.fst").unwrap(),
            m.value_of("sort_type").unwrap(),
            m.value_of("out.fst").unwrap(),
        ),
        (s, _) => Err(format_err!("Unknown subcommand {}.", s)),
    }
    .map_err(|e| e.into())
}
