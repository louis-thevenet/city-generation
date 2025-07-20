use std::collections::HashMap;

use crate::building::Building;
const CITY_BOUNDS_OFFSET: i32 = 20;

/// Represents a city with buildings and roads.
pub struct City {
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
}

impl Default for City {
    fn default() -> Self {
        Self {
            min_x: i32::MAX,
            min_y: i32::MAX,
            max_x: 0,
            max_y: 0,
            buildings: HashMap::new(),
            important_buildings: vec![],
            roads: vec![],
        }
    }
}
impl City {
    pub fn new() -> Self {
        City::default()
    }
    /// Computes the borders of the city
    pub fn update_borders(&mut self) {
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
    pub fn update_borders_from_new_building(&mut self, building: &Building) {
        self.min_x = self.min_x.min(building.x - CITY_BOUNDS_OFFSET);
        self.min_y = self.min_y.min(building.y - CITY_BOUNDS_OFFSET);

        self.max_x = self
            .max_x
            .max(building.x + building.width + CITY_BOUNDS_OFFSET);
        self.max_y = self
            .max_y
            .max(building.y + building.height + CITY_BOUNDS_OFFSET);
    }
}
