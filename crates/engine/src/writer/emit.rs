//! Sends bytes to the caller as they are made and records where each object starts.

use pdf_writer::{Chunk, Ref};

/// The output callback. Chunks must be joined in order, unchanged.
pub type Out<'a> = &'a mut dyn FnMut(&[u8]);

#[derive(Default)]
pub struct Emitter {
    /// Bytes sent so far.
    pos: u64,
    /// Byte offset of each object by number; 0 for numbers not written (yet).
    offsets: Vec<u64>,
}

impl Emitter {
    /// A new object number.
    pub fn alloc(&mut self) -> Ref {
        self.offsets.push(0);
        Ref::new(self.offsets.len() as i32)
    }

    /// The trailer's `/Size`: the highest object number plus one.
    pub fn size(&self) -> usize {
        self.offsets.len() + 1
    }

    pub fn raw(&mut self, out: Out, bytes: &[u8]) {
        out(bytes);
        self.pos += bytes.len() as u64;
    }

    /// Writes a chunk holding exactly one indirect object.
    pub fn obj(&mut self, out: Out, chunk: &Chunk) {
        let mut refs = chunk.refs();
        let (Some(id), None) = (refs.next(), refs.next()) else {
            panic!("one object per chunk");
        };
        self.offsets[id.get() as usize - 1] = self.pos;
        self.raw(out, chunk.as_bytes());
    }

    /// Builds and writes one object.
    pub fn write(&mut self, out: Out, build: impl FnOnce(&mut Chunk)) {
        let mut chunk = Chunk::new();
        build(&mut chunk);
        self.obj(out, &chunk);
    }

    /// The cross-reference table and trailer. Numbers never written are listed as free.
    pub fn end(&mut self, out: Out, trailer: &[u8]) {
        let start = self.pos;
        let mut table = format!("xref\n0 {}\n0000000000 65535 f\r\n", self.offsets.len() + 1);
        for &offset in &self.offsets {
            if offset == 0 {
                table.push_str("0000000000 65535 f\r\n");
            } else {
                table.push_str(&format!("{offset:010} 00000 n\r\n"));
            }
        }
        let tail = format!("startxref\n{start}\n%%EOF\n");
        self.raw(out, table.as_bytes());
        self.raw(out, b"trailer\n");
        self.raw(out, trailer);
        self.raw(out, b"\n");
        self.raw(out, tail.as_bytes());
    }
}
