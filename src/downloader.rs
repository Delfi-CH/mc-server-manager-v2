use std::env;

use app_lib::*;

fn main() {
    println!("Hello, downloader!");
    println!("Downloading Vanilla server.jar...");
    match download_vanilla_server("1.21.11".to_owned(),env::current_dir().unwrap().to_str().unwrap().to_owned()+"/test/vanilla", true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download vanilla server :{e}");
        }
    }
    
    println!("Downloading Forge installer.jar...");
    match download_forge_server("1.21.11".to_owned(),env::current_dir().unwrap().to_str().unwrap().to_owned()+"/test/forge", "61.0.6".to_owned(), true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download/install Forge server :{e}");
        }
    }

    println!("Downloading Fabric installer.jar...");
    match download_fabric_server("1.21.11".to_owned(),env::current_dir().unwrap().to_str().unwrap().to_owned()+"/test/fabric", true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download/install Fabric server :{e}");
        }
    }

    println!("Downloading NeoForge installer.jar...");
    match download_neoforge_server(env::current_dir().unwrap().to_str().unwrap().to_owned()+"/test/neoforge", "21.11.29-beta".to_owned(), true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download/install NeoForge server :{e}");
        }
    }
    

}
