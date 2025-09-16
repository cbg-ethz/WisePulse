//! SILO Data Fetcher - WisePulse Genomic Data Pipeline
//!
//! Fetches COVID-19 genomic sample data from the LAPIS API, working backwards in time
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
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt, time};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const MAX_READS: u64 = 100_000_000;
const MAX_WEEKS: i64 = 6;
const OUTPUT_DIR: &str = "./silo_data_test";
const API_BASE_URL: &str = "https://api.db.wasap.genspectrum.org";

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
    let client = Client::new();
    
    fs::create_dir_all(OUTPUT_DIR).await?;
    println!("Output directory: {}", OUTPUT_DIR);

    let start_date = chrono::Local::now().date_naive();
    let earliest_allowed = start_date - Duration::weeks(MAX_WEEKS);

    let mut stats = ProcessingStats::default();
    let mut all_files = Vec::<FileToDownload>::new();

    println!("Starting data collection...");
    println!("Date range: {} -> {} (max {} weeks)", start_date, earliest_allowed, MAX_WEEKS);
    println!("Max reads: {}", MAX_READS);
    println!();

    let mut current_date = start_date;
    while current_date >= earliest_allowed {
        println!("Processing date: {}", current_date);

        let samples = fetch_samples_for_single_date(&client, current_date).await?;

        if samples.is_empty() {
            println!("   No samples found");
        } else {
            println!("   Found {} samples", samples.len());

            let date_files = process_samples_for_date(&samples)?;
            let date_reads: u64 = date_files.iter().map(|f| f.read_count).sum();

            if stats.total_reads + date_reads > MAX_READS {
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

            println!("   Added {} files, {} reads (total: {})", 
                     date_files.len(), date_reads, stats.total_reads);
            
            all_files.extend(date_files);
        }

        current_date = current_date - Duration::days(1);
        time::sleep(time::Duration::from_millis(100)).await;
    }
    
    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        stats.date_range_days = (latest - earliest).num_days() + 1;
    }

    print_collection_summary(&stats, &all_files);


    // Download files
    println!();
    println!("Starting file downloads...");
    download_all_files(&client, &all_files, &mut stats).await?;

    print_final_summary(&stats);
    Ok(())
}

async fn download_all_files(
    client: &Client,
    files: &[FileToDownload],
    stats: &mut ProcessingStats,
) -> Result<()> {
    for (i, file) in files.iter().enumerate() {
        println!("[{}/{}] Downloading: {} (sample: {})", 
                 i + 1, files.len(), file.name, file.sample_id);

        match download_single_file(client, &file.name, &file.url).await {
            Ok(bytes) => {
                stats.downloaded_files += 1;
                println!("   Success: {} bytes (sample: {})", bytes_downloaded, file.sample_id);
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

async fn download_single_file(client: &Client, filename: &str, url: &str) -> Result<u64> {
    let file_path = Path::new(OUTPUT_DIR).join(filename);

    // Skip if file already exists
    if file_path.exists() {
        let metadata = fs::metadata(&file_path).await?;
        println!("   Already exists ({} bytes)", metadata.len());
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


async fn fetch_samples_for_single_date(client: &Client, date: NaiveDate) -> Result<Vec<SampleData>> {
    let date_str = date.format("%Y-%m-%d");
    let url = format!(
        "{}/covid/sample/details?samplingDate={}&dataFormat=JSON&downloadAsFile=false",
        API_BASE_URL, date_str
    );

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

fn process_samples_for_date(samples: &[SampleData]) -> Result<Vec<FileToDownload>> {
    let mut files = Vec::new();
    let mut seen_sample_ids = HashSet::new();
    let mut duplicates_found = 0;

    for sample in samples {
        let read_count: u64 = sample.count_silo_reads.parse()?;
        let actual_date = sample.sampling_date.parse::<NaiveDate>()?;
        
        if date != actual_date {
            println!("   WARNING: Sampling date mismatch for sample_id {}: expected {}, got {}",
                     sample.sample_id, date, actual_date);
        }
        
        total_reads += read_count;

        println!("   Sample ID: {} ({} reads, sampled: {})", 
                 sample.sample_id, read_count, actual_date);

        let silo_files: Vec<SiloFile> = serde_json::from_str(&sample.silo_reads)?;
        
        // Parse the actual sampling date from the API
        let actual_date = sample.sampling_date.parse::<NaiveDate>()
            .map_err(|e| format!("Failed to parse sampling_date '{}': {}", sample.sampling_date, e))?;

        println!("   Sample ID: {} ({} reads, sampled: {})", sample.sample_id, read_count, actual_date);
        
        // Check for duplicate sample_id
        if seen_sample_ids.contains(&sample.sample_id) {
            duplicates_found += 1;
            println!("     WARNING: Duplicate sample_id found, skipping");
            continue;
        }
        
        seen_sample_ids.insert(sample.sample_id.clone());
        
        for file in silo_files {
            println!("     -> File: {}", file.name);
            files.push(FileToDownload {
                sample_id: sample.sample_id.clone(),
                name: file.name,
                url: file.url,
                date: actual_date, // Use the actual sampling date from API
                read_count,
            });
        }
    }
    
    if duplicates_found > 0 {
        println!("   Found {} duplicate sample_ids (skipped)", duplicates_found);
    }
    
    Ok(files)
}

fn print_collection_summary(stats: &ProcessingStats, files: &[FileToDownload]) {
    println!();
    println!("COLLECTION SUMMARY");
    println!("==================");
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
            println!("   {} [{}] ({}, {} reads)", file.name, file.sample_id, file.date, file.read_count);
        }
        if files.len() > 3 {
            println!("   ... and {} more files", files.len() - 3);
        }
    }
}

fn print_final_summary(stats: &ProcessingStats) {
    println!();
    println!("FINAL SUMMARY");
    println!("=============");
    println!("Downloaded: {}", stats.downloaded_files);
    
    if stats.download_errors > 0 {
        println!("Errors: {}", stats.download_errors);
    }
    
    println!("Location: {}/", OUTPUT_DIR);
    
    if stats.download_errors == 0 && stats.downloaded_files > 0 {
        println!();
        println!("All files downloaded successfully!");
    }
}