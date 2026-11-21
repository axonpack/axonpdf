//! A stand-in for the engine with the same signatures, so the bindings run end to end before the
//! real one lands. It writes one A4 page per `PAGE` node and one Helvetica line per `TEXT` node.
//! ponytail: delete this file once `axonpdf::Engine` exists.

use serde_json::Value;

pub struct RenderArgs {
    pub page_number: u32,
    pub total_pages: Option<u32>,
    pub sub_page_number: u32,
    pub sub_page_total_pages: Option<u32>,
}

type Callback = Box<dyn FnMut(u32, &RenderArgs) -> String>;

pub struct Engine {
    callback: Option<Callback>,
}

impl Engine {
    pub fn new() -> Self {
        Engine { callback: None }
    }

    pub fn add_font(
        &mut self,
        _family: &str,
        _weight: u16,
        _italic: bool,
        _bytes: Vec<u8>,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn add_image(&mut self, _id: u32, _bytes: Vec<u8>) -> Result<(), String> {
        Ok(())
    }

    pub fn set_render_callback(&mut self, cb: Callback) {
        self.callback = Some(cb);
    }

    pub fn render(&mut self, tree_json: &str, out: &mut dyn FnMut(&[u8])) -> Result<(), String> {
        let tree: Value = serde_json::from_str(tree_json).map_err(|e| e.to_string())?;
        let pages: Vec<&Value> = children(&tree)
            .iter()
            .filter(|n| n["type"] == "PAGE")
            .collect();
        let total = pages.len() as u32;
        let mut offset = 0;
        let mut offsets = vec![0; 3];
        let mut emit = |bytes: Vec<u8>, offset: &mut usize| {
            *offset += bytes.len();
            out(&bytes);
        };
        emit(b"%PDF-1.7\n".to_vec(), &mut offset);
        for (i, page) in pages.iter().enumerate() {
            let mut lines = Vec::new();
            let args = RenderArgs {
                page_number: i as u32 + 1,
                total_pages: Some(total),
                sub_page_number: 1,
                sub_page_total_pages: Some(1),
            };
            self.collect(page, &args, &mut lines);
            let mut content = String::from("BT /F1 12 Tf 14 TL 56 780 Td\n");
            for line in &lines {
                let escaped = line
                    .replace('\\', "\\\\")
                    .replace('(', "\\(")
                    .replace(')', "\\)");
                content.push_str(&format!("({escaped}) Tj T*\n"));
            }
            content.push_str("ET");
            let id = offsets.len() + 1;
            offsets.push(offset);
            let mut chunk = format!(
                "{id} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.28 841.89] /Contents {} 0 R /Resources << /Font << /F1 3 0 R >> >> >>\nendobj\n",
                id + 1
            );
            offsets.push(offset + chunk.len());
            chunk.push_str(&format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n{content}\nendstream\nendobj\n",
                id + 1,
                content.len()
            ));
            emit(chunk.into_bytes(), &mut offset);
        }
        let kids: Vec<String> = (0..pages.len())
            .map(|i| format!("{} 0 R", 4 + i * 2))
            .collect();
        let mut tail = String::new();
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            format!(
                "<< /Type /Pages /Kids [{}] /Count {total} >>",
                kids.join(" ")
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
                .to_owned(),
        ];
        for (i, object) in objects.iter().enumerate() {
            offsets[i] = offset + tail.len();
            tail.push_str(&format!("{} 0 obj\n{object}\nendobj\n", i + 1));
        }
        let xref = offset + tail.len();
        tail.push_str(&format!(
            "xref\n0 {}\n0000000000 65535 f \n",
            offsets.len() + 1
        ));
        for o in &offsets {
            tail.push_str(&format!("{o:010} 00000 n \n"));
        }
        tail.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len() + 1
        ));
        emit(tail.into_bytes(), &mut offset);
        Ok(())
    }

    fn collect(&mut self, node: &Value, args: &RenderArgs, lines: &mut Vec<String>) {
        if let Some(id) = node["render"].as_u64() {
            let callback = self.callback.as_mut().expect("render callback not set");
            let result: Value =
                serde_json::from_str(&callback(id as u32, args)).unwrap_or(Value::Null);
            let mut line = String::new();
            for child in children(&result) {
                text(child, &mut line);
            }
            lines.push(line);
            return;
        }
        if node["type"] == "TEXT" {
            let mut line = String::new();
            text(node, &mut line);
            lines.push(line);
            return;
        }
        for child in children(node) {
            self.collect(child, args, lines);
        }
    }
}

fn children(node: &Value) -> &[Value] {
    node["children"].as_array().map_or(&[], Vec::as_slice)
}

fn text(node: &Value, out: &mut String) {
    if let Some(value) = node["value"].as_str() {
        out.push_str(value);
    }
    for child in children(node) {
        text(child, out);
    }
}
