use super::*;
use serde_json::json;

fn r(v: Value) -> Style {
    resolve(
        &Container {
            width: 600.0,
            height: 800.0,
            ..Default::default()
        },
        &v,
    )
}

#[test]
fn box_model_shorthands() {
    let s = r(json!({"margin": "10 auto", "padding": "1 2 3", "marginTop": "5mm"}));
    let mm5 = 5.0 * 72.0 / 25.4;
    assert_eq!(
        s.margin,
        [
            Some(Dim::Pt(mm5)),
            Some(Dim::Auto),
            Some(Dim::Pt(10.0)),
            Some(Dim::Auto)
        ]
    );
    assert_eq!(
        s.padding,
        [
            Some(Dim::Pt(1.0)),
            Some(Dim::Pt(2.0)),
            Some(Dim::Pt(3.0)),
            Some(Dim::Pt(2.0))
        ]
    );
    // Written later, the shorthand wins.
    let s = r(json!({"marginTop": 5, "margin": 1}));
    assert_eq!(s.margin[TOP], Some(Dim::Pt(1.0)));
    let s = r(json!({"paddingHorizontal": "10%", "marginVertical": "1 2 3"}));
    assert_eq!(
        (s.padding[RIGHT], s.padding[LEFT]),
        (Some(Dim::Pct(10.0)), Some(Dim::Pct(10.0)))
    );
    assert_eq!(s.margin, [None; 4]);
}

#[test]
fn borders() {
    let s = r(json!({"border": "2pt dashed red", "borderTopWidth": 4, "borderRadius": 3}));
    assert_eq!(s.border_width, [Some(4.0), Some(2.0), Some(2.0), Some(2.0)]);
    assert_eq!(s.border_style[LEFT], Some(BorderStyle::Dashed));
    assert_eq!(s.border_color[BOTTOM].map(|c| c.r), Some(1.0));
    assert_eq!(s.border_radius, [Some(Dim::Pt(3.0)); 4]);
    // The shorthand takes lengths only; a percentage is an error, as on a corner it is not.
    let s = r(json!({"borderRadius": "50%", "borderTopLeftRadius": "10%"}));
    assert_eq!(s.border_radius[0], Some(Dim::Pct(10.0)));
    assert_eq!(take_error(), Err("Invalid border radius: 50%".into()));
    assert_eq!(take_error(), Ok(()));
    let s = r(json!({"borderBottom": "1 solid blue", "borderColor": "green"}));
    assert_eq!(s.border_width, [None, None, Some(1.0), None]);
    assert!(s.border_color.iter().all(|c| c.is_some()));
}

#[test]
fn flex_shorthand() {
    let s = r(json!({"flex": 1}));
    assert_eq!(
        (s.flex_grow, s.flex_shrink, s.flex_basis),
        (Some(1.0), Some(1.0), Some(Dim::Pt(0.0)))
    );
    let s = r(json!({"flex": "none"}));
    assert_eq!(
        (s.flex_grow, s.flex_shrink, s.flex_basis),
        (Some(0.0), Some(0.0), Some(Dim::Auto))
    );
    let s = r(json!({"flex": "2 3 10%"}));
    assert_eq!(
        (s.flex_grow, s.flex_shrink, s.flex_basis),
        (Some(2.0), Some(3.0), Some(Dim::Pct(10.0)))
    );
}

#[test]
fn text() {
    let s = r(json!({"fontWeight": "bold", "lineHeight": 1.5, "fontFamily": ["A", "B"]}));
    assert_eq!(s.font_weight, Some(700));
    // Unitless line heights scale this style's own font size, 18 when unset.
    assert_eq!(s.line_height, Some(27.0));
    assert_eq!(s.font_family, Some(vec!["A".to_string(), "B".to_string()]));
    let s = r(json!({"fontSize": 10, "lineHeight": "150%", "letterSpacing": "1in"}));
    assert_eq!((s.line_height, s.letter_spacing), (Some(15.0), Some(72.0)));
    assert_eq!(r(json!({"lineHeight": "20pt"})).line_height, Some(20.0));
}

#[test]
fn units_against_the_page() {
    let s = r(json!({"width": "50vw", "height": "50%", "gap": "1 2"}));
    assert_eq!(
        (s.width, s.height),
        (Some(Dim::Pt(300.0)), Some(Dim::Pct(50.0)))
    );
    assert_eq!(
        (s.row_gap, s.column_gap),
        (Some(Dim::Pt(1.0)), Some(Dim::Pt(2.0)))
    );
}

#[test]
fn floats_are_absolute() {
    assert_eq!(
        r(json!({"float": "left"})).position,
        Some(PositionType::Absolute)
    );
}

#[test]
fn inheritance() {
    let parent = r(
        json!({"color": "red", "fontSize": 10, "textDecoration": "underline", "backgroundColor": "blue", "margin": 3}),
    );
    let mut child = r(json!({"textDecoration": "line-through", "fontSize": 12}));
    inherit(&parent, false, &mut child);
    assert_eq!(child.font_size, Some(12.0));
    assert!(child.color.is_some());
    assert_eq!(
        child.text_decoration.as_deref(),
        Some("underline line-through")
    );
    assert_eq!(child.background_color, None);
    assert_eq!(child.margin, [None; 4]);
    let mut nested = Style::default();
    inherit(&parent, true, &mut nested);
    assert!(nested.background_color.is_some());
}

#[test]
fn font_feature_settings_take_a_list_or_an_object_and_are_inherited() {
    let list = r(json!({"fontFeatureSettings": ["tnum", "zero", "toolong"]}));
    assert_eq!(list.font_feature_settings, Some(vec![(*b"tnum", true), (*b"zero", true)]));
    let object = r(json!({"fontFeatureSettings": {"liga": 0, "ss01": 1, "kern": false}}));
    assert_eq!(
        object.font_feature_settings,
        Some(vec![(*b"liga", false), (*b"ss01", true), (*b"kern", false)])
    );
    let mut child = r(json!({}));
    inherit(&list, false, &mut child);
    assert_eq!(child.font_feature_settings, list.font_feature_settings);
}
