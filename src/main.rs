use benri_qr::{MeCard, QrEncode};
use clap::Parser;
use qrcode::render::svg;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let light = svg::Color("transparent");
    let dark = svg::Color("black");
    match args
        .input
        .extension()
        .ok_or("input path error")?
        .to_str()
        .ok_or("input path error")?
    {
        "json" => {
            let card = MeCard::from_json(args.input.as_path())?;
            let image = card.svg(args.width, args.height, light, dark)?;
            println!("{}", image);
        }
        "xlsx" => {
            let file = File::open(&args.input)?;
            let reader = BufReader::new(file);
            let cards = MeCard::from_excel(reader)?;
            let title = if let Some(title) = args.title {
                title
            } else {
                args.input
                    .file_stem()
                    .ok_or("input path error")?
                    .to_str()
                    .ok_or("input path error")?
                    .into()
            };
            let stdout = std::io::stdout();
            let mut writer = std::io::BufWriter::new(stdout.lock());
            MeCard::write_html(
                &mut writer,
                &cards,
                &title,
                &args.lang,
                (args.width, args.height),
                light,
                dark,
            )?;
        }
        _ => {
            eprintln!("Invalid file format.")
        }
    }
    Ok(())
}
