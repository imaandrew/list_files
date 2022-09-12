use clap::Parser;
use comfy_table::presets::ASCII_NO_BORDERS;
use comfy_table::Table;
use std::error::Error;
use std::path::PathBuf;
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
            let f_bytes = match std::fs::read(entry.path()) {
                Ok(bytes) => bytes,
                Err(_) => {
                    continue;
                }
            };
            format!("{:x}", md5::compute(f_bytes))
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
