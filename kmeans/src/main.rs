extern crate image;
extern crate kmeans;

use image::GenericImage;
use kmeans::*;
use std::path::Path;
use std::vec::Vec;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap();

    let mut pixels = Vec::new();
    for p in img.pixels() {
        pixels.push(RGBPixel{r: p.2.data[0] as f64,
                             g: p.2.data[1] as f64,
                             b: p.2.data[2] as f64});
    }

    let mut cluster_centroids = vec![RGBPixel::zero(),
                                     RGBPixel::red(),
                                     RGBPixel::green(),
                                     RGBPixel::blue()];

    let (mut error, mut prev_error) = (0.0, -1.0);
    let mut assignments: Vec<Assignment>;

    while error != prev_error {
        prev_error = error;
        assignments = kmeans_one_iteration(&mut cluster_centroids, &pixels);
        error = get_error_metric(&cluster_centroids, &assignments);
        println!("{}", error);
    }
}
