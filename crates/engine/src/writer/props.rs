//! The `Document` props of the tree JSON as [`DocInfo`].

use serde_json::{Map, Value};

use super::crypt::{Encryption, Permissions, Printing};
use super::info::{DocInfo, PageLayout, PageMode, PdfVersion};
use super::pdfa::Conformance;
use super::strings::Date;

/// Reads `title`, `author`, `subject`, `keywords`, `creator` and `producer` (both `axonpdf` when
/// left out), `language`, `creationDate` and `modificationDate` (ISO 8601 strings, as
/// `JSON.stringify` writes a `Date`), `pdfVersion` (`"1.3"` to `"1.7ext3"`), `pageMode`,
/// `pageLayout`, `conformance` (`"PDF/A-2b"`...), `userPassword`, `ownerPassword`,
/// `permissions` (the components' object), `encryptionSeed` (32 random bytes as an array of
/// numbers, added by the adapter) and `compress` (`false` writes the streams undeflated).
pub fn doc_info(props: &Map<String, Value>) -> DocInfo {
    let text = |k: &str| {
        props
            .get(k)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    let date = |k: &str| props.get(k).and_then(Value::as_str).and_then(iso_date);
    let encryption =
        (text("userPassword").is_some() || text("ownerPassword").is_some()).then(|| Encryption {
            user_password: text("userPassword"),
            owner_password: text("ownerPassword"),
            permissions: props
                .get("permissions")
                .map(permissions)
                .unwrap_or_default(),
            seed: props.get("encryptionSeed").and_then(seed),
        });
    DocInfo {
        title: text("title"),
        author: text("author"),
        subject: text("subject"),
        keywords: text("keywords"),
        creator: text("creator").or_else(|| Some("axonpdf".into())),
        producer: text("producer").or_else(|| Some("axonpdf".into())),
        creation_date: date("creationDate"),
        modification_date: date("modificationDate"),
        language: text("language"),
        pdf_version: match text("pdfVersion").as_deref() {
            Some("1.4") => PdfVersion::V1_4,
            Some("1.5") => PdfVersion::V1_5,
            Some("1.6") => PdfVersion::V1_6,
            Some("1.7") => PdfVersion::V1_7,
            Some("1.7ext3") => PdfVersion::V1_7Ext3,
            _ => PdfVersion::V1_3,
        },
        page_mode: text("pageMode").and_then(|m| match m.as_str() {
            "useNone" => Some(PageMode::UseNone),
            "useOutlines" => Some(PageMode::UseOutlines),
            "useThumbs" => Some(PageMode::UseThumbs),
            "fullScreen" => Some(PageMode::FullScreen),
            "useOC" => Some(PageMode::UseOC),
            "useAttachments" => Some(PageMode::UseAttachments),
            _ => None,
        }),
        page_layout: text("pageLayout").and_then(|l| match l.as_str() {
            "singlePage" => Some(PageLayout::SinglePage),
            "oneColumn" => Some(PageLayout::OneColumn),
            "twoColumnLeft" => Some(PageLayout::TwoColumnLeft),
            "twoColumnRight" => Some(PageLayout::TwoColumnRight),
            "twoPageLeft" => Some(PageLayout::TwoPageLeft),
            "twoPageRight" => Some(PageLayout::TwoPageRight),
            _ => None,
        }),
        encryption,
        conformance: text("conformance").and_then(|c| Conformance::parse(&c)),
        uncompressed: props.get("compress") == Some(&Value::Bool(false)),
    }
}

fn permissions(v: &Value) -> Permissions {
    let on = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
    Permissions {
        printing: match v.get("printing").and_then(Value::as_str) {
            Some("lowResolution") => Printing::LowResolution,
            Some("highResolution") => Printing::HighResolution,
            _ => Printing::None,
        },
        modifying: on("modifying"),
        copying: on("copying"),
        annotating: on("annotating"),
        filling_forms: on("fillingForms"),
        content_accessibility: on("contentAccessibility"),
        document_assembly: on("documentAssembly"),
    }
}

fn seed(v: &Value) -> Option<[u8; 32]> {
    let bytes: Vec<u8> = v
        .as_array()?
        .iter()
        .map(|b| b.as_u64().map(|b| b as u8))
        .collect::<Option<_>>()?;
    bytes.try_into().ok()
}

/// `YYYY-MM-DDTHH:MM:SS[.sss](Z|±HH:MM)`.
fn iso_date(s: &str) -> Option<Date> {
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<u16>().ok();
    let (year, month, day) = (n(0..4)?, n(5..7)? as u8, n(8..10)? as u8);
    let (hour, minute, second) = (
        n(11..13).unwrap_or(0) as u8,
        n(14..16).unwrap_or(0) as u8,
        n(17..19).unwrap_or(0) as u8,
    );
    let zone = s
        .get(19..)
        .unwrap_or("")
        .trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    let utc_offset_minutes = match zone.as_bytes().first() {
        Some(b'+' | b'-') => {
            let sign = if zone.starts_with('-') { -1 } else { 1 };
            let h: i16 = zone.get(1..3)?.parse().ok()?;
            let m: i16 = zone.get(4..6).and_then(|m| m.parse().ok()).unwrap_or(0);
            Some(sign * (h * 60 + m))
        }
        _ => None,
    };
    Some(Date {
        year,
        month,
        day,
        hour,
        minute,
        second,
        utc_offset_minutes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_document_props() {
        let props = serde_json::json!({
            "title": "T", "creationDate": "2026-11-21T06:30:05.123Z", "modificationDate": "2026-11-21T12:00:00+06:00",
            "pdfVersion": "1.7ext3", "pageMode": "useOutlines", "pageLayout": "twoPageLeft", "conformance": "PDF/A-2b",
            "ownerPassword": "o", "permissions": {"printing": "highResolution", "copying": true},
            "encryptionSeed": vec![9; 32], "compress": false,
        });
        let info = doc_info(props.as_object().unwrap());
        assert!(info.uncompressed);
        // Left out, the creator and producer are axonpdf.
        assert_eq!(
            (info.creator.as_deref(), info.producer.as_deref()),
            (Some("axonpdf"), Some("axonpdf"))
        );
        let named = doc_info(
            serde_json::json!({"creator": "c", "producer": "p"})
                .as_object()
                .unwrap(),
        );
        assert_eq!(
            (named.creator.as_deref(), named.producer.as_deref()),
            (Some("c"), Some("p"))
        );
        assert_eq!(info.creation_date.unwrap().pdf(), "D:20261121063005Z");
        assert_eq!(
            info.modification_date.unwrap().pdf(),
            "D:20261121120000+06'00'"
        );
        assert_eq!(
            (info.pdf_version, info.page_mode, info.page_layout),
            (
                PdfVersion::V1_7Ext3,
                Some(PageMode::UseOutlines),
                Some(PageLayout::TwoPageLeft)
            )
        );
        assert_eq!(info.conformance, Conformance::parse("PDF/A-2b"));
        let e = info.encryption.unwrap();
        assert_eq!(
            (e.user_password, e.owner_password.as_deref(), e.seed),
            (None, Some("o"), Some([9; 32]))
        );
        assert_eq!(
            (
                e.permissions.printing,
                e.permissions.copying,
                e.permissions.modifying
            ),
            (Printing::HighResolution, true, false)
        );
    }
}
