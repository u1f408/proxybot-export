use anyhow::Result;
use clap::Parser;
use std::{fs, path::PathBuf};
use serde_json::Value;

use proxybot_export_converter::ExportFormat;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short)]
    input: PathBuf,

    #[arg(short = 'o')]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let data: Value = serde_json::from_str(&fs::read_to_string(&args.input)?)?;

    let input = ExportFormat::detect(&data)?;
    let new_json = serde_json::to_string_pretty(&input.try_convert()?)?;

    if let Some(out_file) = args.output {
        fs::write(out_file, &new_json)?;
    } else {
        println!("{}", &new_json);
    }

    Ok(())
}
