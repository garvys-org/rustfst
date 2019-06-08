use std::time::{Duration, Instant};

use colored::Colorize;
use failure::Fallible;
use log::{debug, info};

use rustfst::prelude::*;

use clap::ArgMatches;

pub trait UnaryFstAlgorithm {
    fn get_path_in(&self) -> &str;
    fn get_path_out(&self) -> &str;
    fn get_algorithm_name() -> String;

    fn read(&self) -> Fallible<VectorFst<TropicalWeight>> {
        VectorFst::<TropicalWeight>::read(self.get_path_in())
    }

    fn run_algorithm(&self, fst: VectorFst<TropicalWeight>) -> Fallible<VectorFst<TropicalWeight>>;

    fn write(&self, fst: &VectorFst<TropicalWeight>) -> Fallible<()> {
        fst.write(self.get_path_out())
    }

    fn run_cli_or_bench(&self, m: &ArgMatches) -> Fallible<()> {
        if m.is_present("bench") {
            // Run bench
            self.run_bench(
                m.value_of("n_warm_ups").unwrap().parse().unwrap(),
                m.value_of("n_iters").unwrap().parse().unwrap(),
            )
        } else {
            // Run cli
            self.run_cli()
        }
    }

    fn run_cli(&self) -> Fallible<()> {
        info!("Running {} algorithm", Self::get_algorithm_name().blue());
        // Parsing
        debug!("Parsing...");
        let parsing_start = Instant::now();
        let mut fst = self.read()?;
        let duration_parsing = parsing_start.elapsed();
        debug!("Duration parsing : {:?}", &duration_parsing);

        // Algorithm
        debug!("Running algorithm...");
        let algo_start = Instant::now();
        fst = self.run_algorithm(fst)?;
        let duration_algo = algo_start.elapsed();
        debug!("Duration running algorithm : {:?}", &duration_algo);

        // Serialization
        debug!("Serialization...");
        let serialization_start = Instant::now();
        self.write(&fst)?;
        let duration_serialization = serialization_start.elapsed();
        debug!("Duration serialization : {:?}", &duration_serialization);

        Ok(())
    }

    fn run_bench(&self, n_warm_ups: usize, n_iters: usize) -> Fallible<()> {
        info!(
            "Running benchmark for algorithm {}",
            Self::get_algorithm_name().blue()
        );
        let mut avg_parsing_time = Duration::default();
        let mut avg_algo_time = Duration::default();
        let mut avg_serialization_time = Duration::default();

        for i in 0..(n_warm_ups + n_iters) {
            // Parsing
            let parsing_start = Instant::now();
            let mut fst = self.read()?;
            let duration_parsing = parsing_start.elapsed();

            // Algorithm
            let algo_start = Instant::now();
            fst = self.run_algorithm(fst)?;
            let duration_algo = algo_start.elapsed();

            // Serialization
            let serialization_start = Instant::now();
            self.write(&fst)?;
            let duration_serialization = serialization_start.elapsed();

            if i >= n_warm_ups {
                info!(
                    "Run #{}/{}: \t{} \t{} \t{}",
                    format!("{}", i + 1 - n_warm_ups).yellow(),
                    format!("{}", n_iters).yellow(),
                    format!("{:?}", &duration_parsing).blue(),
                    format!("{:?}", &duration_algo).magenta(),
                    format!("{:?}", &duration_serialization).cyan(),
                );

                avg_parsing_time = avg_parsing_time.checked_add(duration_parsing).unwrap();
                avg_algo_time = avg_algo_time.checked_add(duration_algo).unwrap();
                avg_serialization_time = avg_serialization_time
                    .checked_add(duration_serialization)
                    .unwrap();
            } else {
                info!(
                    "Warmup #{}/{}: \t{} \t{} \t{}",
                    format!("{}", i + 1).yellow(),
                    format!("{}", n_warm_ups).yellow(),
                    format!("{:?}", &duration_parsing).blue(),
                    format!("{:?}", &duration_algo).magenta(),
                    format!("{:?}", &duration_serialization).cyan(),
                );
            }
        }

        avg_parsing_time = avg_parsing_time.checked_div(n_iters as u32).unwrap();
        avg_algo_time = avg_algo_time.checked_div(n_iters as u32).unwrap();
        avg_serialization_time = avg_serialization_time.checked_div(n_iters as u32).unwrap();

        let s = format!(
            "Bench results (Warmups = {}, Iterations = {}):",
            n_warm_ups, n_iters
        );
        info!("{}", s.bold().underline());

        let s = format!(
            "\t Mean {} : \t\t{}",
            "parsing time".blue(),
            format!("{:?}", avg_parsing_time).blue()
        );
        info!("{}", s.bold());
        let s = format!(
            "\t Mean {} : \t\t{}",
            "algorithm time".magenta(),
            format!("{:?}", avg_algo_time).magenta()
        );
        info!("{}", s.bold());

        let s = format!(
            "\t Mean {} : \t{}",
            "serialization time".cyan(),
            format!("{:?}", avg_serialization_time).cyan()
        );
        info!("{}", s.bold());
        Ok(())
    }
}
