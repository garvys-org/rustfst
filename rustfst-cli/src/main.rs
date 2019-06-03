use std::process;

use clap::{App, Arg, SubCommand};
use failure::format_err;
use log::error;

use crate::pretty_errors::ExitFailure;

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
        .about("Minimization algorithm")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(Arg::with_name("in.fst").help("Path to input fst file").required(true))
        .arg(
            Arg::with_name("allow-nondet")
                .help("Minimize non-deterministic FSTs ?")
                .long("allow-nondet"),
        )
        .arg(Arg::with_name("out.fst").help("Path to output fst file").required(true));
    app = app.subcommand(minimize_cmd);

    // Connect
    let connect_cmd = SubCommand::with_name("connect")
        .about("Connect algorithm")
        .version("1.0")
        .author("Alexandre Caulier <alexandre.caulier@protonmail.com>")
        .arg(Arg::with_name("in.fst").help("Path to input fst file").required(true))
        .arg(Arg::with_name("out.fst").help("Path to output fst file").required(true));
    app = app.subcommand(connect_cmd);

    let matches = app.get_matches();

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");

    env_logger::Builder::from_env(env).default_format_timestamp_nanos(true).init();

    if let Err(e) = handle(matches) {
        error!("{:?}", e);
        process::exit(1)
    }
}

/// Handles the command-line input.
fn handle(matches: clap::ArgMatches) -> Result<(), ExitFailure> {
    match matches.subcommand() {
        ("minimize", Some(m)) => {
            crate::minimize::minimize_cli(
                m.value_of("in.fst").unwrap(),
                m.is_present("allow-nondet"),
                m.value_of("out.fst").unwrap(),
            )
        },
        ("connect", Some(m)) => {
            crate::connect::connect_cli(
                m.value_of("in.fst").unwrap(),
                m.value_of("out.fst").unwrap(),
            )
        }
        (s, _) => Err(format_err!("Unknown subcommand {}.", s)),
    }.map_err(|e| e.into())
}

