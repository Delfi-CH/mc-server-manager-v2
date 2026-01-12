use std::env;

use app_lib::*;

fn main() {
    println!("Hello, downloader!");
    println!("Downloading Vanilla server.jar...");
    download_vanilla_server("1.21.11".to_owned(),env::current_dir().unwrap().to_str().unwrap().to_owned()+"/test", true);
    println!("Done!");
    println!("Downloading Forge installer.jar...");
    download_forge_server("1.21.11".to_owned(),env::current_dir().unwrap().to_str().unwrap().to_owned()+"/test", "61.0.6".to_owned(), true);
    println!("Done!");

}
