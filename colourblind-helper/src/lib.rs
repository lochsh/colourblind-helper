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


/// Structure for holding an RGB point's assignment to a cluster
#[derive(Clone, Debug)]
pub struct Assignment<'a> {
    pub pixel: &'a Rgb,
    pub cluster_ind: usize,
}


pub fn index_of_min_val<I>(floats: I) -> Option<usize> where I: IntoIterator<Item = f64> {
    let mut iter = floats.into_iter()
                         .enumerate();

    let fold_func = |(min_i, min_val), (i, val)| {
                        if val < min_val { (i, val) }
                        else { (min_i, min_val) }
    };

    iter.next().map(|(i, min)| {
        iter.fold((i, min), fold_func).0})
}


/// Assign points to clusters
fn expectation<'a>(data: &'a [Rgb], cluster_centroids: &[Rgb]) -> Vec<Assignment<'a>> {
    data.iter()
        .map(|point| {
            let distances = cluster_centroids.iter()
                                             .map(|cluster| point.sq_euclidean_distance(cluster));
            Assignment {
                pixel: point,
                cluster_ind: index_of_min_val(distances).expect("No min value found"),
            }
        })
        .collect()
}


pub fn points_in_cluster<'a>(assignments: &'a [Assignment],
                             c_ind: usize) -> Box<Iterator<Item = Assignment<'a>> + 'a> {
    let i = assignments.into_iter()
        .cloned()
        .filter(move |&Assignment { cluster_ind, .. }| cluster_ind == c_ind);
    Box::new(i)
}


pub fn count_assignments(assignments: &[Assignment], cluster_ind: usize) -> usize {
    points_in_cluster(assignments, cluster_ind).count()
}


pub fn sum_assigned_values(assignments: &[Assignment], cluster_ind: usize) -> Rgb {
    points_in_cluster(assignments, cluster_ind)
        .into_iter()
        .fold(Rgb::black(), |acc, a| acc + *a.pixel)
}


/// Update cluster centres
fn maximisation(cluster_centroids: &mut [Rgb], assignments: &[Assignment]) {

    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&assignments, i);
        let sum_points = sum_assigned_values(&assignments, i);

        cluster_centroids[i] = Rgb::new(sum_points.r / num_points as f64,
                                        sum_points.g / num_points as f64,
                                        sum_points.b / num_points as f64)
    }
}


pub fn get_error_metric(cluster_centroids: &[Rgb], assignments: &[Assignment]) -> f64 {
    assignments.iter().fold(0.0, |error, assignment| {
        error + assignment.pixel.sq_euclidean_distance(&cluster_centroids[assignment.cluster_ind])
    })
}


pub fn kmeans_one_iteration<'a>(cluster_centroids: &mut [Rgb],
                                data: &'a [Rgb]) -> Vec<Assignment<'a>> {
    let assignments = expectation(data, cluster_centroids);
    maximisation(cluster_centroids, &assignments);
    assignments
}

// fn rgb_to_colour_name(rgb: Rgb, colours: HashMap<Rgb, String>) -> String {
// let mut distances = Vec::new();
//
// for col in colours.keys() {
// distances.push(rgb.sq_euclidean_distance(col));
// }
//
// colours.keys()[index_of_min_val(distances)];
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sq_euclidean_distance_example() {}

    #[test]
    fn test_sq_euclidean_distance_simple_case() {
        let point = Rgb::new(1.0, 1.0, 1.0);
        assert_eq!(3.0, Rgb::black().sq_euclidean_distance(&point));
    }

    #[test]
    fn test_sq_euclidean_distance_gives_0_for_same_point() {
        let point = Rgb::new(200.0, 10.0, 0.0);
        assert_eq!(0.0, point.sq_euclidean_distance(&point));
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
        let dp = Rgb::black();
        let assignments = [Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 1,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 5,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           }];
        assert_eq!(0, count_assignments(&assignments, 4))
    }

    #[test]
    fn test_count_assignments_returns_3_when_3_occurences() {
        let dp = Rgb::black();
        let assignments = [Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 1,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 5,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           }];
        assert_eq!(3, count_assignments(&assignments, 0));
    }

    #[test]
    fn test_sum_assigned_values_returns_0_when_none_assigned() {
        let dp = Rgb::new(5.0, 5.0, 5.0);
        let assignments = [Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 1,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 5,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           }];
        assert_eq!(Rgb::black(), sum_assigned_values(&assignments, 2))
    }

    #[test]
    fn test_sum_assigned_values_returns_correctly_when_some_assigned() {
        let dp = Rgb::new(1.0, 1.0, 1.0);
        let assignments = [Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 1,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 5,
                           },
                           Assignment {
                               pixel: &dp,
                               cluster_ind: 0,
                           }];
        assert_eq!(Rgb::new(3.0, 3.0, 3.0),
                   sum_assigned_values(&assignments, 0));
    }
}
