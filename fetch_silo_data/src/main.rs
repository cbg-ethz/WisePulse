use reqwest;
use tokio;
use serde::{Deserialize};
use chrono::{NaiveDate, Duration};
use std::collections::HashMap;

const MAX_READS: i64 = 1000000;
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

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await; 
    }
    

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