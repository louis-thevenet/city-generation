use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::city_generation::CityGenerator;

const DEFAULT_WIDTH: u32 = 1200;
const DEFAULT_HEIGHT: u32 = 800;

pub struct CityApp {
    window: Option<Window>,
    needs_redraw: bool,
    city: CityGenerator,
    camera_x: f32,
    camera_y: f32,
    zoom: f32,
}

impl CityApp {
    pub fn new(city: CityGenerator) -> Self {
        Self {
            window: None,
            needs_redraw: true,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
            city,
        }
    }

    fn render_city(&self, pixels: &mut Pixels, window_width: u32, window_height: u32) {
        let frame = pixels.frame_mut();
        
        // Clear frame to black
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0xff]);
        }

        // Calculate scaling and offset with camera and zoom
        let city_width = (self.city.max_x - self.city.min_x) as f32;
        let city_height = (self.city.max_y - self.city.min_y) as f32;
        
        let base_scale_x = (window_width as f32 - 20.0) / city_width;
        let base_scale_y = (window_height as f32 - 20.0) / city_height;
        let base_scale = base_scale_x.min(base_scale_y);
        let scale = base_scale * self.zoom;
        
        // Apply camera offset
        let offset_x = ((window_width as f32 - city_width * scale) / 2.0) as i32 + self.camera_x as i32;
        let offset_y = ((window_height as f32 - city_height * scale) / 2.0) as i32 + self.camera_y as i32;

        // Draw roads first (background) - single pixel width like walls
        for road in &self.city.roads {
            // Draw lines between consecutive road points
            for window in road.windows(2) {
                if let [point1, point2] = window {
                    let screen_x1 = ((point1.0 - self.city.min_x) as f32 * scale) as i32 + offset_x;
                    let screen_y1 = ((point1.1 - self.city.min_y) as f32 * scale) as i32 + offset_y;
                    let screen_x2 = ((point2.0 - self.city.min_x) as f32 * scale) as i32 + offset_x;
                    let screen_y2 = ((point2.1 - self.city.min_y) as f32 * scale) as i32 + offset_y;
                    
                    Self::draw_line(frame, screen_x1, screen_y1, screen_x2, screen_y2, [139, 69, 19, 255], window_width, window_height);
                }
            }
            
            // Also draw individual points to ensure single-point roads are visible
            for &(x, y) in road {
                let screen_x = ((x - self.city.min_x) as f32 * scale) as i32 + offset_x;
                let screen_y = ((y - self.city.min_y) as f32 * scale) as i32 + offset_y;
                
                if screen_x >= 0 && screen_x < window_width as i32 && screen_y >= 0 && screen_y < window_height as i32 {
                    Self::set_pixel(frame, screen_x as u32, screen_y as u32, [139, 69, 19, 255], window_width, window_height); // Brown roads
                }
            }
        }

        // Draw buildings
        for building in self.city.buildings.values() {
            let screen_x = ((building.x - self.city.min_x) as f32 * scale) as i32 + offset_x;
            let screen_y = ((building.y - self.city.min_y) as f32 * scale) as i32 + offset_y;
            let screen_width = (building.width as f32 * scale) as i32;
            let screen_height = (building.height as f32 * scale) as i32;

            // Building color based on ID and importance
            let color = if building.is_important {
                [255, 255, 0, 255] // Yellow for important buildings
            } else {
                let ratio = building.id as f32 / self.city.buildings.len() as f32;
                [
                    (255.0 - ratio * 255.0) as u8, // Red component decreases
                    0,                              // No green
                    (ratio * 255.0) as u8,         // Blue component increases
                    255
                ]
            };

            Self::draw_rect(frame, screen_x, screen_y, screen_width, screen_height, color, window_width, window_height);

            // Draw door as a red pixel (or small rectangle when zoomed in)
            let door_screen_x = ((building.door.0 - self.city.min_x) as f32 * scale) as i32 + offset_x;
            let door_screen_y = ((building.door.1 - self.city.min_y) as f32 * scale) as i32 + offset_y;
            
            if door_screen_x >= 0 && door_screen_x < window_width as i32 && door_screen_y >= 0 && door_screen_y < window_height as i32 {
                Self::set_pixel(frame, door_screen_x as u32, door_screen_y as u32, [255, 0, 0, 255], window_width, window_height); // Red door
            }
        }
    }

    fn set_pixel(frame: &mut [u8], x: u32, y: u32, color: [u8; 4], window_width: u32, window_height: u32) {
        if x < window_width && y < window_height {
            let idx = ((y * window_width + x) * 4) as usize;
            if idx + 3 < frame.len() {
                frame[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }

    fn draw_rect(frame: &mut [u8], x: i32, y: i32, width: i32, height: i32, color: [u8; 4], window_width: u32, window_height: u32) {
        for dy in 0..height {
            for dx in 0..width {
                let px = x + dx;
                let py = y + dy;
                
                // Draw outline only (hollow rectangle)
                if dx == 0 || dx == width - 1 || dy == 0 || dy == height - 1 {
                    if px >= 0 && px < window_width as i32 && py >= 0 && py < window_height as i32 {
                        Self::set_pixel(frame, px as u32, py as u32, color, window_width, window_height);
                    }
                }
            }
        }
    }

    // Bresenham's line algorithm for drawing solid lines
    fn draw_line(frame: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4], window_width: u32, window_height: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        
        loop {
            if x >= 0 && x < window_width as i32 && y >= 0 && y < window_height as i32 {
                Self::set_pixel(frame, x as u32, y as u32, color, window_width, window_height);
            }
            
            if x == x1 && y == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

impl ApplicationHandler for CityApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Procedural City Explorer")
            .with_inner_size(LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_min_inner_size(LogicalSize::new(320, 240));

        let window = event_loop.create_window(window_attributes).unwrap();
        
        // Request initial redraw
        window.request_redraw();
        
        self.window = Some(window);
        
        println!("City viewer started!");
        println!("Controls:");
        println!("  Arrow keys: Move camera");
        println!("  +/-: Zoom in/out (faster zoom, up to 15x in, 0.05x out)");
        println!("  0: Fit city to screen");
        println!("  R: Reset camera and zoom");
        println!("  ESC: Exit");
        println!("City size: {}x{}", 
            self.city.max_x - self.city.min_x, 
            self.city.max_y - self.city.min_y
        );
        println!("Buildings: {}", self.city.buildings.len());
        println!("Roads: {}", self.city.roads.len());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let movement_speed = 20.0; // Constant movement speed regardless of zoom
                    let mut moved = false;
                    
                    match event.logical_key {
                        Key::Named(NamedKey::Escape) => {
                            event_loop.exit();
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            self.camera_y += movement_speed;
                            moved = true;
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            self.camera_y -= movement_speed;
                            moved = true;
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            self.camera_x += movement_speed;
                            moved = true;
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            self.camera_x -= movement_speed;
                            moved = true;
                        }
                        Key::Character(c) => {
                            match c.as_str() {
                                "=" | "+" => {
                                    self.zoom = (self.zoom * 1.2).min(15.0); // Zoom in faster, max 15x
                                    println!("Zoom: {:.2}x", self.zoom);
                                    moved = true;
                                }
                                "-" => {
                                    self.zoom = (self.zoom / 1.2).max(0.05); // Zoom out faster, min 0.05x
                                    println!("Zoom: {:.2}x", self.zoom);
                                    moved = true;
                                }
                                "r" | "R" => {
                                    // Reset camera and zoom
                                    self.camera_x = 0.0;
                                    self.camera_y = 0.0;
                                    self.zoom = 1.0;
                                    moved = true;
                                    println!("Camera and zoom reset");
                                }
                                "0" => {
                                    // Fit the entire city on screen
                                    self.zoom = 1.0;
                                    self.camera_x = 0.0;
                                    self.camera_y = 0.0;
                                    moved = true;
                                    println!("Zoom set to fit city");
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    
                    if moved {
                        self.needs_redraw = true;
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Only redraw if we actually need to
                if self.needs_redraw {
                    if let Some(window) = &self.window {
                        let window_size = window.inner_size();
                        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);
                        
                        match Pixels::new(window_size.width, window_size.height, surface_texture) {
                            Ok(mut pixels) => {
                                self.render_city(&mut pixels, window_size.width, window_size.height);
                                if let Err(err) = pixels.render() {
                                    eprintln!("pixels.render() failed: {err}");
                                    event_loop.exit();
                                } else {
                                    // Mark as drawn - we don't need to redraw unless something changes
                                    self.needs_redraw = false;
                                }
                            }
                            Err(err) => {
                                eprintln!("Failed to create pixels: {err}");
                                event_loop.exit();
                            }
                        }
                    }
                }
            }
            WindowEvent::Resized(_) => {
                // Mark for redraw when window is resized
                self.needs_redraw = true;
                if let Some(window) = &self.window {
                    if window.id() == window_id {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn run_city_viewer(city: CityGenerator) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = CityApp::new(city);

    event_loop.run_app(&mut app)?;
    Ok(())
}
