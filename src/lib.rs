use calamine::{RangeDeserializerBuilder, Reader, Xlsx};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
static CSS: &str = include_str!("default.css");

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
    fn from_excel<Reader: std::io::Read + std::io::Seek>(
        reader: Reader,
    ) -> Result<Vec<Self>, calamine::Error> {
        let mut workbook = Xlsx::new(reader)?; //<_> = open_workbook(path)?;
        let sheet_name = workbook.sheet_names()[0].to_owned();
        let range = workbook
            .worksheet_range(&sheet_name)
            .ok_or(calamine::Error::Msg("Cannot find a sheet"))??;

        let iter = RangeDeserializerBuilder::new().from_range(&range)?;
        let cards: Vec<Self> = iter.map(|row| row.unwrap()).collect();
        Ok(cards)
    }
    fn from_json(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let card = serde_json::from_reader(reader)?;
        Ok(card)
    }
    fn write_html<Writer: std::io::Write>(
        writer: &mut Writer,
        cards: &Vec<Self>,
        title: &str,
        lang: &str,
        width: u32,
        height: u32,
        light: svg::Color,
        dark: svg::Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(
            writer,
            "<html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{}</title><style>",
            lang, title
        )?;
        writeln!(writer, "{}", CSS)?;
        writeln!(writer, "</style></head>\n<body>")?;
        for card in cards {
            write!(writer, "<div><figure><figcaption>")?;
            write!(writer, "{}", card.display())?;
            writeln!(writer, "</figcaption>")?;
            writeln!(writer, "{}", card.svg(width, height, light, dark,))?;
            writeln!(writer, "</figure></div>")?;
        }
        writeln!(writer, "</body></html>")?;
        Ok(())
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

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn xlsx2html(xlsx: &[u8], title: &str) -> Result<String, JsValue> {
    let cursor = std::io::Cursor::new(xlsx);
    let buf = BufReader::new(cursor);
    let cards = MeCard::from_excel(buf).map_err(|e| JsValue::from(e.to_string()))?;
    let mut writer = BufWriter::new(Vec::new());
    let light = svg::Color("transparent");
    let dark = svg::Color("black");
    MeCard::write_html(&mut writer, &cards, title, "ja", 128, 128, light, dark)
        .map_err(|e| JsValue::from(e.to_string()))?;

    let bytes = writer
        .into_inner()
        .map_err(|e| JsValue::from(e.to_string()))?;
    let html = String::from_utf8(bytes).map_err(|e| JsValue::from(e.to_string()))?;
    Ok(html)
}
