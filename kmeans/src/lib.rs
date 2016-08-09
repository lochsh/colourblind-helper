use std::path::Path;

extern crate csv;
extern crate rustc_serialize;

/// Store one data point's (or one cluster centroid's) x and y co-ordinates
#[derive(Clone, Debug, RustcDecodable)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl DataPoint {
    fn zero() -> DataPoint {
        DataPoint {
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn squared_euclidean_distance(&self, other: &DataPoint) -> f64 {
        (other.x - self.x).powi(2) + (other.y - self.y).powi(2)
    }
}

impl std::ops::Add for DataPoint {
    type Output = DataPoint;

    fn add(self, other: DataPoint) -> DataPoint {
        DataPoint {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Structure for holding data point's assignments to clusters
#[derive(Clone, Debug)]
pub struct Assignment<'a> {
    data_point: &'a DataPoint,
    cluster_ind: usize,
}


pub fn read_data<P>(file_path: P) -> Vec<DataPoint>
    where P: AsRef<Path> {
    let mut reader = csv::Reader::from_file(file_path).unwrap();
    reader.decode().map(|point| point.unwrap()).collect()
}


pub fn index_of_min_val(floats: &Vec<f64>) -> Option<usize> {
    if floats.is_empty() {
        None
    }
    else {
        Some(floats.iter()
                   .enumerate()
                   .fold(0,
                         |min_ind, (ind, &val)|
                         if val == f64::min(floats[min_ind], val) { ind }
                         else { min_ind }))
    }
}


/// Assign points to clusters
fn expectation<'a>(data: &'a [DataPoint],
                   cluster_centroids: &[DataPoint]) -> Vec<Assignment<'a>>
{
    data.iter().map(|point| {
        let distances = cluster_centroids.iter()
                                         .map(|cluster| point.squared_euclidean_distance(cluster));
        let index = index_of_min_val(distances).expect("No minimum value found");
        Assignment { data_point: point, cluster_ind: index }
    }).collect()
}

pub fn count_assignments(assignments: &Vec<Assignment>,
                         cluster_ind: usize) -> usize {
    points_in_cluster(assignments, cluster_ind).count()
}

pub fn points_in_cluster<'a>(assignments: &'a Vec<Assignment>,
                                 cluster_ind: usize) -> Vec<Assignment<'a>> {
    let mut points_in_cluster = assignments.clone();
    points_in_cluster.retain(|&Assignment{data_point: _,
                                          cluster_ind: a}| a == cluster_ind);
    points_in_cluster
}
    
pub fn sum_assigned_values(assignments: &Vec<Assignment>,
                           cluster_ind: usize) -> DataPoint
{
    points_in_cluster(assignments, cluster_ind)
        .into_iter()
        .fold(DataPoint::zero(), |acc, point| acc + *point.data_point)
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

pub fn get_error_metric(cluster_centroids: &Vec<DataPoint>,
                        assignments: &Vec<Assignment>) -> f64 {
        assignments.iter()
                   .fold(0.0, |error, assignment| {
                       let centroid = &cluster_centroids[assignment.cluster_ind];
                       error + assignment.data_point.squared_euclidean_distance(centroid)
                   })
}

pub fn kmeans_one_iteration<'a>(cluster_centroids: &mut Vec<DataPoint>,
                                data: &'a Vec<DataPoint>) -> Vec<Assignment<'a>> {
    let assignments = expectation(data, cluster_centroids);
    maximisation(cluster_centroids, &assignments);
    assignments
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
    fn test_index_of_min_val() {
        let floats = vec![0.0_f64, 1.0_f64, 3.0_f64, -5.5_f64];
        let expected = 3;
        let actual = index_of_min_val(&floats);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_count_assignments_returns_0_when_no_occurences() {
        let dp = DataPoint{x: 0.0, y: 0.0};
        let assignments = vec![Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 1},
                               Assignment{data_point: &dp, cluster_ind: 5},
                               Assignment{data_point: &dp, cluster_ind: 0}];
        let cluster_ind: usize = 4;
        let expected = 0;
        let actual = count_assignments(&assignments, cluster_ind);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_count_assignments_returns_3_when_3_occurences() {
        let dp = DataPoint{x: 0.0, y: 0.0};
        let assignments = vec![Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 1},
                               Assignment{data_point: &dp, cluster_ind: 5},
                               Assignment{data_point: &dp, cluster_ind: 0}];
        let cluster_ind: usize = 0;
        let expected = 3;
        let actual = count_assignments(&assignments, cluster_ind);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_sum_assigned_values_returns_0_when_none_assigned() {
        let dp = DataPoint{x: 5.0, y: 5.0};
        let assignments = vec![Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 1},
                               Assignment{data_point: &dp, cluster_ind: 5},
                               Assignment{data_point: &dp, cluster_ind: 0}];
        let cluster_ind: usize = 2;
        let expected = DataPoint{x: 0.0, y: 0.0};
        let actual = sum_assigned_values(&assignments, cluster_ind);
        assert_eq!(expected.x, actual.x);
        assert_eq!(expected.y, actual.y)
    }

    #[test]
    fn test_sum_assigned_values_returns_correctly_when_some_assigned() {
        let dp = DataPoint{x: 1.0, y: 1.0};
        let assignments = vec![Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 0},
                               Assignment{data_point: &dp, cluster_ind: 1},
                               Assignment{data_point: &dp, cluster_ind: 5},
                               Assignment{data_point: &dp, cluster_ind: 0}];
        let cluster_ind: usize = 0;
        let expected = DataPoint{x: 3.0, y: 3.0};
        let actual = sum_assigned_values(&assignments, cluster_ind);
        assert_eq!(expected.x, actual.x);
        assert_eq!(expected.y, actual.y)
    }
}
