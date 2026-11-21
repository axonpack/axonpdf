//! Document settings, the info dictionary and the catalog.

use pdf_writer::{Finish, Name, Ref, Str};

use super::Writer;
use super::emit::Out;
use super::nav::NavRefs;
use super::strings::{Date, text};

/// Everything about the document as a whole. The caller fills in the creation date; the engine
/// has no clock.
#[derive(Clone, Debug, Default)]
pub struct DocInfo {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<Date>,
    pub modification_date: Option<Date>,
    /// BCP 47 language tag (`Lang`).
    pub language: Option<String>,
    pub pdf_version: PdfVersion,
    pub page_mode: Option<PageMode>,
    pub page_layout: Option<PageLayout>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PdfVersion {
    #[default]
    V1_3,
    V1_4,
    V1_5,
    V1_6,
    V1_7,
    /// 1.7 with Adobe extension level 3 (AES-256 encryption).
    V1_7Ext3,
}

impl PdfVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            PdfVersion::V1_3 => "1.3",
            PdfVersion::V1_4 => "1.4",
            PdfVersion::V1_5 => "1.5",
            PdfVersion::V1_6 => "1.6",
            PdfVersion::V1_7 | PdfVersion::V1_7Ext3 => "1.7",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageMode {
    UseNone,
    UseOutlines,
    UseThumbs,
    FullScreen,
    UseOC,
    UseAttachments,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageLayout {
    SinglePage,
    OneColumn,
    TwoColumnLeft,
    TwoColumnRight,
    TwoPageLeft,
    TwoPageRight,
}

impl Writer {
    pub(super) fn write_info(&mut self, out: Out) -> Ref {
        let id = self.emit.alloc();
        let i = &self.info;
        let mut entries: Vec<(&[u8], Vec<u8>)> = Vec::new();
        let texts: [(&[u8], &Option<String>); 6] = [
            (b"Title", &i.title),
            (b"Author", &i.author),
            (b"Subject", &i.subject),
            (b"Keywords", &i.keywords),
            (b"Creator", &i.creator),
            (b"Producer", &i.producer),
        ];
        for (key, value) in texts {
            if let Some(v) = value {
                entries.push((key, self.seal(id, text(v))));
            }
        }
        for (key, date) in [
            (b"CreationDate".as_slice(), i.creation_date),
            (b"ModDate", i.modification_date),
        ] {
            if let Some(d) = date {
                entries.push((key, self.seal(id, d.pdf().into_bytes())));
            }
        }
        self.emit.write(out, |c| {
            let mut dict = c.indirect(id).dict();
            for (key, value) in &entries {
                dict.pair(Name(key), Str(value));
            }
        });
        id
    }

    pub(super) fn write_catalog(&mut self, nav: NavRefs, out: Out) {
        let (catalog, pages) = (self.catalog, self.pages);
        let lang = self
            .info
            .language
            .as_ref()
            .map(|l| self.seal(catalog, text(l)));
        let page_mode = self
            .info
            .page_mode
            .map(page_mode_name)
            .or(nav.outline.map(|_| "UseOutlines"));
        let page_layout = self.info.page_layout.map(page_layout_name);
        self.emit.write(out, |c| {
            let mut cat = c.indirect(catalog).dict();
            cat.pair(Name(b"Type"), Name(b"Catalog"));
            cat.pair(Name(b"Pages"), pages);
            if let Some(outline) = nav.outline {
                cat.pair(Name(b"Outlines"), outline);
            }
            if let Some(form) = nav.form {
                cat.pair(Name(b"AcroForm"), form);
            }
            if let Some(dests) = nav.dests {
                cat.insert(Name(b"Names"))
                    .dict()
                    .pair(Name(b"Dests"), dests);
            }
            if let Some(lang) = &lang {
                cat.pair(Name(b"Lang"), Str(lang));
            }
            if let Some(mode) = page_mode {
                cat.pair(Name(b"PageMode"), Name(mode.as_bytes()));
            }
            if let Some(layout) = page_layout {
                cat.pair(Name(b"PageLayout"), Name(layout.as_bytes()));
            }
            cat.insert(Name(b"ViewerPreferences"))
                .dict()
                .pair(Name(b"DisplayDocTitle"), true);
            cat.finish();
        });
    }
}

fn page_mode_name(mode: PageMode) -> &'static str {
    match mode {
        PageMode::UseNone => "UseNone",
        PageMode::UseOutlines => "UseOutlines",
        PageMode::UseThumbs => "UseThumbs",
        PageMode::FullScreen => "FullScreen",
        PageMode::UseOC => "UseOC",
        PageMode::UseAttachments => "UseAttachments",
    }
}

fn page_layout_name(layout: PageLayout) -> &'static str {
    match layout {
        PageLayout::SinglePage => "SinglePage",
        PageLayout::OneColumn => "OneColumn",
        PageLayout::TwoColumnLeft => "TwoColumnLeft",
        PageLayout::TwoColumnRight => "TwoColumnRight",
        PageLayout::TwoPageLeft => "TwoPageLeft",
        PageLayout::TwoPageRight => "TwoPageRight",
    }
}
