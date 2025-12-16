//! SILO Data Fetcher - WisePulse Genomic Data Pipeline
//!
//! Fetches genomic sample data from the LAPIS API, working backwards in time
//! from the current date. Downloads .ndjson.zst files containing sequencing reads.
//!
//! Key behaviors:
//! - Assumes sample_id uniqueness within each date
//! - Deduplicates samples by sample_id, warns about duplicates
//! - Stops when read count limit or time limit is reached
//! - Atomic file downloads with resume capability
//! - Uses actual sampling_date from API for data integrity
//!
//! Integration: Downloads to silo_input/ for processing by existing WisePulse pipeline

use chrono::{Duration, NaiveDate};
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt, time};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(name = "fetch_silo_data")]
#[command(about = "Fetches genomic data files from LAPIS API")]
struct Args {
    /// Start date for fetching (YYYY-MM-DD format)
    #[arg(long)]
    start_date: NaiveDate,

    /// Number of days to fetch backwards
    #[arg(long)]
    days: i64,

    /// Maximum number of reads to fetch
    #[arg(long)]
    max_reads: u64,

    /// Output directory for downloaded files
    #[arg(long)]
    output_dir: String,

    /// Base URL for the LAPIS API
    #[arg(long)]
    api_base_url: String,

    /// Organism/virus identifier for the API endpoint (e.g., "covid", "rsva", "rsvb")
    #[arg(long, default_value = "covid")]
    organism: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    data: Vec<SampleData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SampleData {
    sample_id: String,
    sampling_date: String,
    count_silo_reads: String,
    silo_reads: String,
}

#[derive(Deserialize, Debug)]
struct SiloFile {
    name: String,
    url: String,
}

#[derive(Debug, Default)]
struct ProcessingStats {
    total_reads: u64,
    total_files: u32,
    date_range_days: i64,
    earliest_date: Option<NaiveDate>,
    latest_date: Option<NaiveDate>,
    downloaded_files: u32,
    download_errors: u32,
}

#[derive(Debug)]
struct FileToDownload {
    sample_id: String,
    name: String,
    url: String,
    date: NaiveDate,
    read_count: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    run_fetch(&args).await
}

async fn run_fetch(args: &Args) -> Result<()> {
    let client = Client::new();

    // Print starting banner
    println!("___ WisePulse SILO Data Fetcher ___");
    println!("Fetching genomic data from LAPIS API");
    println!();

    fs::create_dir_all(&args.output_dir).await?;
    println!("Configuration:");
    println!("  Output directory: {}", args.output_dir);
    println!("  API base URL: {}", args.api_base_url);
    println!("  Organism: {}", args.organism);

    let start_date = args.start_date;
    let earliest_allowed = start_date - Duration::days(args.days);

    let mut stats = ProcessingStats::default();
    let mut all_files = Vec::<FileToDownload>::new();

    println!("  Start date: {}", start_date);
    println!(
        "  Date range: {} -> {} (max {} days)",
        start_date, earliest_allowed, args.days
    );
    println!("  Max reads: {}", args.max_reads);
    println!();
    println!("Starting data collection...");
    println!();

    let mut current_date = start_date;
    let mut days_processed = 0;
    let mut consecutive_empty_days = 0;
    let total_days_to_check = (start_date - earliest_allowed).num_days() + 1;

    while current_date >= earliest_allowed {
        days_processed += 1;
        let progress = (days_processed as f32 / total_days_to_check as f32 * 100.0) as u32;

        let samples = fetch_samples_for_single_date(
            &client,
            current_date,
            &args.api_base_url,
            &args.organism,
        )
        .await?;

        if samples.is_empty() {
            consecutive_empty_days += 1;

            // Only show individual empty days for the first few, then summarize
            if consecutive_empty_days <= 3 {
                println!(
                    "Processing date: {} - No samples found ({}%)",
                    current_date, progress
                );
            } else if consecutive_empty_days == 4 {
                println!(
                    "Processing date: {} - No samples found ({}%)",
                    current_date, progress
                );
                println!("   Continuing to check dates (will summarize empty streaks)...");
            }
            // For days 5+ with no samples, we'll just count them silently
        } else {
            // If we had a streak of empty days, summarize them
            if consecutive_empty_days > 3 {
                let days_to_summarize = consecutive_empty_days - 3; // Don't count the first 3 we already showed
                println!("   Checked {} additional empty days", days_to_summarize);
            }
            consecutive_empty_days = 0;

            println!(
                "Processing date: {} - SUCCESS ({}%)",
                current_date, progress
            );
            println!("   Found {} samples", samples.len());

            let date_files = process_samples_for_date(&samples, current_date)?;
            let date_reads: u64 = date_files.iter().map(|f| f.read_count).sum();

            if stats.total_reads + date_reads > args.max_reads {
                println!("   Would exceed read limit, stopping");
                break;
            }

            // Update stats
            stats.total_reads += date_reads;
            stats.total_files += date_files.len() as u32;

            // Track date range
            if stats.latest_date.is_none() {
                stats.latest_date = Some(current_date);
            }
            stats.earliest_date = Some(current_date);

            println!(
                "   Added {} files, {} reads (total: {})",
                date_files.len(),
                date_reads,
                stats.total_reads
            );

            all_files.extend(date_files);
        }

        current_date -= Duration::days(1);
        time::sleep(time::Duration::from_millis(100)).await;
    }

    // Show final summary of empty days if we ended on a streak
    if consecutive_empty_days > 3 {
        let days_to_summarize = consecutive_empty_days - 3;
        println!("   Final {} empty days checked", days_to_summarize);
    }

    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        stats.date_range_days = (latest - earliest).num_days() + 1;
    }

