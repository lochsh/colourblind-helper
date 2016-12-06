extern crate image;
extern crate colourblind_helper;

use image::GenericImage;
use colourblind_helper::*;
use std::path::Path;
use std::vec::Vec;


fn write_assignments_to_file(assignments: Vec<Assignment>,
                             cluster_centroids: Vec<RgbPixel>,
                             width: u32, height: u32)
{
   let mut img_out = image::RgbImage::new(width, height);

   for (a, i) in assignments.iter().zip(0..width*height) {
       let rgb = cluster_centroids[a.cluster_ind];
       img_out.put_pixel(i % width, i / width,
                         image::Rgb{data: [rgb.r as u8,
                                           rgb.g as u8,
                                           rgb.b as u8]});
   }

   img_out.save("test.jpg").unwrap();
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap();

    let mut pixels = Vec::new();
    for p in img.pixels() {
        pixels.push(RgbPixel{r: p.2.data[0] as f64,
                             g: p.2.data[1] as f64,
                             b: p.2.data[2] as f64});
    }

    let mut cluster_centroids = vec![RgbPixel::black(),
                                     RgbPixel::red(),
                                     RgbPixel::green(),
                                     RgbPixel::blue(),
                                     RgbPixel::white(),
                                     RgbPixel{r: 200.0, g: 0.0, b: 200.0}];

    let (mut error, mut prev_error) = (0.0, -1.0);
    let init_pixel = RgbPixel::black();
    let mut assignments: Vec<Assignment> = vec![Assignment{pixel: &init_pixel,
                                                           cluster_ind: 0}];

    while error != prev_error {
        prev_error = error;
        assignments = kmeans_one_iteration(&mut cluster_centroids, &pixels);
        error = get_error_metric(&cluster_centroids, &assignments);
    }

   let (width, height) = img.dimensions();
   write_assignments_to_file(assignments, cluster_centroids, width, height);
}
