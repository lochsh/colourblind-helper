extern crate image;
extern crate colourblind_helper;

use image::GenericImage;
use colourblind_helper::*;

use std::path::Path;
use std::vec::Vec;


fn rgb_scale(rgb: Rgb) -> image::Rgb<u8> {
    image::Rgb { data: [(rgb.r) as u8, (rgb.g) as u8, (rgb.b) as u8] }
}


fn output_image(assignments: Vec<Assignment>,
                cluster_centroids: Vec<Rgb>,
                width: u32,
                height: u32) {
    let mut img_out = image::RgbImage::new(width, height);

    for (a, i) in assignments.iter().zip(0..width * height) {
        img_out.put_pixel(i % width,
                          i / width,
                          rgb_scale(cluster_centroids[a.cluster_ind]));
    }

    img_out.save("test.jpg").unwrap();
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let img = image::open(&Path::new(&args[1])).unwrap();

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
