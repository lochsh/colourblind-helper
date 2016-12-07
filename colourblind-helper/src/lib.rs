extern crate image;
extern crate num;


pub trait ColourVal: image::Primitive + num::Float {}
impl<T> ColourVal for T where T: image::Primitive + num::Float {}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbPixel<T>(pub image::Rgb<T>) where T: ColourVal;


impl <T> RgbPixel<T> where T: ColourVal {

    pub fn new(r: T, g: T, b: T) -> RgbPixel<T> where T: ColourVal {
        RgbPixel(image::Rgb {data: [r, g, b]})
    }

    pub fn black() -> RgbPixel<T> where T: ColourVal {
        RgbPixel::new(T::zero(), T::zero(), T::zero())
    }

    pub fn sq_euclidean_distance(&self, other: &RgbPixel<T>) -> T
        where T: ColourVal
    {
        self.0.data.iter()
                   .zip(other.0.data.iter())
                   .fold(T::zero(), |acc, x| acc + x.0.powi(2) + x.1.powi(2))
    }

    pub fn as_u8(&self) -> image::Rgb<u8> {
        image::Rgb {data: [self.0.data[0] as u8,
                           self.0.data[1] as u8,
                           self.0.data[2] as u8]}
    }
}

impl <T> std::ops::Add for RgbPixel<T> where T: ColourVal {
    type Output = RgbPixel<T>;

    fn add(self, other: RgbPixel<T>) -> RgbPixel<T> where T: ColourVal {
        let mut sum = RgbPixel::black();

        for i in 0..3 {
            sum.0.data[i] = self.0.data[i] + other.0.data[i];
        }

        sum
    }
}

/// Structure for holding data point's assignments to clusters
#[derive(Clone, Debug)]
pub struct Assignment<'a, T> where T: ColourVal + 'a {
    pub pixel: &'a RgbPixel<T>,
    pub cluster_ind: usize,
}


pub fn index_of_min_val<I, T>(floats: I) -> Option<usize>
    where I: IntoIterator<Item = T>, T: ColourVal,
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
fn expectation<'a, T>(data: &'a [RgbPixel<T>],
                      cluster_centroids: &[RgbPixel<T>]) -> Vec<Assignment<'a, T>>
    where T: ColourVal
{
    data.iter().map(|point| {
        let distances = cluster_centroids.iter()
                                         .map(|cluster| point.sq_euclidean_distance(cluster));
        let index = index_of_min_val(distances).expect("No min value found");
        Assignment {pixel: point, cluster_ind: index}
    }).collect()
}


pub fn points_in_cluster<'a, T>(assignments: &'a [Assignment<T>], expected_cluster_ind: usize)
    -> Box<Iterator<Item = Assignment<'a, T>> + 'a> where T: ColourVal
{
    let i = assignments.into_iter()
        .cloned()
        .filter(move |&Assignment { cluster_ind, .. }| expected_cluster_ind == cluster_ind);
    Box::new(i)
}


pub fn count_assignments<T>(assignments: &[Assignment<T>],
                            cluster_ind: usize) -> usize where T: ColourVal {
    points_in_cluster(assignments, cluster_ind).count()
}


pub fn sum_assigned_values<T>(assignments: &[Assignment<T>],
                              cluster_ind: usize) -> RgbPixel<T>
    where T: ColourVal
{
    points_in_cluster(assignments, cluster_ind)
        .into_iter()
        .fold(RgbPixel::black(), |acc, point| acc + *point.pixel)
}


/// Update cluster centres
fn maximisation<T>(cluster_centroids: &mut [RgbPixel<T>],
                   assignments: &[Assignment<T>]) where T: ColourVal {

    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&assignments, i);
        let sum_points = sum_assigned_values(&assignments, i);
        cluster_centroids[i] = RgbPixel::new(sum_points.0.data[0] / T::from(num_points).unwrap(),
                                             sum_points.0.data[1] / T::from(num_points).unwrap(),
                                             sum_points.0.data[2] / T::from(num_points).unwrap())
    }
}

pub fn get_error_metric<T>(cluster_centroids: &[RgbPixel<T>],
                           assignments: &[Assignment<T>]) -> T where T: ColourVal
{
    assignments.iter().fold(T::zero(), |error, assignment| {
        let centroid = &cluster_centroids[assignment.cluster_ind];
        error + assignment.pixel.sq_euclidean_distance(centroid)
    })
}

pub fn kmeans_one_iteration<'a, T>(cluster_centroids: &mut [RgbPixel<T>],
                                   data: &'a [RgbPixel<T>])
    -> Vec<Assignment<'a, T>> where T: ColourVal {
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
        let point = RgbPixel::new([1.0; 3]);
        assert_eq!(3.0, RgbPixel::black().sq_euclidean_distance(&point));
    }

    #[test]
    fn test_sq_euclidean_distance_gives_0_for_same_point() {
        let point = RgbPixel::new([-999.3, 10.5, 0.15]);
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
        let dp = RgbPixel::new([5.0; 3]);
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(RgbPixel::black(), sum_assigned_values(&assignments, 2))
    }

    #[test]
    fn test_sum_assigned_values_returns_correctly_when_some_assigned() {
        let dp = RgbPixel::new([1.0; 3]);
        let assignments = [Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 0 },
                           Assignment { pixel: &dp, cluster_ind: 1 },
                           Assignment { pixel: &dp, cluster_ind: 5 },
                           Assignment { pixel: &dp, cluster_ind: 0 }];
        assert_eq!(RgbPixel::new([3.0; 3]),
                   sum_assigned_values(&assignments, 0));
    }
}
