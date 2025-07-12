use std::{fs, time::Instant};

use clap::Parser;
use image::{ImageBuffer, ImageResult, Rgb};
use rand::random;

use crate::{city_generation::CityGenerator, image_utils::draw_rect};

mod city_generation;
mod image_utils;

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
fn city_generator(cli: &Cli) -> ImageResult<()> {
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
    println!("Seed is {seed}",);

    let mut img = ImageBuffer::new(
        10 + (city_gen.max_x - city_gen.min_x) as u32,
        10 + (city_gen.max_y - city_gen.min_y) as u32,
    );
    println!(
        "size : {}x{}",
        city_gen.max_x - city_gen.min_x,
        city_gen.max_y - city_gen.min_y
    );

    // roads
    for road in &city_gen.roads {
        for (x, y) in road {
            img.put_pixel(
                *x as u32 - city_gen.min_x as u32,
                *y as u32 - city_gen.min_y as u32,
                Rgb([139, 69, 19]),
            );
        }
    }
    for building in city_gen.buildings.values() {
        draw_rect(
            &mut img,
            (
                building.x as u32 - city_gen.min_x as u32,
                building.y as u32 - city_gen.min_y as u32,
            ),
            building.width as u32,
            building.height as u32,
            // color based on id, the less the more red
            Rgb([
                255 - (building.id as f32 / buildings as f32 * 255.0) as u8,
                if building.is_important { 255 } else { 0 },
                (building.id as f32 / buildings as f32 * 255.0) as u8,
            ]),
        );
        draw_rect(
            &mut img,
            (
                building.door.0 as u32 - city_gen.min_x as u32,
                building.door.1 as u32 - city_gen.min_y as u32,
            ),
            0,
            0,
            Rgb([255, 0, 0]),
        );
    }

    img.save("output/city.png")
}
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn main() -> ImageResult<()> {
    let cli = Cli::parse();
    fs::create_dir("output").unwrap_or_default();
    city_generator(&cli)
}
