use ureq::*;
use serde::*;

pub fn sanity_check() -> String {
    return "This works".to_string();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let result = sanity_check();
        assert_eq!(result, "This works");
    }
}

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
    pub releaseTime: String,
    pub sha1: String,
    pub complianceLevel: i32,
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

pub fn download_vanilla_server() {
    let ver = "1.21.11".to_owned();
    let intermediate_url = download_vanilla_get_version_data_url(ver);
    if intermediate_url != "none" {
        download_vanilla_get_version_download_url(intermediate_url)
    }
}

fn download_vanilla_get_version_data_url(version: String) -> String {

    let manifest = download_vanilla_fetch_available_vannila_versions();
    let mut return_data = String::new();
    return_data = "none".to_owned();    

    for manifest_version in manifest.versions {
        if manifest_version.id == version {
            return_data = manifest_version.url;
        } 
    }
    return return_data;
}

fn download_vanilla_get_version_download_url(data_url: String) {
    let mut response = ureq::get(data_url).call().unwrap();
    let body = response.body_mut();
    let text = body.read_to_string().unwrap();
    let version_data: MojangMinecraftVersion = serde_json::from_str(&text).unwrap();
    println!("{}", version_data.downloads.server.url);
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
