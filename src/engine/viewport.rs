use engine::context::Window;
use engine::geo_utils::GeoUtils;
use sdl2::rect::{Point, Rect};

pub struct Viewport {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Viewport {
    pub fn new(window: &Window, center: (i32, i32)) -> Viewport {
        Viewport {
            x: center.0,
            y: center.1,
            width: window.width,
            height: window.height,
        }
    }

    pub fn update(&mut self, change: (i32, i32)) {
        self.x += change.0;
        self.y += change.1;
    }

    pub fn in_viewport(&self, point: Point) -> bool {
        let x_min = self.x - (self.width / 2) as i32;
        let x_max = self.x + (self.width / 2) as i32;
        let y_min = self.y - (self.height / 2) as i32;
        let y_max = self.y + (self.height / 2) as i32;

        point.x() >= x_min && point.x() <= x_max && point.y() >= y_min && point.y() <= y_max
    }

    /// Returns the point in the game relative to the viewpoint
    pub fn relative_point(&self, map_point: (i32, i32)) -> (i32, i32) {
        (map_point.0 - self.x, map_point.1 - self.y)
    }
}