use super::*;

/// Words of the given widths joined by 3pt spaces, ending in the mandatory final glue and break.
fn words(ws: &[f64]) -> Vec<Item> {
    let mut items = Vec::new();
    let mut at = 0;
    for (i, &w) in ws.iter().enumerate() {
        if i > 0 {
            items.push(Item::Glue {
                width: 3.0,
                start: at,
                end: at + 1,
                stretch: 1.5,
                shrink: 1.0,
            });
            at += 1;
        }
        items.push(Item::Box {
            width: w,
            start: at,
            end: at + 1,
        });
        at += 1;
    }
    items.push(Item::Glue {
        width: 0.0,
        start: at,
        end: at,
        stretch: INFINITY,
        shrink: 0.0,
    });
    items.push(Item::Penalty {
        width: 0.0,
        penalty: -INFINITY,
        flagged: 1.0,
    });
    items
}

#[test]
fn total_fit() {
    let items = words(&[10.0, 10.0, 10.0]);
    // Two words fit a 23pt line exactly; the third goes to the next line.
    assert_eq!(knuth_plass(&items, &[23.0], 4.0), vec![0, 3, 6]);
    // Everything fits one wide line.
    assert_eq!(knuth_plass(&items, &[100.0], 4.0), vec![0, 6]);
}

#[test]
fn nothing_fits_a_narrow_line() {
    let items = words(&[10.0, 10.0]);
    assert!(knuth_plass(&items, &[5.0], 4.0).is_empty());
    // Best fit still breaks between the words.
    assert_eq!(best_fit(&items, &[5.0]), vec![0, 1, 4]);
}

#[test]
fn hyphenation_points() {
    // "aaaa-bbbb cc": a penalty between the two syllables of the first word.
    let items = vec![
        Item::Box {
            width: 10.0,
            start: 0,
            end: 4,
        },
        Item::Penalty {
            width: 5.0,
            penalty: 600.0,
            flagged: 1.0,
        },
        Item::Box {
            width: 10.0,
            start: 4,
            end: 8,
        },
        Item::Glue {
            width: 3.0,
            start: 8,
            end: 9,
            stretch: 1.5,
            shrink: 1.0,
        },
        Item::Box {
            width: 2.0,
            start: 9,
            end: 11,
        },
        Item::Glue {
            width: 0.0,
            start: 11,
            end: 11,
            stretch: INFINITY,
            shrink: 0.0,
        },
        Item::Penalty {
            width: 0.0,
            penalty: -INFINITY,
            flagged: 1.0,
        },
    ];
    assert_eq!(knuth_plass(&items, &[15.0], 4.0), vec![0, 1, 6]);
}
