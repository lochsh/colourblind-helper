extern crate image;
use image::{RgbImage, Luma, ImageBuffer, GrayImage};

extern crate itertools;
use itertools::{Itertools, multizip};


type EdgeImage = ImageBuffer<Luma<f64>, Vec<f64>>;


pub fn f64_pixels_to_u8(edge_image: EdgeImage) -> image::GrayImage {
    let mut gray_image = GrayImage::new(edge_image.width(), edge_image.height());
    for (x, y, p) in edge_image.enumerate_pixels() {
        gray_image.put_pixel(x, y, Luma { data: [(p[0] / 255.0) as u8] });
    }

    gray_image
}


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
        if i <= max_i {
            Bounded { i: i, max_i: max_i }
        } else {
            Bounded { i: max_i, max_i: max_i }
        }
    }

    fn sub(self, other: u32) -> u32 {
        self.i.saturating_sub(other)
    }

    fn add(self, other: u32) -> u32 {
        if self.i + other <= self.max_i {
            self.i + other
        } else {
            self.max_i
        }
    }
}


fn channel_change(rgb_image: &RgbImage, x: u32, y: u32, channel: Channel, axis: Axis) -> f64 {
    let c: usize;

    match channel {
        Channel::Red => c = 0,
        Channel::Blue => c = 1,
        Channel::Green => c = 2,
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
        let diff = brightness_change(rgb_image, x, y, Axis::X).abs() -
                   brightness_change(rgb_image, x, y, Axis::Y).abs();

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

        let x_pixel_group = ((x_sat.sub(1))..(x_sat.add(2))).map(|i| edge_image.get_pixel(i, y)[0].abs());
        let y_pixel_group = ((y_sat.sub(1))..(y_sat.add(2))).map(|i| edge_image.get_pixel(x, i)[0].abs());

        match d {
            Axis::X => if edge_image.get_pixel(x, y)[0].abs() != x_pixel_group.fold(0./0.,
                                                                                    f64::max) {
                            supp_image.put_pixel(x, y, Luma { data: [0.0] });
                       } else { },

            Axis::Y => if edge_image.get_pixel(x, y)[0].abs() != y_pixel_group.fold(0./0.,
                                                                                    f64::max) {
                           supp_image.put_pixel(x, y, Luma { data: [0.0] });
                       } else { }
        }
    }

    supp_image
}


#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
use quickcheck::TestResult;

#[cfg(test)]
extern crate rand;


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

        if x >= img.width() || y >= img.height() {
            TestResult::discard()
        } else {
            TestResult::from_bool(channel_change(&RgbImage::new(10, 10),
                                                 x, y, channel, axis) == 0.0)
        }
    }

    fn test_colour_change_zero_on_black_image(x: u32, y: u32) -> TestResult {
        let img = RgbImage::new(10, 10);

        if x >= img.width() || y >= img.height() {
            TestResult::discard()
        } else {
            TestResult::from_bool(colour_change(&RgbImage::new(10, 10), x, y) == 0.0)
        }
    }

    fn test_brightness_change_zero_on_black_image(x: u32, y: u32, axis: Axis) -> TestResult{
        let img = RgbImage::new(10, 10);

        if x >= img.width() || y >= img.height() {
            TestResult::discard()
        } else {
            TestResult::from_bool(brightness_change(&img, x, y, axis) == 0.0)
        }
    }

    fn test_edge_strength_zero_on_black_image(x: u32, y: u32) -> TestResult{
        let img = RgbImage::new(10, 10);

        if x >= img.width() || y >= img.height() {
            TestResult::discard()
        } else {
            TestResult::from_bool(edge_strength(img, x, y) == 0.0)
        }
    }

}
