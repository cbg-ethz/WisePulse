use reqwest;
use tokio;
use serde::{Deserialize};
use chrono::{NaiveDate, Duration};
use std::{collections::HashMap, f32::consts::E};

const MAX_READS: u64 = 100_000_000;
const MAX_WEEKS: i64 = 6;

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
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client: reqwest::Client = reqwest::Client::new();

    let start_date = chrono::Local::now().date_naive();
    let mut current_date = start_date;
    let earliest_allowed = start_date - Duration::weeks(MAX_WEEKS);

    let mut stats = ProcessingStats {
        total_reads: 0,
        total_files: 0,
        date_range_days: 0,
        earliest_date: None, 
        latest_date: None,
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

    print_final_summary(&stats, &all_files);

    // let url = format!(
    //     "https://api.db.wasap.genspectrum.org/covid/sample/details?samplingDateFrom={}&samplingDateTo={}&limit=1000&dataFormat=JSON&downloadAsFile=false",
    //     SAMPLE_FROM_DATE, SAMPLE_TO_DATE
    // );

    // println!("Fetching data from: {}", url); 

    // let response: reqwest::Response = client
    //     .get(url)
    //     .header("Accept", "application/json")
    //     .send()
    //     .await?;

    // println!("Response Status: {}", response.status());

    // let api_response: ApiResponse = response.json().await?;

    // print!("Fetched {} records\n", api_response.data.len());

    // for (i, sample) in api_response.data.iter().enumerate() {

    //     println!("Sample #{}", i + 1);
    //     println!("  Sample ID: {}", sample.sample_id);
    //     println!("  Sampling Date: {}", sample.sampling_date);
    //     println!("  Read Count: {}", sample.count_silo_reads);

    //     let silo_files: Vec<SiloFile> = serde_json::from_str(&sample.silo_reads)?;

    //     println!("  Files:");
    //     for file in silo_files {
    //         println!(" {}", file.name);
    //         println!(" {}", file.url);
    //     }
    //     println!()
    // }


    Ok(())
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

fn print_final_summary(stats: &ProcessingStats, files: &[(String, String, NaiveDate, u64)]) {
    println!();
    println!("FINAL SUMMARY");
    println!("================");
    println!("Total reads: {}", stats.total_reads);
    println!("Total files: {}", files.len());
    println!("Date range: {} days", stats.date_range_days);
    if let (Some(earliest), Some(latest)) = (stats.earliest_date, stats.latest_date) {
        println!("From {} to {}", earliest, latest);
    }
    println!();
    
    println!(" Files to download:");
    for (name, _url, date, reads) in files.iter().take(5) { // Show first 5
        println!("   {} ({}) - {} reads", name, date, reads);
    }
    if files.len() > 5 {
        println!("  ... and {} more files", files.len() - 5);
    }
}