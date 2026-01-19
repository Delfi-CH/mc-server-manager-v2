use std::{thread::sleep, time::Duration};

use app_lib::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, version, long_about = None)]
struct Args {
    //Download Java
    #[arg(long="download-java")]
    java_version: Option<JavaVersion>,
}

fn main() -> Result<(), LibError> {
    config_create_config()?;
    let config = config_read_config()?;
    let args = Args::parse();

    match args.java_version {
        Some(java_version) => {
            let java_dir = config.directories.java_dir;
            match java_version {
                JavaVersion::Java8 => {
                    println!("Downloading Java 8...");
                    download_java_openjdk_amazon_correto(LINUX_JAVA_8_URL, LINUX_JAVA_8_SHA256, true, java_dir, JavaVersion::Java8)?;
                }
                JavaVersion::Java17 => {
                    println!("Downloading Java 17...");
                    download_java_openjdk_amazon_correto(LINUX_JAVA_17_URL, LINUX_JAVA_17_SHA256, true, java_dir, JavaVersion::Java17)?;
                }
                JavaVersion::Java21 => {
                    println!("Downloading Java 21...");
                    download_java_openjdk_amazon_correto(LINUX_JAVA_21_URL, LINUX_JAVA_21_SHA256, true, java_dir, JavaVersion::Java21)?;
                }
                JavaVersion::Java25 => {
                    println!("Downloading Java 25...");
                    download_java_openjdk_amazon_correto(LINUX_JAVA_25_URL, LINUX_JAVA_25_SHA256, true, java_dir, JavaVersion::Java25)?;
                }
            }
        }
        None => sleep(Duration::from_nanos(0)),
    }

    println!("Hello, cli!");
    
    
    Ok(())
}
