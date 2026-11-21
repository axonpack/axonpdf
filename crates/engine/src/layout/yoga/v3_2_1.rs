//! The C API and enum values of Yoga 3.2.1, from `vendor/yoga/3.2.1`. Enum values match its
//! `YGEnums.h`. Another Yoga version gets a file of its own next to this one, with the same items.

use std::ffi::c_void;

pub type YgNode = *mut c_void;
pub type YgConfig = *mut c_void;

#[repr(C)]
pub struct YgSize {
    pub width: f32,
    pub height: f32,
}

pub type YgMeasureFunc = extern "C" fn(YgNode, f32, i32, f32, i32) -> YgSize;

unsafe extern "C" {
    pub fn YGConfigNew() -> YgConfig;
    pub fn YGConfigFree(config: YgConfig);
    pub fn YGConfigSetPointScaleFactor(config: YgConfig, pixels_in_point: f32);
    pub fn YGNodeNewWithConfig(config: YgConfig) -> YgNode;
    pub fn YGNodeFree(node: YgNode);
    pub fn YGNodeInsertChild(node: YgNode, child: YgNode, index: usize);
    pub fn YGNodeRemoveAllChildren(node: YgNode);
    pub fn YGNodeMarkDirty(node: YgNode);
    pub fn YGNodeGetChildCount(node: YgNode) -> usize;
    pub fn YGNodeSetContext(node: YgNode, context: *mut c_void);
    pub fn YGNodeGetContext(node: YgNode) -> *mut c_void;
    pub fn YGNodeSetMeasureFunc(node: YgNode, measure: YgMeasureFunc);
    pub fn YGNodeCalculateLayout(node: YgNode, width: f32, height: f32, direction: i32);
    pub fn YGNodeLayoutGetLeft(node: YgNode) -> f32;
    pub fn YGNodeLayoutGetTop(node: YgNode) -> f32;
    pub fn YGNodeLayoutGetWidth(node: YgNode) -> f32;
    pub fn YGNodeLayoutGetHeight(node: YgNode) -> f32;
    pub fn YGNodeLayoutGetMargin(node: YgNode, edge: i32) -> f32;
    pub fn YGNodeLayoutGetPadding(node: YgNode, edge: i32) -> f32;
    pub fn YGNodeLayoutGetBorder(node: YgNode, edge: i32) -> f32;

    pub fn YGNodeStyleSetFlexDirection(node: YgNode, v: i32);
    pub fn YGNodeStyleSetJustifyContent(node: YgNode, v: i32);
    pub fn YGNodeStyleSetAlignContent(node: YgNode, v: i32);
    pub fn YGNodeStyleSetAlignItems(node: YgNode, v: i32);
    pub fn YGNodeStyleSetAlignSelf(node: YgNode, v: i32);
    pub fn YGNodeStyleSetPositionType(node: YgNode, v: i32);
    pub fn YGNodeStyleSetFlexWrap(node: YgNode, v: i32);
    pub fn YGNodeStyleSetOverflow(node: YgNode, v: i32);
    pub fn YGNodeStyleSetDisplay(node: YgNode, v: i32);
    pub fn YGNodeStyleSetFlexGrow(node: YgNode, v: f32);
    pub fn YGNodeStyleSetFlexShrink(node: YgNode, v: f32);
    pub fn YGNodeStyleSetFlexBasis(node: YgNode, v: f32);
    pub fn YGNodeStyleSetFlexBasisPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetFlexBasisAuto(node: YgNode);
    pub fn YGNodeStyleSetWidth(node: YgNode, v: f32);
    pub fn YGNodeStyleSetWidthPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetWidthAuto(node: YgNode);
    pub fn YGNodeStyleSetHeight(node: YgNode, v: f32);
    pub fn YGNodeStyleSetHeightPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetHeightAuto(node: YgNode);
    pub fn YGNodeStyleSetMinWidth(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMinWidthPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMinHeight(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMinHeightPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMaxWidth(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMaxWidthPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMaxHeight(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMaxHeightPercent(node: YgNode, v: f32);
    pub fn YGNodeStyleSetMargin(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetMarginPercent(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetMarginAuto(node: YgNode, edge: i32);
    pub fn YGNodeStyleSetPadding(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetPaddingPercent(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetBorder(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetPosition(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetPositionPercent(node: YgNode, edge: i32, v: f32);
    pub fn YGNodeStyleSetPositionAuto(node: YgNode, edge: i32);
    pub fn YGNodeStyleSetGap(node: YgNode, gutter: i32, v: f32);
    pub fn YGNodeStyleSetGapPercent(node: YgNode, gutter: i32, v: f32);
    pub fn YGNodeStyleSetAspectRatio(node: YgNode, v: f32);
}

pub const DIRECTION_LTR: i32 = 1;

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
    pub(super) fn from_raw(v: i32) -> Self {
        match v {
            1 => Self::Exactly,
            2 => Self::AtMost,
            _ => Self::Undefined,
        }
    }
}
