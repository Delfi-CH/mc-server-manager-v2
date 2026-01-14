use std::path::PathBuf;

use app_lib::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, version, long_about = None)]
struct Args {
    /// Minecraft Version to download
    #[arg(short='v', long="mc-version")]
    mcversion: String,

    /// Which Modloader to use
    #[arg(short='m', long="modloader")]
    modloader: Modloaders,

    /// Path to download to
    #[arg(short='p', long="path")]
    path: Option<PathBuf>,

    /// Neoforge Version (only required if --modloader neo-forge)
    #[arg(long="neoforge-version")]
    neoforge_ver: Option<String>,
}


fn main() -> Result<(), LibError>{
    let args = Args::parse();
    let versions = meta_fetch_game_versions()?;
    if !versions.contains(&args.mcversion){
        eprintln!("Invalid Minecraft Version!");
        std::process::exit(1);
    }
    let mut path_str = "".to_owned();
    match args.path {
        Some(path)=>path_str = path.display().to_string(),
        None => {
            eprintln!("Invalid path!");
            std::process::exit(1);
        }
    } 
    let mut neofroge_ver = "".to_owned();
    match args.neoforge_ver {
        Some(ver) => neofroge_ver = ver,
        None => {
            if args.modloader == Modloaders::NeoForge {
                eprintln!("No NeoForge Version specified!");
                std::process::exit(1);
            }
        }
    }
    match args.modloader {
        Modloaders::Vanilla => wrap_download_vanilla_server(args.mcversion, path_str),
        Modloaders::Forge => wrap_download_forge_server(args.mcversion, path_str),
        Modloaders::NeoForge => wrap_download_neoforge_server(args.mcversion, path_str, neofroge_ver),
        Modloaders::Fabric => wrap_download_fabric_server(args.mcversion, path_str),
        Modloaders::Paper => wrap_download_paper_server(args.mcversion, path_str),
        Modloaders::Folia => wrap_download_folia_server(args.mcversion, path_str),
    }
    Ok(())
}

fn wrap_download_vanilla_server(ver: String, path: String) {
    println!("Downloading Vanilla server.jar...");
    match download_vanilla_server(ver,path,true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download vanilla server :{e}");
        }
    }
}
fn wrap_download_forge_server(ver: String, path: String) {
    println!("Downloading Forge installer.jar...");
    match download_forge_server(ver,path, true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download/install Forge server :{e}");
        }
    }
}
fn wrap_download_neoforge_server(ver: String, path: String, neoforge_ver: String) {
    println!("Downloading NeoForge installer.jar...");
    match download_neoforge_server(path, ver,true, neoforge_ver) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download/install NeoForge server :{e}");
        }
    }
}
fn wrap_download_fabric_server(ver: String, path: String) {
    println!("Downloading Fabric installer.jar...");
    match download_fabric_server(ver,path,true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download/install Fabric server :{e}");
        }
    }
}
fn wrap_download_paper_server(ver: String, path: String) {
    println!("Downloading Paper server.jar...");
    match download_paper_server(ver,path,true, false) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download Paper server :{e}");
        }
    }
}
fn wrap_download_folia_server(ver: String, path: String) {
    println!("Downloading Folia server.jar...");
    match download_paper_server(ver,path,true, true) {
        Ok(_) => {
            println!("Done!");
        }
        Err(e) => {
            eprintln!("Could not download Folia server :{e}");
        }
    }
}