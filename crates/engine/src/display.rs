//! The display list: everything one finished page draws, handed to the writer.
//!
//! Coordinates are points with the origin at the page's top left and y growing down, the way
//! layout sees the page. The writer flips them into PDF space.

mod field;

pub use field::{Align, Field, FieldFormat, FieldKind};

/// A font in the [`crate::fonts::FontStore`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

/// An image in the [`crate::images::ImageStore`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(pub u32);

/// One laid out page.
#[derive(Clone, Debug, Default)]
pub struct PageOut {
    pub width: f32,
    pub height: f32,
    pub ops: Vec<Op>,
}

/// One drawing or navigation instruction. Ops draw in order; later ones paint over earlier ones.
#[derive(Clone, Debug)]
pub enum Op {
    /// Saves the graphics state (transform, clip, opacity) until the matching `Restore`.
    Save,
    Restore,
    /// Concatenates `[a, b, c, d, e, f]` (x' = a·x + c·y + e, y' = b·x + d·y + f) onto the
    /// current transform.
    Transform([f32; 6]),
    /// Intersects the clip with a path.
    Clip { path: Vec<Seg>, rule: FillRule },
    /// Multiplies the alpha of everything drawn until the matching `Restore`.
    Opacity(f32),
    /// A rectangle with per-corner radii (top left, top right, bottom right, bottom left),
    /// clamped to half the width and height.
    Rect { rect: Rect, radii: [f32; 4], fill: Option<Paint>, stroke: Option<Stroke> },
    /// Box borders, each side with its own width, color and style.
    Border(Border),
    Path { path: Vec<Seg>, fill: Option<Fill>, stroke: Option<Stroke> },
    Text(TextRun),
    /// An image stretched over `rect`. JPEG EXIF orientation is applied by the writer.
    Image { id: ImageId, rect: Rect, opacity: f32 },
    /// A clickable area.
    Link { rect: Rect, target: LinkTarget },
    /// A named destination (`id` prop) at a point of this page.
    Dest { name: String, x: f32, y: f32 },
    Bookmark(Bookmark),
    /// A sticky-note annotation at a point.
    Note { x: f32, y: f32, text: String, color: Option<Color> },
    /// A form field group. Fields and groups name it as their `parent`.
    FieldSet { id: u32, name: String, parent: Option<u32> },
    Field(Field),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// sRGB color, every channel 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
}

/// A path segment. Quadratic curves and arcs become cubics (see [`crate::path`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
    Move(f32, f32),
    Line(f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Clone, Debug)]
pub struct Fill {
    pub paint: Paint,
    pub rule: FillRule,
}

#[derive(Clone, Debug)]
pub enum Paint {
    Solid(Color),
    Gradient(Box<Gradient>),
}

/// A gradient. Stops with alpha below 1 are drawn through a soft mask.
#[derive(Clone, Debug)]
pub struct Gradient {
    pub kind: GradientKind,
    /// Offsets 0 to 1, ascending.
    pub stops: Vec<(f32, Color)>,
    /// Maps gradient coordinates to the current user space.
    pub transform: [f32; 6],
}

#[derive(Clone, Copy, Debug)]
pub enum GradientKind {
    Linear { x1: f32, y1: f32, x2: f32, y2: f32 },
    /// From the focal circle (`fx`, `fy`, `fr`) to the outer circle (`cx`, `cy`, `r`).
    Radial { fx: f32, fy: f32, fr: f32, cx: f32, cy: f32, r: f32 },
}

#[derive(Clone, Debug)]
pub struct Stroke {
    pub paint: Paint,
    pub width: f32,
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter_limit: f32,
    /// Dash and gap lengths; empty for a solid line.
    pub dash: Vec<f32>,
    pub dash_phase: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// Sides are top, right, bottom, left; radii are top left, top right, bottom right, bottom left.
#[derive(Clone, Debug)]
pub struct Border {
    pub rect: Rect,
    pub widths: [f32; 4],
    pub colors: [Color; 4],
    pub styles: [BorderStyle; 4],
    pub radii: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Shaped text on one baseline. Build `glyphs` with [`crate::fonts::FontStore::shape`].
#[derive(Clone, Debug)]
pub struct TextRun {
    pub font: FontId,
    pub size: f32,
    pub color: Color,
    /// Pen position of the first glyph, on the baseline.
    pub x: f32,
    pub y: f32,
    pub glyphs: Vec<Glyph>,
    /// The source text; glyph clusters index into it (for copy and search in viewers).
    pub text: String,
}

/// One positioned glyph. Lengths are points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glyph {
    pub id: u16,
    /// Byte offset in [`TextRun::text`] of the first character this glyph draws.
    pub cluster: u32,
    /// Distance to the next glyph, letter spacing and justification included.
    pub x_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LinkTarget {
    Uri(String),
    /// A named destination (`href="#name"` without the `#`).
    Dest(String),
}

/// An outline entry. `id`s are unique in the document; a child names its parent's `id`.
#[derive(Clone, Debug)]
pub struct Bookmark {
    pub id: u32,
    pub parent: Option<u32>,
    pub title: String,
    pub left: f32,
    pub top: f32,
    pub zoom: Option<f32>,
    pub fit: bool,
    pub expanded: bool,
}
