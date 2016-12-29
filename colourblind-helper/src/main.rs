extern crate colourblind_helper;

use colourblind_helper::utils;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    utils::cluster_image(&args[1], 10);
}
