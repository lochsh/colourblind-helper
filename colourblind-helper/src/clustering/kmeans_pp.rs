extern crate std;
extern crate rand;

use self::rand::Rng;
use self::rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use self::std::vec::Vec;
use super::kmeans;


pub fn compute_distances(data: &Vec<kmeans::Rgb>, centroids: &Vec<kmeans::Rgb>) -> Vec<f64> {
    let mut distances = Vec::<f64>::new();

    for d in data {
        distances.push(centroids.iter()
                                .cloned()
                                .map(|x| x.sq_euclidean_distance(d))
                                .fold(0./0., f64::min));
    }

    distances
}


fn compute_weights<'a>(data: &'a Vec<kmeans::Rgb>, distances: &Vec<f64>) -> Vec<Weighted<&'a kmeans::Rgb>>{
    let mut weights = Vec::new();
    let factor: f64 = distances.iter()
                               .map(|x| x.powi(2))
                               .sum();

    for d in data.iter().zip(distances.iter()) {
        weights.push(Weighted {item: d.0, weight: (*d.1/factor * u32::max_value() as f64) as u32});
    }

    weights
}


pub fn choose_centres(data: &Vec<kmeans::Rgb>, num_centroids: usize) -> Vec<kmeans::Rgb> {
    let mut centroids = vec![*rand::thread_rng().choose(data).unwrap()];

    for _ in 0..num_centroids {
        let distances = compute_distances(&data, &centroids);

        centroids.push(*WeightedChoice::new(&mut compute_weights(&data,
                                            &distances)).ind_sample(&mut rand::thread_rng()));
    }

    centroids
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::kmeans;

    #[test]
    fn test_compute_distances() {
        let data = vec![kmeans::Rgb {r: 0., g: 0., b: 0.}];
        let centroids = vec![kmeans::Rgb {r: 0., g: 0., b: 0.},
                             kmeans::Rgb {r: 10.4, g: 1., b: 4.9}];
        assert_eq!(vec![0.0], compute_distances(&data, &centroids));
    }
}
