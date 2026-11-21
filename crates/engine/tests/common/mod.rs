//! A sample document that draws every op type, shared by the writer tests.
#![allow(dead_code)]

use axonpdf::display::*;
use axonpdf::fonts::FontStore;
use axonpdf::images::ImageStore;
use axonpdf::writer::{Date, DocInfo, Writer};

pub fn asset(name: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../fixtures/assets/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

pub fn info() -> DocInfo {
    DocInfo {
        title: Some("Sample — every op".into()),
        author: Some("axonpdf".into()),
        creator: Some("axonpdf tests".into()),
        producer: Some("axonpdf".into()),
        creation_date: Some(Date {
            year: 2026,
            month: 11,
            day: 21,
            hour: 12,
            minute: 0,
            second: 0,
            utc_offset_minutes: Some(360),
        }),
        language: Some("en-US".into()),
        ..Default::default()
    }
}

pub struct Assets {
    pub fonts: FontStore,
    pub images: ImageStore,
    pub lato: FontId,
    pub png: ImageId,
    pub jpg: ImageId,
}

pub fn assets() -> Assets {
    let mut fonts = FontStore::new();
    let lato = fonts
        .add_font("Lato", 400, false, asset("Lato-Regular.ttf"))
        .unwrap();
    let mut images = ImageStore::new();
    let png = images.add(&asset("badge.png")).unwrap();
    let jpg = images.add(&asset("photo.jpg")).unwrap();
    Assets {
        fonts,
        images,
        lato,
        png,
        jpg,
    }
}

pub fn text(
    fonts: &FontStore,
    font: FontId,
    s: &str,
    size: f32,
    x: f32,
    y: f32,
    color: Color,
) -> Op {
    Op::Text(TextRun {
        font,
        size,
        color,
        x,
        y,
        glyphs: fonts.shape(font, s, size, 0.0, &[]),
        text: s.into(),
    })
}

/// One page with every op. `n` is the page number, drawn in the corner.
pub fn sample_page(a: &Assets, n: usize) -> PageOut {
    let f = &a.fonts;
    let helvetica = f.resolve("Helvetica", 400, false).unwrap();
    let bold = f.resolve("Helvetica", 700, false).unwrap();
    let r = |x, y, w, h| Rect { x, y, w, h };
    let solid = |c| Some(Paint::Solid(c));
    let mut ops = vec![
        Op::Dest {
            name: format!("page-{n}"),
            x: 0.0,
            y: 0.0,
        },
        Op::Bookmark(Bookmark {
            id: n as u32 * 2,
            parent: None,
            title: format!("Page {n}"),
            left: 0.0,
            top: 0.0,
            zoom: None,
            fit: false,
            expanded: true,
        }),
        Op::Bookmark(Bookmark {
            id: n as u32 * 2 + 1,
            parent: Some(n as u32 * 2),
            title: "Shapes".into(),
            left: 0.0,
            top: 100.0,
            zoom: None,
            fit: true,
            expanded: false,
        }),
        Op::Rect {
            rect: r(40.0, 40.0, 515.0, 60.0),
            radii: [12.0, 12.0, 0.0, 0.0],
            fill: solid(rgb(0.13, 0.4, 0.8)),
            stroke: None,
        },
        text(
            f,
            bold,
            "AVATAR Typography: Kerned Helvetica",
            20.0,
            56.0,
            78.0,
            rgb(1.0, 1.0, 1.0),
        ),
        text(
            f,
            a.lato,
            "Lato: office fifty flights, café — “quotes”",
            16.0,
            40.0,
            130.0,
            Color::BLACK,
        ),
        text(
            f,
            helvetica,
            &format!("Page {n}"),
            10.0,
            520.0,
            820.0,
            rgb(0.4, 0.4, 0.4),
        ),
        Op::Border(Border {
            rect: r(40.0, 150.0, 160.0, 100.0),
            widths: [4.0, 8.0, 4.0, 2.0],
            colors: [
                rgb(0.9, 0.1, 0.1),
                rgb(0.1, 0.7, 0.2),
                rgb(0.1, 0.2, 0.9),
                rgb(0.8, 0.6, 0.0),
            ],
            styles: [
                BorderStyle::Solid,
                BorderStyle::Dashed,
                BorderStyle::Dotted,
                BorderStyle::Solid,
            ],
            radii: [20.0, 0.0, 30.0, 10.0],
        }),
        Op::Image {
            id: a.png,
            rect: r(220.0, 150.0, 100.0, 100.0),
            opacity: 1.0,
        },
        Op::Save,
        Op::Clip {
            path: axonpdf::path::rounded_rect(r(340.0, 150.0, 160.0, 100.0), [50.0; 4]),
            rule: FillRule::NonZero,
        },
        Op::Image {
            id: a.jpg,
            rect: r(340.0, 150.0, 160.0, 100.0),
            opacity: 0.8,
        },
        Op::Restore,
        Op::Save,
        Op::Opacity(0.5),
        Op::Rect {
            rect: r(40.0, 280.0, 200.0, 80.0),
            radii: [0.0; 4],
            fill: solid(rgb(1.0, 0.0, 0.0)),
            stroke: None,
        },
        Op::Restore,
        Op::Rect {
            rect: r(140.0, 300.0, 200.0, 80.0),
            radii: [0.0; 4],
            fill: solid(Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 0.5,
            }),
            stroke: None,
        },
        Op::Save,
        Op::Transform([0.866, 0.5, -0.5, 0.866, 420.0, 300.0]),
        Op::Rect {
            rect: r(0.0, 0.0, 80.0, 40.0),
            radii: [6.0; 4],
            fill: solid(rgb(0.2, 0.8, 0.5)),
            stroke: Some(stroke(rgb(0.0, 0.3, 0.1), 2.0, vec![])),
        },
        text(f, helvetica, "turned", 12.0, 8.0, 25.0, Color::BLACK),
        Op::Restore,
        Op::Path {
            path: vec![
                Seg::Move(40.0, 420.0),
                Seg::Cubic(140.0, 360.0, 240.0, 480.0, 340.0, 420.0),
                Seg::Line(340.0, 470.0),
                Seg::Close,
            ],
            fill: Some(Fill {
                paint: Paint::Solid(rgb(1.0, 0.9, 0.6)),
                rule: FillRule::EvenOdd,
            }),
            stroke: Some(stroke(rgb(0.5, 0.3, 0.0), 3.0, vec![6.0, 3.0])),
        },
        text(
            f,
            helvetica,
            "Link to the web",
            12.0,
            40.0,
            520.0,
            rgb(0.0, 0.0, 0.8),
        ),
        Op::Link {
            rect: r(40.0, 508.0, 90.0, 14.0),
            target: LinkTarget::Uri("https://axonpack.dev".into()),
        },
        text(
            f,
            helvetica,
            "Link to page 1",
            12.0,
            40.0,
            540.0,
            rgb(0.0, 0.0, 0.8),
        ),
        Op::Link {
            rect: r(40.0, 528.0, 90.0, 14.0),
            target: LinkTarget::Dest("page-1".into()),
        },
        Op::Note {
            x: 300.0,
            y: 520.0,
            text: "A sticky note".into(),
            color: None,
        },
        Op::Rect {
            rect: r(360.0, 380.0, 190.0, 50.0),
            radii: [8.0; 4],
            fill: Some(gradient(
                GradientKind::Linear {
                    x1: 360.0,
                    y1: 0.0,
                    x2: 550.0,
                    y2: 0.0,
                },
                1.0,
            )),
            stroke: None,
        },
        Op::Path {
            path: axonpdf::path::ellipse(455.0, 480.0, 60.0, 35.0),
            fill: Some(Fill {
                paint: gradient(
                    GradientKind::Radial {
                        fx: 440.0,
                        fy: 470.0,
                        fr: 0.0,
                        cx: 455.0,
                        cy: 480.0,
                        r: 60.0,
                    },
                    0.2,
                ),
                rule: FillRule::NonZero,
            }),
            stroke: Some(Stroke {
                paint: gradient(
                    GradientKind::Linear {
                        x1: 395.0,
                        y1: 0.0,
                        x2: 515.0,
                        y2: 0.0,
                    },
                    1.0,
                ),
                ..stroke(Color::BLACK, 4.0, vec![])
            }),
        },
        Op::Path {
            path: axonpdf::path::svg::parse(
                "M300 600 a40 40 0 0 1 80 0 a40 40 0 0 1 80 0 q0 60 -80 100 q-80 -40 -80 -100z",
            ),
            fill: Some(Fill {
                paint: Paint::Solid(rgb(0.85, 0.1, 0.3)),
                rule: FillRule::NonZero,
            }),
            stroke: Some(stroke(rgb(0.4, 0.0, 0.1), 2.0, vec![])),
        },
        Op::Path {
            path: axonpdf::path::svg::polyline(
                &axonpdf::path::svg::points("40,700 80,620 120,700 160,620 200,700"),
                false,
            ),
            fill: None,
            stroke: Some(Stroke {
                join: LineJoin::Miter,
                cap: LineCap::Square,
                ..stroke(rgb(0.1, 0.5, 0.1), 6.0, vec![])
            }),
        },
    ];
    ops.push(Op::Rect {
        rect: r(0.0, 0.0, 0.0, 0.0),
        radii: [0.0; 4],
        fill: None,
        stroke: None,
    });
    PageOut {
        width: 595.28,
        height: 841.89,
        ops,
    }
}

