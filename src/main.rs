extern crate image;
extern crate colourblind_helper;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap().to_rgb();

    println!("{}", colourblind_helper::edge_strength(img, 2_u32, 2_u32));
}

