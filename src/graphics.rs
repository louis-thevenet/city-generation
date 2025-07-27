use pixels::{Error, Pixels, SurfaceTexture};
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

        // Handle input events
        if input.update(&event) {
            // Handle keyboard events
            if input.key_pressed(KeyCode::ArrowLeft) || input.key_held(KeyCode::ArrowLeft) {
                city_explorer.origin.0 -= 10;
            }
            if input.key_pressed(KeyCode::ArrowRight) || input.key_held(KeyCode::ArrowRight) {
                city_explorer.origin.0 += 10;
            }
            if input.key_pressed(KeyCode::ArrowUp) || input.key_held(KeyCode::ArrowUp) {
                city_explorer.origin.1 -= 10;
            }
            if input.key_pressed(KeyCode::ArrowDown) || input.key_held(KeyCode::ArrowDown) {
                city_explorer.origin.1 += 10;
            }
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                city_explorer.window_size = size.into();
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
            // Update internal state and request a redraw
            city_explorer.update();
            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

#[allow(clippy::pedantic)]
impl CityExplorer {
    /// Create a new `World` instance that can draw a moving box.
    fn new(city: City, window_size: (u32, u32)) -> Self {
        Self {
            city,
            origin: (0, 0),
            window_size,
        }
    }

    fn update(&mut self) {}

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x_frame = (i % self.window_size.0 as usize) as i32;
            let y_frame = (i / self.window_size.0 as usize) as i32;
            let x1 = self.origin.0 - (self.window_size.0 as i32 / 2);
            // let x2 = self.origin.0 + (self.window_size.0 as i32 / 2);
            let y1 = self.origin.1 - (self.window_size.1 as i32 / 2);
            // let y2 = self.origin.1 + (self.window_size.1 as i32 / 2);

            // println!(
            //     "{},{} <=> {},{}",
            //     x_frame,
            //     y_frame,
            //     x1 + x_frame,
            //     y1 + y_frame
            // );

            let rgba = match self.city.is_something.get(&(x1 + x_frame, y1 + y_frame)) {
                Some(CellType::Building) => [255, 0, 0, 255],
                Some(CellType::Road) => [0, 255, 0, 255],
                None => [0, 0, 0, 0],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
