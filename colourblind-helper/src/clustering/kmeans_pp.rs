extern crate std;
extern crate rand;

use self::rand::Rng;
use self::std::vec::Vec;
use super::kmeans;


fn index_of_max_val<I>(floats: I) -> Option<usize> where I: IntoIterator<Item = f64> {
    let mut iter = floats.into_iter()
                         .enumerate();

    let fold_func = |(max_i, max_val), (i, val)| {
                        if val > max_val { (i, val) }
                        else { (max_i, max_val) }
    };

    iter.next().map(|(i, max)| {
        iter.fold((i, max), fold_func).0})
}


pub fn choose_centres(data: &Vec<kmeans::Rgb>, num_centroids: usize) -> Vec<kmeans::Rgb> {
    let mut centroids = Vec::new();
    let mut c = *rand::thread_rng().choose(data).unwrap();

    for _ in 0..num_centroids {
        let distances: Vec<f64> = data.iter().map(|x| c.sq_euclidean_distance(x)).collect();
        c = data[index_of_max_val(distances).unwrap()];
        centroids.push(c)
    }

    centroids
}
