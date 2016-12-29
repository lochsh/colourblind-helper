extern crate std;
extern crate image;

use self::image::GenericImage;
use super::clustering::kmeans::*;
use super::clustering::init::choose_centres;
use std::path::Path;


/// Struct to hold floating point colour channel values, for use in calculations
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}


impl Rgb {
    pub fn new(r: f64, g: f64, b: f64) -> Rgb {
        Rgb { r: r, g: g, b: b }
    }

    pub fn black() -> Rgb {
        Rgb::new(0.0, 0.0, 0.0)
    }

    pub fn sq_euclidean_distance(&self, other: &Rgb) -> f64 {
        ((self.r - other.r).powi(2) +
         (self.g - other.g).powi(2) +
         (self.b - other.b).powi(2)).abs()
    }
}


impl std::ops::Add for Rgb {
    type Output = Rgb;

    fn add(self, other: Rgb) -> Rgb {
        Rgb::new(self.r + other.r,
                 self.g + other.g,
                 self.b + other.b)
    }
}


fn rgb_scale(rgb: Rgb) -> image::Rgb<u8> {
    image::Rgb { data: [(rgb.r) as u8, (rgb.g) as u8, (rgb.b) as u8] }
}


fn output_image(assignments: Vec<Assignment>, cluster_centroids: Vec<Rgb>,
                width: u32, height: u32) {
    let mut img_out = image::RgbImage::new(width, height);

    for (a, i) in assignments.iter().zip(0..width * height) {
        img_out.put_pixel(i % width,
                          i / width,
                          rgb_scale(cluster_centroids[a.cluster_ind]));
    }

    img_out.save("test.jpg").unwrap();
}


pub fn cluster_image(input_image: &String, num_clusters: u32) {
    let img = image::open(&Path::new(input_image)).unwrap();

    let mut pixels = Vec::new();
    for p in img.pixels() {
        pixels.push(Rgb::new(p.2.data[0] as f64, p.2.data[1] as f64, p.2.data[2] as f64));
    }

    let mut cluster_centroids = choose_centres(&pixels, 10);

    let (mut error, mut prev_error) = (0.0, -1.0);
    let init_pixel = Rgb::black();
    let mut assignments: Vec<Assignment> = vec![Assignment {
                                                    pixel: &init_pixel,
                                                    cluster_ind: 0,
                                                }];

    while error != prev_error {
        prev_error = error;
        assignments = kmeans_one_iteration(&mut cluster_centroids, &pixels);
        error = get_error_metric(&cluster_centroids, &assignments);
        println!("{}", error);
    }

    let (width, height) = img.dimensions();
    output_image(assignments, cluster_centroids, width, height);
}
