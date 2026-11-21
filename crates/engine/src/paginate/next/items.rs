//! What the newer engine packs into pages: items made from the laid out content, the fragments
//! still to place, and the placements a page ends up with.

use std::rc::Rc;

use crate::tree::Node;

pub type ItemRef = Rc<Item>;

pub enum Item {
    /// Content, or blank space when `data` is `None`, of a fixed height.
    Leaf(Leaf),
    /// Children stacked down the page. `data` is the box they belong to, if any.
    Column {
        data: Option<Rc<Data>>,
        repeat: bool,
        children: Vec<ItemRef>,
    },
    /// Children side by side, cut together at one line.
    Row { children: Vec<ItemRef> },
    /// A break that must happen here (`force`), or must not; `ahead` > 0 keeps any break out of
    /// that much space below.
    Penalty { force: bool, ahead: f64 },
    /// A node with `render` functions: rendered when the flow reaches it, for that page.
    Lazy {
        node: Box<Node>,
        repeat: bool,
        absolute: bool,
    },
}

pub struct Leaf {
    pub height: f64,
    pub repeat: bool,
    pub data: Option<Rc<Data>>,
    pub split: Split,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Split {
    No,
    /// Blank space; `collapse` drops what is left of it at a break.
    Space {
        collapse: bool,
    },
    /// The node's own split: text between lines, a childless box by height.
    Node,
}

/// The node an item stands for. A container's children are items of their own, so it keeps
/// none but `kept`: a row's absolutely placed children, which are not in its flow.
pub struct Data {
    pub node: Node,
    pub kept: Vec<(usize, Node)>,
}

impl Item {
    pub fn data(&self) -> Option<&Rc<Data>> {
        match self {
            Item::Leaf(l) => l.data.as_ref(),
            Item::Column { data, .. } => data.as_ref(),
            _ => None,
        }
    }

    pub fn is_repeat(&self) -> bool {
        match self {
            Item::Leaf(l) => l.repeat,
            Item::Column { repeat, .. } | Item::Lazy { repeat, .. } => *repeat,
            _ => false,
        }
    }

    pub fn set_repeat(&mut self) {
        match self {
            Item::Leaf(l) => l.repeat = true,
            Item::Column { repeat, .. } | Item::Lazy { repeat, .. } => *repeat = true,
            _ => {}
        }
    }

    pub fn is_group(&self) -> bool {
        matches!(self, Item::Column { .. } | Item::Row { .. })
    }

    pub fn is_forbid(&self) -> bool {
        matches!(self, Item::Penalty { force: false, .. })
    }
}

pub fn space(height: f64, collapse: bool) -> Item {
    Item::Leaf(Leaf {
        height,
        repeat: false,
        data: None,
        split: Split::Space { collapse },
    })
}

/// An item still to place, or the part of one left over from the page before.
#[derive(Clone)]
pub struct Fragment {
    pub item: ItemRef,
    pub first: bool,
    pub children: Vec<Fragment>,
    /// The repeating `render` node this came out of.
    pub origin: Option<ItemRef>,
}

pub fn to_fragments(items: &[ItemRef]) -> Vec<Fragment> {
    items
        .iter()
        .map(|item| Fragment {
            item: item.clone(),
            first: true,
            children: match &**item {
                Item::Column { children, .. } | Item::Row { children } => to_fragments(children),
                _ => Vec::new(),
            },
            origin: None,
        })
        .collect()
}

/// The height of what is still to place of a fragment.
pub fn height(f: &Fragment) -> f64 {
    match &*f.item {
        Item::Penalty { .. } | Item::Lazy { .. } => 0.0,
        Item::Row { .. } => f.children.iter().map(height).fold(0.0, f64::max),
        Item::Column { .. } => f.children.iter().map(height).fold(0.0, |a, h| a + h),
        Item::Leaf(l) => l.height,
    }
}

/// Where an item landed on its page.
pub struct Placed {
    pub item: ItemRef,
    pub y: f64,
    pub height: f64,
    pub first: bool,
    pub last: bool,
    pub children: Option<Vec<Placed>>,
}

/// How far down placements reached.
pub fn reach(placed: &[Placed]) -> f64 {
    placed.iter().map(|p| p.y + p.height).fold(0.0, f64::max)
}

/// Nothing but repeating items.
pub fn is_repeat_placed(p: &Placed) -> bool {
    p.item.is_repeat()
        || p.children
            .as_ref()
            .is_some_and(|c| !c.is_empty() && c.iter().all(is_repeat_placed))
}

pub fn fully_placed(placed: &[Placed], item: &ItemRef) -> bool {
    placed.iter().any(|p| Rc::ptr_eq(&p.item, item) && p.last)
}

pub struct FillResult {
    pub placed: Vec<Placed>,
    pub remaining: Vec<Fragment>,
}

pub enum Step {
    Continue,
    Rewind,
    Done(FillResult),
    Decline,
}

/// Renders a lazy node for a page (1-based within its `<Page>`) and makes it an item.
pub type Materialize<'a> = dyn FnMut(&Node, bool, u32) -> Result<Item, String> + 'a;
