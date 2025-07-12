use image::{ImageBuffer, Rgb};

pub fn draw_rect(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    pos: (u32, u32),
    width: u32,
    height: u32,
    color: Rgb<u8>,
) {
    for x in 0..=width {
        *img.get_pixel_mut(pos.0 + x, pos.1) = color;
        *img.get_pixel_mut(pos.0 + x, pos.1 + height) = color;
    }
    for y in 0..=height {
        *img.get_pixel_mut(pos.0, pos.1 + y) = color;
        *img.get_pixel_mut(pos.0 + width, pos.1 + y) = color;
    }
}
