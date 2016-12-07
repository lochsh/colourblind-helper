extern crate image;
extern crate num;


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbPixel(pub image::Rgb<u8>);


impl RgbPixel {

    pub fn new(r: u8, g: u8, b: u8) -> RgbPixel {
        RgbPixel(image::Rgb {data: [r, g, b]})
    }

    pub fn black() -> RgbPixel {
        RgbPixel::new(0_u8, 0_u8, 0_u8)
    }

    pub fn sq_euclidean_distance(&self, other: &RgbPixel) -> f64 {
        self.0.data.iter()
                   .zip(other.0.data.iter())
                   .fold(0.0, |acc, x| acc + (*x.0 as f64).powi(2) - (*x.1 as f64).powi(2))
                   .abs()
    }
}

impl std::ops::Add for RgbPixel {
    type Output = RgbPixel;

    fn add(self, other: RgbPixel) -> RgbPixel {
        let mut sum = RgbPixel::black();

        for i in 0..3 {
            sum.0.data[i] = self.0.data[i] + other.0.data[i];
        }

        sum
    }
}

/// Structure for holding data point's assignments to clusters
#[derive(Clone, Debug)]
pub struct Assignment<'a> {
    pub pixel: &'a RgbPixel,
    pub cluster_ind: usize,
}


pub fn index_of_min_val<I>(floats: I) -> Option<usize>
    where I: IntoIterator<Item = f64>
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
fn expectation<'a>(data: &'a [RgbPixel], cluster_centroids: &[RgbPixel]) -> Vec<Assignment<'a>> {
    data.iter().map(|point| {
        let distances = cluster_centroids.iter()
                                         .map(|cluster| point.sq_euclidean_distance(cluster));
        let index = index_of_min_val(distances).expect("No min value found");
        Assignment {pixel: point, cluster_ind: index}
    }).collect()
}


pub fn points_in_cluster<'a>(assignments: &'a [Assignment], expected_cluster_ind: usize)
    -> Box<Iterator<Item = Assignment<'a>> + 'a>
{
    let i = assignments.into_iter()
        .cloned()
        .filter(move |&Assignment { cluster_ind, .. }| expected_cluster_ind == cluster_ind);
    Box::new(i)
}


pub fn count_assignments(assignments: &[Assignment], cluster_ind: usize) -> usize {
    points_in_cluster(assignments, cluster_ind).count()
}


pub fn sum_assigned_values(assignments: &[Assignment], cluster_ind: usize) -> RgbPixel {
    points_in_cluster(assignments, cluster_ind)
        .into_iter()
        .fold(RgbPixel::black(), |acc, point| acc + *point.pixel)
}


/// Update cluster centres
fn maximisation(cluster_centroids: &mut [RgbPixel], assignments: &[Assignment]) {

    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&assignments, i);
        let sum_points = sum_assigned_values(&assignments, i);
        cluster_centroids[i] = RgbPixel::new(sum_points.0.data[0] / num_points as u8,
                                             sum_points.0.data[1] / num_points as u8,
                                             sum_points.0.data[2] / num_points as u8)
    }
}

pub fn get_error_metric(cluster_centroids: &[RgbPixel], assignments: &[Assignment]) -> f64 {
    assignments.iter().fold(0.0, |error, assignment| {
        let centroid = &cluster_centroids[assignment.cluster_ind];
        error + assignment.pixel.sq_euclidean_distance(centroid)
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
        distances.push(rgb.sq_euclidean_distance(col));
    }

    colours.keys()[index_of_min_val(distances)];
}*/


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sq_euclidean_distance_simple_case() {
        let point = RgbPixel::new(1, 1, 1);
        assert_eq!(3.0, RgbPixel::black().sq_euclidean_distance(&point));
    }

    #[test]
    fn test_sq_euclidean_distance_gives_0_for_same_point() {
        let point = RgbPixel::new(200, 10, 0);
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
        let dp = RgbPixel::new(5, 5, 5);
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(RgbPixel::black(), sum_assigned_values(&assignments, 2))
    }

    #[test]
    fn test_sum_assigned_values_returns_correctly_when_some_assigned() {
        let dp = RgbPixel::new(1, 1, 1);
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(RgbPixel::new(3, 3, 3),
                   sum_assigned_values(&assignments, 0));
    }
}
