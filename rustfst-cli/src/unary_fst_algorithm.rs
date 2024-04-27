use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::ArgMatches;
use colored::Colorize;
use log::{debug, info};

use rustfst::prelude::*;

fn duration_to_seconds(duration: &Duration) -> f64 {
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1.0e-9
}

fn standard_deviation(data: &[f64]) -> f64 {
    let sum: f64 = data.iter().sum();
    let mean: f64 = sum / data.len() as f64;

    let a: f64 = data.iter().map(|v| (*v - mean).powf(2.0)).sum();
    let b: f64 = a / data.len() as f64;
    b.sqrt()
}

pub trait UnaryFstAlgorithm {
    fn get_path_in(&self) -> &str;
    fn get_path_out(&self) -> &str;
    fn get_algorithm_name(&self) -> String;

    fn read(&self) -> Result<VectorFst<TropicalWeight>> {
        VectorFst::<TropicalWeight>::read(self.get_path_in())
    }

    fn run_algorithm(&self, fst: VectorFst<TropicalWeight>) -> Result<VectorFst<TropicalWeight>>;

    fn write(&self, fst: &VectorFst<TropicalWeight>) -> Result<()> {
        fst.write(self.get_path_out())
    }

    fn run_cli_or_bench(&self, m: &ArgMatches) -> Result<()> {
        if m.contains_id("bench") {
            // Run bench
            self.run_bench(
                m.get_one::<String>("n_warm_ups").unwrap().parse().unwrap(),
                m.get_one::<String>("n_iters").unwrap().parse().unwrap(),
                m.get_one::<String>("export-markdown").map(|s| s.as_str()),
            )
        } else {
            // Run cli
            self.run_cli()
        }
    }

    fn run_cli(&self) -> Result<()> {
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

    fn run_bench(
        &self,
        n_warm_ups: usize,
        n_iters: usize,
        path_markdown_report: Option<&str>,
    ) -> Result<()> {
        println!(
            "Running benchmark for algorithm {}",
            self.get_algorithm_name().blue()
        );
        let mut avg_parsing_time = Duration::default();
        let mut avg_algo_time = Duration::default();
        let mut avg_serialization_time = Duration::default();

        let mut parsing_times = vec![];
        let mut algo_times = vec![];
        let mut serialization_times = vec![];
        let mut cli_times = vec![];

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

                parsing_times.push(duration_to_seconds(&duration_parsing));
                algo_times.push(duration_to_seconds(&duration_algo));
                serialization_times.push(duration_to_seconds(&duration_serialization));
                cli_times.push(
                    duration_to_seconds(&duration_parsing)
                        + duration_to_seconds(&duration_algo)
                        + duration_to_seconds(&duration_serialization),
                )
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
            writeln!(
                file,
                "| {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} | {:.3} ± {:.3} |",
                duration_to_seconds(&avg_parsing_time),
                standard_deviation(&parsing_times),
                duration_to_seconds(&avg_algo_time),
                standard_deviation(&algo_times),
                duration_to_seconds(&avg_serialization_time),
                standard_deviation(&serialization_times),
                duration_to_seconds(&mean_total_time),
                standard_deviation(&cli_times),
            )?;
        }
        Ok(())
    }
}
