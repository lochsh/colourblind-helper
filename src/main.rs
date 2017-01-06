extern crate image;
extern crate imageproc;
extern crate colourblind_helper;

use std::path::Path;
use colourblind_helper::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap().to_rgb();
    let blur_img = imageproc::filter::gaussian_blur_f32(&img, 0.4);

    f64_pixels_to_u8(hysteresis(&non_max_suppression(&blur_img), 4000.0)).save(&args[2]).unwrap();
}

