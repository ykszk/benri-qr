use benri_qr::{MeCard, QrEncode};
use clap::Parser;
use qrcode::render::svg;
use std::path::PathBuf;

static ABOUT: &str =
    "Generate QrCode for contact information. Input file with the following fields.
Name, Reading, TEL, Email, Memo, Birthday, Address, URL, Nickname
Any fields but <Name> are optional.";
#[derive(Parser, Debug)]
#[clap(author, version, about = ABOUT)]
struct Args {
    /// Input file, json or xlsx
    input: PathBuf,
    /// Minimum width
    #[clap(long, default_value_t = 128)]
    width: u32,
    /// Minimum height
    #[clap(long, default_value_t = 128)]
    height: u32,
    /// Html title
    title: Option<String>,
    #[clap(long, default_value = "ja")]
    /// Html lang
    lang: String,
}

fn main() {
    let args = Args::parse();
    let light = svg::Color("transparent");
    let dark = svg::Color("black");
    match args.input.extension().unwrap().to_str().unwrap() {
        "json" => {
            let card = MeCard::from_json(args.input.as_path()).unwrap();
            let image = card.svg(args.width, args.height, light, dark);
            println!("{}", image);
        }
        "xlsx" => {
            let cards = MeCard::from_excel(args.input.as_path()).unwrap();
            let title = if let Some(title) = args.title {
                title
            } else {
                args.input.file_stem().unwrap().to_str().unwrap().into()
            };
            let stdout = std::io::stdout();
            let mut writer = std::io::BufWriter::new(stdout.lock());
            MeCard::write_html(
                &mut writer,
                &cards,
                &title,
                &args.lang,
                args.width,
                args.height,
                light,
                dark,
            ).unwrap();
        }
        _ => {
            eprintln!("Invalid file format.")
        }
    }
}
