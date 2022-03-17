use calamine::{RangeDeserializerBuilder, Reader, Xlsx};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

static CSS: &str = include_str!("default.css");

/// MeCard data format.
///
/// `TEL-AV` omitted since it seems obsolete.
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

impl MeCard {
    pub fn new(
        name: String,
        reading: Option<String>,
        tel: Option<String>,
        email: Option<String>,
        memo: Option<String>,
        birthday: Option<String>,
        address: Option<String>,
        url: Option<String>,
        nickname: Option<String>,
    ) -> Self {
        Self {
            Name: name,
            Reading: reading,
            TEL: tel,
            EMail: email,
            Memo: memo,
            Birthday: birthday,
            Address: address,
            URL: url,
            Nickname: nickname,
        }
    }
    pub fn init(name: String) -> Self {
        Self {
            Name: name,
            Reading: None,
            TEL: None,
            EMail: None,
            Memo: None,
            Birthday: None,
            Address: None,
            URL: None,
            Nickname: None,
        }
    }
}

/// QrEncodable object.
/// Implement `encode` and `display` for custom types and use blanket implementations.
pub trait QrEncode: Sized
where
    for<'de> Self: Deserialize<'de>,
{
    /// Encode fields into string for QrCode
    fn encode(&self) -> String;
    /// Short descriptive name (e.g. Name in MeCard) used for `figcaption` in html
    fn display(&self) -> String;
    /// Create SVG string. Internally, `self.encode()` is converted into SVG.
    fn svg(
        &self,
        width: u32,
        height: u32,
        light: svg::Color,
        dark: svg::Color,
    ) -> Result<String, Box<dyn Error>> {
        let code = QrCode::with_error_correction_level(self.encode(), EcLevel::L)?;
        let qr = code
            .render()
            .min_dimensions(width, height)
            .dark_color(dark)
            .light_color(light)
            .build();
        Ok(qr)
    }
    /// Batch load from excel
    fn from_excel<Reader: std::io::Read + std::io::Seek>(
        reader: Reader,
    ) -> Result<Vec<Self>, calamine::Error> {
        let mut workbook = Xlsx::new(reader)?;
        let sheet_name = workbook.sheet_names()[0].to_owned();
        let range = workbook
            .worksheet_range(&sheet_name)
            .ok_or(calamine::Error::Msg("Cannot find a sheet"))??;
        let iter = RangeDeserializerBuilder::new().from_range(&range)?;
        let mut cards: Vec<Self> = Vec::with_capacity(range.height());
        for result in iter {
            cards.push(result?);
        }
        Ok(cards)
    }
    /// Load single instance
    fn from_json(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let card = serde_json::from_reader(reader)?;
        Ok(card)
    }
    fn write_html<Writer: std::io::Write>(
        writer: &mut Writer,
        cards: &[Self],
        title: &str,
        lang: &str,
        w_h: (u32, u32),
        light: svg::Color,
        dark: svg::Color,
    ) -> Result<(), Box<dyn Error>> {
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
            writeln!(writer, "{}", card.svg(w_h.0, w_h.1, light, dark)?)?;
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

/// Convert bunch of MeCard information from xlsx into an html file.
/// # Arguments
/// - xlsx: raw bytes of xlsx file
/// - title: title of html
/// - lang: `<html lang="lang">`
#[wasm_bindgen]
pub fn xlsx2html(xlsx: &[u8], title: &str, lang: &str) -> Result<String, JsValue> {
    let cursor = std::io::Cursor::new(xlsx);
    let buf = BufReader::new(cursor);
    let cards = MeCard::from_excel(buf).map_err(|e| JsValue::from(e.to_string()))?;
    let mut writer = BufWriter::new(Vec::new());
    let light = svg::Color("transparent");
    let dark = svg::Color("black");
    MeCard::write_html(&mut writer, &cards, title, lang, (128, 128), light, dark)
        .map_err(|e| JsValue::from(e.to_string()))?;

    let bytes = writer
        .into_inner()
        .map_err(|e| JsValue::from(e.to_string()))?;
    let html = String::from_utf8(bytes).map_err(|e| JsValue::from(e.to_string()))?;
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let mut card = MeCard::init(String::from("John"));
        card.TEL = Some("1234-5678".into());
        assert_eq!(card.encode(), "MECARD:N:John;TEL:1234-5678;");
        card.EMail = Some("john@example.com".into());
        assert_eq!(
            card.encode(),
            "MECARD:N:John;TEL:1234-5678;EMAIL:john@example.com;"
        );
    }
}
