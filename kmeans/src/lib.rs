/// Store one data point's (or one cluster centroid's) x and y co-ordinates
#[derive(Clone)]
pub struct DataPoint {
    x: f64,
    y: f64,
}

/*
fn read_data(file_path: &Path) -> Vec<DataPoint> {
}*/

pub fn squared_euclidean_distance(point_a: &DataPoint, point_b: &DataPoint) -> f64 {
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
fn expectation(data: Vec<&DataPoint>,
                   cluster_centroids: Vec<DataPoint>)
                   -> Vec<(&DataPoint, usize)> {

    let mut distance: Vec<f64> = vec![];
    let mut cluster_assignments: Vec<(&DataPoint, usize)> = vec![];
    for point in data {
        for cluster in &cluster_centroids {
            distance.push(squared_euclidean_distance(point, cluster));
        }

        // Index of cluster centroid that point is nearest to
        cluster_assignments.push((point, get_index_of_min_val(&distance)));
    }
    cluster_assignments
}

pub fn count_assignments(assignments: &Vec<(&DataPoint, usize)>,
                         cluster_index: usize) -> usize {
    let mut assignments_copy = assignments.clone();
    assignments_copy.retain(|&(_, a)| a == cluster_index);
    assignments_copy.len()
}

pub fn sum_assigned_values(assignments: &Vec<(&DataPoint, usize)>,
                           cluster_index: usize) -> DataPoint {
    let mut assignments_copy = assignments.clone();
    assignments_copy.retain(|&(_, a)| a == cluster_index);
    let (mut x_tot, mut y_tot) = (0.0_f64, 0.0_f64);
    for (dp, a) in assignments_copy {
        x_tot += dp.x;
        y_tot += dp.y;
    }
    DataPoint{x: x_tot, y: y_tot}
}

/// Update cluster centres
fn maximisation(cluster_centroids: &mut Vec<DataPoint>,
                cluster_assignments: Vec<(&DataPoint, usize)>) {

    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&cluster_assignments, i);
        let sum_points = sum_assigned_values(&cluster_assignments, i);
        cluster_centroids[i] = DataPoint{x: sum_points.x/num_points as f64,
                                         y: sum_points.y/num_points as f64};
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squared_euclidean_distance_simple_case() {
        let origin = DataPoint { x: 0.0, y: 0.0};
        let point = DataPoint {x: 1.0, y: 1.0};
        let expected = 2.0;
        let actual = squared_euclidean_distance(&origin, &point);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_squared_euclidean_distance_gives_0_for_same_point() {
        let point_a = DataPoint { x: -999.3, y: 10.5};
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
        let assignments: Vec<(&DataPoint, usize)> = vec![(&dp, 0), (&dp, 0), (&dp, 1),
                                                  (&dp, 5), (&dp, 0)];
        let val: usize = 4;
        let expected = 0;
        let actual = count_assignments(&assignments, val);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_count_assignments_returns_3_when_3_occurences() {
        let dp = DataPoint{x: 0.0, y: 0.0};
        let assignments: Vec<(&DataPoint, usize)> = vec![(&dp, 0), (&dp, 0), (&dp, 1),
                                                  (&dp, 5), (&dp, 0)];
        let val: usize = 0;
        let expected = 3;
        let actual = count_assignments(&assignments, val);
        assert_eq!(expected, actual)
    }
}
