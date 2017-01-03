extern crate image;
use image::RgbImage;

extern crate itertools;
use itertools::Itertools;


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


/// Bound image co-ordinate
///
/// Bounded image co-ordinate, e.g. x, with a maximum value.
/// Used to prevent out-of-bounds access attempts and overflow +/- attempts.
#[derive(Copy, Clone)]
struct ImageCoord {
    i: u32,
    max_i: u32,
}


impl ImageCoord {

    fn new(i: u32, max_i: u32) -> ImageCoord {
        if i < max_i {
            ImageCoord { i: i, max_i: max_i - 1}
        } else {
            ImageCoord { i: max_i - 1, max_i: max_i - 1}
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

    let x_sat = ImageCoord::new(x, rgb_image.width());
    let y_sat = ImageCoord::new(y, rgb_image.height());

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


pub fn edge_strength(rgb_image: RgbImage, x: u32, y: u32) -> f64 {
    brightness_change(&rgb_image, x, y, Axis::X).powi(2) +
    brightness_change(&rgb_image, x, y, Axis::Y).powi(2) +
    colour_change(&rgb_image, x, y) * 3.0
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

}



#[test]
fn test_brightness_change_zero_on_black_image() {
    let img = RgbImage::new(10, 10);
    assert_eq!(brightness_change(&img, 4, 4, Axis::X), 0.0);
}


#[test]
fn test_edge_strength_zero_on_black_image() {
    let img = RgbImage::new(10, 10);
    assert_eq!(edge_strength(img, 4, 4), 0.0);
}
