use std::collections::HashMap;
use std::path::Path;

extern crate csv;
extern crate rustc_serialize;

/// Store one RGB pixel's colour channel values
#[derive(Clone, Copy, Debug, PartialEq, RustcDecodable)]
pub struct RgbPixel {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}


impl RgbPixel {
    pub fn black() -> RgbPixel {
        RgbPixel {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn white() -> RgbPixel {
        RgbPixel {
            r: 255.0,
            g: 255.0,
            b: 255.0,
        }
    }

    pub fn red() -> RgbPixel {
        RgbPixel {
            r: 255.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn green() -> RgbPixel {
        RgbPixel {
            r: 0.0,
            g: 255.0,
            b: 0.0,
        }
    }

    pub fn blue() -> RgbPixel {
        RgbPixel {
            r: 0.0,
            g: 0.0,
            b: 255.0,
        }
    }

    pub fn squared_euclidean_distance(&self, other: &RgbPixel) -> f64 {
        (other.r - self.r).powi(2) + (other.g - self.g).powi(2) +
        (other.b - self.b).powi(2)
    }
}

impl std::ops::Add for RgbPixel {
    type Output = RgbPixel;

    fn add(self, other: RgbPixel) -> RgbPixel {
        RgbPixel {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

/// Structure for holding data point's assignments to clusters
#[derive(Clone, Debug)]
pub struct Assignment<'a> {
    pub pixel: &'a RgbPixel,
    pub cluster_ind: usize,
}


pub fn read_data<P>(file_path: P) -> Vec<RgbPixel>
    where P: AsRef<Path>
{
    let mut reader = csv::Reader::from_file(file_path).unwrap();
    reader.decode().map(|point| point.unwrap()).collect()
}


pub fn index_of_min_val<I>(floats: I) -> Option<usize>
    where I: IntoIterator<Item = f64>,
{
    let mut iter = floats.into_iter()
                         .enumerate();

    iter.next()
        .map(|(i, min)| {
            iter.fold((i, min), |(min_i, min_val), (i, val)| {
                if val < min_val { (i, val) }
                else { (min_i, min_val) }
            }).0
        })
}


/// Assign points to clusters
fn expectation<'a>(data: &'a [RgbPixel],
                   cluster_centroids: &[RgbPixel]) -> Vec<Assignment<'a>>
{
    data.iter().map(|point| {
        let distances = cluster_centroids.iter()
                                         .map(|cluster| point.squared_euclidean_distance(cluster));
        let index = index_of_min_val(distances).expect("No minimum value found");
        Assignment { pixel: point, cluster_ind: index }
    }).collect()
}


pub fn points_in_cluster<'a>(assignments: &'a [Assignment],
                             expected_cluster_ind: usize) -> Box<Iterator<Item = Assignment<'a>> + 'a>
{
    let i = assignments.into_iter()
        .cloned()
        .filter(move |&Assignment { cluster_ind, .. }| expected_cluster_ind == cluster_ind);
    Box::new(i)
}


pub fn count_assignments(assignments: &[Assignment],
                         cluster_ind: usize) -> usize {
    points_in_cluster(assignments, cluster_ind).count()
}


pub fn sum_assigned_values(assignments: &[Assignment],
                           cluster_ind: usize) -> RgbPixel
{
    points_in_cluster(assignments, cluster_ind)
        .into_iter()
        .fold(RgbPixel::black(), |acc, point| acc + *point.pixel)
}


/// Update cluster centres
fn maximisation(cluster_centroids: &mut [RgbPixel],
                assignments: &[Assignment]) {

    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&assignments, i);
        let sum_points = sum_assigned_values(&assignments, i);
        cluster_centroids[i] = RgbPixel{
            r: sum_points.r/num_points as f64,
            g: sum_points.g/num_points as f64,
            b: sum_points.b/num_points as f64};
    }
}

pub fn get_error_metric(cluster_centroids: &[RgbPixel],
                        assignments: &[Assignment]) -> f64
{
    assignments.iter().fold(0.0, |error, assignment| {
        let centroid = &cluster_centroids[assignment.cluster_ind];
        error + assignment.pixel.squared_euclidean_distance(centroid)
    })
}

pub fn kmeans_one_iteration<'a>(cluster_centroids: &mut [RgbPixel],
                                data: &'a [RgbPixel]) -> Vec<Assignment<'a>> {
    let assignments = expectation(data, cluster_centroids);
    maximisation(cluster_centroids, &assignments);
    assignments
}

/*
fn rgb_to_colour_name(rgb: RgbPixel, colours: HashMap<RgbPixel, String>) -> String {
    let mut distances = Vec::new();

    for col in colours.keys() {
        distances.push(rgb.squared_euclidean_distance(col));
    }

    colours.keys()[index_of_min_val(distances)];
}*/


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squared_euclidean_distance_simple_case() {
        let point = RgbPixel { r: 1.0, g: 1.0, b: 1.0};
        assert_eq!(3.0, RgbPixel::black().squared_euclidean_distance(&point));
    }

    #[test]
    fn test_squared_euclidean_distance_gives_0_for_same_point() {
        let point = RgbPixel { r: -999.3, g: 10.5, b: 0.15};
        assert_eq!(0.0, point.squared_euclidean_distance(&point));
    }

    #[test]
    fn test_index_of_min_val_end() {
        let floats = vec![0.0_f64, 1.0_f64, 3.0_f64, -5.5_f64];
        assert_eq!(Some(3), index_of_min_val(floats))
    }

    #[test]
    fn test_index_of_min_val_start() {
        let floats = vec![-7.0_f64, 1.0_f64, 3.0_f64, -5.5_f64];
        assert_eq!(Some(0), index_of_min_val(floats))
    }

    #[test]
    fn test_index_of_min_val_middle() {
        let floats = vec![2.0_f64, 1.0_f64, 0.1_f64, 5.5_f64];
        assert_eq!(Some(2), index_of_min_val(floats))
    }

    #[test]
    fn test_count_assignments_returns_0_when_no_occurences() {
        let dp = RgbPixel::black();
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(0, count_assignments(&assignments, 4))
    }

    #[test]
    fn test_count_assignments_returns_3_when_3_occurences() {
        let dp = RgbPixel::black();
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(3, count_assignments(&assignments, 0));
    }

    #[test]
    fn test_sum_assigned_values_returns_0_when_none_assigned() {
        let dp = RgbPixel { r: 5.0, g: 5.0, b: 5.0};
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(RgbPixel::black(), sum_assigned_values(&assignments, 2))
    }

    #[test]
    fn test_sum_assigned_values_returns_correctly_when_some_assigned() {
        let dp = RgbPixel { r: 1.0, g: 1.0, b: 1.0};
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(RgbPixel{r: 3.0, g: 3.0, b: 3.0},
                   sum_assigned_values(&assignments, 0));
    }
}
