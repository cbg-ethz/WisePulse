//! Update Timestamp - WisePulse Data Pipeline
//!
//! Simple utility to update the .last_update timestamp file
//! Should be called after successful pipeline completion

use chrono::Utc;
use std::env;
use tokio::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let timestamp_file = if args.len() > 1 {
        &args[1]
    } else {
        ".last_update"
    };

    let now = Utc::now();
    let timestamp = now.timestamp();

    fs::write(timestamp_file, timestamp.to_string()).await?;
    println!(
        "Updated timestamp to: {}",
        now.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("File: {}", timestamp_file);

    Ok(())
}
