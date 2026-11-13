//! Yoga 3.2.1 from `vendor/yoga`, set up for PDF layout.
//!
//! A [`Tree`] owns one Yoga config and every node made from it. Enum values match `YGEnums.h`.

use std::ffi::c_void;

type YgNode = *mut c_void;
type YgConfig = *mut c_void;

#[repr(C)]
struct YgSize {
    width: f32,
    height: f32,
}

type YgMeasureFunc = extern "C" fn(YgNode, f32, i32, f32, i32) -> YgSize;

unsafe extern "C" {
    fn YGConfigNew() -> YgConfig;
    fn YGConfigFree(config: YgConfig);
    fn YGConfigSetPointScaleFactor(config: YgConfig, pixels_in_point: f32);
    fn YGNodeNewWithConfig(config: YgConfig) -> YgNode;
    fn YGNodeFree(node: YgNode);
    fn YGNodeInsertChild(node: YgNode, child: YgNode, index: usize);
    fn YGNodeGetChildCount(node: YgNode) -> usize;
    fn YGNodeSetContext(node: YgNode, context: *mut c_void);
    fn YGNodeGetContext(node: YgNode) -> *mut c_void;
    fn YGNodeSetMeasureFunc(node: YgNode, measure: YgMeasureFunc);
    fn YGNodeCalculateLayout(node: YgNode, width: f32, height: f32, direction: i32);
    fn YGNodeLayoutGetLeft(node: YgNode) -> f32;
    fn YGNodeLayoutGetTop(node: YgNode) -> f32;
    fn YGNodeLayoutGetWidth(node: YgNode) -> f32;
    fn YGNodeLayoutGetHeight(node: YgNode) -> f32;

    fn YGNodeStyleSetFlexDirection(node: YgNode, v: i32);
    fn YGNodeStyleSetJustifyContent(node: YgNode, v: i32);
    fn YGNodeStyleSetAlignContent(node: YgNode, v: i32);
    fn YGNodeStyleSetAlignItems(node: YgNode, v: i32);
    fn YGNodeStyleSetAlignSelf(node: YgNode, v: i32);
    fn YGNodeStyleSetPositionType(node: YgNode, v: i32);
    fn YGNodeStyleSetFlexWrap(node: YgNode, v: i32);
    fn YGNodeStyleSetOverflow(node: YgNode, v: i32);
    fn YGNodeStyleSetDisplay(node: YgNode, v: i32);
    fn YGNodeStyleSetFlexGrow(node: YgNode, v: f32);
    fn YGNodeStyleSetFlexShrink(node: YgNode, v: f32);
    fn YGNodeStyleSetFlexBasis(node: YgNode, v: f32);
    fn YGNodeStyleSetFlexBasisPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetFlexBasisAuto(node: YgNode);
    fn YGNodeStyleSetWidth(node: YgNode, v: f32);
    fn YGNodeStyleSetWidthPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetWidthAuto(node: YgNode);
    fn YGNodeStyleSetHeight(node: YgNode, v: f32);
    fn YGNodeStyleSetHeightPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetHeightAuto(node: YgNode);
    fn YGNodeStyleSetMinWidth(node: YgNode, v: f32);
    fn YGNodeStyleSetMinWidthPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetMinHeight(node: YgNode, v: f32);
    fn YGNodeStyleSetMinHeightPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetMaxWidth(node: YgNode, v: f32);
    fn YGNodeStyleSetMaxWidthPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetMaxHeight(node: YgNode, v: f32);
    fn YGNodeStyleSetMaxHeightPercent(node: YgNode, v: f32);
    fn YGNodeStyleSetMargin(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetMarginPercent(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetMarginAuto(node: YgNode, edge: i32);
    fn YGNodeStyleSetPadding(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetPaddingPercent(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetBorder(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetPosition(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetPositionPercent(node: YgNode, edge: i32, v: f32);
    fn YGNodeStyleSetPositionAuto(node: YgNode, edge: i32);
    fn YGNodeStyleSetGap(node: YgNode, gutter: i32, v: f32);
    fn YGNodeStyleSetGapPercent(node: YgNode, gutter: i32, v: f32);
    fn YGNodeStyleSetAspectRatio(node: YgNode, v: f32);
}

const DIRECTION_LTR: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum FlexDirection {
    Column,
    ColumnReverse,
    Row,
    RowReverse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Justify {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Align {
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    Baseline,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum PositionType {
    Static,
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Wrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Display {
    Flex,
    None,
    Contents,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Edge {
    Left,
    Top,
    Right,
    Bottom,
    Start,
    End,
    Horizontal,
    Vertical,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(i32)]
pub enum Gutter {
    Column,
    Row,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeasureMode {
    Undefined,
    Exactly,
    AtMost,
}

impl MeasureMode {
    fn from_raw(v: i32) -> Self {
        match v {
            1 => Self::Exactly,
            2 => Self::AtMost,
            _ => Self::Undefined,
        }
    }
}

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
