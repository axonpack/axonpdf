//! Form fields (AcroForm widgets).

use super::{Color, Rect};

#[derive(Clone, Debug)]
pub struct Field {
    pub rect: Rect,
    pub name: String,
    /// The `id` of the enclosing [`super::Op::FieldSet`].
    pub parent: Option<u32>,
    pub kind: FieldKind,
    pub required: bool,
    pub no_export: bool,
    pub read_only: bool,
    pub value: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug)]
pub enum FieldKind {
    Text {
        align: Align,
        multiline: bool,
        password: bool,
        no_spell: bool,
        format: Option<FieldFormat>,
        font_size: Option<f32>,
        max_len: Option<u32>,
    },
    /// `value` and `default_value` are ignored; `checked` picks the state.
    Checkbox {
        checked: bool,
        on_state: String,
        off_state: String,
        /// Draws a cross instead of a check mark.
        x_mark: bool,
        background: Option<Color>,
        border: Option<Color>,
    },
    /// A drop-down (`Select`) when `combo`, else a list box (`List`).
    Choice { combo: bool, options: Vec<String>, sort: bool, edit: bool, multi_select: bool, no_spell: bool },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// How a viewer formats a text field's value.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldFormat {
    Zip,
    ZipPlus4,
    Phone,
    Ssn,
    /// A date pattern such as `"yyyy-mm-dd"`.
    Date(String),
    /// 0 to 3, the viewer's time styles.
    Time(u32),
    Number { decimals: u32, sep_comma: bool, neg_style: String, currency: String, currency_prepend: bool },
    Percent { decimals: u32, sep_comma: bool },
}
