use std::path::Path;

extern crate csv;

extern crate generic_array;
use generic_array::GenericArray;

extern crate rustc_serialize;
use rustc_serialize::{Decodable, Decoder};

#[derive(Clone, Debug)]
pub struct DataPoint<N>(GenericArray<f64, N>)
    where N: generic_array::ArrayLength<f64>;


impl <N> DataPoint<N>
    where N: generic_array::ArrayLength<f64>
{
    fn squared_euclidean_distance(&self, other: &DataPoint<N>) -> f64
        where N: generic_array::ArrayLength<f64>
    {
        let iter = self.0.iter().zip(other.0.iter());
        iter.fold(0.0, |acc, x| acc + (x.0 - x.1).powi(2))
    }
}


impl <N: Default+Copy+Decodable> Decodable for DataPoint<N>
    where N: generic_array::ArrayLength<f64>
{
    fn decode<S: Decoder>(decoder: &mut S) -> Result<DataPoint<N>, S::Error> {
        decoder.read_seq(|decoder, _| {
            let mut arr  = GenericArray::<f64, N>::new();
            for (i, val) in arr.iter_mut().enumerate() {
                *val = try!(decoder.read_seq_elt(i, Decodable::decode));
            }
            Ok(DataPoint(arr))
        })
    }
}


impl <N> std::ops::Add for DataPoint<N>
    where N: generic_array::ArrayLength<f64>
{
    type Output = DataPoint<N>;

    fn add(self, other: DataPoint<N>) -> DataPoint<N> {
        let mut arr = GenericArray::<f64, N>::new();
        for (i, val) in self.0.iter().zip(other.0.iter()).enumerate() {
            arr[i] = val.0 + val.1;
        }
        DataPoint(arr)
    }
}


/// Structure for holding data point's assignments to clusters
#[derive(Clone, Debug)]
pub struct Assignment<'a, N>
    where N: generic_array::ArrayLength<f64> + 'a
{
    data_point: &'a DataPoint<N>,
    cluster_ind: usize,
}


pub fn read_data<P, N>(file_path: P) -> Vec<DataPoint<N>>
    where P: AsRef<Path>,
    N: generic_array::ArrayLength<f64> + std::marker::Copy + std::default::Default + rustc_serialize::Decodable
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
fn expectation<'a, N>(data: &'a [DataPoint<N>],
                   cluster_centroids: &[DataPoint<N>]) -> Vec<Assignment<'a, N>>
    where N: generic_array::ArrayLength<f64>
{
    data.iter().map(|point| {
        let distances = cluster_centroids.iter()
                                         .map(|cluster| point.squared_euclidean_distance(cluster));
        let index = index_of_min_val(distances).expect("No minimum value found");
        Assignment { data_point: point, cluster_ind: index }
    }).collect()
}


pub fn points_in_cluster<'a, N>(assignments: &'a [Assignment<'a, N>],
                             expected_cluster_ind: usize) -> Box<Iterator<Item = Assignment<'a, N>> + 'a>
    where N: generic_array::ArrayLength<f64> + std::clone::Clone
{
    let i = assignments.into_iter()
        .cloned()
        .filter(move |&Assignment { cluster_ind, .. }| expected_cluster_ind == cluster_ind);
    Box::new(i)
}


pub fn count_assignments<'a, N>(assignments: &[Assignment<'a, N>],
                                cluster_ind: usize) -> usize
    where N: generic_array::ArrayLength<f64> + std::clone::Clone
{
    points_in_cluster(assignments, cluster_ind).count()
}


pub fn sum_assigned_values<'a, N>(assignments: &[Assignment<'a, N>],
                                  cluster_ind: usize) -> DataPoint<N>
where N: generic_array::ArrayLength<f64> + std::clone::Clone
{
    points_in_cluster(assignments, cluster_ind)
        .into_iter()
        .fold(DataPoint(GenericArray::<f64, N>::new()), |acc, point| acc + point.data_point.clone())
}


/// Update cluster centres
fn maximisation<'a, N>(cluster_centroids: &mut [DataPoint<N>],
                       assignments: &[Assignment<'a, N>])
where N: generic_array::ArrayLength<f64> + std::clone::Clone
{
    for i in 0..cluster_centroids.len() {
        let num_points = count_assignments(&assignments, i);
        let sum_points = sum_assigned_values(&assignments, i);
        let mut arr = GenericArray::<f64, N>::new();

        for j in 0..arr.len() {
            arr[j] = sum_points.0[j]/num_points as f64;
        }

        cluster_centroids[i] = DataPoint(arr);
    }
}


pub fn get_error_metric<'a, N>(cluster_centroids: &[DataPoint<N>],
                               assignments: &[Assignment<'a, N>]) -> f64
where N: generic_array::ArrayLength<f64>
{
    assignments.iter().fold(0.0, |error, assignment| {
        let centroid = &cluster_centroids[assignment.cluster_ind];
        error + assignment.data_point.squared_euclidean_distance(centroid)
    })
}
