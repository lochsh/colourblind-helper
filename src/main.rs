extern crate image;
extern crate colourblind_helper;
use std::path::Path;
use colourblind_helper::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap().to_rgb();

    f64_pixels_to_u8(non_max_suppression(&img)).save("test.jpg").unwrap();
}