    print_collection_summary(&stats, &all_files);

    println!();
    println!("Starting file downloads...");
    download_all_files(&client, &all_files, &mut stats, &args.output_dir).await?;

    print_final_summary(&stats, &args.output_dir);
    Ok(())
}

async fn download_all_files(
    client: &Client,
    files: &[FileToDownload],
    stats: &mut ProcessingStats,
    output_dir: &str,
) -> Result<()> {
    for (i, file) in files.iter().enumerate() {
        let progress = ((i + 1) as f32 / files.len() as f32 * 100.0) as u32;
        println!(
            "[{}/{}] Downloading: {} ({}%)",
            i + 1,
            files.len(),
            file.name,
            progress
        );

        match download_single_file(client, &file.name, &file.url, output_dir).await {
            Ok(bytes) => {
                stats.downloaded_files += 1;
                let size_mb = bytes as f64 / 1024.0 / 1024.0;
                println!("   Success: {:.1} MB (sample: {})", size_mb, file.sample_id);
            }
            Err(e) => {
                stats.download_errors += 1;
                println!("   Failed: {} (sample: {})", e, file.sample_id);
            }
        }

        time::sleep(time::Duration::from_millis(100)).await;
    }
    Ok(())
}

async fn download_single_file(
    client: &Client,
    filename: &str,
    url: &str,
    output_dir: &str,
) -> Result<u64> {
    let file_path = Path::new(output_dir).join(filename);

    // Skip if file already exists
    if file_path.exists() {
        let metadata = fs::metadata(&file_path).await?;
        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
        println!("   Already exists ({:.1} MB)", size_mb);
        return Ok(metadata.len());
    }

    // Download the file
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), filename).into());
    }

    let content = response.bytes().await?;
    let bytes_downloaded = content.len() as u64;

    // Write to file atomically
    let temp_path = file_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path).await?;
    file.write_all(&content).await?;
    file.sync_all().await?;
    drop(file);

    fs::rename(temp_path, file_path).await?;
    Ok(bytes_downloaded)
}

