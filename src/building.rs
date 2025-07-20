use rand::Rng;
use rand_chacha::ChaCha8Rng;

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
    pub fn overlaps(&self, other: &Building, offset: i32) -> bool {
        self.x - offset < other.x + other.width
            && self.x + self.width + offset > other.x
            && self.y - offset < other.y + other.height
            && self.y + self.height + offset > other.y
    }
    /// Check if a point is inside the building (including its walls)
    pub fn contains(&self, pos: (i32, i32)) -> bool {
        let (x, y) = pos;
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Create a building from a rectangle and ID, randomizes the door
    pub fn with_random_door(
        rng: &mut ChaCha8Rng,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: usize,
    ) -> Self {
        let (door_x, door_y) = if rng.random_bool(0.5) {
            // on northern or southern side
            if rng.random_bool(0.5) {
                // northern side
                (rng.random_range(x..x + width), y)
            } else {
                // southern side
                (rng.random_range(x..x + width), y + height)
            }
        } else {
            // on eastern or western side
            if rng.random_bool(0.5) {
                // eastern side
                (x + width, rng.random_range(y..y + height))
            } else {
                // western side
                (x, rng.random_range(y..y + height))
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
    pub fn make_important(self) -> Self {
        Self {
            is_important: true,
            ..self
        }
    }
}
