use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use thiserror::Error;

use indicatif::*;
use serde::*;

//
// Core Library Stuff
//

#[derive(Debug, Error)]
pub enum LibError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parsing error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Network error: {0}")]
    Net(#[from] ureq::Error),
    #[error("Version error: {0}")]
    Ver(String),
    #[error("JSON Parse error: {0}")]
    Json(#[from] serde_json::Error),
}


//
// The download Stuff
//

//
// Vanilla Server
//

#[derive(Debug, Deserialize)]
pub struct MojangVersionManifest {
    pub latest: Latest,
    pub versions: Vec<MojangVersionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize)]
pub struct MojangVersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: i32,
}

#[derive(Deserialize, Debug)]
struct MojangMinecraftVersion {
    downloads: MojangDownloads,
}

#[derive(Deserialize, Debug)]
struct MojangDownloads {
    server: MojangServerDownload,
}

#[derive(Deserialize, Debug)]
struct MojangServerDownload {
    url: String,
}

pub fn download_vanilla_server(ver: String, path:String, term: bool) -> Result<(), LibError>{

    let intermediate_url = download_vanilla_get_version_data_url(ver.clone())?;
    if intermediate_url != "none" {

        //The Download
        let downlad_url = download_vanilla_get_version_download_url(intermediate_url)?;

        let mut response = ureq::get(downlad_url).call()?;

        let size = response.body().content_length().unwrap_or(0);
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("server.jar");
        let mut server_jar = File::create(save_path)?;

        let progress = if term {
            Some(ProgressBar::new(size))
        } else {
            None
        };        

        if let Some(pb) = &progress {
            pb.set_style(
                ProgressStyle::default_bar()
                .template(
                    "{bar:80.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
                )
                .unwrap()
                .progress_chars("=> "),
            );

        }

        let mut buffer = [0u8; 8 * 1024];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            server_jar.write_all(&buffer[..n])?;
            if let Some(pb) = &progress {
                pb.inc(n as u64);
            }
        }
        Ok(())
    } else {
        return Err(LibError::Ver(ver));
    }
}

fn download_vanilla_get_version_data_url(version: String) -> Result<String, LibError> {

    let manifest = download_vanilla_fetch_available_vannila_versions()?;
    let mut return_data = "none".to_owned();    

    for manifest_version in manifest.versions {
        if manifest_version.id == version {
            return_data = manifest_version.url;
        } 
    }
    return Ok(return_data);
}

fn download_vanilla_get_version_download_url(data_url: String) -> Result<String, LibError> {
    let mut response = ureq::get(data_url).call()?;
    let body = response.body_mut();
    let text = body.read_to_string()?;
    let version_data: MojangMinecraftVersion = serde_json::from_str(&text)?;
    return Ok(version_data.downloads.server.url);
}

fn download_vanilla_fetch_available_vannila_versions() -> Result<MojangVersionManifest, LibError> {
    let mut response = ureq::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
    .call()?;

    let body = response.body_mut();
    let text = body.read_to_string()?;

    let manifest: MojangVersionManifest = serde_json::from_str(&text).unwrap();
    return Ok(manifest);
}

//
// Forge Server
//

pub fn download_forge_server(mc_ver: String, path:String, forge_ver: String, term: bool) -> Result<(), LibError>{
    download_forge_installer(mc_ver, path.clone(), forge_ver, term)?;

    if term {
        println!("Installing Forge Server...");
    }

    let spinner = if term {
            Some(ProgressBar::new_spinner())
        } else {
            None
    };

    if let Some (spinner) = &spinner {
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_style(
            ProgressStyle::default_spinner()
            .tick_chars("|/-\\")
            .template("{spinner} {msg}")
            .unwrap(),
        );
    }

    let mut child = Command::new("java")
    .args(["-jar", "installer.jar", "--installServer"])
    .current_dir(path)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()?;

    
    child.wait()?;

    if let Some (spinner) = &spinner {
        spinner.finish();
    }
    return Ok(());
    
}

