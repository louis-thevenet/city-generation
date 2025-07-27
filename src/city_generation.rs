use pathfinding::prelude::astar;
use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::slice::ParallelSliceMut;
use std::{collections::HashMap, ops::Range, time::Instant};

use crate::{building::Building, city::City};

#[derive(Clone)]
pub enum CellType {
    Road,
    Building,
}

/// Random city generator
pub struct CityGenerator {
    rng: ChaCha8Rng,
    /// Lets us know if a point is not free
    is_something: HashMap<(i32, i32), CellType>,
    /// Min and max width of the buildings
    width_bound: Range<i32>,
    /// Min and max height of the buildings
    height_bound: Range<i32>,
    /// Min and max distance between buildings
    distance_bound: Range<i32>,
    /// Max distance between important buildings
    important_buildings_max_distance: i32,
}

impl CityGenerator {
    #[must_use]
    pub fn new(
        seed: u64,
        width_bound: Range<i32>,
        height_bound: Range<i32>,
        distance_bound: Range<i32>,
        important_buildings_max_distance: i32,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            is_something: HashMap::new(),
            width_bound,
            height_bound,
            distance_bound,
            important_buildings_max_distance,
        }
    }
    pub fn generate(
        &mut self,
        normal_buildings: usize,
        important_buildings: usize,
        important_building_scale: i32,
    ) -> City {
        println!("Generating important buildings");

        let now = Instant::now();
        let mut city =
            self.generate_important_buildings(important_buildings, important_building_scale);
        let duration = now.elapsed();
        println!(
            "Generated {} important buildings in {}",
            important_buildings,
            duration.as_secs_f32()
        );

        println!("Generating normal buildings");
        let now = Instant::now();
        self.generate_buildings(&mut city, normal_buildings);
        let duration = now.elapsed();
        println!(
            "Generated {} buildings in {}",
            normal_buildings,
            duration.as_secs_f32()
        );
        city.update_borders();
        city.is_something = self.is_something.clone();
        city
    }
    fn generate_important_buildings(&mut self, n: usize, important_building_scale: i32) -> City {
        let mut city = City::new();
        // generate the important buildings with a smaller scale

        for _ in 0..n {
            // New building
            let b1 = self.generate_random_important_building(&mut city, important_building_scale);
            // Register the building in the map
            for x in b1.x..=b1.x + b1.width {
                for y in b1.y..=b1.y + b1.height {
                    self.is_something.insert((x, y), CellType::Building);
                }
            }
            // Keep door free to go through
            self.is_something.remove(&b1.door);

            // Keep track of the important building
            city.important_buildings.push((b1.x, b1.y));
            city.update_borders_from_new_building(&b1);
            city.buildings.insert((b1.x, b1.y), b1);
        }
        let mut buildings = city.buildings.values().collect::<Vec<&Building>>(); // We'll iterate over the buildings
        buildings.par_sort_by(|b1, b2| b1.x.cmp(&b2.x).then(b1.y.cmp(&b2.y)));

        for &b1 in &buildings {
            for &b2 in &buildings {
                if b1 == b2 {
                    continue;
                }

                let road = if let Some((road, _)) = self.generate_road(&city, b1, b2) {
                    road
                } else {
                    vec![]
                };
                for (x, y) in &road {
                    self.is_something.insert((*x, *y), CellType::Road);
                }
                city.roads.push(road);
            }
        }
        // Now, we will update everywthing to scale, so multiply everything by the scale factor
        if important_building_scale > 1 {
            self.is_something.clear();
            city.min_x *= important_building_scale;
            city.min_y *= important_building_scale;
            city.max_x *= important_building_scale;
            city.max_y *= important_building_scale;

            for building in city.buildings.values_mut() {
                building.x *= important_building_scale;
                building.y *= important_building_scale;
                building.width *= important_building_scale;
                building.height *= important_building_scale;

                building.door.0 *= important_building_scale;
                building.door.1 *= important_building_scale;

                for x in building.x..=building.x + building.width {
                    for y in building.y..=building.y + building.height {
                        self.is_something.insert((x, y), CellType::Building);
                    }
                }
                self.is_something.remove(&building.door);
            }
            for road in &mut city.roads {
                let mut scaled_road = vec![];
                for i in 0..road.len() - 1 {
                    let mut direction = (road[i + 1].0 - road[i].0, road[i + 1].1 - road[i].1);
                    direction = (
                        if direction.0 == 0 {
                            0
                        } else {
                            direction.0 / direction.0.abs()
                        },
                        if direction.1 == 0 {
                            0
                        } else {
                            direction.1 / direction.1.abs()
                        },
                    );

                    let mut position = (
                        road[i].0 * important_building_scale,
                        road[i].1 * important_building_scale,
                    );
                    for _i in 0..important_building_scale {
                        scaled_road.push(position);
                        self.is_something.insert(position, CellType::Road);
                        position = (position.0 + direction.0, position.1 + direction.1);
                    }
                }
                *road = scaled_road;
            }
        }
        city
    }
    /// Generate a random important building
    fn generate_random_important_building(
        &mut self,
        city: &mut City,
        scale_factor: i32,
    ) -> Building {
        let (x, y) = (
            self.rng.random_range(
                -(self.important_buildings_max_distance / (scale_factor * 2))
                    ..(self.important_buildings_max_distance / (scale_factor * 2)),
            ),
            self.rng.random_range(
                (-self.important_buildings_max_distance / (scale_factor * 2))
                    ..(self.important_buildings_max_distance / (scale_factor * 2)),
            ),
        );
        let width = (self.rng.random_range(self.width_bound.clone()) + scale_factor) / scale_factor;
        let height =
            (self.rng.random_range(self.height_bound.clone()) + scale_factor) / scale_factor;

        let building =
            Building::with_random_door(&mut self.rng, x, y, width, height, 0).make_important();
        if city.buildings.values().any(|b| b.overlaps(&building, 3)) {
            self.generate_random_important_building(city, scale_factor)
        } else {
            building
        }
    }
    fn generate_buildings(&mut self, city: &mut City, mut n: usize) {
        let init_n = n as f32;
        while n > 0 {
            let Building {
                door: _,
                is_important: _,
                x,
                y,
                width,
                height,
                id: _,
            } = {
                let values = city.buildings.values();
                let mut a = values.into_iter().collect::<Vec<&Building>>();
                a.par_sort_by(|b1, b2| b1.x.cmp(&b2.x).then(b1.y.cmp(&b2.y)));
                a.into_iter().choose(&mut self.rng).unwrap()
            };
            let x_center = x + width / 2;
            let y_center = y + height / 2;

            // let distance_x = self.rng.random_range(self.distance_bound.clone());
            // let distance_y = self.rng.random_range(self.distance_bound.clone());

            let distance_x = ((self.distance_bound.end - self.distance_bound.start) as f32
                * n as f32
                / init_n) as i32
                + self.distance_bound.start;

            let distance_y = ((self.distance_bound.end - self.distance_bound.start) as f32
                * (n as f32 / init_n)) as i32
                + self.distance_bound.start;

            let spawn_x = if self.rng.random_bool(0.5) {
                x_center + distance_x
            } else {
                x_center - distance_x
            };

            let spawn_y = if self.rng.random_bool(0.5) {
                y_center + distance_y
            } else {
                y_center - distance_y
            };

            let width = self.rng.random_range(self.width_bound.clone());
            let height = self.rng.random_range(self.height_bound.clone());

            let offset = 8; // minimum distance between buildings
            let new_building =
                Building::with_random_door(&mut self.rng, spawn_x, spawn_y, width, height, n);
            let overlaps =
                        // seems inefficient but it's A* that's the bottleneck
                            city
                            .buildings
                            .iter()
                            .any(|(_, b)| b.overlaps(&new_building, offset) && b != &new_building)
                            // it's okay to only check on building walls and not inside

                            || (spawn_x..=spawn_x + width)
                                .any(|x| self.is_something.contains_key(&(x, spawn_y)))

                            || (spawn_x..=spawn_x + width
            )                    .any(|x| self.is_something.contains_key(&(x, spawn_y + height)))

                            || (spawn_y..=spawn_y + height)
                                .any(|y| self.is_something.contains_key(&(spawn_x, y)))

                            ||( spawn_y..=spawn_y + height
            )                    .any(|y| self.is_something.contains_key(&(spawn_x + width, y)))

                            ;

            if !overlaps {
                let buildings_clone = city.buildings.clone();
                let closest_important_building = buildings_clone
                    .get(
                        city.important_buildings
                            .iter()
                            .min_by_key(|(x, y)| (x - spawn_x).abs() + (y - spawn_y).abs())
                            .unwrap(),
                    )
                    .unwrap();

                for x in spawn_x..=spawn_x + width {
                    for y in spawn_y..=spawn_y + height {
                        self.is_something.insert((x, y), CellType::Building);
                    }
                }
                self.is_something.remove(&(*x, *y));

                city.update_borders_from_new_building(&new_building);
                let road = if let Some((road, _)) =
                    self.generate_road(city, &new_building, closest_important_building)
                {
                    road
                } else {
                    println!(
                        "No road found between {closest_important_building:?} and {spawn_x},{spawn_y}"
                    );

                    vec![]
                };
                for (x, y) in &road {
                    self.is_something.insert((*x, *y), CellType::Road);
                }
                city.buildings.insert((spawn_x, spawn_y), new_building);
                city.roads.push(road);

                n -= 1;
            }
        }
    }

    fn successors(&self, city: &City, p: (i32, i32)) -> Vec<((i32, i32), i32)> {
        let (x, y) = p;

        let mut successors = vec![];
        for i in -1..=1 {
            for j in -1..=1 {
                // Don't go diagonally
                if i != 0 && j != 0 {
                    continue;
                }

                // Don't go back to the same point
                // Don't go out of known bounds
                if i == 0 && j == 0
                    || (x + i < city.min_x
                        || x + i >= city.max_x
                        || y + j < city.min_y
                        || y + j >= city.max_y)
                {
                    continue;
                }

                let base_score = if i != 0 && j != 0 { 14 } else { 10 }; // if we go diagonally, the cost is sqrt(2)

                match self.is_something.get(&(x + i, y + j)) {
                    Some(CellType::Building) => match city.buildings.get(&(x + i, y + j)) {
                        Some(building) => {
                            // if we are in the door of the building, we can go through
                            if building.door == (x + i, y + j) {
                                successors.push(((x + i, y + j), base_score));
                            }
                        }
                        None => continue,
                    },
                    Some(CellType::Road) => successors.push(((x + i, y + j), base_score)),
                    None => successors.push(((x + i, y + j), base_score * 5)), // penalize going through nothing
                }
            }
        }

        successors
    }

    /// Wrapper around A* to generate a road between two buildings
    /// Road will start at the door of the first building and end as close as possible to the second
    /// building or an existing road
    #[allow(clippy::cast_possible_truncation)]
    fn generate_road(
        &self,
        city: &City,
        start: &Building,
        end: &Building,
    ) -> Option<(Vec<(i32, i32)>, i32)> {
        let (x2, y2) = start.door;
        astar(
            &(x2, y2),
            |&p| self.successors(city, p),
            |&p| {
                let (x, y) = p;
                f64::from(((x - end.x + end.width).abs() + (y - end.y + end.height).abs()) * 10)
                    .sqrt() as i32
            },
            |&p| matches!(self.is_something.get(&p), Some(CellType::Road)) || end.contains(p),
        )
    }
}
