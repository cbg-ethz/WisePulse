use reqwest;
use tokio;
use serde::{Deserialize};
use chrono::{NaiveDate, Duration};
use std::{collections::HashMap, f32::consts::E};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::path::Path;

const MAX_READS: u64 = 100_000_000;
const MAX_WEEKS: i64 = 6;
const OUTPUT_DIR: &str = "./silo_data_test";

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

#[derive(Debug)]
struct ProcessingStats {
    total_reads: u64,
    total_files: u32,
    date_range_days: i64,
    earliest_date: Option<NaiveDate>,
    latest_date: Option<NaiveDate>,
    downloaded_files: u32,
    download_errors: u32,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client: reqwest::Client = reqwest::Client::new();

    fs::create_dir_all(OUTPUT_DIR).await?;
    println!("Output directory: {}", OUTPUT_DIR);

    let start_date = chrono::Local::now().date_naive();
    let mut current_date = start_date;
    let earliest_allowed = start_date - Duration::weeks(MAX_WEEKS);

    let mut stats = ProcessingStats {
        total_reads: 0,
        total_files: 0,
        date_range_days: 0,
        earliest_date: None, 
        latest_date: None,
        downloaded_files: 0,
        download_errors: 0,
    };

    let mut all_files: Vec<(String, String, NaiveDate, u64)> = Vec::new(); // (name, url, date, read_count)

    println!("Starting dynamic fetching...");
    println!("Start date: {}", start_date);
    println!("Max reads: {}", MAX_READS);
    println!("Max weeks back: {}", MAX_WEEKS);
    println!();

    while current_date >= earliest_allowed {
        println!("Processing date: {}", current_date);

        current_date = current_date - Duration::days(1);

        let samples = fetch_samples_for_single_date(&client, current_date).await?;

        if samples.is_empty() {
            println!(" No samples found for this date.");
        } else {
            println!(" Found {} samples.", samples.len());
        

            let (date_files, date_reads) = process_samples_for_date(&samples, current_date)?;

            if stats.total_reads + date_reads > MAX_READS {
                println!("Reached max reads limit. Stopping.");
                break;
            }
            
            all_files.extend(date_files);
            stats.total_reads += date_reads;
            stats.total_files = all_files.len() as u32;

            if stats.latest_date.is_none() {
                stats.latest_date = Some(current_date);
            }
            stats.earliest_date = Some(current_date);

            println!("   Added {} files, {} reads (total: {} reads)", 
                        samples.len(), date_reads, stats.total_reads);
        }

        current_date = current_date - Duration::days(1);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await; 
    }
    
    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        stats.date_range_days = (latest - earliest).num_days() + 1;
    }

    print_collection_summary(&stats, &all_files);


    println!("");
    println!(" Starting file downloads...");
    download_all_files(&client, &all_files, &mut stats).await?;


    print_final_summary(&stats);

    Ok(())
}


async fn download_all_files(
    client: &reqwest::Client, 
    files: &[(String, String, NaiveDate, u64)],
    stats: &mut ProcessingStats,
) -> Result<(), Box<dyn std::error::Error>> {
    for (i, (filename, url, date, reads)) in files.iter().enumerate() {
        println!(" [{}/{}] Downloading: {} ", i+1, files.len(), filename);

        match download_single_file(client, filename, url).await {
            Ok(bytes_downloaded) => {
                stats.downloaded_files += 1;
                println!("    Downloaded {} bytes", bytes_downloaded);
            }
            Err(e) => {
                stats.download_errors += 1;
                println!("    Error downloading {}: {}", filename, e);
            }
        }
    }
    Ok(())
}

async fn download_single_file(
    client: &reqwest::Client,
    filename: &str,
    url: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    let file_path = Path::new(OUTPUT_DIR).join(filename);
    
    // Skip if file already exists
    if file_path.exists() {
        let metadata = fs::metadata(&file_path).await?;
        println!("     File already exists ({} bytes)", metadata.len());
        return Ok(metadata.len());
    }
    
    // Download the file
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let content = response.bytes().await?;
    let bytes_downloaded = content.len() as u64;
    
    // Write to file
    let mut file = fs::File::create(&file_path).await?;
    file.write_all(&content).await?;
    
    Ok(bytes_downloaded)
}


async fn fetch_samples_for_single_date(
    client: &reqwest::Client, 
    date: NaiveDate
) -> Result<Vec<SampleData>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.db.wasap.genspectrum.org/covid/sample/details?samplingDateFrom={}&samplingDateTo={}&dataFormat=JSON&downloadAsFile=false",
        date.format("%Y-%m-%d"),
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
    date: NaiveDate
) -> Result<(Vec<(String, String, NaiveDate, u64)>, u64), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut total_reads: u64 = 0;

    for sample in samples {
        let read_count: u64 = sample.count_silo_reads.parse()?;
        total_reads += read_count;

        let silo_files: Vec<SiloFile> = serde_json::from_str(&sample.silo_reads)?;

        for file in silo_files {
            files.push((file.name, file.url, date, read_count));
        }
    }
    Ok((files, total_reads))
}

fn print_collection_summary(stats: &ProcessingStats, files: &[(String, String, NaiveDate, u64)]) {
    println!();
    println!(" COLLECTION SUMMARY");
    println!("====================");
    println!(" Total reads: {}", stats.total_reads);
    println!(" Total files found: {}", files.len());
    println!(" Date range: {} days", stats.date_range_days);
    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        println!(" From {} to {}", earliest, latest);
    }
    println!();
    
    println!(" Files to download:");
    for (name, _url, date, reads) in files.iter().take(3) {
        println!("    {} ({}) - {} reads", name, date, reads);
    }
    if files.len() > 3 {
        println!("   ... and {} more files", files.len() - 3);
    }
}

fn print_final_summary(stats: &ProcessingStats) {
    println!();
    println!(" FINAL SUMMARY");
    println!("================");
    println!(" Files downloaded: {}", stats.downloaded_files);
    println!(" Download errors: {}", stats.download_errors);
    println!(" Files saved to: {}/", OUTPUT_DIR);
    println!();
    
    if stats.download_errors == 0 {
        println!(" All files downloaded successfully!");
        println!(" Ready for processing with: make");
    } else {
        println!("  Some downloads failed. Check the logs above.");
    }
}