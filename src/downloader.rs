use std::env;

use app_lib::*;

fn main() {
    println!("Hello, downloader!");
    println!("Downloading server.jar");
    download_vanilla_server("1.21.11".to_owned(),env::current_dir().unwrap().to_str().unwrap().to_owned());
}
