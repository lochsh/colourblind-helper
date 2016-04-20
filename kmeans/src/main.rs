extern crate kmeans;

use kmeans::*;

fn main() {
    let data = read_data("../../data/faithful.csv");
    let mut cluster_centroids = vec![DataPoint{x: 2.0, y: 50.0},
                                     DataPoint{x: 7.0, y: 100.0}];
    let (mut error, mut prev_error) = (0.0, -1.0);
    let mut assignments: Vec<Assignment>;
    while error != prev_error {
        prev_error = error;
        assignments = kmeans_one_iteration(&mut cluster_centroids, &data);
        error = get_error_metric(&cluster_centroids, &assignments);
        println!("{}", error);
    }
}
