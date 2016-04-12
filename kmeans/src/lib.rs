/// Store one data point's (or one cluster centroid's) x and y co-ordinates
#[derive(Clone)]
pub struct DataPoint {
    x: f64,
    y: f64,
}

/*
fn read_data(file_path: &Path) -> Vec<DataPoint> {
}*/

pub fn squared_euclidean_distance(point_1: &DataPoint, point_2: &DataPoint) -> f64 {
   (point_2.x - point_1.x).powi(2) + (point_2.y - point_1.y).powi(2)
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
               cluster_centroids: Vec<DataPoint>) -> Vec<usize> {

    let mut distance: Vec<f64> = vec![];
    let mut assigned_clusters: Vec<usize> = vec![];
    for point in &data {
        for cluster in &cluster_centroids {
            distance.push(squared_euclidean_distance(point, cluster));
        }

        // Index of cluster centroid that point is nearest to
        assigned_clusters.push(get_index_of_min_val(&distance));
    }
    assigned_clusters
}

fn count_occurences(list: &Vec<usize>, value: usize) -> usize {
    let mut list_copy = list.clone();
    list_copy.retain(|&x| x == value);
    list_copy.len()
}
    

/// Update cluster centres
fn maximisation(cluster_centroids: &mut Vec<DataPoint>,
                assigned_clusters: Vec<usize>) {

    for i in 0..cluster_centroids.len() {
        let num_points = count_occurences(&assigned_clusters, i);
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
        let point_1 = DataPoint { x: -999.3, y: 10.5};
        let point_2 = point_1.clone();
        let expected = 0.0;
        let actual = squared_euclidean_distance(&point_1, &point_2);
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_index_of_min_val() {
        let floats = vec![0.0_f64, 1.0_f64, 3.0_f64, -5.5_f64];
        let expected = 3;
        let actual = get_index_of_min_val(&floats);
        assert_eq!(expected, actual)
    }
}
