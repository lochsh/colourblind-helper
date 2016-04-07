use std::path::Path;

/// Store one data point's (or one cluster centroid's) x and y co-ordinates
struct DataPoint {
    x: f64,
    y: f64,
}

fn read_data(file_path: &Path) -> Vec<DataPoint> {
}

fn squared_euclidean_distance(point_1: Vec<f64>, point_2: Vec<f64>) -> f64 {
}

/// Assign points to clusters
fn expectation(data: Vec<DataPoint>) -> Vec<i32> {
}

/// Update cluster centres
fn maximisation(cluster_centroids: &mut Vec<DataPoint>) -> Vec<DataPoint> {
}
