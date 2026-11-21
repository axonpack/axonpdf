mod common;

use axonpdf::writer::{Encryption, PdfVersion, Permissions, Printing, Writer};

/// Writes the sample with `version`'s method; qpdf must report `method` (the cipher, or the
/// revision for RC4), open it with each password and find
/// no errors, and the text must not be readable without decrypting.
fn check(version: PdfVersion, method: &str) {
    let mut a = common::assets();
    let mut info = common::info();
    info.pdf_version = version;
    info.encryption = Some(Encryption {
        user_password: Some("reader".into()),
        owner_password: Some("owner".into()),
        permissions: Permissions {
            printing: Printing::HighResolution,
            copying: true,
            ..Default::default()
        },
        seed: Some([7; 32]),
    });
    let mut pdf = Vec::new();
    let mut out = |c: &[u8]| pdf.extend_from_slice(c);
    let mut w = Writer::new(info);
    w.begin(&mut out);
    w.write_page(
        &common::sample_page(&a, 1),
        &a.fonts,
        &mut a.images,
        &mut out,
    );
    w.finish(&a.fonts, &mut out);
    assert!(
        !pdf.windows(13).any(|w| w == b"Sample \xe2\x80\x94 every"),
        "title in the clear"
    );
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("encrypted-{version:?}.pdf"));
    std::fs::write(&path, &pdf).unwrap();
    let p = path.to_str().unwrap();
    for password in ["reader", "owner"] {
        let Some(r) = common::tool("qpdf", &["--check", &format!("--password={password}"), p])
        else {
            return;
        };
        let s = String::from_utf8_lossy(&r.stdout);
        assert!(
            r.status.success() && !s.contains("WARNING"),
            "{method} with {password}: {s}{}",
            String::from_utf8_lossy(&r.stderr)
        );
        assert!(s.contains(method), "{method}: {s}");
    }
    let Some(r) = common::tool("qpdf", &["--check", "--password=wrong", p]) else {
        return;
    };
    assert!(!r.status.success(), "{method} opened with a wrong password");
    let Some(r) = common::tool("pdftotext", &["-upw", "reader", p, "-"]) else {
        return;
    };
    assert!(
        String::from_utf8_lossy(&r.stdout).contains("Kerned Helvetica"),
        "{method} text"
    );
}

#[test]
fn rc4_40() {
    check(PdfVersion::V1_3, "R = 2");
}

#[test]
fn rc4_128() {
    check(PdfVersion::V1_4, "R = 3");
}

#[test]
fn aes_128() {
    check(PdfVersion::V1_6, "AESv2");
}

#[test]
fn aes_256() {
    check(PdfVersion::V1_7Ext3, "AESv3");
}
