mod clock;
mod config;
mod customer;
mod duration;
mod event;
mod history;
mod output;
mod simulation;
mod state;
mod statistics;

use clap::Parser;
use config::Config;
use simulation::Simulation;
use std::io::{self, Write};

/// Drive-Through Simulation System
#[derive(Parser, Debug)]
#[command(name = "drive-through-simulation")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = match Config::load(&args.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load {}: {}", args.config, e);
            eprintln!(
                "Please ensure the config file exists and at least one simulation is enabled."
            );
            eprintln!("\nUsage: drive-through-simulation [--config <FILE>]");
            eprintln!("  Default config file: config.yaml");
            eprintln!("\nExample:");
            eprintln!("  drive-through-simulation --config my-config.yaml");
            return;
        }
    };

    println!("=== Drive-Through Simulation System ===");
    println!("Using config file: {}", args.config);
    println!("Enabled simulations:");
    if config.fixed_simulation.enabled {
        println!("  ✓ Fixed simulation");
    }
    if config.random_simulation.enabled {
        println!("  ✓ Random simulation");
    }
    println!();
    let _ = io::stdout().flush();

    if config.fixed_simulation.enabled {
        println!("=== Drive-Through Simulation (Fixed Data from Config) ===");
        let _ = io::stdout().flush();
        let mut sim_fixed = Simulation::new(config.fixed_simulation.num_windows);
        for cust in &config.fixed_simulation.customers {
            sim_fixed.add_customer(cust.arrival, cust.service);
        }

        sim_fixed.run(None, Some(&config.fixed_simulation.history_file)).await;
        sim_fixed.print_statistics();

        if config.random_simulation.enabled {
            println!("\n");
        }
    }

    if config.random_simulation.enabled {
        println!("=== Drive-Through Simulation (Random Data from Config) ===");
        let _ = io::stdout().flush();
        let mut sim_random = Simulation::new(config.random_simulation.num_windows);
        let r = &config.random_simulation;
        sim_random.generate_random_customers(
            r.max_simulation_time,
            r.avg_arrival_interval,
            r.min_service_time,
            r.max_service_time,
        );
        sim_random.run(Some(r.max_simulation_time), Some(&r.history_file)).await;
        sim_random.print_statistics();
    }

    println!("\nSimulation(s) completed.");
}
