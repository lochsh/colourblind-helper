use std::cmp::Ordering;

extern crate image;
use image::{RgbImage, Luma, ImageBuffer, GrayImage};

extern crate itertools;
use itertools::{Itertools, multizip};


type EdgeImage = ImageBuffer<Luma<f64>, Vec<f64>>;


#[derive(Copy, Clone, Debug)]
enum Channel {
    Red,
    Green,
    Blue,
}


#[derive(Copy, Clone, Debug)]
enum Axis {
    X,
    Y,
}


/// Ceiling bounded image co-ordinate
///
/// Bounded image co-ordinate, e.g. x, with a maximum value.
/// Used to prevent out-of-bounds access attempts and overflow +/- attempts.
#[derive(Copy, Clone)]
struct Bounded {
    i: u32,
    max_i: u32,
}


impl Bounded {

    fn new(i: u32, max_i: u32) -> Bounded {
        match i.cmp(&max_i) {
            Ordering::Less | Ordering::Equal => Bounded { i: i, max_i: max_i },
            Ordering::Greater => Bounded { i: max_i, max_i: max_i },
        }
    }

    fn sub(self, other: u32) -> u32 {
        self.i.saturating_sub(other)
    }

    fn add(self, other: u32) -> u32 {
        match (self.i + other).cmp(&self.max_i) {
            Ordering::Less | Ordering::Equal => self.i + other,
            Ordering::Greater => self.max_i,
        }
    }
}


fn channel_change(rgb_image: &RgbImage, x: u32, y: u32, channel: Channel, axis: Axis) -> f64 {
    let c: usize;

    match channel {
        Channel::Red => c = 0,
        Channel::Green => c = 1,
        Channel::Blue => c = 2,
    }

    let x_sat = Bounded::new(x, rgb_image.width() - 1);
    let y_sat = Bounded::new(y, rgb_image.height() - 1);

    match axis {
        Axis::X => rgb_image.get_pixel(x_sat.sub(2), y)[c] as f64 +
                   rgb_image.get_pixel(x_sat.sub(1), y)[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x_sat.add(1), y)[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x_sat.add(2), y)[c] as f64,

        Axis::Y => rgb_image.get_pixel(x, y_sat.sub(2))[c] as f64 +
                   rgb_image.get_pixel(x, y_sat.sub(1))[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x, y_sat.add(1))[c] as f64 * 0.5 -
                   rgb_image.get_pixel(x, y_sat.add(2))[c] as f64,
    }
}


fn colour_change(rgb_image: &RgbImage, x: u32, y: u32) -> f64 {
    let channels = [Channel::Red, Channel::Green, Channel::Blue];
    let pairs = channels.iter().cloned().combinations(2);

    pairs.map(|pair| (channel_change(rgb_image, x, y, pair[0], Axis::X) -
                      channel_change(rgb_image, x, y, pair[1], Axis::X)).powi(2) +
                     (channel_change(rgb_image, x, y, pair[0], Axis::Y) -
                      channel_change(rgb_image, x, y, pair[1], Axis::Y)).powi(2)).sum::<f64>()
                                                                                 .sqrt()
}


fn brightness_change(rgb_image: &RgbImage, x: u32, y: u32, axis: Axis) -> f64 {
    channel_change(rgb_image, x, y, Channel::Red, axis) +
    channel_change(rgb_image, x, y, Channel::Green, axis) +
    channel_change(rgb_image, x, y, Channel::Blue, axis)
}


fn edge_strength(rgb_image: &RgbImage, x: u32, y: u32) -> f64 {
    brightness_change(rgb_image, x, y, Axis::X).powi(2) +
    brightness_change(rgb_image, x, y, Axis::Y).powi(2) +
    colour_change(rgb_image, x, y) * 3.0
}


fn edge_strengths(rgb_image: &RgbImage) -> Vec<f64> {
    let (width, height) = rgb_image.dimensions();
    (0..(width * height)).map(|i| edge_strength(rgb_image, i % width, i / width))
                         .collect::<Vec<f64>>()
}


fn edge_orientations(rgb_image: &RgbImage) -> Vec<Axis> {

    fn axis_max(rgb_image: &RgbImage, x: u32, y: u32) -> Axis {
        let diff = brightness_change(rgb_image, x, y, Axis::X) -
                   brightness_change(rgb_image, x, y, Axis::Y);

        if diff >= 0.0 { Axis::X } else { Axis::Y }
    }

    let (width, height) = rgb_image.dimensions();
    (0..(width * height)).map(|i| axis_max(rgb_image, i % width, i / width))
                         .collect::<Vec<Axis>>()
}


