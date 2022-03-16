use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
static CSS: &str = include_str!("default.css");
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct MeCard {
    pub Name: String,
    pub Reading: Option<String>,
    pub TEL: Option<String>,
    pub EMail: Option<String>,
    pub Memo: Option<String>,
    pub Birthday: Option<String>,
    pub Address: Option<String>,
    pub URL: Option<String>,
    pub Nickname: Option<String>,
}

pub trait QrEncode: Sized
where
    for<'de> Self: Deserialize<'de>,
{
    fn encode(&self) -> String;
    fn display(&self) -> String;
    fn svg(&self, width: u32, height: u32, light: svg::Color, dark: svg::Color) -> String {
        let code = QrCode::with_error_correction_level(self.encode(), EcLevel::L).unwrap();
        code.render()
            .min_dimensions(width, height)
            .dark_color(dark)
            .light_color(light)
            .build()
    }
    fn from_excel(path: &Path) -> Result<Vec<Self>, calamine::Error> {
        let mut workbook: Xlsx<_> = open_workbook(path)?;
        let sheet_name = workbook.sheet_names()[0].to_owned();
        let range = workbook
            .worksheet_range(&sheet_name)
            .ok_or(calamine::Error::Msg("Cannot find 'Sheet1'"))??;

        let iter = RangeDeserializerBuilder::new().from_range(&range)?;
        let cards: Vec<Self> = iter.map(|row| row.unwrap()).collect();
        Ok(cards)
    }
    fn print_html(
        cards: &Vec<Self>,
        title: &str,
        lang: &str,
        width: u32,
        height: u32,
        light: svg::Color,
        dark: svg::Color,
    ) {
        println!(
            "<html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{}</title><style>",
            lang, title
        );
        println!("{}", CSS);
        println!("</style></head>\n<body>");
        for card in cards {
            print!("<div><figure><figcaption>");
            print!("{}", card.display());
            println!("</figcaption>");
            println!("{}", card.svg(width, height, light, dark,));
            println!("</figure></div>");
        }
        println!("</body></html>");
    }
}

impl QrEncode for MeCard {
    fn encode(&self) -> String {
        let mut fields = Vec::with_capacity(9);
        fields.push(("N", &self.Name));
        for (name, opt) in [
            ("SOUND", &self.Reading),
            ("TEL", &self.TEL),
            ("EMAIL", &self.EMail),
            ("NOTE", &self.Memo),
            ("BDAY", &self.Birthday),
            ("ADR", &self.Address),
            ("URL", &self.URL),
            ("NICKNAME", &self.Nickname),
        ] {
            if let Some(val) = opt {
                fields.push((name, val));
            };
        }
        let code = fields
            .iter()
            .map(|(name, val)| std::format!("{}:{};", name, val))
            .collect::<Vec<String>>()
            .join("");
        String::from("MECARD:") + &code
    }
    fn display(&self) -> String {
        self.Name.clone()
    }
}
