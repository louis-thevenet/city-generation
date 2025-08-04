use pixels::{Error, Pixels, SurfaceTexture};
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use crate::city::City;
use crate::city_generation::CellType;

/// Representation of the application state. In this example, a box will bounce around the screen.
pub struct CityExplorer {
    origin: (i32, i32),
    city: City,
    window_size: (u32, u32),
    pixels: Vec<u8>,
    zoom_factor: f32,
}

pub fn start_city_explorer(city: City) -> Result<(), Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        WindowBuilder::new()
            .with_title("City Explorer")
            // .with_min_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(window_size.width, window_size.height, surface_texture)?
    };
    let mut city_explorer = CityExplorer::new(city, window.inner_size().into());

    let res = event_loop.run(|event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            city_explorer.draw(pixels.frame_mut());
            if let Err(_err) = pixels.render() {
                elwt.exit();
                return;
            }
        }

        let mut delta_x = 0;
        let mut delta_y = 0;
        if input.update(&event) {
            if input.key_pressed(KeyCode::ArrowLeft) || input.key_held(KeyCode::ArrowLeft) {
                delta_x += -30;
            }
            if input.key_pressed(KeyCode::ArrowRight) || input.key_held(KeyCode::ArrowRight) {
                delta_x += 30;
            }
            if input.key_pressed(KeyCode::ArrowUp) || input.key_held(KeyCode::ArrowUp) {
                delta_y += -30;
            }
            if input.key_pressed(KeyCode::ArrowDown) || input.key_held(KeyCode::ArrowDown) {
                delta_y += 30;
            }
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }
        }
        let zoom_delta = input.scroll_diff().1 * 0.05;

        let speed_multiplier = if input.key_held(KeyCode::ShiftLeft) {
            2
        } else {
            1
        };
        let zoom_adjusted_speed = (speed_multiplier as f32 / city_explorer.zoom_factor) as i32;
        delta_x *= zoom_adjusted_speed;
        delta_y *= zoom_adjusted_speed;

        city_explorer.update((delta_x, delta_y), zoom_delta);
        if delta_x != 0 || delta_y != 0 || zoom_delta != 0.0 {
            city_explorer.redraw_pixels();
        }

        // Resize the window
        if let Some(size) = input.window_resized() {
            city_explorer.resize(size);
            println!("Window resized");
            if let Err(_err) = pixels.resize_buffer(size.width, size.height) {
                elwt.exit();
                return;
            }
            if let Err(_err) = pixels.resize_surface(size.width, size.height) {
                elwt.exit();
                return;
            }
        }
        window.request_redraw();
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

#[allow(clippy::pedantic)]
impl CityExplorer {
    /// Create a new `World` instance that can draw a moving box.
    fn new(city: City, window_size: (u32, u32)) -> Self {
        let mut res = Self {
            city,
            origin: (0, 0),
            window_size,
            pixels: vec![0; (window_size.0 * window_size.1 * 4) as usize],
            zoom_factor: 1.0,
        };
        res.redraw_pixels();
        res
    }

    fn update(&mut self, delta: (i32, i32), zoom_delta: f32) {
        self.origin.0 += delta.0 as i32;
        self.origin.1 += delta.1 as i32;
        self.zoom_factor += zoom_delta;
        self.zoom_factor = self.zoom_factor.clamp(0.1, 30.0);
    }

    /// There is potential to clerverly use the `delta` parameter to only redraw the pixels that have changed. But for now, it's okay as it is.
    fn redraw_pixels(&mut self) {
        self.pixels
            .par_chunks_mut(4)
            .enumerate()
            .for_each(|(i, pixel)| {
                let x_frame = (i % self.window_size.0 as usize) as i32;
                let y_frame = (i / self.window_size.0 as usize) as i32;
                let x1 = {
                    let half_width = self.window_size.0 as f32 / 2.0;
                    let world_offset = (x_frame as f32 - half_width) / self.zoom_factor;
                    self.origin.0 + world_offset as i32
                };
                let y1 = {
                    let half_height = self.window_size.1 as f32 / 2.0;
                    let world_offset = (y_frame as f32 - half_height) / self.zoom_factor;
                    self.origin.1 + world_offset as i32
                };
                let rgba = match self.city.is_something.get(&(x1, y1)) {
                    Some(CellType::Building) => [255, 0, 0, 255],
                    Some(CellType::Road) => [0, 255, 0, 255],
                    None => [0, 0, 0, 0],
                };

                pixel.copy_from_slice(&rgba);
            });
    }
    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        frame.copy_from_slice(&self.pixels);
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = (new_size.width, new_size.height);
        self.pixels
            .resize((self.window_size.0 * self.window_size.1 * 4) as usize, 0);
        self.redraw_pixels();
    }
}