pub fn non_max_suppression(rgb_image: &RgbImage) -> EdgeImage {
    let edge_image = EdgeImage::from_vec(rgb_image.width(), rgb_image.height(),
                                         edge_strengths(rgb_image)).unwrap();

    let mut supp_image = EdgeImage::new(rgb_image.width(), rgb_image.height());

    for (d, (x, y, _)) in multizip((edge_orientations(rgb_image), rgb_image.enumerate_pixels())) {
        let x_sat = Bounded::new(x, rgb_image.width());
        let y_sat = Bounded::new(y, rgb_image.height());

        let x_pixel_group = ((x_sat.sub(1))..(x_sat.add(2))).map(|i| edge_image.get_pixel(i, y)[0]);
        let y_pixel_group = ((y_sat.sub(1))..(y_sat.add(2))).map(|i| edge_image.get_pixel(x, i)[0]);
        let p = edge_image.get_pixel(x, y);

        match d {

            Axis::X => if p[0] != y_pixel_group.fold(0./0., f64::max) {
                           supp_image.put_pixel(x, y, Luma { data: [0.0] });
                       } else {
                           supp_image.put_pixel(x, y, *p);
                       },

            Axis::Y => if p[0] != x_pixel_group.fold(0./0., f64::max) {
                           supp_image.put_pixel(x, y, Luma { data: [0.0] });
                       } else {
                           supp_image.put_pixel(x, y, *p);
                       },
        }
    }

    supp_image
}


pub fn hysteresis(edge_image: &EdgeImage, threshold: f64) -> EdgeImage {
    let mut hyst_image = EdgeImage::new(edge_image.width(), edge_image.height());

    for (x, y, p) in edge_image.enumerate_pixels() {

        match p[0].partial_cmp(&threshold).unwrap() {
            Ordering::Less => hyst_image.put_pixel(x, y, Luma { data: [0.0] }),
            Ordering::Greater | Ordering::Equal => hyst_image.put_pixel(x, y, Luma {data: [1.0] }),
        }
    }

    hyst_image
}


pub fn f64_pixels_to_u8(edge_image: EdgeImage) -> image::GrayImage {
    let mut gray_image = GrayImage::new(edge_image.width(), edge_image.height());
    let scale_factor = edge_image.pixels().map(|p| p[0]).fold(0./0., f64::max);

    for (x, y, p) in edge_image.enumerate_pixels() {
        gray_image.put_pixel(x, y, Luma { data: [(255.0 * p[0] / scale_factor) as u8] });
    }

    gray_image
}


#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
use quickcheck::TestResult;

#[cfg(test)]
extern crate rand;

#[cfg(test)]
use image::GenericImage;


#[cfg(test)]
impl quickcheck::Arbitrary for Channel {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Channel {
        g.gen()
    }
}


#[cfg(test)]
impl rand::Rand for Channel {
    fn rand<R: rand::Rng>(rng: &mut R) -> Channel {
        *rng.choose(&[Channel::Red, Channel::Blue, Channel::Green]).unwrap()
    }
}


#[cfg(test)]
impl quickcheck::Arbitrary for Axis {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Axis {
        g.gen()
    }
}


#[cfg(test)]
impl rand::Rand for Axis {
    fn rand<R: rand::Rng>(rng: &mut R) -> Axis {
        *rng.choose(&[Axis::X, Axis::Y]).unwrap()
    }
}


#[cfg(test)]
quickcheck! {
    fn test_channel_change_zero_on_black_image(x: u32, y: u32,
                                               channel: Channel, axis: Axis) -> TestResult {
        let img = RgbImage::new(10, 10);

        match img.in_bounds(x, y) {
            false => TestResult::discard(),
            true => TestResult::from_bool(channel_change(&RgbImage::new(10, 10),
                                                         x, y, channel, axis) == 0.0),
        }
    }

    fn test_colour_change_zero_on_black_image(x: u32, y: u32) -> TestResult {
        let img = RgbImage::new(10, 10);

        match img.in_bounds(x, y) {
            false => TestResult::discard(),
            true => TestResult::from_bool(colour_change(&RgbImage::new(10, 10), x, y) == 0.0),
        }
    }

    fn test_brightness_change_zero_on_black_image(x: u32, y: u32, axis: Axis) -> TestResult{
        let img = RgbImage::new(10, 10);

        match img.in_bounds(x, y) {
            false => TestResult::discard(),
            true => TestResult::from_bool(brightness_change(&img, x, y, axis) == 0.0),
        }
    }

    fn test_edge_strength_zero_on_black_image(x: u32, y: u32) -> TestResult{
        let img = RgbImage::new(10, 10);

        match img.in_bounds(x, y) {
            false => TestResult::discard(),
            true => TestResult::from_bool(edge_strength(&img, x, y) == 0.0),
        }
    }
}


#[test]
fn test_channel_change_example() {
   let pixels = vec!(255, 0, 0, 255, 0, 0, 255, 0, 0,
                      0, 255, 0, 0, 255, 0, 0, 255, 0,
                      0, 0, 255, 0, 0, 255, 0, 0, 255);

    let img = RgbImage::from_vec(3, 3, pixels).unwrap();

    assert_eq!(channel_change(&img, 0, 0, Channel::Red, Axis::Y), 382.5);
    assert_eq!(channel_change(&img, 0, 0, Channel::Blue, Axis::Y), -255.0);
    assert_eq!(channel_change(&img, 2, 1, Channel::Green, Axis::X), 0.0);
}
