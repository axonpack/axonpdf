//! PDF/A: the XMP metadata naming the part and level, and an sRGB output intent. Text in the
//! Standard 14 fonts is never embedded, so a conforming file uses registered fonts only.

use pdf_writer::{Name, Ref, Str};

use super::Writer;
use super::emit::Out;

/// A compact sRGB profile (CC0, vendor/icc/).
const SRGB: &[u8] = include_bytes!("../../../../vendor/icc/sRGB-v2-micro.icc");

/// `PDF/A-<part><level>`, as in `PDF/A-2b`. Level is `A`, `B` or `U`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conformance {
    pub part: u8,
    pub level: char,
}

impl Conformance {
    /// Reads `PDF/A-1`, `PDF/A-1a`, `PDF/A-2b`, ... A missing level is `B`.
    pub fn parse(s: &str) -> Option<Conformance> {
        let rest = s.strip_prefix("PDF/A-")?;
        let mut chars = rest.chars();
        let part = chars.next()?.to_digit(10)? as u8;
        let level = chars.next().map_or('B', |c| c.to_ascii_uppercase());
        (matches!(part, 1..=3) && matches!(level, 'A' | 'B' | 'U') && chars.next().is_none())
            .then_some(Conformance { part, level })
    }
}

/// What the catalog points to.
pub struct PdfaRefs {
    pub metadata: Ref,
    pub intent_profile: Ref,
}

impl Writer {
    pub(super) fn write_pdfa(&mut self, out: Out) -> Option<PdfaRefs> {
        let c = self.info.conformance?;
        let metadata = self.emit.alloc();
        let xmp = self.seal(metadata, xmp(self, c).into_bytes());
        self.emit.write(out, |chunk| {
            chunk.metadata(metadata, &xmp);
        });
        let profile = self.emit.alloc();
        let data = self.seal(profile, SRGB.to_vec());
        self.emit.write(out, |chunk| {
            chunk.icc_profile(profile, &data).n(3);
        });
        Some(PdfaRefs {
            metadata,
            intent_profile: profile,
        })
    }
}

/// The catalog's `Metadata` and `OutputIntents`; `id` is the sealed profile name.
pub fn catalog_entries(cat: &mut pdf_writer::Dict, refs: &PdfaRefs, id: &[u8]) {
    cat.pair(Name(b"Metadata"), refs.metadata);
    let mut list = cat.insert(Name(b"OutputIntents")).array();
    let mut intent = list.push().dict();
    intent.pair(Name(b"Type"), Name(b"OutputIntent"));
    intent.pair(Name(b"S"), Name(b"GTS_PDFA1"));
    intent.pair(Name(b"OutputConditionIdentifier"), Str(id));
    intent.pair(Name(b"Info"), Str(id));
    intent.pair(Name(b"DestOutputProfile"), refs.intent_profile);
}

fn xmp(w: &Writer, c: Conformance) -> String {
    let i = &w.info;
    let esc = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };
    let mut body = format!(
        "<rdf:Description rdf:about=\"\" xmlns:pdfaid=\"http://www.aiim.org/pdfa/ns/id/\">\
         <pdfaid:part>{}</pdfaid:part><pdfaid:conformance>{}</pdfaid:conformance></rdf:Description>\n",
        c.part, c.level
    );
    body += "<rdf:Description rdf:about=\"\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:format>application/pdf</dc:format>";
    let alt = |v: &str| {
        format!(
            "<rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt>",
            esc(v)
        )
    };
    if let Some(t) = &i.title {
        body += &format!("<dc:title>{}</dc:title>", alt(t));
    }
    if let Some(a) = &i.author {
        body += &format!(
            "<dc:creator><rdf:Seq><rdf:li>{}</rdf:li></rdf:Seq></dc:creator>",
            esc(a)
        );
    }
    if let Some(s) = &i.subject {
        body += &format!("<dc:description>{}</dc:description>", alt(s));
    }
    body += "</rdf:Description>\n<rdf:Description rdf:about=\"\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">";
    if let Some(t) = &i.creator {
        body += &format!("<xmp:CreatorTool>{}</xmp:CreatorTool>", esc(t));
    }
    if let Some(d) = i.creation_date {
        body += &format!("<xmp:CreateDate>{}</xmp:CreateDate>", d.iso());
    }
    if let Some(d) = i.modification_date {
        body += &format!("<xmp:ModifyDate>{}</xmp:ModifyDate>", d.iso());
    }
    body += "</rdf:Description>\n<rdf:Description rdf:about=\"\" xmlns:pdf=\"http://ns.adobe.com/pdf/1.3/\">";
    if let Some(p) = &i.producer {
        body += &format!("<pdf:Producer>{}</pdf:Producer>", esc(p));
    }
    if let Some(k) = &i.keywords {
        body += &format!("<pdf:Keywords>{}</pdf:Keywords>", esc(k));
    }
    body += "</rdf:Description>\n";
    format!(
        "<?xpacket begin=\"\u{FEFF}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n\
         <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n{body}</rdf:RDF>\n</x:xmpmeta>\n<?xpacket end=\"w\"?>"
    )
}
