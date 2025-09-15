use reqwest;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.db.wasap.genspectrum.org/covid/sample/details?samplingDateFrom=2025-06-15&samplingDateTo=2025-09-15&limit=100&dataFormat=JSON&downloadAsFile=false";

    println!("Fetching data from: {}", url); 

    Ok(())
}