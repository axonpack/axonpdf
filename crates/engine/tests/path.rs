use axonpdf::display::Seg;
use axonpdf::path::svg::{parse, points, vertices};

fn end(segs: &[Seg]) -> (f32, f32) {
    match *segs.last().unwrap() {
        Seg::Move(x, y) | Seg::Line(x, y) | Seg::Cubic(.., x, y) => (x, y),
        Seg::Close => panic!("ends with close"),
    }
}

fn close(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
}

#[test]
fn parses_every_command_relative_and_absolute() {
    let segs = parse("M10 10h20v10H10zm5,5 l1-1L20 20 c1 1 2 2 3 3s1 1 2 2Q40 40 50 50t10 0");
    assert_eq!(segs[0], Seg::Move(10.0, 10.0));
    assert_eq!(segs[1], Seg::Line(30.0, 10.0));
    assert_eq!(segs[2], Seg::Line(30.0, 20.0));
    assert_eq!(segs[4], Seg::Close);
    // After z the pen is back at 10,10, so m5,5 moves to 15,15.
    assert_eq!(segs[5], Seg::Move(15.0, 15.0));
    assert_eq!(segs[6], Seg::Line(16.0, 14.0));
    assert!(close(end(&segs), (60.0, 50.0)));
    // Implicit line-tos after a move, numbers touching signs and dots.
    assert_eq!(
        parse("M0 0 10 0-5.5.5"),
        vec![
            Seg::Move(0.0, 0.0),
            Seg::Line(10.0, 0.0),
            Seg::Line(-5.5, 0.5)
        ]
    );
    // Bad data keeps what came before.
    assert_eq!(parse("M1 1 L2 2 X 3 3").len(), 2);
}

#[test]
fn arcs_end_where_asked_and_stay_on_the_circle() {
    let segs = parse("M0 0 A10 10 0 0 1 20 0");
    assert!(close(end(&segs), (20.0, 0.0)));
    assert_eq!(segs.len(), 3, "a half circle is two quarter cubics");
    // Flags can touch: large-arc 1, sweep 0, then x.
    assert!(close(end(&parse("M0 0a5 5 0 1020 0")), (20.0, 0.0)));
    // A radius too small is scaled up to reach the end point.
    assert!(close(end(&parse("M0 0 A1 1 0 0 0 30 0")), (30.0, 0.0)));
}

#[test]
fn points_and_marker_angles() {
    assert_eq!(
        points("0,0 10 0, 10,10 7"),
        vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]
    );
    let v = vertices(&parse("M0 0 L10 0 L10 10"));
    assert_eq!(v.len(), 3);
    assert_eq!(v[0].angle, 0.0);
    assert!(
        (v[1].angle - std::f32::consts::FRAC_PI_4).abs() < 1e-6,
        "bisects the corner"
    );
    assert!((v[2].angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
}
