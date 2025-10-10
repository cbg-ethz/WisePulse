//! Update Timestamp - WisePulse Data Pipeline
//!
//! Updates the .last_update timestamp to mark successful pipeline completion.
//! 
//! The timestamp represents "last time we successfully checked for updates"
//! rather than "most recent data timestamp" because:
//! - Data can be added/modified retroactively (old dates, recent submissions)
//! - Revocations have recent submission times but reference old data
//! - We use submittedAtTimestampFrom to catch ALL changes since last check
//!
//! Only call this after the pipeline completes successfully.

use chrono::Utc;
use std::env;
use std::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let timestamp_file = if args.len() > 1 {
        &args[1]
    } else {
        ".last_update"
    };

    let now = Utc::now();
    let timestamp = now.timestamp();

    fs::write(timestamp_file, timestamp.to_string())?;
    println!(
        "Updated timestamp to: {}",
        now.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("File: {}", timestamp_file);

    Ok(())
}