fn download_forge_installer(mc_ver: String, path:String, forge_ver: String, term: bool) -> Result<(), LibError>{
    let mut url = "".to_owned();
    if mc_ver == "1.9.4" || mc_ver == "1.8.9" || mc_ver == "1.7.10"{
        url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}-{}/forge-{}-{}-{}-installer.jar", mc_ver, forge_ver, mc_ver, mc_ver, forge_ver, mc_ver);
    } else {
        url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar", mc_ver, forge_ver, mc_ver, forge_ver)
    }
    let mut response = ureq::get(&url).call()?;

        let size = response.body().content_length().unwrap_or(0);
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("installer.jar");
        let mut server_jar = File::create(save_path)?;
        let progress = if term {
            Some(ProgressBar::new(size))
        } else {
            None
        };

        if let Some(pb) = &progress {
            pb.set_style(
                ProgressStyle::default_bar()
                .template(
                    "{bar:80.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
                )
                .unwrap()
                .progress_chars("=> "),
            );

        }

        let mut buffer = [0u8; 8 * 1024];
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            server_jar.write_all(&buffer[..n]).unwrap();
            if let Some(pb) = &progress {
                pb.inc(n as u64);
            }
        }
        Ok(())
}

//
// Neoforge Server
//

pub fn download_neoforge_server(path: String, neoforge_ver: String, term: bool) -> Result<(), LibError> {
    download_neoforge_installer(neoforge_ver, path.clone(), term)?;

    if term {
        println!("Installing NeoForge Server...");
    }

    let spinner = if term {
            Some(ProgressBar::new_spinner())
        } else {
            None
    };

    if let Some (spinner) = &spinner {
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_style(
            ProgressStyle::default_spinner()
            .tick_chars("|/-\\")
            .template("{spinner} {msg}")
            .unwrap(),
        );
    }
    
    let mut child = Command::new("java")
    .args(["-jar", "installer.jar", "--installServer"])
    .current_dir(path)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()?;

    child.wait()?;

    if let Some (spinner) = &spinner {
        spinner.finish();
    }
    return Ok(());

}

fn download_neoforge_installer(neoforge_ver: String, path: String, term: bool) -> Result<(), LibError>{
    let url = format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{neoforge_ver}/neoforge-{neoforge_ver}-installer.jar");
    let mut response = ureq::get(&url).call()?;

        let size = response.body().content_length().unwrap_or(0);
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("installer.jar");
        let mut server_jar = File::create(save_path)?;
        let progress = if term {
            Some(ProgressBar::new(size))
        } else {
            None
        };

        if let Some(pb) = &progress {
            pb.set_style(
                ProgressStyle::default_bar()
                .template(
                    "{bar:80.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
                )
                .unwrap()
                .progress_chars("=> "),
            );

        }

        let mut buffer = [0u8; 8 * 1024];
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            server_jar.write_all(&buffer[..n]).unwrap();
            if let Some(pb) = &progress {
                pb.inc(n as u64);
            }
        }
        Ok(())
}

//
// Fabric Server
//

pub fn download_fabric_server(mc_ver: String, path: String, term: bool) -> Result<(), LibError> {
    download_fabric_installer(path.clone(), term)?;
    if term {
        println!("Installing Fabric Server...");
    }

    let spinner = if term {
            Some(ProgressBar::new_spinner())
        } else {
            None
    };

    if let Some (spinner) = &spinner {
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_style(
            ProgressStyle::default_spinner()
            .tick_chars("|/-\\")
            .template("{spinner} {msg}")
            .unwrap(),
        );
    }

    let mut child = Command::new("java")
    .args(["-jar", "installer.jar", "server", "-mcversion", &mc_ver, "-dir", &path])
    .current_dir(path)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()?;

    child.wait()?;

    if let Some (spinner) = &spinner {
        spinner.finish();
    }
    return Ok(());
}

fn download_fabric_installer(path: String, term: bool) -> Result<(), LibError> {
    let downlad_url = String::from("https://maven.fabricmc.net/net/fabricmc/fabric-installer/1.0.0/fabric-installer-1.0.0.jar");

        let mut response = ureq::get(downlad_url).call()?;

        let size = response.body().content_length().unwrap_or(0);
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("installer.jar");
        let mut server_jar = File::create(save_path)?;

        let progress = if term {
            Some(ProgressBar::new(size))
        } else {
            None
        };        

        if let Some(pb) = &progress {
            pb.set_style(
                ProgressStyle::default_bar()
                .template(
                    "{bar:80.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
                )
                .unwrap()
                .progress_chars("=> "),
            );

        }

        let mut buffer = [0u8; 8 * 1024];
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            server_jar.write_all(&buffer[..n]).unwrap();
            if let Some(pb) = &progress {
                pb.inc(n as u64);
            }
        }
        Ok(())
}

//
// Paper Server
//

