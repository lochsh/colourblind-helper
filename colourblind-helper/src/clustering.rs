pub mod kmeans {
    use super::super::utils::Rgb;

    /// Structure for holding an RGB point's assignment to a cluster
    #[derive(Clone, Debug)]
    pub struct Assignment<'a> {
        pub pixel: &'a Rgb,
        pub cluster_ind: usize,
    }


    pub fn index_of_min_val(floats: Vec<f64>) -> Option<usize> {
        let min_val = floats.iter().cloned().fold(0./0., f64::min);
        floats.iter().position(|x| *x == min_val)
    }


    /// Assign points to clusters
    fn expectation<'a>(data: &'a [Rgb], cluster_centroids: &[Rgb]) -> Vec<Assignment<'a>> {
        data.iter()
            .map(|point| {
                let distances = cluster_centroids.iter()
                                                 .map(|clust| point.sq_euclidean_distance(clust))
                                                 .collect();
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
        assignments.iter().fold(0.0, |error, asgmnt| {
            error + asgmnt.pixel.sq_euclidean_distance(&cluster_centroids[asgmnt.cluster_ind])
        })
    }


    pub fn kmeans_one_iteration<'a>(cluster_centroids: &mut [Rgb],
                                    data: &'a [Rgb]) -> Vec<Assignment<'a>> {
        let assignments = expectation(data, cluster_centroids);
        maximisation(cluster_centroids, &assignments);
        assignments
    }
}

pub mod init {
    use super::super::utils::Rgb;
    extern crate rand;

    use self::rand::Rng;
    use self::rand::distributions::{Weighted, WeightedChoice, IndependentSample};


    pub fn compute_distances(data: &Vec<Rgb>, centroids: &Vec<Rgb>) -> Vec<f64> {
        let mut distances = Vec::<f64>::new();

        for d in data {
            distances.push(centroids.iter()
                                    .cloned()
                                    .map(|x| x.sq_euclidean_distance(d))
                                    .fold(0./0., f64::min));
        }

        distances
    }


    fn compute_weights<'a>(data: &'a Vec<Rgb>, distances: &Vec<f64>) -> Vec<Weighted<&'a Rgb>> {
        let mut weights = Vec::new();
        let factor: f64 = distances.iter()
                                   .map(|x| x.powi(2))
                                   .sum();

        for d in data.iter().zip(distances.iter()) {
            weights.push(Weighted {item: d.0,
                                   weight: (*d.1/factor * u32::max_value() as f64) as u32});
        }

        weights
    }


    pub fn choose_centres(data: &Vec<Rgb>, num_centroids: usize) -> Vec<Rgb> {
        let mut centroids = vec![*rand::thread_rng().choose(data).unwrap()];

        for _ in 0..num_centroids {
            let distances = compute_distances(&data, &centroids);

            centroids.push(*WeightedChoice::new(&mut compute_weights(&data,
                                                &distances)).ind_sample(&mut rand::thread_rng()));
        }

        centroids
    }
}


#[cfg(test)]
mod tests {
    use super::kmeans::*;
    use super::init::*;
    use super::super::utils::*;

    #[test]
    fn test_compute_distances() {
        let data = vec![Rgb {r: 0., g: 0., b: 0.}];
        let centroids = vec![Rgb {r: 0., g: 0., b: 0.},
                             Rgb {r: 10.4, g: 1., b: 4.9}];
        assert_eq!(vec![0.0], compute_distances(&data, &centroids));
    }

    #[test]
    fn test_sq_euclidean_distance_example() {
        assert_eq!(28789.16,
                   Rgb::new(10.0, 98.3, 29.1).sq_euclidean_distance(&Rgb::new(5.0, 98.3, 198.7)));
    }

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
