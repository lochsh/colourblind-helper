extern crate image;
extern crate itertools;
use itertools::Itertools;


#[derive(Copy, Clone)]
enum Channel {
    Red,
    Green,
    Blue,
}


#[derive(Copy, Clone)]
enum Axis {
    X,
    Y,
}


fn channel_change(rgb_image: &image::RgbImage, x: u32, y: u32,
                  channel: Channel, axis: Axis) -> f64 {
    let c: usize;

    match channel {
        Channel::Red => c = 0,
        Channel::Blue => c = 1,
        Channel::Green => c = 2,
    }

    match axis {
        Axis::X => rgb_image.get_pixel(x - 2, y).data[c] as f64 +
                   rgb_image.get_pixel(x - 1, y).data[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x + 1, y).data[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x + 2, y).data[c] as f64,

        Axis::Y => rgb_image.get_pixel(x, y - 2).data[c] as f64 +
                   rgb_image.get_pixel(x, y - 1).data[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x, y + 1).data[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x, y + 2).data[c] as f64,
    }
}


fn colour_change(rgb_image: &image::RgbImage, x: u32, y: u32) -> f64 {
    let channels = [Channel::Red, Channel::Green, Channel::Blue];
    let pairs = channels.iter().cloned().combinations(2);

    pairs.map(|pair| (channel_change(rgb_image, x, y, pair[0], Axis::X) -
                      channel_change(rgb_image, x, y, pair[1], Axis::X)).powi(2) +
                     (channel_change(rgb_image, x, y, pair[0], Axis::Y) -
                      channel_change(rgb_image, x, y, pair[1], Axis::Y)).powi(2)).sum::<f64>()
}

fn brightness_change(rgb_image: &image::RgbImage, x: u32, y: u32, axis: Axis) -> f64 {
    channel_change(rgb_image, x, y, Channel::Red, axis) +
    channel_change(rgb_image, x, y, Channel::Green, axis) +
    channel_change(rgb_image, x, y, Channel::Blue, axis)
}


pub fn edge_strength(rgb_image: image::RgbImage, x: u32, y: u32) -> f64 {
    brightness_change(&rgb_image, x, y, Axis::X).powi(2) +
    brightness_change(&rgb_image, x, y, Axis::Y).powi(2) +
    colour_change(&rgb_image, x, y) * 3.0
}
