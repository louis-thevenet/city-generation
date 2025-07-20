use clap::Parser;
use rand::random;

use crate::{city_generation::CityGenerator, graphics::run_city_viewer};

mod city_generation;
mod graphics;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Number of buildings
    #[arg(short, long, default_value_t = 500)]
    buildings: usize,
    /// Number of important buildings
    #[arg(short, long, default_value_t = 3)]
    important_buildings: usize,
    /// Maximum distance between important buildings
    #[arg(short, long, default_value_t = 500)]
    max_distance_seeds: i32,
    /// Scale of the important buildings
    #[arg(short, long, default_value_t = 1)]
    scale_seeds: i32,
    /// Seed
    #[arg(long)]
    seed: Option<u64>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn city_generator(cli: &Cli) -> Result<CityGenerator, Box<dyn std::error::Error>> {
    let buildings = cli.buildings;
    let important_buildings = cli.important_buildings;
    let important_buildings_max_distance = cli.max_distance_seeds;
    let important_buildings_scale = cli.scale_seeds;
    let seed = cli.seed.unwrap_or_else(random);

    let mut city_gen = CityGenerator::new(
        seed,
        8..30,
        8..30,
        20..100,
        important_buildings_max_distance,
    );

    city_gen.generate(buildings, important_buildings, important_buildings_scale);
    println!("Seed is {seed}");

    Ok(city_gen)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let city = city_generator(&cli)?;
    
    println!("Starting city viewer...");
    run_city_viewer(city)?;
    
    Ok(())
}
