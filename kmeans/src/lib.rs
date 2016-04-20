use std::path::Path;

extern crate csv;
extern crate rustc_serialize;

/// Store one data point's (or one cluster centroid's) x and y co-ordinates
#[derive(Clone, RustcDecodable)]
pub struct DataPoint {
    x: f64,
    y: f64,
}

/// Structure for holding data point's assignments to clusters
#[derive(Clone)]
pub struct Assignment {
    data_point: DataPoint,
    cluster_ind: usize,
}


fn read_data<P>(file_path: P) -> Vec<DataPoint> where P: AsRef<Path> {
    let mut data = vec![];
    let mut reader = csv::Reader::from_file(file_path).unwrap();
    for data_point in reader.decode() {
        let data_point: DataPoint = data_point.unwrap();
        data.push(data_point);
    }
    data
}

/*
fn read_data(file_path: &Path) -> Vec<DataPoint> {
    let lines = 
}*/

pub fn squared_euclidean_distance(point_a: &DataPoint,
                                  point_b: &DataPoint) -> f64 {
   (point_b.x - point_a.x).powi(2) + (point_b.y - point_a.y).powi(2)
}


pub fn get_index_of_min_val(floats: &Vec<f64>) -> usize {

    floats.iter()
          .enumerate()
          .fold(0,
                | min_ind, (ind, &val) |
                if val == f64::min(floats[min_ind], val) { ind }
                else { min_ind })
}

/// Assign points to clusters
fn expectation(data: Vec<DataPoint>,
               cluster_centroids: &Vec<DataPoint>) -> Vec<(Assignment)> {

    let mut distance: Vec<f64> = vec![];
    let mut assignments: Vec<(Assignment)> = vec![];
    for point in data {
        for cluster in cluster_centroids {
            distance.push(squared_euclidean_distance(&point, cluster));
        }
        assignments.push(Assignment{data_point: point,
                                    cluster_ind: get_index_of_min_val(&distance)});
    }
    assignments
}

pub fn count_assignments(assignments: &Vec<Assignment>,
                         cluster_ind: usize) -> usize {
    let points_in_cluster = get_points_in_cluster(assignments, cluster_ind);
    points_in_cluster.len()
}

pub fn get_points_in_cluster(assignments: &Vec<Assignment>,
                             cluster_ind: usize) -> Vec<Assignment> {
    let mut points_in_cluster = assignments.clone();
    points_in_cluster.retain(|&Assignment{data_point: _,
                                          cluster_ind: a}| a == cluster_ind);
    points_in_cluster
}
    
pub fn sum_assigned_values(assignments: &Vec<Assignment>,
                           cluster_ind: usize) -> DataPoint {
    let points_in_cluster = get_points_in_cluster(assignments, cluster_ind);
    let (mut x_tot, mut y_tot) = (0.0_f64, 0.0_f64);
    for point in points_in_cluster {
        x_tot += point.data_point.x;
        y_tot += point.data_point.y;
    }
    DataPoint{x: x_tot, y: y_tot}
}

/// Update cluster centres
fn maximisation(cluster_centroids: &mut Vec<DataPoint>,
                assignments: &Vec<(Assignment)>) {

    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&assignments, i);
        let sum_points = sum_assigned_values(&assignments, i);
        cluster_centroids[i] = DataPoint{
            x: sum_points.x/num_points as f64,
            y: sum_points.y/num_points as f64};
    }
}

fn get_error_metric(cluster_centroids: Vec<DataPoint>,
                    assignments: Vec<(&Assignment)>) -> f64 {
        let mut error = 0.0;
        for i in 0..assignments.len() {
            let cluster_ind = assignments[i].cluster_ind;
            error += squared_euclidean_distance(&assignments[i].data_point,
                                                &cluster_centroids[cluster_ind]);
        }
        error
    }

fn kmeans_one_iteration(mut cluster_centroids: Vec<DataPoint>,
                        data: Vec<DataPoint>, num_clusters: i32) {
    let assignments = expectation(data, &cluster_centroids);
    maximisation(&mut cluster_centroids, &assignments);
    }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squared_euclidean_distance_simple_case() {
        let origin = DataPoint{x: 0.0, y: 0.0};
        let point = DataPoint{x: 1.0, y: 1.0};
        let expected = 2.0;
        let actual = squared_euclidean_distance(&origin, &point);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_squared_euclidean_distance_gives_0_for_same_point() {
        let point_a = DataPoint{x: -999.3, y: 10.5};
        let point_b = point_a.clone();
        let expected = 0.0;
        let actual = squared_euclidean_distance(&point_a, &point_b);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_index_of_min_val() {
        let floats = vec![0.0_f64, 1.0_f64, 3.0_f64, -5.5_f64];
        let expected = 3;
        let actual = get_index_of_min_val(&floats);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_count_assignments_returns_0_when_no_occurences() {
        let dp = DataPoint{x: 0.0, y: 0.0};
        let assignments: Vec<(&Assignment)> = vec![(&dp, 0), (&dp, 0),
                                                         (&dp, 1), (&dp, 5),
                                                         (&dp, 0)];
        let val: usize = 4;
        let expected = 0;
        let actual = count_assignments(&assignments, val);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_count_assignments_returns_3_when_3_occurences() {
        let dp = DataPoint{x: 0.0, y: 0.0};
        let assignments: Vec<(&Assignment)> = vec![(&dp, 0), (&dp, 0),
                                                         (&dp, 1), (&dp, 5),
                                                         (&dp, 0)];
        let val: usize = 0;
        let expected = 3;
        let actual = count_assignments(&assignments, val);
        assert_eq!(expected, actual)
    }
}
