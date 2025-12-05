use std::fs::File;

use clap::Parser;
use paltoquet::phonetics::phonogram;

#[derive(Parser, Debug)]
struct Args {
    /// Path to target CSV file
    path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file = File::open(args.path)?;
    let mut reader = simd_csv::Reader::from_reader(file);
    let mut writer = simd_csv::Writer::from_writer(std::io::stdout());

    let mut record = reader.byte_headers()?.clone();
    record.push_field(b"phonogram");

    writer.write_byte_record(&record)?;

    while reader.read_byte_record(&mut record)? {
        let code = phonogram(std::str::from_utf8(&record[0])?);
        record.push_field(code.as_bytes());

        writer.write_byte_record(&record)?;
    }

    writer.flush()?;

    Ok(())
}
