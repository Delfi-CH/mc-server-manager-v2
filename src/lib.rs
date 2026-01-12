use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use indicatif::ProgressBar;
use serde::*;

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

pub fn download_vanilla_server(ver: String, path:String, term: bool) {

    let intermediate_url = download_vanilla_get_version_data_url(ver);
    if intermediate_url != "none" {

        //The Download
        let downlad_url = download_vanilla_get_version_download_url(intermediate_url);

        let mut response = ureq::get(downlad_url).call().unwrap();

        let size = response.body().content_length().unwrap();
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("server.jar");
        let mut server_jar = File::create(save_path).unwrap();

        let progress = if term {
            Some(ProgressBar::new(size))
        } else {
            None
        };        

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
    }
}

fn download_vanilla_get_version_data_url(version: String) -> String {

    let manifest = download_vanilla_fetch_available_vannila_versions();
    let mut return_data = "none".to_owned();    

    for manifest_version in manifest.versions {
        if manifest_version.id == version {
            return_data = manifest_version.url;
        } 
    }
    return return_data;
}

fn download_vanilla_get_version_download_url(data_url: String) -> String {
    let mut response = ureq::get(data_url).call().unwrap();
    let body = response.body_mut();
    let text = body.read_to_string().unwrap();
    let version_data: MojangMinecraftVersion = serde_json::from_str(&text).unwrap();
    return version_data.downloads.server.url;
}

fn download_vanilla_fetch_available_vannila_versions() -> MojangVersionManifest {
    let mut response = ureq::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
    .call()
    .unwrap();

    let body = response.body_mut();
    let text = body.read_to_string().unwrap();

    let manifest: MojangVersionManifest = serde_json::from_str(&text).unwrap();
    return manifest;
}

//
// Forge Server
//

pub fn download_forge_server(mc_ver: String, path:String, forge_ver: String, term: bool) {
    download_forge_installer(mc_ver, path.clone(), forge_ver, term);

    if term {
        println!("Installing Forge Server...");
        thread::sleep(Duration::new(3, 0 ));
    }

    Command::new("java")
    .args(["-jar", "installer.jar", "--installServer"])
    .current_dir(path)
    .status()
    .unwrap();
}

fn download_forge_installer(mc_ver: String, path:String, forge_ver: String, term: bool) {
    let mut url = "".to_owned();
    if mc_ver == "1.9.4" || mc_ver == "1.8.9" || mc_ver == "1.7.10"{
        url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}-{}/forge-{}-{}-{}-installer.jar", mc_ver, forge_ver, mc_ver, mc_ver, forge_ver, mc_ver);
    } else {
        url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar", mc_ver, forge_ver, mc_ver, forge_ver)
    }
    let mut response = ureq::get(&url).call().unwrap();

        let size = response.body().content_length().unwrap_or(1);
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("installer.jar");
        let mut server_jar = File::create(save_path).unwrap();
        let progress = if term {
            Some(ProgressBar::new(size))
        } else {
            None
        };        

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
}

//
// Fabric Server
//

