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
    /// Base URL for the Loculus LAPIS API
    #[arg(long, default_value = "https://api.db.wasap.genspectrum.org")]
    api_base_url: String,

    /// Path to read last update timestamp from
    #[arg(long, default_value = ".last_update")]
    timestamp_file: String,

    /// Number of days back to check for sampling dates (rolling window)
    #[arg(long, default_value = "90")]
    days_back: i64,

    /// Path to write the maximum submittedAtTimestamp found (for pipeline use)
    #[arg(long, default_value = ".next_timestamp")]
    output_timestamp_file: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Vec<SampleData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SampleData {
    sample_id: Option<String>,
    submitted_at_timestamp: i64,
    #[serde(default)]
    is_revocation: Option<bool>,
    #[serde(default)]
    version_status: Option<String>,
    #[serde(default)]
    version_comment: Option<String>,
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

            let (has_new_data, max_timestamp) = check_for_data_changes(&args, last_date).await?;

            if has_new_data {
                if let Some(max_ts) = max_timestamp {
                    // Write the max timestamp to file for the pipeline to use
                    fs::write(&args.output_timestamp_file, max_ts.to_string()).await?;
                    let max_dt = DateTime::from_timestamp(max_ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| max_ts.to_string());
                    println!("Max submission timestamp: {} ({})", max_ts, max_dt);
                    println!("Written to: {}", args.output_timestamp_file);
                }
                println!("✓ New data available!");
                println!("  Pipeline should run to fetch and process new sequences.");
            } else {
                println!("• No new data found.");
                println!("  Pipeline can skip this run.");
            }

            Ok(has_new_data)
        }
        None => {
            println!("No previous update timestamp found - first run.");
            println!("Creating initial timestamp from current time...");
            
            // For first run, use a timestamp far enough in the past to catch recent data
            // but query the API to get the actual max timestamp
            let initial_timestamp = (Utc::now() - chrono::Duration::days(args.days_back)).timestamp();
            let initial_date = DateTime::from_timestamp(initial_timestamp, 0)
                .ok_or("Failed to create initial timestamp")?;
            
            println!("Querying from: {}", initial_date.format("%Y-%m-%d %H:%M:%S UTC"));
            
            let (has_new_data, max_timestamp) = check_for_data_changes(&args, initial_date).await?;
            
            if has_new_data {
                if let Some(max_ts) = max_timestamp {
                    fs::write(&args.output_timestamp_file, max_ts.to_string()).await?;
                    let max_dt = DateTime::from_timestamp(max_ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| max_ts.to_string());
                    println!("Max submission timestamp: {} ({})", max_ts, max_dt);
                    println!("Written to: {}", args.output_timestamp_file);
                }
                println!("✓ Data available - pipeline should fetch initial data.");
            } else {
                println!("• No data found in rolling window.");
            }
            
            Ok(has_new_data)
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
    let datetime = DateTime::from_timestamp(timestamp, 0).ok_or("Invalid timestamp in file")?;

    Ok(Some(datetime))
}

/// Checks if there are any data changes (new submissions or revocations) after the given timestamp.
///
/// Makes two separate API calls:
/// 1. New submissions within the rolling window (uses samplingDateFrom filter)  
/// 2. All revocations since last update (revocations have no sampling date)
///
/// Returns `Ok((has_data, max_timestamp))` where:
/// - has_data: true if any relevant changes found
/// - max_timestamp: the maximum submittedAtTimestamp from the results (for updating the checkpoint)
async fn check_for_data_changes(args: &Args, last_update: DateTime<Utc>) -> Result<(bool, Option<i64>)> {
    let client = Client::new();
    let timestamp = last_update.timestamp();
    
    // Calculate the sampling date range (rolling window)
    let now = Utc::now();
    let sampling_date_from = (now - chrono::Duration::days(args.days_back))
        .format("%Y-%m-%d")
        .to_string();

    println!("Querying API for changes after {}", last_update.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("  (submittedAtTimestampFrom: {})", timestamp);

    // Call 1: Get new submissions within the rolling window
    let submissions_url = format!(
        "{}/covid/sample/details?submittedAtTimestampFrom={}&samplingDateFrom={}&dataFormat=JSON&downloadAsFile=false",
        args.api_base_url, timestamp, sampling_date_from
    );

    println!("  Fetching new submissions in rolling window: {} to now ({} days)", sampling_date_from, args.days_back);
    let submissions_response = client
        .get(&submissions_url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !submissions_response.status().is_success() {
        return Err(format!("New submissions API request failed: {}", submissions_response.status()).into());
    }

    let submissions_data: ApiResponse = submissions_response.json().await?;

    // Call 2: Get all revocations since last update
    let revocations_url = format!(
        "{}/covid/sample/details?submittedAtTimestampFrom={}&isRevocation=true&dataFormat=JSON&downloadAsFile=false",
        args.api_base_url, timestamp
    );

    println!("  Fetching revocations since last update");
    let revocations_response = client
        .get(&revocations_url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !revocations_response.status().is_success() {
        return Err(format!("Revocations API request failed: {}", revocations_response.status()).into());
    }

    let revocations_data: ApiResponse = revocations_response.json().await?;

    // Combine and analyze results
    let new_submissions_count = submissions_data.data.len();
    let revocations_count = revocations_data.data.len();
    let total_changes = new_submissions_count + revocations_count;
    let has_data = total_changes > 0;

    // Calculate max timestamp from both datasets
    let mut max_timestamp: Option<i64> = None;
    
    for sample in submissions_data.data.iter().chain(revocations_data.data.iter()) {
        max_timestamp = Some(max_timestamp.map_or(sample.submitted_at_timestamp, |max| {
            max.max(sample.submitted_at_timestamp)
        }));
    }

    // Log summary
    if new_submissions_count > 0 {
        println!("Found {} new submission(s) in rolling window (samplingDate: {} to now)", new_submissions_count, sampling_date_from);
    }
    if revocations_count > 0 {
        println!("Found {} revocation(s) since last update (submittedAtTimestamp >= {})", revocations_count, timestamp);
    }
    if has_data {
        println!("Total: {} changes detected - pipeline should run", total_changes);
        
        // Log sample details (first few from each category)
        log_sample_details(&submissions_data.data, "New submissions", false);
        log_sample_details(&revocations_data.data, "Revocations", true);
    } else {
        println!("No new submissions or revocations found");
    }

    Ok((has_data, max_timestamp))
}

/// Helper function to log sample details in a consistent format
fn log_sample_details(samples: &[SampleData], category: &str, is_revocation_category: bool) {
    if samples.is_empty() {
        return;
    }
    
    println!("  {} details:", category);
    for (i, sample) in samples.iter().enumerate().take(3) {
        let sample_id = sample.sample_id.as_deref().unwrap_or("<unknown sample id>");
        
        if is_revocation_category {
            let status = sample
                .version_status
                .as_deref()
                .map(|s| format!(" [status: {}]", s))
                .unwrap_or_default();
            let comment = sample
                .version_comment
                .as_deref()
                .map(|c| format!(" - {}", c))
                .unwrap_or_default();
            println!("    [{}] {}{}{}", i + 1, sample_id, status, comment);
        } else {
            let status = sample
                .version_status
                .as_deref()
                .map(|s| format!(" [status: {}]", s))
                .unwrap_or_default();
            println!("    [{}] {}{}", i + 1, sample_id, status);
        }
    }
    
    if samples.len() > 3 {
        println!("    ... and {} more", samples.len() - 3);
    }
}
