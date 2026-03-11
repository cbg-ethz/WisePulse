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

    /// Organism/virus identifier for the API endpoint (e.g., "covid", "rsva", "rsvb")
    /// This is appended to the API base URL: {api_base_url}/{organism}/sample/details
    #[arg(long, default_value = "covid")]
    organism: String,

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
    println!("Organism: {}", args.organism);

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
            let initial_timestamp =
                (Utc::now() - chrono::Duration::days(args.days_back)).timestamp();
            let initial_date = DateTime::from_timestamp(initial_timestamp, 0)
                .ok_or("Failed to create initial timestamp")?;

            println!(
                "Querying from: {}",
                initial_date.format("%Y-%m-%d %H:%M:%S UTC")
            );

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

/// Builds the URL for fetching new submissions from the LAPIS API.
///
/// # Arguments
/// * `api_base_url` - Base URL of the API (e.g., "https://api.db.wasap.genspectrum.org")
/// * `organism` - Organism identifier (e.g., "covid", "rsva")
/// * `timestamp` - Unix timestamp for submittedAtTimestampFrom filter
/// * `sampling_date_from` - Date string (YYYY-MM-DD) for samplingDateFrom filter
fn build_submissions_url(
    api_base_url: &str,
    organism: &str,
    timestamp: i64,
    sampling_date_from: &str,
) -> String {
    format!(
        "{}/{}/sample/details?submittedAtTimestampFrom={}&samplingDateFrom={}&dataFormat=JSON&downloadAsFile=false",
        api_base_url, organism, timestamp, sampling_date_from
    )
}

/// Builds the URL for fetching revocations from the LAPIS API.
///
/// # Arguments
/// * `api_base_url` - Base URL of the API
/// * `organism` - Organism identifier
/// * `timestamp` - Unix timestamp for submittedAtTimestampFrom filter
fn build_revocations_url(api_base_url: &str, organism: &str, timestamp: i64) -> String {
    format!(
        "{}/{}/sample/details?submittedAtTimestampFrom={}&isRevocation=true&dataFormat=JSON&downloadAsFile=false",
        api_base_url, organism, timestamp
    )
}

/// Calculates the maximum timestamp from an iterator of samples.
///
/// Returns `None` if the iterator is empty.
fn calculate_max_timestamp<'a>(samples: impl Iterator<Item = &'a SampleData>) -> Option<i64> {
    samples.map(|s| s.submitted_at_timestamp).max()
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
async fn check_for_data_changes(
    args: &Args,
    last_update: DateTime<Utc>,
) -> Result<(bool, Option<i64>)> {
    let client = Client::new();
    // Use strictly greater than logic to avoid infinite loop on identical max timestamp
    let timestamp = last_update.timestamp() + 1;

    // Calculate the sampling date range (rolling window)
    let now = Utc::now();
    let sampling_date_from = (now - chrono::Duration::days(args.days_back))
        .format("%Y-%m-%d")
        .to_string();

    println!(
        "Querying API for changes after {}",
        last_update.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  (submittedAtTimestampFrom: {})", timestamp);

    // Call 1: Get new submissions within the rolling window
    let submissions_url = build_submissions_url(
        &args.api_base_url,
        &args.organism,
        timestamp,
        &sampling_date_from,
    );

    println!(
        "  Fetching new submissions in rolling window: {} to now ({} days)",
        sampling_date_from, args.days_back
    );
    let submissions_response = client
        .get(&submissions_url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !submissions_response.status().is_success() {
        return Err(format!(
            "New submissions API request failed: {}",
            submissions_response.status()
        )
        .into());
    }

    let submissions_data: ApiResponse = submissions_response.json().await?;

    // Call 2: Get all revocations since last update
    let revocations_url = build_revocations_url(&args.api_base_url, &args.organism, timestamp);

    println!("  Fetching revocations since last update");
    let revocations_response = client
        .get(&revocations_url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !revocations_response.status().is_success() {
        return Err(format!(
            "Revocations API request failed: {}",
            revocations_response.status()
        )
        .into());
    }

    let revocations_data: ApiResponse = revocations_response.json().await?;

    // Combine and analyze results
    let new_submissions_count = submissions_data.data.len();
    let revocations_count = revocations_data.data.len();
    let total_changes = new_submissions_count + revocations_count;
    let has_data = total_changes > 0;

    // Calculate max timestamp from both datasets (no cloning needed)
    let max_timestamp = calculate_max_timestamp(
        submissions_data
            .data
            .iter()
            .chain(revocations_data.data.iter()),
    );

    // Log summary
    if new_submissions_count > 0 {
        println!(
            "Found {} new submission(s) in rolling window (samplingDate: {} to now)",
            new_submissions_count, sampling_date_from
        );
    }
    if revocations_count > 0 {
        println!(
            "Found {} revocation(s) since last update (submittedAtTimestamp >= {})",
            revocations_count, timestamp
        );
    }
    if has_data {
        println!(
            "Total: {} changes detected - pipeline should run",
            total_changes
        );

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_submissions_url() {
        let url =
            build_submissions_url("https://api.example.org", "covid", 1700000000, "2024-01-01");
        assert_eq!(
            url,
            "https://api.example.org/covid/sample/details?submittedAtTimestampFrom=1700000000&samplingDateFrom=2024-01-01&dataFormat=JSON&downloadAsFile=false"
        );
    }

    #[test]
    fn test_build_submissions_url_rsva() {
        let url = build_submissions_url(
            "https://api.db.wasap.genspectrum.org",
            "rsva",
            1700000000,
            "2024-06-15",
        );
        assert!(url.contains("/rsva/sample/details"));
        assert!(url.contains("submittedAtTimestampFrom=1700000000"));
        assert!(url.contains("samplingDateFrom=2024-06-15"));
    }

    #[test]
    fn test_build_revocations_url() {
        let url = build_revocations_url("https://api.example.org", "covid", 1700000000);
        assert_eq!(
            url,
            "https://api.example.org/covid/sample/details?submittedAtTimestampFrom=1700000000&isRevocation=true&dataFormat=JSON&downloadAsFile=false"
        );
    }

    #[test]
    fn test_build_revocations_url_rsvb() {
        let url = build_revocations_url("https://api.example.org", "rsvb", 1600000000);
        assert!(url.contains("/rsvb/sample/details"));
        assert!(url.contains("isRevocation=true"));
    }

    #[test]
    fn test_calculate_max_timestamp_empty() {
        let samples: Vec<SampleData> = vec![];
        assert_eq!(calculate_max_timestamp(samples.iter()), None);
    }

    #[test]
    fn test_calculate_max_timestamp_single() {
        let samples = [SampleData {
            sample_id: Some("test1".to_string()),
            submitted_at_timestamp: 1700000000,
            version_status: None,
            version_comment: None,
        }];
        assert_eq!(calculate_max_timestamp(samples.iter()), Some(1700000000));
    }

    #[test]
    fn test_calculate_max_timestamp_multiple() {
        let samples = [
            SampleData {
                sample_id: Some("test1".to_string()),
                submitted_at_timestamp: 1700000000,
                version_status: None,
                version_comment: None,
            },
            SampleData {
                sample_id: Some("test2".to_string()),
                submitted_at_timestamp: 1700000500,
                version_status: None,
                version_comment: None,
            },
            SampleData {
                sample_id: Some("test3".to_string()),
                submitted_at_timestamp: 1700000100,
                version_status: None,
                version_comment: None,
            },
        ];
        assert_eq!(calculate_max_timestamp(samples.iter()), Some(1700000500));
    }
}
