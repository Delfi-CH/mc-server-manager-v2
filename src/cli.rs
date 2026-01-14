use app_lib::*;

fn main() -> Result<(), LibError> {
    println!("Hello, cli!");
    config_create_config()?;
    Ok(())
}