/// Red, yellow, blue; the last stop's alpha is `end_alpha`.
pub fn gradient(kind: GradientKind, end_alpha: f32) -> Paint {
    let stops = vec![
        (0.0, rgb(0.9, 0.1, 0.1)),
        (0.5, rgb(1.0, 0.85, 0.0)),
        (
            1.0,
            Color {
                a: end_alpha,
                ..rgb(0.1, 0.2, 0.9)
            },
        ),
    ];
    Paint::Gradient(Box::new(Gradient {
        kind,
        stops,
        transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    }))
}

pub fn stroke(c: Color, width: f32, dash: Vec<f32>) -> Stroke {
    Stroke {
        paint: Paint::Solid(c),
        width,
        cap: LineCap::Round,
        join: LineJoin::Round,
        miter_limit: 10.0,
        dash,
        dash_phase: 0.0,
    }
}

/// Writes `pages` sample pages and returns the file.
pub fn render(pages: usize) -> Vec<u8> {
    let mut a = assets();
    let mut bytes = Vec::new();
    let mut out = |chunk: &[u8]| bytes.extend_from_slice(chunk);
    let mut w = Writer::new(info());
    w.begin(&mut out);
    for n in 1..=pages {
        w.write_page(&sample_page(&a, n), &a.fonts, &mut a.images, &mut out);
    }
    w.finish(&a.fonts, &mut out);
    bytes
}

/// Runs a tool on a PDF; `None` when the tool is not installed.
pub fn tool(cmd: &str, args: &[&str]) -> Option<std::process::Output> {
    std::process::Command::new(cmd).args(args).output().ok()
}
