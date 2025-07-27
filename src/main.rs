use std::fs;

use clap::Parser;
use rand::random;

use crate::{city::City, city_generation::CityGenerator};

mod building;
mod city;
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
fn city_generator(cli: &Cli) -> City {
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

    let city = city_gen.generate(buildings, important_buildings, important_buildings_scale);
    println!("Seed is {seed}",);

    // let mut img = ImageBuffer::new(
    //     10 + (city.max_x - city.min_x) as u32,
    //     10 + (city.max_y - city.min_y) as u32,
    // );
    println!(
        "size : {}x{}",
        city.max_x - city.min_x,
        city.max_y - city.min_y
    );
    city

    // img.save("output/city.png")
}
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn main() {
    let cli = Cli::parse();
    fs::create_dir("output").unwrap_or_default();
    let city = city_generator(&cli);
    graphics::start_city_explorer(city);
}