/// Builds the URL for fetching samples for a specific date from the LAPIS API.
///
/// # Arguments
/// * `api_base_url` - Base URL of the API (e.g., "https://api.db.wasap.genspectrum.org")
/// * `organism` - Organism identifier (e.g., "covid", "rsva")
/// * `date` - The sampling date to query
fn build_samples_url(api_base_url: &str, organism: &str, date: NaiveDate) -> String {
    let date_str = date.format("%Y-%m-%d");
    format!(
        "{}/{}/sample/details?samplingDate={}&dataFormat=JSON&downloadAsFile=false",
        api_base_url, organism, date_str
    )
}

async fn fetch_samples_for_single_date(
    client: &Client,
    date: NaiveDate,
    api_base_url: &str,
    organism: &str,
) -> Result<Vec<SampleData>> {
    let url = build_samples_url(api_base_url, organism, date);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API request failed: {}", response.status()).into());
    }

    let api_response: ApiResponse = response.json().await?;
    Ok(api_response.data)
}

fn process_samples_for_date(
    samples: &[SampleData],
    current_date: NaiveDate,
) -> Result<Vec<FileToDownload>> {
    let mut files = Vec::new();
    let mut sample_map = std::collections::HashMap::new();
    let mut duplicates_found = 0;

    // First pass: collect all samples, keeping the latest occurrence of each sample_id
    for sample in samples {
        let read_count: u64 = sample.count_silo_reads.parse()?;
        let actual_date = sample.sampling_date.parse::<NaiveDate>()?;

        if current_date != actual_date {
            println!(
                "   WARNING: Sampling date mismatch for sample_id {}: expected {}, got {}",
                sample.sample_id, current_date, actual_date
            );
        }

        // Check if this sample_id was already seen
        if sample_map.contains_key(&sample.sample_id) {
            duplicates_found += 1;
            println!(
                "   Sample ID: {} ({} reads, sampled: {}) - REPLACING PREVIOUS",
                sample.sample_id, read_count, actual_date
            );
        } else {
            println!(
                "   Sample ID: {} ({} reads, sampled: {})",
                sample.sample_id, read_count, actual_date
            );
        }

        // Always insert/replace - this keeps the latest occurrence
        sample_map.insert(sample.sample_id.clone(), sample);
    }

    // Second pass: process the deduplicated samples
    for (sample_id, sample) in sample_map {
        let read_count: u64 = sample.count_silo_reads.parse()?;
        let actual_date = sample.sampling_date.parse::<NaiveDate>().map_err(|e| {
            format!(
                "Failed to parse sampling_date '{}': {}",
                sample.sampling_date, e
            )
        })?;

        let silo_files: Vec<SiloFile> = serde_json::from_str(&sample.silo_reads)?;

        for file in silo_files {
            println!("     -> File: {}", file.name);
            files.push(FileToDownload {
                sample_id: sample_id.clone(),
                name: file.name,
                url: file.url,
                date: actual_date,
                read_count,
            });
        }
    }

    if duplicates_found > 0 {
        println!(
            "   Found {} duplicate sample_ids (kept latest version)",
            duplicates_found
        );
    }

    Ok(files)
}

fn print_collection_summary(stats: &ProcessingStats, files: &[FileToDownload]) {
    println!();
    println!("COLLECTION SUMMARY");
    println!("------------------");
    println!("Total reads: {}", stats.total_reads);
    println!("Files found: {}", files.len());

    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        let days = (latest - earliest).num_days() + 1;
        println!("Date range: {} days ({} to {})", days, earliest, latest);
    }

    if !files.is_empty() {
        println!();
        println!("Sample files:");
        for file in files.iter().take(3) {
            let reads_millions = file.read_count as f64 / 1_000_000.0; // Number of reads in millions
            println!(
                "   {} [{}] ({}, ~{:.1}M reads)",
                file.name, file.sample_id, file.date, reads_millions
            );
        }
        if files.len() > 3 {
            println!("   ... and {} more files", files.len() - 3);
        }
    }
}

