//! Text strings and dates as PDF string bytes.

/// ASCII as is, anything else as UTF-16BE with a byte order mark.
pub fn text(s: &str) -> Vec<u8> {
    if s.is_ascii() {
        return s.as_bytes().to_vec();
    }
    let mut out = vec![0xFE, 0xFF];
    for unit in s.encode_utf16() {
        out.extend(unit.to_be_bytes());
    }
    out
}

/// A moment, as the caller's clock gave it (the engine has no clock).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// Offset from UTC in minutes; `None` writes `Z`.
    pub utc_offset_minutes: Option<i16>,
}

impl Date {
    /// `D:YYYYMMDDHHmmSS` followed by `Z` or `+HH'mm'`.
    pub fn pdf(&self) -> String {
        let Date {
            year,
            month,
            day,
            hour,
            minute,
            second,
            ..
        } = *self;
        let zone = match self.utc_offset_minutes {
            None | Some(0) => "Z".to_string(),
            Some(m) => {
                let sign = if m < 0 { '-' } else { '+' };
                format!("{sign}{:02}'{:02}'", m.abs() / 60, m.abs() % 60)
            }
        };
        format!("D:{year:04}{month:02}{day:02}{hour:02}{minute:02}{second:02}{zone}")
    }

    /// ISO 8601, for XMP metadata.
    pub fn iso(&self) -> String {
        let Date {
            year,
            month,
            day,
            hour,
            minute,
            second,
            ..
        } = *self;
        let zone = match self.utc_offset_minutes {
            None | Some(0) => "Z".to_string(),
            Some(m) => format!(
                "{}{:02}:{:02}",
                if m < 0 { '-' } else { '+' },
                m.abs() / 60,
                m.abs() % 60
            ),
        };
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{zone}")
    }
}
