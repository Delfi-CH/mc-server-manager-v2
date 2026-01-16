use app_lib::*;

fn main() -> Result<(), LibError> {
    println!("Hello, cli!");
    config_create_config()?;
    println!("Downloading Java 17...");
    download_java_openjdk_amazon_correto(LINUX_JAVA_17_URL, LINUX_JAVA_17_SHA256, true, "./test/java/".to_owned(), JavaVersion::Java17)?;
    Ok(())
}
