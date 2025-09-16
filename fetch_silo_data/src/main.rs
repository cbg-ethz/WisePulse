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
use clap::Parser;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt, time};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(version, about = "Fetch SILO genomic data from LAPIS API", long_about = None)]
struct Args {
    /// Maximum number of reads to accumulate before stopping
    #[arg(long, default_value_t = 100_000_000)]
    max_reads: u64,

    /// Maximum weeks to go back in time
    #[arg(long, default_value_t = 6)]
    max_weeks: i64,

    /// Output directory for downloaded files
    #[arg(long, default_value = "silo_input")]
    output_dir: PathBuf,

    /// API base URL
    #[arg(long, default_value = "https://api.db.wasap.genspectrum.org")]
    api_base_url: String,
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
    date_range_days: i64,
    earliest_date: Option<NaiveDate>,
    latest_date: Option<NaiveDate>,
    downloaded_files: u32,
    download_errors: u32,
    duplicate_samples: u32,
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
    
    println!("SILO Data Fetcher - WisePulse Pipeline");
    println!("======================================");
    println!("Output directory: {}", args.output_dir.display());
    println!("Max reads: {}", args.max_reads);
    println!("Max weeks back: {}", args.max_weeks);
    println!();

    fs::create_dir_all(&args.output_dir).await?;

    let client = Client::new();
    let start_date = chrono::Local::now().date_naive();
    let mut current_date = start_date;
    let earliest_allowed = start_date - Duration::weeks(args.max_weeks);

    let mut stats = ProcessingStats::default();
    let mut all_files = Vec::new();

    println!("Starting data collection from {}", start_date);

    while current_date >= earliest_allowed {
        println!("Processing date: {}", current_date);

        let samples = fetch_samples_for_single_date(&client, &args.api_base_url, current_date).await?;

        if samples.is_empty() {
            println!("   No samples found for this date");
        } else {
            println!("   Found {} samples", samples.len());

            let (date_files, date_reads, duplicates) = process_samples_for_date(&samples, current_date)?;

            if stats.total_reads + date_reads > args.max_reads {
                println!("   Reached max reads limit ({} + {} > {}). Stopping.", 
                         stats.total_reads, date_reads, args.max_reads);
                break;
            }

            all_files.extend(date_files);
            stats.total_reads += date_reads;
            stats.duplicate_samples += duplicates;

            if stats.latest_date.is_none() {
                stats.latest_date = Some(current_date);
            }
            stats.earliest_date = Some(current_date);

            println!("   Added {} unique samples, {} reads (total: {} reads)", 
                     samples.len() - duplicates as usize, date_reads, stats.total_reads);
        }

        current_date = current_date - Duration::days(1);
        time::sleep(time::Duration::from_millis(50)).await;
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
    output_dir: &Path,
) -> Result<()> {
    for (i, file) in files.iter().enumerate() {
        println!("[{}/{}] Downloading: {}", i + 1, files.len(), file.name);

        match download_single_file(client, &file.name, &file.url, output_dir).await {
            Ok(bytes_downloaded) => {
                stats.downloaded_files += 1;
                println!("   Success: {} bytes", bytes_downloaded);
            }
            Err(e) => {
                stats.download_errors += 1;
                println!("   Failed: {}", e);
            }
        }
    }
    Ok(())
}

async fn download_single_file(
    client: &Client,
    filename: &str,
    url: &str,
    output_dir: &Path,
) -> Result<u64> {
    let file_path = output_dir.join(filename);

    if file_path.exists() {
        let metadata = fs::metadata(&file_path).await?;
        println!("   Already exists ({} bytes)", metadata.len());
        return Ok(metadata.len());
    }

    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP {} for {}", response.status(), filename).into());
    }

    let content = response.bytes().await?;
    let bytes_downloaded = content.len() as u64;

    let temp_path = file_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path).await?;
    file.write_all(&content).await?;
    file.sync_all().await?;
    drop(file);
    
    fs::rename(temp_path, file_path).await?;
    Ok(bytes_downloaded)
}

async fn fetch_samples_for_single_date(
    client: &Client,
    api_base_url: &str,
    date: NaiveDate,
) -> Result<Vec<SampleData>> {
    let url = format!(
        "{}/covid/sample/details?samplingDate={}&dataFormat=JSON&downloadAsFile=false",
        api_base_url,
        date.format("%Y-%m-%d")
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

fn process_samples_for_date(
    samples: &[SampleData],
    date: NaiveDate,
) -> Result<(Vec<FileToDownload>, u64, u32)> {
    let mut files = Vec::new();
    let mut total_reads = 0u64;
    let mut seen_sample_ids = HashSet::new();
    let mut duplicates = 0u32;

    for sample in samples {
        if !seen_sample_ids.insert(sample.sample_id.clone()) {
            println!("   WARNING: Duplicate sample_id found: {}, skipping", sample.sample_id);
            duplicates += 1;
            continue;
        }

        let read_count: u64 = sample.count_silo_reads.parse()?;
        let actual_date = sample.sampling_date.parse::<NaiveDate>()?;
        total_reads += read_count;

        println!("   Sample ID: {} ({} reads, sampled: {})", 
                 sample.sample_id, read_count, actual_date);

        let silo_files: Vec<SiloFile> = serde_json::from_str(&sample.silo_reads)?;

        for file in silo_files {
            println!("     -> File: {}", file.name);
            files.push(FileToDownload {
                sample_id: sample.sample_id.clone(),
                name: file.name,
                url: file.url,
                date: actual_date,
                read_count,
            });
        }
    }

    if duplicates > 0 {
        println!("   Found {} duplicate sample_ids (skipped)", duplicates);
    }

    Ok((files, total_reads, duplicates))
}

fn print_collection_summary(stats: &ProcessingStats, files: &[FileToDownload]) {
    println!();
    println!("COLLECTION SUMMARY");
    println!("==================");
    println!("Total reads: {}", stats.total_reads);
    println!("Total files found: {}", files.len());
    println!("Date range: {} days", stats.date_range_days);
    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        println!("From {} to {}", earliest, latest);
    }
    if stats.duplicate_samples > 0 {
        println!("Duplicate samples skipped: {}", stats.duplicate_samples);
    }
    println!();
    
    println!("Sample files:");
    for (name, _url, date, reads) in files.iter().take(3).map(|f| (&f.name, &f.url, f.date, f.read_count)) {
        println!("   {} ({}, {} reads)", name, date, reads);
    }
    if files.len() > 3 {
        println!("   ... and {} more files", files.len() - 3);
    }
}

fn print_final_summary(stats: &ProcessingStats, output_dir: &Path) {
    println!();
    println!("FINAL SUMMARY");
    println!("=============");
    println!("Files downloaded: {}", stats.downloaded_files);
    println!("Download errors: {}", stats.download_errors);
    println!("Files saved to: {}/", output_dir.display());
    println!();
    
    if stats.download_errors == 0 {
        println!("All files downloaded successfully!");
        println!("Ready for processing with: make");
    } else {
        println!("Some downloads failed. Check the logs above.");
        println!("You can still proceed with: make");
    }
}