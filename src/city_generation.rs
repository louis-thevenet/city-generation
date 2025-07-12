use pathfinding::prelude::astar;
use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::slice::ParallelSliceMut;
use std::{collections::HashMap, ops::Range};

const CITY_BOUNDS_OFFSET: i32 = 20;

enum CellType {
    Road,
    Building,
}

/// Building of the city
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Building {
    /// Coordinates of the door
    pub door: (i32, i32),
    /// x coordinate of top left corner
    pub x: i32,
    /// y coordinate of top left corner
    pub y: i32,
    /// Width of the building
    pub width: i32,
    /// Height of the building
    pub height: i32,
    /// If the building is important
    pub is_important: bool,
    /// Unique identifier
    pub id: usize,
}
impl Building {
    /// Check if two buildings overlap
    fn overlaps(&self, other: &Building, offset: i32) -> bool {
        self.x - offset < other.x + other.width
            && self.x + self.width + offset > other.x
            && self.y - offset < other.y + other.height
            && self.y + self.height + offset > other.y
    }
    /// Check if a point is inside the building (including its walls)
    fn contains(&self, pos: (i32, i32)) -> bool {
        let (x, y) = pos;
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Create a building from a rectangle and ID, randomizes the door
    fn with_random_door(
        rng: &mut ChaCha8Rng,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: usize,
    ) -> Self {
        let (door_x, door_y) = if rng.gen_bool(0.5) {
            // on northern or southern side
            if rng.gen_bool(0.5) {
                // northern side
                (rng.gen_range(x..x + width), y)
            } else {
                // southern side
                (rng.gen_range(x..x + width), y + height)
            }
        } else {
            // on eastern or western side
            if rng.gen_bool(0.5) {
                // eastern side
                (x + width, rng.gen_range(y..y + height))
            } else {
                // western side
                (x, rng.gen_range(y..y + height))
            }
        };
        Self {
            is_important: false,
            door: (door_x, door_y),
            x,
            y,
            width,
            height,
            id,
        }
    }
    /// Make the building important
    fn make_important(self) -> Self {
        Self {
            is_important: true,
            ..self
        }
    }
}

/// Random city generator
pub struct CityGenerator {
    rng: ChaCha8Rng,
    /// Buildings of the city
    pub buildings: HashMap<(i32, i32), Building>,
    /// Buildings of the city
    pub important_buildings: Vec<(i32, i32)>,
    /// Roads of the city
    pub roads: Vec<Vec<(i32, i32)>>,
    /// x coordinate of the leftmost building
    pub min_x: i32,
    /// y coordinate of the topmost building
    pub min_y: i32,
    /// x coordinate of the rightmost building
    pub max_x: i32,
    /// y coordinate of the bottommost building
    pub max_y: i32,
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
            min_x: i32::MAX,
            min_y: i32::MAX,
            max_x: 0,
            max_y: 0,
            buildings: HashMap::new(),
            important_buildings: vec![],
            is_something: HashMap::new(),
            roads: vec![],
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
    ) {
        println!("Generating important buildings");
        self.generate_important_buildings(important_buildings, important_building_scale);
        println!("Generating normal buildings");
        self.generate_buildings(normal_buildings);
        self.update_borders();
    }
    fn generate_important_buildings(&mut self, n: usize, important_building_scale: i32) {
        // generate the important buildings with a smaller scale

        for _ in 0..n {
            // New building
            let b1 = self.generate_random_important_building(important_building_scale);
            // Register the building in the map
            for x in b1.x..=b1.x + b1.width {
                for y in b1.y..=b1.y + b1.height {
                    self.is_something.insert((x, y), CellType::Building);
                }
            }
            // Keep door free to go through
            self.is_something.remove(&b1.door);

            // Keep track of the important building
            self.important_buildings.push((b1.x, b1.y));
            self.update_borders_from_new_building(&b1);
            self.buildings.insert((b1.x, b1.y), b1);
        }
        let mut buildings = self.buildings.values().collect::<Vec<&Building>>(); // We'll iterate over the buildings
        buildings.par_sort_by(|b1, b2| b1.x.cmp(&b2.x).then(b1.y.cmp(&b2.y)));

        for &b1 in &buildings {
            for &b2 in &buildings {
                if b1 == b2 {
                    continue;
                }

                let road = if let Some((road, _)) = self.generate_road(b1, b2) {
                    road
                } else {
                    vec![]
                };
                for (x, y) in &road {
                    self.is_something.insert((*x, *y), CellType::Road);
                }
                self.roads.push(road);
            }
        }
        // Now, we will update everywthing to scale, so multiply everything by the scale factor
        if important_building_scale > 1 {
            self.is_something.clear();
            self.min_x *= important_building_scale;
            self.min_y *= important_building_scale;
            self.max_x *= important_building_scale;
            self.max_y *= important_building_scale;

            for building in self.buildings.values_mut() {
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
            for road in &mut self.roads {
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
    }
    /// Generate a random important building
    fn generate_random_important_building(&mut self, scale_factor: i32) -> Building {
        let (x, y) = (
            self.rng.gen_range(
                -(self.important_buildings_max_distance / (scale_factor * 2))
                    ..(self.important_buildings_max_distance / (scale_factor * 2)),
            ),
            self.rng.gen_range(
                (-self.important_buildings_max_distance / (scale_factor * 2))
                    ..(self.important_buildings_max_distance / (scale_factor * 2)),
            ),
        );
        let width = (self.rng.gen_range(self.width_bound.clone()) + scale_factor) / scale_factor;
        let height = (self.rng.gen_range(self.height_bound.clone()) + scale_factor) / scale_factor;

        let building =
            Building::with_random_door(&mut self.rng, x, y, width, height, 0).make_important();
        if self.buildings.values().any(|b| b.overlaps(&building, 3)) {
            self.generate_random_important_building(scale_factor)
        } else {
            building
        }
    }
    fn generate_buildings(&mut self, mut n: usize) {
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
                let values = self.buildings.values();
                let mut a = values.into_iter().collect::<Vec<&Building>>();
                a.par_sort_by(|b1, b2| b1.x.cmp(&b2.x).then(b1.y.cmp(&b2.y)));
                a.into_iter().choose(&mut self.rng).unwrap()
            };
            let x_center = x + width / 2;
            let y_center = y + height / 2;

            // let distance_x = self.rng.gen_range(self.distance_bound.clone());
            // let distance_y = self.rng.gen_range(self.distance_bound.clone());

            let distance_x = ((self.distance_bound.end - self.distance_bound.start) as f32
                * n as f32
                / init_n) as i32
                + self.distance_bound.start;

            let distance_y = ((self.distance_bound.end - self.distance_bound.start) as f32
                * (n as f32 / init_n)) as i32
                + self.distance_bound.start;

            let spawn_x = if self.rng.gen_bool(0.5) {
                x_center + distance_x
            } else {
                x_center - distance_x
            };

            let spawn_y = if self.rng.gen_bool(0.5) {
                y_center + distance_y
            } else {
                y_center - distance_y
            };

            let width = self.rng.gen_range(self.width_bound.clone());
            let height = self.rng.gen_range(self.height_bound.clone());

            let offset = 8; // minimum distance between buildings
            let new_building =
                Building::with_random_door(&mut self.rng, spawn_x, spawn_y, width, height, n);
            let overlaps =
                        // seems inefficient but it's A* that's the bottleneck
                            self
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
                let buildings_clone = self.buildings.clone();
                let closest_important_building = buildings_clone
                    .get(
                        self.important_buildings
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

                self.update_borders_from_new_building(&new_building);
                let road = if let Some((road, _)) =
                    self.generate_road(&new_building, closest_important_building)
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
                self.buildings.insert((spawn_x, spawn_y), new_building);
                self.roads.push(road);

                n -= 1;
            }
        }
    }

    /// Computes the borders of the city
    fn update_borders(&mut self) {
        self.min_x = self.buildings.values().map(|b| b.x).min().unwrap() - CITY_BOUNDS_OFFSET;
        self.min_y = self.buildings.values().map(|b| b.y).min().unwrap() - CITY_BOUNDS_OFFSET;

        self.max_x = self
            .buildings
            .values()
            .map(|b| b.x + b.width)
            .max()
            .unwrap()
            + CITY_BOUNDS_OFFSET;
        self.max_y = self
            .buildings
            .values()
            .map(|b| b.y + b.height)
            .max()
            .unwrap()
            + CITY_BOUNDS_OFFSET;
    }

    /// Update the borders of the city based on a new building
    fn update_borders_from_new_building(&mut self, building: &Building) {
        self.min_x = self.min_x.min(building.x - CITY_BOUNDS_OFFSET);
        self.min_y = self.min_y.min(building.y - CITY_BOUNDS_OFFSET);

        self.max_x = self
            .max_x
            .max(building.x + building.width + CITY_BOUNDS_OFFSET);
        self.max_y = self
            .max_y
            .max(building.y + building.height + CITY_BOUNDS_OFFSET);
    }

    fn successors(&self, p: (i32, i32)) -> Vec<((i32, i32), i32)> {
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
                    || (x + i < self.min_x
                        || x + i >= self.max_x
                        || y + j < self.min_y
                        || y + j >= self.max_y)
                {
                    continue;
                }

                let base_score = if i != 0 && j != 0 { 14 } else { 10 }; // if we go diagonally, the cost is sqrt(2)

                match self.is_something.get(&(x + i, y + j)) {
                    Some(CellType::Building) => match self.buildings.get(&(x + i, y + j)) {
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
    fn generate_road(&self, start: &Building, end: &Building) -> Option<(Vec<(i32, i32)>, i32)> {
        let (x2, y2) = start.door;
        astar(
            &(x2, y2),
            |&p| self.successors(p),
            |&p| {
                let (x, y) = p;
                f64::from(((x - end.x + end.width).abs() + (y - end.y + end.height).abs()) * 10)
                    .sqrt() as i32
            },
            |&p| matches!(self.is_something.get(&p), Some(CellType::Road)) || end.contains(p),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::CityGenerator;

    #[test]
    fn test_different_seeds() {
        let seed = 1;
        let mut city_gen1 = CityGenerator::new(seed, 10..30, 10..30, 20..100, 1000);
        let mut city_gen2 = CityGenerator::new(seed, 10..30, 10..30, 20..100, 1000);
        city_gen1.generate(100, 6, 10);
        city_gen2.generate(100, 6, 10);

        assert_eq!(city_gen1.min_x, city_gen2.min_x);
        assert_eq!(city_gen1.min_y, city_gen2.min_y);
        assert_eq!(city_gen1.max_x, city_gen2.max_x);
        assert_eq!(city_gen1.max_y, city_gen2.max_y);
        assert_eq!(city_gen1.buildings, city_gen2.buildings);
    }
}
