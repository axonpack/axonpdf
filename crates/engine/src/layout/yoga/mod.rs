//! Yoga, set up for PDF layout.
//!
//! A [`Tree`] owns one Yoga config and every node made from it. The C API and enum values come
//! from the binding of the Yoga version being built (`YOGA_VERSION` in `consts/yoga.rs`); this
//! file works with any binding that has the same items.

#[cfg(yoga_version = "3.2.1")]
#[path = "v3_2_1.rs"]
mod binding;
#[cfg(not(yoga_version = "3.2.1"))]
compile_error!("no binding for this Yoga version; add one next to v3_2_1.rs");

use binding::*;
pub use binding::{
    Align, Display, Edge, FlexDirection, Gutter, Justify, MeasureMode, Overflow, PositionType, Wrap,
};
use std::ffi::c_void;

/// A length that may also be `auto`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dim {
    Auto,
    Pt(f32),
    Pct(f32),
}

/// A length for the properties Yoga gives no `auto` (min, max, padding, gap).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    Pt(f32),
    Pct(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

/// Index of a node in its [`Tree`].
pub type NodeId = usize;

/// Measures a leaf: `(width, width mode, height, height mode) -> (width, height)`.
pub type Measure = dyn FnMut(f32, MeasureMode, f32, MeasureMode) -> (f32, f32);

pub struct Tree {
    config: YgConfig,
    nodes: Vec<YgNode>,
    // Yoga keeps a thin pointer to each closure in the node's context, so each closure is boxed
    // twice and the outer box is kept here for as long as the node lives. Without the outer box the
    // pointer would move whenever the Vec grows.
    #[allow(clippy::vec_box)]
    measures: Vec<Box<Box<Measure>>>,
}

impl Tree {
    pub fn new() -> Self {
        let config = unsafe { YGConfigNew() };
        // No rounding to a pixel grid: PDF works in points.
        unsafe { YGConfigSetPointScaleFactor(config, 0.0) };
        Self {
            config,
            nodes: Vec::new(),
            measures: Vec::new(),
        }
    }

    pub fn node(&mut self) -> NodeId {
        self.nodes.push(unsafe { YGNodeNewWithConfig(self.config) });
        self.nodes.len() - 1
    }

    pub fn push_child(&mut self, parent: NodeId, child: NodeId) {
        let (p, c) = (self.nodes[parent], self.nodes[child]);
        unsafe { YGNodeInsertChild(p, c, YGNodeGetChildCount(p)) };
    }

    pub fn set_measure(&mut self, id: NodeId, measure: Box<Measure>) {
        let mut boxed = Box::new(measure);
        let context = &mut *boxed as *mut Box<Measure> as *mut c_void;
        unsafe {
            YGNodeSetContext(self.nodes[id], context);
            YGNodeSetMeasureFunc(self.nodes[id], measure_trampoline);
        }
        self.measures.push(boxed);
    }

    /// Lays out the tree under `root`. `None` leaves that side unconstrained.
    pub fn calculate(&mut self, root: NodeId, width: Option<f32>, height: Option<f32>) {
        let (w, h) = (width.unwrap_or(f32::NAN), height.unwrap_or(f32::NAN));
        unsafe { YGNodeCalculateLayout(self.nodes[root], w, h, DIRECTION_LTR) };
    }

    pub fn layout(&self, id: NodeId) -> Rect {
        let n = self.nodes[id];
        unsafe {
            Rect {
                left: YGNodeLayoutGetLeft(n),
                top: YGNodeLayoutGetTop(n),
                width: YGNodeLayoutGetWidth(n),
                height: YGNodeLayoutGetHeight(n),
            }
        }
    }

    /// Computed margin, padding and border of one edge.
    pub fn edges(&self, id: NodeId, edge: Edge) -> (f32, f32, f32) {
        let (n, e) = (self.nodes[id], edge as i32);
        unsafe {
            (
                YGNodeLayoutGetMargin(n, e),
                YGNodeLayoutGetPadding(n, e),
                YGNodeLayoutGetBorder(n, e),
            )
        }
    }

    pub fn flex_direction(&mut self, id: NodeId, v: FlexDirection) {
        unsafe { YGNodeStyleSetFlexDirection(self.nodes[id], v as i32) }
    }
    pub fn justify_content(&mut self, id: NodeId, v: Justify) {
        unsafe { YGNodeStyleSetJustifyContent(self.nodes[id], v as i32) }
    }
    pub fn align_content(&mut self, id: NodeId, v: Align) {
        unsafe { YGNodeStyleSetAlignContent(self.nodes[id], v as i32) }
    }
    pub fn align_items(&mut self, id: NodeId, v: Align) {
        unsafe { YGNodeStyleSetAlignItems(self.nodes[id], v as i32) }
    }
    pub fn align_self(&mut self, id: NodeId, v: Align) {
        unsafe { YGNodeStyleSetAlignSelf(self.nodes[id], v as i32) }
    }
    pub fn position_type(&mut self, id: NodeId, v: PositionType) {
        unsafe { YGNodeStyleSetPositionType(self.nodes[id], v as i32) }
    }
    pub fn flex_wrap(&mut self, id: NodeId, v: Wrap) {
        unsafe { YGNodeStyleSetFlexWrap(self.nodes[id], v as i32) }
    }
    pub fn overflow(&mut self, id: NodeId, v: Overflow) {
        unsafe { YGNodeStyleSetOverflow(self.nodes[id], v as i32) }
    }
    pub fn display(&mut self, id: NodeId, v: Display) {
        unsafe { YGNodeStyleSetDisplay(self.nodes[id], v as i32) }
    }
    pub fn flex_grow(&mut self, id: NodeId, v: f32) {
        unsafe { YGNodeStyleSetFlexGrow(self.nodes[id], v) }
    }
    pub fn flex_shrink(&mut self, id: NodeId, v: f32) {
        unsafe { YGNodeStyleSetFlexShrink(self.nodes[id], v) }
    }
    pub fn aspect_ratio(&mut self, id: NodeId, v: f32) {
        unsafe { YGNodeStyleSetAspectRatio(self.nodes[id], v) }
    }
    pub fn flex_basis(&mut self, id: NodeId, v: Dim) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Dim::Auto => YGNodeStyleSetFlexBasisAuto(n),
                Dim::Pt(x) => YGNodeStyleSetFlexBasis(n, x),
                Dim::Pct(x) => YGNodeStyleSetFlexBasisPercent(n, x),
            }
        }
    }
    pub fn width(&mut self, id: NodeId, v: Dim) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Dim::Auto => YGNodeStyleSetWidthAuto(n),
                Dim::Pt(x) => YGNodeStyleSetWidth(n, x),
                Dim::Pct(x) => YGNodeStyleSetWidthPercent(n, x),
            }
        }
    }
    pub fn height(&mut self, id: NodeId, v: Dim) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Dim::Auto => YGNodeStyleSetHeightAuto(n),
                Dim::Pt(x) => YGNodeStyleSetHeight(n, x),
                Dim::Pct(x) => YGNodeStyleSetHeightPercent(n, x),
            }
        }
    }
    pub fn min_width(&mut self, id: NodeId, v: Len) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Len::Pt(x) => YGNodeStyleSetMinWidth(n, x),
                Len::Pct(x) => YGNodeStyleSetMinWidthPercent(n, x),
            }
        }
    }
    pub fn min_height(&mut self, id: NodeId, v: Len) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Len::Pt(x) => YGNodeStyleSetMinHeight(n, x),
                Len::Pct(x) => YGNodeStyleSetMinHeightPercent(n, x),
            }
        }
    }
    pub fn max_width(&mut self, id: NodeId, v: Len) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Len::Pt(x) => YGNodeStyleSetMaxWidth(n, x),
                Len::Pct(x) => YGNodeStyleSetMaxWidthPercent(n, x),
            }
        }
    }
    pub fn max_height(&mut self, id: NodeId, v: Len) {
        let n = self.nodes[id];
        unsafe {
            match v {
                Len::Pt(x) => YGNodeStyleSetMaxHeight(n, x),
                Len::Pct(x) => YGNodeStyleSetMaxHeightPercent(n, x),
            }
        }
    }
    pub fn margin(&mut self, id: NodeId, edge: Edge, v: Dim) {
        let (n, e) = (self.nodes[id], edge as i32);
        unsafe {
            match v {
                Dim::Auto => YGNodeStyleSetMarginAuto(n, e),
                Dim::Pt(x) => YGNodeStyleSetMargin(n, e, x),
                Dim::Pct(x) => YGNodeStyleSetMarginPercent(n, e, x),
            }
        }
    }
    pub fn padding(&mut self, id: NodeId, edge: Edge, v: Len) {
        let (n, e) = (self.nodes[id], edge as i32);
        unsafe {
            match v {
                Len::Pt(x) => YGNodeStyleSetPadding(n, e, x),
                Len::Pct(x) => YGNodeStyleSetPaddingPercent(n, e, x),
            }
        }
    }
    pub fn border(&mut self, id: NodeId, edge: Edge, v: f32) {
        unsafe { YGNodeStyleSetBorder(self.nodes[id], edge as i32, v) }
    }
    pub fn position(&mut self, id: NodeId, edge: Edge, v: Dim) {
        let (n, e) = (self.nodes[id], edge as i32);
        unsafe {
            match v {
                Dim::Auto => YGNodeStyleSetPositionAuto(n, e),
                Dim::Pt(x) => YGNodeStyleSetPosition(n, e, x),
                Dim::Pct(x) => YGNodeStyleSetPositionPercent(n, e, x),
            }
        }
    }
    pub fn gap(&mut self, id: NodeId, gutter: Gutter, v: Len) {
        let (n, g) = (self.nodes[id], gutter as i32);
        unsafe {
            match v {
                Len::Pt(x) => YGNodeStyleSetGap(n, g, x),
                Len::Pct(x) => YGNodeStyleSetGapPercent(n, g, x),
            }
        }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        // YGNodeFree detaches a node from its owner and children, so freeing each one is safe in
        // any order. The closures in `measures` are dropped after the nodes that point at them.
        for &n in &self.nodes {
            unsafe { YGNodeFree(n) };
        }
        unsafe { YGConfigFree(self.config) };
    }
}

extern "C" fn measure_trampoline(node: YgNode, w: f32, wm: i32, h: f32, hm: i32) -> YgSize {
    // A panic must not unwind into Yoga's C++, which is built without exceptions.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let measure = unsafe { &mut *(YGNodeGetContext(node) as *mut Box<Measure>) };
        measure(w, MeasureMode::from_raw(wm), h, MeasureMode::from_raw(hm))
    }));
    let (width, height) = result.unwrap_or_else(|_| std::process::abort());
    YgSize { width, height }
}