fn print_final_summary(stats: &ProcessingStats, output_dir: &str) {
    println!();
    println!("FINAL SUMMARY");
    println!("--------------");
    println!("Downloaded: {}", stats.downloaded_files);

    if stats.download_errors > 0 {
        println!("Errors: {}", stats.download_errors);
    }

    println!("Location: {}/", output_dir);

    if stats.download_errors == 0 && stats.downloaded_files > 0 {
        println!();
        println!("All files downloaded successfully!");
    } else if stats.download_errors > 0 {
        println!();
        println!("Some downloads failed. Check the logs above for details.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_samples_url() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let url = build_samples_url("https://api.example.org", "covid", date);
        assert_eq!(
            url,
            "https://api.example.org/covid/sample/details?samplingDate=2024-06-15&dataFormat=JSON&downloadAsFile=false"
        );
    }

    #[test]
    fn test_build_samples_url_rsva() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let url = build_samples_url("https://api.db.wasap.genspectrum.org", "rsva", date);
        assert!(url.contains("/rsva/sample/details"));
        assert!(url.contains("samplingDate=2024-12-01"));
    }

    #[test]
    fn test_process_samples_deduplication() {
        // Test that duplicate sample_ids are deduplicated (keeping the last one)
        let samples = vec![
            SampleData {
                sample_id: "sample1".to_string(),
                sampling_date: "2024-06-15".to_string(),
                count_silo_reads: "1000".to_string(),
                silo_reads: r#"[{"name": "file1.ndjson.zst", "url": "http://example.com/file1"}]"#
                    .to_string(),
            },
            SampleData {
                sample_id: "sample1".to_string(), // duplicate - this one should be kept
                sampling_date: "2024-06-15".to_string(),
                count_silo_reads: "2000".to_string(), // different read count
                silo_reads:
                    r#"[{"name": "file1_v2.ndjson.zst", "url": "http://example.com/file1_v2"}]"#
                        .to_string(),
            },
            SampleData {
                sample_id: "sample2".to_string(),
                sampling_date: "2024-06-15".to_string(),
                count_silo_reads: "500".to_string(),
                silo_reads: r#"[{"name": "file2.ndjson.zst", "url": "http://example.com/file2"}]"#
                    .to_string(),
            },
        ];

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let files = process_samples_for_date(&samples, date).unwrap();

        // Should have 2 files (sample1 deduplicated, sample2 kept)
        assert_eq!(files.len(), 2);

        // Verify the "keep last" behavior: sample1 should have read_count 2000 (from the second entry)
        let sample1_file = files.iter().find(|f| f.sample_id == "sample1").unwrap();
        assert_eq!(
            sample1_file.read_count, 2000,
            "Deduplication should keep the last occurrence (read_count 2000, not 1000)"
        );

        // Verify sample2 is also present
        let sample2_file = files.iter().find(|f| f.sample_id == "sample2").unwrap();
        assert_eq!(sample2_file.read_count, 500);
    }

    #[test]
    fn test_process_samples_multiple_files_per_sample() {
        let samples = vec![SampleData {
            sample_id: "sample1".to_string(),
            sampling_date: "2024-06-15".to_string(),
            count_silo_reads: "1000".to_string(),
            silo_reads: r#"[
                {"name": "file1a.ndjson.zst", "url": "http://example.com/file1a"},
                {"name": "file1b.ndjson.zst", "url": "http://example.com/file1b"}
            ]"#
            .to_string(),
        }];

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let files = process_samples_for_date(&samples, date).unwrap();

        // Should have 2 files from the same sample
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.sample_id == "sample1"));
    }

    #[test]
    fn test_process_samples_empty() {
        let samples: Vec<SampleData> = vec![];
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let files = process_samples_for_date(&samples, date).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_process_samples_read_count_parsing() {
        let samples = vec![SampleData {
            sample_id: "sample1".to_string(),
            sampling_date: "2024-06-15".to_string(),
            count_silo_reads: "12345678".to_string(),
            silo_reads: r#"[{"name": "file1.ndjson.zst", "url": "http://example.com/file1"}]"#
                .to_string(),
        }];

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let files = process_samples_for_date(&samples, date).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].read_count, 12345678);
    }
}
