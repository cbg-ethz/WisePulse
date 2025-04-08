use serde_json::Value;
use std::io::{self, BufRead, Write};

fn count_leading_ns(seq: &str) -> usize {
    seq.chars().take_while(|&c| c == 'N').count()
}

fn main() -> std::io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let reader = stdin.lock();
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = line?;
        let mut json_value: Value = serde_json::from_str(&line).expect("Invalid JSON format");

        if let Some(main_seq) = json_value.pointer("/alignedNucleotideSequences/main") {
            if let Some(main_seq_str) = main_seq.as_str() {
                let offset = count_leading_ns(main_seq_str);

                if let Value::Object(ref mut obj) = json_value {
                    obj.insert("offset".to_string(), Value::from(offset));
                }
            }
        }

        writeln!(writer, "{}", json_value.to_string())?;
    }

    Ok(())
}
