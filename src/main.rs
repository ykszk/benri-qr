use clap::Parser;
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use benri_qr::{MeCard, QrEncode};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file
    input: PathBuf,

    /// Minimum width
    #[clap(short, long, default_value_t = 128)]
    width: u32,
    /// Minimum height
    #[clap(short, long, default_value_t = 128)]
    height: u32,
}

fn read_mecard_from_file(path: &Path) -> Result<MeCard, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let card = serde_json::from_reader(reader)?;
    Ok(card)
}

fn main() {
    let args = Args::parse();
    let light = svg::Color("transparent");
    let dark = svg::Color("black");
    match args.input.extension().unwrap().to_str().unwrap() {
        "json" => {
            let card = read_mecard_from_file(args.input.as_path()).unwrap();
            let image = card.svg(args.width, args.height, light, dark);
            println!("{}", image);
        }
        "xlsx" => {
            let cards = MeCard::from_excel(args.input.as_path()).unwrap();
            MeCard::print_html(&cards, "title", "ja", args.width, args.height, light, dark);
        }
        _ => {
            eprintln!("Invalid file format.")
        }
    }
}
