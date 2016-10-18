extern crate generic_array;

trait Euclidean<N> {
    fn squared_euclidean_distance(&self, other: &generic_array::GenericArray<f64, N>) -> f64
        where N: generic_array::ArrayLength<f64>;
}


impl <N> Euclidean<N> for generic_array::GenericArray<f64, N>
    where N: generic_array::ArrayLength<f64>
{
    fn squared_euclidean_distance(&self, other: &generic_array::GenericArray<f64, N>) -> f64
        where N: generic_array::ArrayLength<f64>
    {
        let iter = self.iter().zip(other.iter());
        iter.fold(0.0, |acc, x| acc + (x.0 - x.1).powi(2))
    }
}


/// Structure for holding data point's assignments to clusters
#[derive(Clone, Debug)]
pub struct Assignment<N>
    where N: generic_array::ArrayLength<f64>
{
    data_point: generic_array::GenericArray<f64, N>,
    cluster_ind: usize,
}
