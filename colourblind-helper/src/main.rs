extern crate image;
extern crate colourblind_helper;

use image::GenericImage;
use colourblind_helper::*;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;


fn read_lines<P>(file_path: P) -> Vec<String> where P: AsRef<Path> {
    let file_path = file_path.as_ref();

    let file = match File::open(file_path) {
        Err(why) => panic!("Couldn't open file {}: {}",
                           file_path.display(), why.description()),
        Ok(file) => file,
    };

    BufReader::new(file).lines().map(|line| {
        match line {
            Ok(l) => l,
            Err(why) => panic!("Couldn't read file {}: {}",
                               file_path.display(), why.description()),
        }
    }).collect()
}


fn write_assignments_to_file(assignments: Vec<Assignment>, cluster_centroids: Vec<RgbPixel>,
                             width: u32, height: u32)
{
   let mut img_out = image::RgbImage::new(width, height);

   for (a, i) in assignments.iter().zip(0..width*height) {
       img_out.put_pixel(i % width, i / width, scale_to_255(cluster_centroids[a.cluster_ind].0));
   }

   img_out.save("test.jpg").unwrap();
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap();

    let mut pixels = Vec::new();
    for p in img.pixels() {
        pixels.push(RgbPixel::new(p.2.data[0] as f64, p.2.data[1] as f64, p.2.data[2] as f64));
    }

    let mut cluster_centroids = vec![RgbPixel::black(),
                                     RgbPixel::new(255.0, 0.0, 0.0),
                                     RgbPixel::new(0.0, 255.0, 0.0),
                                     RgbPixel::new(0.0, 0.0, 255.0),
                                     RgbPixel::new(0.0, 0.0, 0.0),
                                     RgbPixel::new(255.0, 0.0, 255.0)];;

    let (mut error, mut prev_error) = (0.0, -1.0);
    let init_pixel = RgbPixel::black();
    let mut assignments: Vec<Assignment> = vec![Assignment{pixel: &init_pixel, cluster_ind: 0}];

    while error != prev_error {
        prev_error = error;
        assignments = kmeans_one_iteration(&mut cluster_centroids, &pixels);
        error = get_error_metric(&cluster_centroids, &assignments);
        println!("{}", error);
    }

   let (width, height) = img.dimensions();
   write_assignments_to_file(assignments, cluster_centroids, width, height);
}
