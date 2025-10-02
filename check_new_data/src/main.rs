//! Check New Data - WisePulse Data Pipeline
//!
//! Queries the LAPIS API to check if any new sequences have been submitted
//! since the last successful pipeline run using submittedAtTimestampFrom.
//!
//! Exit codes:
//! - 0: New data available (pipeline should run)
//! - 1: No new data (pipeline can skip)
//! - 2: Error occurred

use chrono::{DateTime, Utc};
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(name = "check_new_data")]
#[command(about = "Check if new genomic data is available from LAPIS API")]
struct Args {
    /// Base URL for the LAPIS API
    #[arg(long, default_value = "https://api.db.wasap.genspectrum.org")]
    api_base_url: String,

    /// Path to store last update timestamp
    #[arg(long, default_value = ".last_update")]
    timestamp_file: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Vec<SampleData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SampleData {
    sample_id: String,
    submitted_at_timestamp: i64,
}

#[tokio::main]
async fn main() {
    let exit_code = match run().await {
        Ok(has_new_data) => {
            if has_new_data {
                0 // New data available
            } else {
                1 // No new data
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            2 // Error
        }
    };
    
    std::process::exit(exit_code);
}

async fn run() -> Result<bool> {
    let args = Args::parse();
    
    println!("=== Checking for new data ===");
    println!("API: {}", args.api_base_url);
    
    let last_update = read_last_update(&args.timestamp_file).await?;
    
    match last_update {
        Some(last_date) => {
            println!("Last update: {}", last_date.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Last update timestamp: {}", last_date.timestamp());
            
            let has_new_data = check_for_new_submissions(&args, last_date).await?;
            
            if has_new_data {
                println!("✓ New data available!");
                println!("  Pipeline should run to fetch and process new sequences.");
            } else {
                println!("• No new data found.");
                println!("  Pipeline can skip this run.");
            }
            
            Ok(has_new_data)
        }
        None => {
            println!("No previous update timestamp found.");
            println!("✓ First run - pipeline should fetch initial data.");
            Ok(true)
        }
    }
}

async fn read_last_update(path: &str) -> Result<Option<DateTime<Utc>>> {
    let file_path = Path::new(path);
    
    if !file_path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(file_path).await?;
    let timestamp = content.trim().parse::<i64>()?;
    let datetime = DateTime::from_timestamp(timestamp, 0)
        .ok_or("Invalid timestamp in file")?;
    
    Ok(Some(datetime))
}

async fn check_for_new_submissions(args: &Args, last_update: DateTime<Utc>) -> Result<bool> {
    let client = Client::new();
    let timestamp = last_update.timestamp();
    
    let url = format!(
        "{}/covid/sample/details?submittedAtTimestampFrom={}&limit=1&dataFormat=JSON&downloadAsFile=false",
        args.api_base_url,
        timestamp
    );
    
    println!("Querying API for submissions after timestamp {}", timestamp);
    
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API request failed: {}", response.status()).into());
    }

    let api_response: ApiResponse = response.json().await?;
    
    if !api_response.data.is_empty() {
        if let Some(sample) = api_response.data.first() {
            let submitted_time = DateTime::from_timestamp(sample.submitted_at_timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| sample.submitted_at_timestamp.to_string());
            println!("Found new submission: {} (submitted at: {})", 
                     sample.sample_id, submitted_time);
        }
    }
    
    Ok(!api_response.data.is_empty())
}
