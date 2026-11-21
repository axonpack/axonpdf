//! The file identifier, the encryption dictionary and the trailer.

use md5::{Digest, Md5};
use pdf_writer::{Finish, Name, Ref, Str};

use super::Writer;
use super::emit::Out;
use super::info::DocInfo;

/// A digest of the document's info: the same document gets the same identifier.
pub fn file_id(info: &DocInfo) -> [u8; 16] {
    let fields = [
        &info.title,
        &info.author,
        &info.subject,
        &info.keywords,
        &info.creator,
        &info.producer,
    ];
    let mut m = Md5::new();
    for f in fields {
        m.update(f.as_deref().unwrap_or("").as_bytes());
        m.update([0]);
    }
    m.update(
        info.creation_date
            .map(|d| d.pdf())
            .unwrap_or_default()
            .as_bytes(),
    );
    m.finalize().into()
}

impl Writer {
    pub(super) fn write_encrypt(&mut self, out: Out) -> Option<Ref> {
        let c = self.crypt.as_ref()?;
        let r = self.emit.alloc();
        let mut chunk = pdf_writer::Chunk::new();
        let mut d = chunk.indirect(r).dict();
        d.pair(Name(b"Filter"), Name(b"Standard"));
        d.pair(Name(b"V"), c.v as i32).pair(Name(b"R"), c.r as i32);
        if c.v >= 2 {
            d.pair(Name(b"Length"), if c.v == 5 { 256 } else { 128 });
        }
        d.pair(Name(b"O"), Str(&c.o))
            .pair(Name(b"U"), Str(&c.u))
            .pair(Name(b"P"), c.p);
        if c.v >= 4 {
            let mut cf = d.insert(Name(b"CF")).dict();
            let mut std_cf = cf.insert(Name(b"StdCF")).dict();
            std_cf.pair(Name(b"AuthEvent"), Name(b"DocOpen"));
            std_cf.pair(
                Name(b"CFM"),
                Name(if c.v == 5 { b"AESV3" } else { b"AESV2" }),
            );
            std_cf.pair(Name(b"Length"), if c.v == 5 { 32 } else { 16 });
            std_cf.finish();
            cf.finish();
            d.pair(Name(b"StmF"), Name(b"StdCF"))
                .pair(Name(b"StrF"), Name(b"StdCF"));
        }
        if c.v == 5 {
            d.pair(Name(b"OE"), Str(&c.oe))
                .pair(Name(b"UE"), Str(&c.ue))
                .pair(Name(b"Perms"), Str(&c.perms));
        }
        d.finish();
        self.emit.obj(out, &chunk);
        Some(r)
    }

    pub(super) fn trailer(&self, info: Ref, encrypt: Option<Ref>) -> String {
        let hex: String = self.id.iter().map(|b| format!("{b:02X}")).collect();
        let encrypt = encrypt
            .map(|e| format!("  /Encrypt {} 0 R\n", e.get()))
            .unwrap_or_default();
        format!(
            "<<\n  /Size {}\n  /Root {} 0 R\n  /Info {} 0 R\n{encrypt}  /ID [<{hex}> <{hex}>]\n>>",
            self.emit.size(),
            self.catalog.get(),
            info.get()
        )
    }
}
