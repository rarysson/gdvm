mod commands;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    commands::available::run().await?;

    Ok(())
}
