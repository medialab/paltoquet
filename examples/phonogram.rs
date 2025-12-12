use std::fs::File;

use clap::Parser;
use paltoquet::phonetics::phonogram;

#[derive(Parser, Debug)]
struct Args {
    /// Path to target CSV file
    #[arg(long)]
    path: Option<String>,

    /// Name to test
    name: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(path) = &args.path {
        let file = File::open(path)?;
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
    } else {
        for name in args.name {
            println!("{} => {}", &name, phonogram(&name));
        }
    }

    Ok(())
}
