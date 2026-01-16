use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::fmt;

use clap::ValueEnum;
use thiserror::Error;
use indicatif::*;
use serde::*;
use directories::*;
use toml::Value;
use sha256::*;

//
// Core Library Stuff
//

pub const APP_NAME: &str = "mc-server-manager-v2";
pub const APP_VERSION: &str = "0.0.1";

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
    #[error("Miscelanious Error: {0}")]
    Misc(String),
    #[error("Variable Error: {0}")]
    Var(#[from] std::env::VarError),
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

pub fn download_forge_server(ver: String, path:String, term: bool) -> Result<(), LibError>{
    let forge_ver = meta_get_forge_version_for_corresponding_mc_version(ver.clone())?;
    download_forge_installer(ver, path.clone(), forge_ver, term)?;

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
    let url;
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

pub fn download_neoforge_server(path: String, ver: String, term: bool, neoforge_ver: String) -> Result<(), LibError> {
    download_neoforge_installer(ver, neoforge_ver, path.clone(), term)?;

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

fn download_neoforge_installer(_ver: String, neoforge_ver: String, path: String, term: bool) -> Result<(), LibError>{
    println!("{}", neoforge_ver);
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

#[derive(Debug, Deserialize)]
pub struct PaperProjectVersions {
    pub project_id: String,
    pub project_name: String,
    pub versions: Vec<String>,
    pub builds: Option<Vec<u32>>,
}
#[derive(Debug, Deserialize)]
pub struct PaperProjectBuilds {
    pub project_id: String,
    pub project_name: String,
    pub builds: Option<Vec<u32>>,
}

pub fn download_paper_server(ver: String, path:String, term: bool, folia: bool) -> Result<(), LibError>{

    let build = download_paper_fetch_latest_build(ver.clone(), folia)?;
    if build != 0 {

        //The Download
        let downlad_url;
        if folia {
            downlad_url = format!("https://api.papermc.io/v2/projects/folia/versions/{ver}/builds/{build}/downloads/folia-{ver}-{build}.jar");
        } else {
            downlad_url =  format!("https://api.papermc.io/v2/projects/paper/versions/{ver}/builds/{build}/downloads/paper-{ver}-{build}.jar");
        };

        let mut response = ureq::get(downlad_url).call()?;
        
        let mut reader = response.body_mut().as_reader();
        let save_path = Path::new(&path).join("server.jar");
        let mut server_jar = File::create(save_path)?;

        let progress = if term {
            Some(ProgressBar::new_spinner())
        } else {
            None
        };        

        if let Some(pb) = &progress {
            pb.enable_steady_tick(Duration::from_millis(100));
            pb.set_style(
                ProgressStyle::default_spinner()
                .tick_chars("|/-\\")
                .template(
                    "{bytes} ({bytes_per_sec}, {eta})"
                )
                .unwrap()
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
        if let Some (pb) = &progress {
        pb.finish();
        }
        Ok(())
    } else {
        return Err(LibError::Ver(ver));
    }
}

fn download_paper_fetch_latest_build(ver: String, folia: bool) -> Result<u32, LibError> {
    let project = download_paper_fetch_versions(folia)?;

    let mut result: u32 = 0;
    for v in project.versions {
        if v == ver {
            let url = if folia {
                "https://api.papermc.io/v2/projects/folia/versions/"
            } else {
                "https://api.papermc.io/v2/projects/paper/versions/"
            };
            let mut response = ureq::get(url.to_owned()+&ver)
            .call()?;
            
            let body = response.body_mut();
            let text = body.read_to_string()?;
            let project2: PaperProjectBuilds = serde_json::from_str(&text).unwrap();
            if let Some(builds) = project2.builds {
                result = builds[builds.len()-1]
            }
        }
    }
    return Ok(result);
}


fn download_paper_fetch_versions(folia: bool) -> Result<PaperProjectVersions, LibError> {
    let url = if folia {
        "https://api.papermc.io/v2/projects/folia"
    } else {
        "https://api.papermc.io/v2/projects/paper"
    };

    let mut response = ureq::get(url).call()?;

    let body = response.body_mut();
    let text = body.read_to_string()?;

    let project: PaperProjectVersions = serde_json::from_str(&text).unwrap();
    return Ok(project);
}

//
// Java Downloads
//

pub const LINUX_JAVA_8_URL: &str = "https://corretto.aws/downloads/latest/amazon-corretto-8-x64-linux-jdk.tar.gz";
pub const LINUX_JAVA_17_URL: &str = "https://corretto.aws/downloads/latest/amazon-corretto-17-x64-linux-jdk.tar.gz";
pub const LINUX_JAVA_21_URL: &str = "https://corretto.aws/downloads/latest/amazon-corretto-21-x64-linux-jdk.tar.gz";
pub const LINUX_JAVA_25_URL: &str = "https://corretto.aws/downloads/latest/amazon-corretto-25-x64-linux-jdk.tar.gz";

pub const LINUX_JAVA_8_SHA256: &str = "https://corretto.aws/downloads/latest_sha256/amazon-corretto-8-x64-linux-jdk.tar.gz";
pub const LINUX_JAVA_17_SHA256: &str = "https://corretto.aws/downloads/latest_sha256/amazon-corretto-17-x64-linux-jdk.tar.gz";
pub const LINUX_JAVA_21_SHA256: &str = "https://corretto.aws/downloads/latest_sha256/amazon-corretto-21-x64-linux-jdk.tar.gz";
pub const LINUX_JAVA_25_SHA256: &str = "https://corretto.aws/downloads/latest_sha256/amazon-corretto-25-x64-linux-jdk.tar.gz";

#[derive(Debug)]
pub enum JavaVersion {
    Java8,
    Java17,
    Java21,
    Java25,
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn download_java_openjdk_amazon_correto(url: &str, hash: &str, term: bool, path: String, java_ver: JavaVersion) -> Result<(), LibError> {
    let mut response = ureq::get(url).call()?;
    let size = response.body().content_length().unwrap_or(0);
        
    let mut reader = response.body_mut().as_reader();
    let save_path = Path::new(&path).join(java_ver.to_string()+"/java.tar.gz");
    let mut java_tar = File::create(save_path)?;

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
        java_tar.write_all(&buffer[..n]).unwrap();
        if let Some(pb) = &progress {
            pb.inc(n as u64);
        }
    }

    if term {
        println!("Verifying Integrety...");
    }

    let mut sha_response = ureq::get(hash).call()?;
    let sha_body = sha_response.body_mut();
    let downloaded_hash = sha_body.read_to_string()?;
    let local_hash = try_digest(Path::new(&path).join(java_ver.to_string()+"/java.tar.gz"))?;
    if downloaded_hash == local_hash {
        if term {
            println!("Done!");
        }        
    } else {
        LibError::Misc("Could not verify the Integrety of the file!".to_owned());
    }
    Ok(())

}

//
// Metadata
//

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq)]
pub enum Modloaders {
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    Paper,
    Folia,
}
 
pub fn meta_fetch_game_versions() -> Result<Vec<String>, LibError> {
    let mut result: Vec<String> = Vec::new();
    let mut response = ureq::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
    .call()?;

    let body = response.body_mut();
    let text = body.read_to_string()?;

    let manifest: MojangVersionManifest = serde_json::from_str(&text)?;
    let versions: Vec<MojangVersionEntry> = manifest.versions;
    for ver in versions {
        if ver.kind == "release" {
            result.push(ver.id);
        }
    }
    Ok(result)
}

//
// Forge Versions
//

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeMetadata {
    #[serde(flatten, deserialize_with = "meta_forge_minecraft_versions_from_map")]
    pub minecraft_versions: HashMap<String, ForgeMinecraftVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeMinecraftVersion {
    pub version: String,
    pub builds: Vec<ForgeBuild>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForgeBuild {
    pub id: String,
}

fn meta_forge_minecraft_versions_from_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ForgeMinecraftVersion>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, Vec<String>> = HashMap::deserialize(deserializer)?;
    let result = map
        .into_iter()
        .map(|(version, build_ids)| {
            Ok((
                version.clone(),
                ForgeMinecraftVersion {
                    version,
                    builds: build_ids.into_iter().map(|id| ForgeBuild { id }).collect(),
                },
            ))
        })
        .collect::<Result<HashMap<_, _>, D::Error>>()?;
    Ok(result)
}

fn meta_get_forge_version_for_corresponding_mc_version(ver: String) -> Result<String, LibError> {
    let mut response = ureq::get("https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json")
        .call()?;

    let body = response.body_mut();
    let text = body.read_to_string()?;
    let meta: ForgeMetadata = serde_json::from_str(&text)?;

    let mut builds: Vec<String> = vec![];

    for (mc_version, forge_version) in &meta.minecraft_versions {
        for build in &forge_version.builds {
            if ver == *mc_version {
                let build_tmp = build.id.replace(mc_version, "");
                let build_flat = build_tmp.replace("-", "");
                builds.push(build_flat);
            }
        }
    }

    Ok(builds[builds.len()-1].clone())
}

//
// Config
//

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub title: String,
    pub version: String,
    pub directories: Directories,
    pub java_paths: JavaPaths,
}

#[derive(Serialize)]
pub struct System {
    pub os_type: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Directories {
    pub config_dir: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub home_dir: String,
    pub server_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JavaPaths {
    // All of these must point to the java binary
    pub java8_path: PathBuf,
    pub java17_path: PathBuf,
    pub java21_path: PathBuf,
    pub java25_path: PathBuf,
}

pub fn config_fetch_directories() -> Directories {
    let mut config_dir = String::new();
    let mut data_dir = String::new();
    let mut cache_dir = String::new();
    let mut home_dir = String::new();
    let mut server_dir = String::new();

    if let Some(project_dirs) = ProjectDirs::from("dev", "delfi", "MC Server Manager V2") {
        config_dir = project_dirs.config_dir().to_string_lossy().to_string();
        data_dir = project_dirs.data_dir().to_string_lossy().to_string();
        cache_dir = project_dirs.cache_dir().to_string_lossy().to_string();
        server_dir = data_dir.clone() + "/servers"
    }

    if let Some(user_dirs) = UserDirs::new() {
        home_dir = user_dirs.home_dir().to_string_lossy().to_string();
    }
    
    let dirs = Directories {
        config_dir: config_dir,
        cache_dir: cache_dir,
        data_dir: data_dir,
        home_dir: home_dir,
        server_dir: server_dir,
    };
    return dirs;
}

pub fn config_create_config() -> Result<(), LibError> {
    let dirs = config_fetch_directories(); 
    let path = PathBuf::from(dirs.clone().config_dir + "/config.toml");
    if !path.exists() {
        if !PathBuf::from(dirs.clone().config_dir).exists(){
            std::fs::create_dir(dirs.clone().config_dir)?;
        } 
        let config = Config {
            title: APP_NAME.to_owned(),
            version: APP_VERSION.to_owned(),
            directories: Directories {
                config_dir: dirs.config_dir,
                data_dir: dirs.data_dir,
                cache_dir: dirs.cache_dir,
                home_dir: dirs.home_dir,
                server_dir: dirs.server_dir
            },
            java_paths: JavaPaths { java8_path: PathBuf::new(), java17_path: PathBuf::new(), java21_path: PathBuf::new(), java25_path: PathBuf::new() }
        };
        let toml_string = toml::to_string_pretty(&config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, toml_string)?;
    }
    Ok(())
}

pub fn config_read_config() -> Result<Value, LibError> {
    let dirs = config_fetch_directories(); 
    let path = PathBuf::from(dirs.config_dir + "/config.toml");
    let content = std::fs::read_to_string(path)?;
    let value: Value = content
        .parse::<Value>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(value)
}

pub fn config_get_value<'a>(config: &'a Value, key: &str) -> Option<&'a Value> {
    let mut current = config;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}


pub fn config_set_value(config: &mut Value, key: &str, new_value: Value) -> Result<bool, LibError> {
    let parts: Vec<&str> = key.split('.').collect();
    let last = match parts.last() {
        Some(k) => *k,
        None => return Ok(false),
    };
    let mut current = config;
    for part in &parts[..parts.len() - 1] {
        current = current.get_mut(part).ok_or(LibError::Misc("Invalid Key".to_owned()))?;
    }
    current[last] = new_value;
    Ok(true)
}

pub fn config_write_config(config: &Value) -> Result<(), LibError> {
    let dirs = config_fetch_directories(); 
    let path = PathBuf::from(dirs.config_dir + "/config.toml");
    let toml_string = toml::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, toml_string)?;
    Ok(())
}

//
// Server Structs
//