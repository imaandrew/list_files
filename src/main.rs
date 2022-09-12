use clap::Parser;
use comfy_table::presets::ASCII_NO_BORDERS;
use comfy_table::Table;
use md5::Context;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::{error::Error, fs::File};
use time::{format_description, OffsetDateTime, UtcOffset};
use walkdir::WalkDir;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directory to scan for files
    #[clap(value_parser)]
    path: PathBuf,

    /// File to write the output to
    #[clap(short, long, value_parser, value_name = "output")]
    output: Option<PathBuf>,

    /// Enable calculating file hashes
    #[clap(short, long, action)]
    md5: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut table = Table::new();
    table.load_preset(ASCII_NO_BORDERS).set_header(vec![
        "Path",
        "Size (KiB)",
        "Date Created",
        "Date Modified",
        "MD5 Hash",
    ]);

    if let Some(file) = &args.output {
        std::fs::create_dir_all(match file.parent() {
            Some(x) => x,
            None => {
                eprintln!("Cannot create output file: `{}`", file.to_string_lossy());
                std::process::exit(1);
            }
        })?;
        std::fs::write(file, "test")?;
    }

    for entry in WalkDir::new(args.path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.metadata()?.is_dir() {
            continue;
        }
        let f_name = entry.path().to_string_lossy();
        println!("Parsing: {}", f_name);
        let f_size = entry.metadata()?.len();
        let format = format_description::parse(
            "[year]-[month]-[day] [hour repr:12]:[minute]:[second] [period]",
        )?;
        let mut date: OffsetDateTime = entry.metadata()?.created()?.into();
        let offset = UtcOffset::local_offset_at(date)?;
        date = date.to_offset(offset);
        let f_date = date.format(&format).unwrap();

        let mut date: OffsetDateTime = entry.metadata()?.modified()?.into();
        let offset = UtcOffset::local_offset_at(date)?;
        date = date.to_offset(offset);
        let f_date_modified = date.format(&format).unwrap();

        let f_hash = if args.md5 {
            let file = File::open(entry.path())?;
            let mut reader = BufReader::with_capacity(64000, file);
            let mut md5 = Context::new();

            loop {
                let len = {
                    let buf = reader.fill_buf()?;
                    md5.consume(buf);
                    buf.len()
                };

                if len == 0 {
                    break;
                }
                reader.consume(len);
            }

            format!("{:x}", md5.compute())
        } else {
            "".to_string()
        };
        table.add_row(vec![
            f_name.to_string(),
            format!("{:.2}", f_size as f64 / 1024f64),
            f_date,
            f_date_modified,
            f_hash,
        ]);
    }

    if let Some(file) = args.output {
        std::fs::write(file, format!("{}", table))?;
    } else {
        println!("{}", table);
    }

    Ok(())
}
