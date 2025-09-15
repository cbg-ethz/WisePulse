use reqwest;
use tokio;
use serde::{Deserialize};


const SAMPLE_FROM_DATE: &str = "2025-06-15";
const SAMPLE_TO_DATE: &str = "2025-09-15";
const MAX_READS: u32 = 1000000;

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let url = format!(
        "https://api.db.wasap.genspectrum.org/covid/sample/details?samplingDateFrom={}&samplingDateTo={}&limit=1000&dataFormat=JSON&downloadAsFile=false",
        SAMPLE_FROM_DATE, SAMPLE_TO_DATE
    );

    println!("Fetching data from: {}", url); 

    let client: reqwest::Client = reqwest::Client::new();

    let response: reqwest::Response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?;

    println!("Response Status: {}", response.status());

    let api_response: ApiResponse = response.json().await?;

    print!("Fetched {} records\n", api_response.data.len());

    for (i, sample) in api_response.data.iter().enumerate() {

        println!("Sample #{}", i + 1);
        println!("  Sample ID: {}", sample.sample_id);
        println!("  Sampling Date: {}", sample.sampling_date);
        println!("  Read Count: {}", sample.count_silo_reads);

        let silo_files: Vec<SiloFile> = serde_json::from_str(&sample.silo_reads)?;

        println!("  Files:");
        for file in silo_files {
            println!(" {}", file.name);
            println!(" {}", file.url);
        }
        println!()
    }


    Ok(())
}