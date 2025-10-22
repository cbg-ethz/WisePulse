use clap::Parser;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use zstd::stream::Encoder;

fn write_ndjson_lines<W: Write>(writer: &mut W, lines: &[Value]) -> std::io::Result<()> {
    let mut writer = BufWriter::new(writer);
    for line in lines {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}

fn sort_by(mut lines: Vec<Value>, sort_field_path: &String) -> Vec<Value> {
    lines.sort_by(|a, b| {
        let a_length = a
            .pointer(sort_field_path)
            .unwrap_or_else(|| panic!("Did not find field {sort_field_path} in object {a}"))
            .as_i64()
            .unwrap_or(0);
        let b_length = b
            .pointer(sort_field_path)
            .unwrap_or_else(|| panic!("Did not find field {sort_field_path} in object {b}"))
            .as_i64()
            .unwrap_or(0);
        a_length.cmp(&b_length)
    });
    lines
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    output_path: String,

    #[arg(long, default_value = "chunk")]
    filename_stem: String,

    #[arg(long)]
    sort_field_path: String,

    #[arg(long, default_value_t = 10000)]
    chunk_size: usize,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let output_path = Path::new(&args.output_path);

    if output_path.exists() {
        assert_eq!(
            fs::read_dir(output_path)?.count(),
            0,
            "The given output directory is not empty"
        );
    } else {
        fs::create_dir_all(output_path)?
    };

    let mut chunk_counter = 0;

    let reader = stdin();
    let reader = reader.lock();
    let mut lines = Vec::new();

    for line in BufReader::new(reader).lines() {
        let line = line?;
        let json: Value = serde_json::from_str(&line)?;
        lines.push(json);

        if lines.len() >= args.chunk_size {
            let sorted_lines = sort_by(lines, &args.sort_field_path);
            let chunk_file: PathBuf = Path::join(
                output_path,
                format!("{}_{}.ndjson.zst", args.filename_stem, chunk_counter),
            );
            let file = File::create(chunk_file.clone())?;
            let mut encoder = Encoder::new(file, 3)?;
            write_ndjson_lines(&mut encoder, &sorted_lines)?;
            encoder.finish()?;
            println!("{}", chunk_file.as_path().to_str().unwrap());
            lines = Vec::new();
            chunk_counter += 1;
        }
    }

    // Process any remaining lines
    if !lines.is_empty() {
        let sorted_lines = sort_by(lines, &args.sort_field_path);
        let chunk_file: PathBuf = Path::join(
            output_path,
            format!("{}_{}.ndjson.zst", args.filename_stem, chunk_counter),
        );
        let file = File::create(chunk_file.clone())?;
        let mut encoder = Encoder::new(file, 3)?;
        write_ndjson_lines(&mut encoder, &sorted_lines)?;
        encoder.finish()?;
        println!("{}", chunk_file.as_path().to_str().unwrap());
    }
    Ok(())
}
