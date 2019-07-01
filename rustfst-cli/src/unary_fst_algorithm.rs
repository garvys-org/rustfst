use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

use clap::ArgMatches;
use colored::Colorize;
use failure::Fallible;
use log::{debug, info};

use rustfst::prelude::*;

fn duration_to_seconds(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1.0e-9
}

pub trait UnaryFstAlgorithm {
    fn get_path_in(&self) -> &str;
    fn get_path_out(&self) -> &str;
    fn get_algorithm_name(&self) -> String;

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
                m.value_of("export-markdown")
            )
        } else {
            // Run cli
            self.run_cli()
        }
    }

    fn run_cli(&self) -> Fallible<()> {
        info!("Running {} algorithm", self.get_algorithm_name().blue());
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

    fn run_bench(&self, n_warm_ups: usize, n_iters: usize, path_markdown_report: Option<&str>) -> Fallible<()> {
        println!(
            "Running benchmark for algorithm {}",
            self.get_algorithm_name().blue()
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
                println!(
                    "Run #{}/{}: \t{} \t{} \t{}",
                    format!("{}", i + 1 - n_warm_ups).yellow(),
                    format!("{}", n_iters).yellow(),
                    format!("{:.6}s", duration_to_seconds(&duration_parsing)).blue(),
                    format!("{:.6}s", duration_to_seconds(&duration_algo)).magenta(),
                    format!("{:.6}s", duration_to_seconds(&duration_serialization)).cyan(),
                );

                avg_parsing_time = avg_parsing_time.checked_add(duration_parsing).unwrap();
                avg_algo_time = avg_algo_time.checked_add(duration_algo).unwrap();
                avg_serialization_time = avg_serialization_time
                    .checked_add(duration_serialization)
                    .unwrap();
            } else {
                println!(
                    "Warmup #{}/{}: \t{} \t{} \t{}",
                    format!("{}", i + 1).yellow(),
                    format!("{}", n_warm_ups).yellow(),
                    format!("{:.6}s", duration_to_seconds(&duration_parsing)).blue(),
                    format!("{:.6}s", duration_to_seconds(&duration_algo)).magenta(),
                    format!("{:.6}s", duration_to_seconds(&duration_serialization)).cyan(),
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
        println!("{}", s.bold().underline());

        let s = format!(
            "\t Mean {} : \t\t{}",
            "parsing time".blue(),
            format!("{:.6}s", duration_to_seconds(&avg_parsing_time)).blue()
        );
        println!("{}", s.bold());
        let s = format!(
            "\t Mean {} : \t\t{}",
            "algorithm time".magenta(),
            format!("{:.6}s", duration_to_seconds(&avg_algo_time)).magenta()
        );
        println!("{}", s.bold());

        let s = format!(
            "\t Mean {} : \t{}",
            "serialization time".cyan(),
            format!("{:.6}s", duration_to_seconds(&avg_serialization_time)).cyan()
        );
        println!("{}", s.bold());

        let mean_total_time = avg_parsing_time + avg_algo_time + avg_serialization_time;
        let s = format!(
            "\t Mean {} : \t\t{}",
            "CLI time".red(),
            format!("{:.6}s", duration_to_seconds(&mean_total_time)).red()
        );
        println!("{}", s.bold());

        if let Some(_path) = path_markdown_report {
            let mut file = File::create(_path)?;
            writeln!(file, "| {:.6} | {:.6} | {:.6} | {:.6} |",
                     duration_to_seconds(&avg_parsing_time),
                     duration_to_seconds(&avg_algo_time),
                     duration_to_seconds(&avg_serialization_time),
                     duration_to_seconds(&mean_total_time)
            )?;
        }
        Ok(())
    }
}
