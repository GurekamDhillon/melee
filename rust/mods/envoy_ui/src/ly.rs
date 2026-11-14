//! Layout for the kit (uiremake): flexbox boxes computed by Taffy, so a screen says what it holds (a column of
//! rows, a header, a card as high as its words) and the boxes come out of the words' own sizes instead of typed
//! coordinates that stop fitting when a word, a font size or the screen's shape changes.
//!
//! The shape follows gpui's wrapper (zed crates/gpui/src/taffy.rs): the tree is rebuilt every frame, text is a
//! leaf measured through a callback (here the game's own font metrics, `gd_ui_measure`), and one compute pass gives
//! every node its rectangle. A screen builds its nodes (`Lay::node`, `Lay::boxed`, `Lay::text`, `Lay::leaf` ..),
//! each carrying what it draws (`P`: a sticker face, its words, an icon, or a box the screen fills itself), calls
//! `Lay::compute`, then `Lay::draw` paints the tree, parents first.
//!
//! Cost: the mod runs in an interpreter, where Taffy takes milliseconds for a panel, so the tree a screen builds is
//! hashed (styles, words, shape) and Taffy only runs when the hash differs from the screen's last one (`Slot`); a
//! present that changes nothing reuses the rectangles. A hover or a colour is paint, not layout, and costs nothing.
//!
//! Memory: Taffy allocates, and a mod has no heap, so the mod's allocator is a bump arena that is emptied when a
//! frame's first layout begins (`begin`). Nothing allocated here may be kept past the frame; a `Lay` is made,
//! used and dropped inside one draw call. Presentation only: no layout value ever reaches the sim.
use crate::kit::*;
use alloc::string::String;
use alloc::vec::Vec;
use gd_ui_enc::*;
use taffy::prelude::*;

// ------------------------------------------------------------------ the frame arena

pub struct Bump;
const ARENA: usize = 384 * 1024;
#[repr(align(16))]
struct Arena([u8; ARENA]);
static mut MEM: Arena = Arena([0; ARENA]);
static mut TOP: usize = 0;
static mut PEAK: usize = 0;
/// The last block handed out (grown in place by `realloc`: a Vec that doubles does not leave its old halves behind).
static mut LAST: (usize, usize) = (0, 0);

unsafe impl core::alloc::GlobalAlloc for Bump {
    unsafe fn alloc(&self, l: core::alloc::Layout) -> *mut u8 {
        let at = (TOP + l.align() - 1) & !(l.align() - 1);
        if at + l.size() > ARENA {
            return core::ptr::null_mut();
        }
        TOP = at + l.size();
        if TOP > PEAK {
            PEAK = TOP;
        }
        LAST = (at, l.size());
        core::ptr::addr_of_mut!(MEM.0).cast::<u8>().add(at)
    }
    unsafe fn dealloc(&self, _p: *mut u8, _l: core::alloc::Layout) {}
    unsafe fn realloc(&self, p: *mut u8, l: core::alloc::Layout, new: usize) -> *mut u8 {
        let base = core::ptr::addr_of_mut!(MEM.0).cast::<u8>();
        let at = p as usize - base as usize;
        if LAST == (at, l.size()) && at + new <= ARENA {
            TOP = at + new;
            if TOP > PEAK {
                PEAK = TOP;
            }
            LAST = (at, new);
            return p;
        }
        let q = self.alloc(core::alloc::Layout::from_size_align_unchecked(new, l.align()));
        if !q.is_null() {
            core::ptr::copy_nonoverlapping(p, q, l.size().min(new));
        }
        q
    }
}

/// Empty the arena: once per present, before the frame's first layout. Everything laid out earlier is gone.
pub fn begin() {
    unsafe {
        TOP = 0;
        LAST = (0, 0);
    }
}

/// The most the arena has held (bytes), for the cost report.
pub fn peak() -> usize {
    unsafe { PEAK }
}

// ------------------------------------------------------------------ the style vocabulary

/// A size: what the content needs, band units, or a share of the parent.
#[derive(Clone, Copy, PartialEq)]
pub enum D {
    Auto,
    Px(f32),
    Pct(f32),
}

/// Where children sit across the box's direction (`align`) and along it (`justify`).
#[derive(Clone, Copy, PartialEq)]
pub enum A {
    Start,
    Center,
    End,
    Stretch,
    Between,
}

/// A box's style, built by chaining (`St::row().gap(6.0).pad(10.0).align(A::Center)`), in the manner of gpui's
/// `Styled`. Rows and columns, gap, padding, fixed and percentage sizes, grow, align and justify, aspect-ratio
/// boxes, wrapping rows.
#[derive(Clone, Copy)]
pub struct St {
    col: bool,
    wrap: bool,
    gap: f32,
    pad: [f32; 4],
    w: D,
    h: D,
    min_w: D,
    max_w: D,
    min_h: D,
    grow: f32,
    shrink: f32,
    align: Option<A>,
    justify: Option<A>,
    this: Option<A>,
    aspect: Option<f32>,
    abs: Option<(f32, f32)>,
}

impl St {
    pub const fn row() -> St {
        St { col: false, wrap: false, gap: 0.0, pad: [0.0; 4], w: D::Auto, h: D::Auto, min_w: D::Auto, max_w: D::Auto, min_h: D::Auto, grow: 0.0, shrink: 1.0, align: None, justify: None, this: None, aspect: None, abs: None }
    }
    pub const fn col() -> St {
        let mut s = St::row();
        s.col = true;
        s
    }
    /// A leaf box of a fixed size.
    pub const fn boxed(w: f32, h: f32) -> St {
        St::row().w(w).h(h).fixed()
    }
    pub const fn gap(mut self, g: f32) -> St {
        self.gap = g;
        self
    }
    /// Padding on every side.
    pub const fn pad(mut self, p: f32) -> St {
        self.pad = [p; 4];
        self
    }
    /// Padding: left, top, right, bottom.
    pub const fn pad4(mut self, l: f32, t: f32, r: f32, b: f32) -> St {
        self.pad = [l, t, r, b];
        self
    }
    pub const fn w(mut self, w: f32) -> St {
        self.w = D::Px(w);
        self
    }
    pub const fn h(mut self, h: f32) -> St {
        self.h = D::Px(h);
        self
    }
    /// Width as a share of the parent (0..1).
    pub const fn w_pct(mut self, p: f32) -> St {
        self.w = D::Pct(p);
        self
    }
    pub const fn h_pct(mut self, p: f32) -> St {
        self.h = D::Pct(p);
        self
    }
    pub const fn min_w(mut self, w: f32) -> St {
        self.min_w = D::Px(w);
        self
    }
    pub const fn max_w(mut self, w: f32) -> St {
        self.max_w = D::Px(w);
        self
    }
    pub const fn min_h(mut self, h: f32) -> St {
        self.min_h = D::Px(h);
        self
    }
    /// Take a share of the room left over along the parent's direction.
    pub const fn grow(mut self, g: f32) -> St {
        self.grow = g;
        self
    }
    /// Never shrink below the size asked for.
    pub const fn fixed(mut self) -> St {
        self.shrink = 0.0;
        self
    }
    /// Where the children sit across the direction.
    pub const fn align(mut self, a: A) -> St {
        self.align = Some(a);
        self
    }
    /// Where the children sit along the direction.
    pub const fn justify(mut self, a: A) -> St {
        self.justify = Some(a);
        self
    }
    /// Where this box sits across its parent's direction (over the parent's `align`).
    pub const fn this(mut self, a: A) -> St {
        self.this = Some(a);
        self
    }
    /// Width / height: the box keeps this shape whatever size it is given (art is never stretched).
    pub const fn aspect(mut self, ratio: f32) -> St {
        self.aspect = Some(ratio);
        self
    }
    /// A row whose children go onto further lines when one cannot hold them.
    pub const fn wrap(mut self) -> St {
        self.wrap = true;
        self
    }
    /// Out of the flow, at (x, y) inside its parent (a badge on a corner, a marker).
    pub const fn at(mut self, x: f32, y: f32) -> St {
        self.abs = Some((x, y));
        self
    }

    fn dim(d: D) -> Dimension {
        match d {
            D::Auto => Dimension::auto(),
            D::Px(v) => Dimension::length(v),
            D::Pct(v) => Dimension::percent(v),
        }
    }
    fn lpa(d: D) -> LengthPercentageAuto {
        match d {
            D::Auto => LengthPercentageAuto::auto(),
            D::Px(v) => LengthPercentageAuto::length(v),
            D::Pct(v) => LengthPercentageAuto::percent(v),
        }
    }
    fn style(&self) -> Style {
        let items = |a: A| match a {
            A::Start => AlignItems::FLEX_START,
            A::Center => AlignItems::CENTER,
            A::End => AlignItems::FLEX_END,
            _ => AlignItems::STRETCH,
        };
        let mut s = Style {
            display: Display::Flex,
            flex_direction: if self.col { FlexDirection::Column } else { FlexDirection::Row },
            flex_wrap: if self.wrap { FlexWrap::Wrap } else { FlexWrap::NoWrap },
            gap: Size { width: LengthPercentage::length(self.gap), height: LengthPercentage::length(self.gap) },
            padding: Rect { left: LengthPercentage::length(self.pad[0]), top: LengthPercentage::length(self.pad[1]), right: LengthPercentage::length(self.pad[2]), bottom: LengthPercentage::length(self.pad[3]) },
            size: Size { width: St::dim(self.w), height: St::dim(self.h) },
            min_size: Size { width: St::lpa(self.min_w), height: St::lpa(self.min_h) },
            max_size: Size { width: St::lpa(self.max_w), height: LengthPercentageAuto::auto() },
            flex_grow: self.grow,
            flex_shrink: self.shrink,
            aspect_ratio: self.aspect,
            ..Style::DEFAULT
        };
        if let Some(a) = self.align {
            s.align_items = items(a);
        }
        if let Some(a) = self.this {
            s.align_self = Some(items(a));
        }
        if let Some(a) = self.justify {
            s.justify_content = match a {
                A::Start => JustifyContent::FLEX_START,
                A::Center => JustifyContent::CENTER,
                A::End => JustifyContent::FLEX_END,
                A::Between => JustifyContent::SPACE_BETWEEN,
                A::Stretch => JustifyContent::STRETCH,
            };
        }
        if let Some((x, y)) = self.abs {
            s.position = Position::Absolute;
            s.inset = Rect { left: St::lpa(D::Px(x)), top: St::lpa(D::Px(y)), right: LengthPercentageAuto::auto(), bottom: LengthPercentageAuto::auto() };
        }
        s
    }
}

// ------------------------------------------------------------------ the tree

/// A node's handle (its index in the frame's node list).
#[derive(Clone, Copy, PartialEq)]
pub struct N(u32);

/// A rectangle in the space the layout was placed in (band units).
#[derive(Clone, Copy, Default)]
pub struct R {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl R {
    pub fn cx(&self) -> f32 {
        self.x + self.w * 0.5
    }
    pub fn cy(&self) -> f32 {
        self.y + self.h * 0.5
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// A text leaf: its words, face, size and line pitch, and how many lines it may take (1: a single line that is
/// set smaller when its box is narrower; more: it wraps on whole words, and the last line takes what is left).
pub struct Words {
    s: String,
    role: fn(u32) -> i32,
    px: u32,
    lh: f32,
    lines: usize,
    /// rich: the item keyword language (kit::rich), measured and drawn with its coloured keywords
    rich: bool,
}

/// What a node draws in its rectangle, once the layout is known (`Lay::draw` walks the tree parents first).
#[derive(Clone, Copy)]
pub enum P {
    None,
    /// a sticker face (`kit::face`): S_FACE_*
    Face(usize),
    /// a flat rounded fill: colour, corner radius
    Fill(u32, f32),
    /// a flat fill with a radius for each corner: colour, [top-left, top-right, bottom-right, bottom-left]
    Fill4(u32, [f32; 4]),
    /// the node's words: colour, ALIGN_*
    Text(u32, i32),
    /// the node's words with an ink edge: colour, ALIGN_*, edge width
    TextEdge(u32, i32, f32),
    /// a kit sprite contained in the box (never stretched): S_*
    Spr(usize),
    /// an item's icon (frame and sprite) filling the box: item id
    Item(i32),
    /// a disc filling the box: colour
    Disc(u32),
    /// something the screen draws itself (`Lay::draw`'s callback): a tag and a word of data
    Own(u8, i32),
}

struct Node {
    st: St,
    /// its children: a run of `Lay::kids`
    kids: (u32, u32),
    parent: u32,
    words: Option<Words>,
    paint: P,
}

/// The most nodes one layout may have (a slot of the cache keeps a rectangle for each).
pub const MAX_NODES: usize = 224;

/// One screen's last layout: the hash of everything that decides it and each node's rectangle relative to its
/// parent. Taffy runs in the wasm interpreter at a few milliseconds a panel, so it only runs when a screen's
/// words, sizes or structure change; a present whose tree hashes the same takes last present's rectangles
/// (measured: the pause panel 2.8 ms a present recomputed every time).
struct Slot {
    key: u64,
    n: u32,
    r: [[f32; 4]; MAX_NODES],
}
/// The screens that keep a layout: the pause panels of both players, the Lab panel, the run-end card.
pub const SLOT_PAUSE: usize = 0;
pub const SLOT_LAB: usize = 2;
pub const SLOT_RESULTS: usize = 3;
const SLOTS_N: usize = 4;
static mut CACHE: [Slot; SLOTS_N] = [const { Slot { key: 0, n: 0, r: [[0.0; 4]; MAX_NODES] } }; SLOTS_N];
/// Layouts computed and layouts taken from the cache since load (the cost report).
static mut STATS: (u32, u32) = (0, 0);
pub fn stats() -> (u32, u32) {
    unsafe { STATS }
}

const NO: u32 = u32::MAX;

pub struct Lay {
    nodes: Vec<Node>,
    kids: Vec<u32>,
    /// FNV-1a over every style, every word and the tree's shape, in creation order
    hash: u64,
    slot: usize,
    origin: (f32, f32),
}

impl Lay {
    /// A layout whose result is kept in cache slot `slot` (SLOT_*) from one present to the next.
    pub fn new(slot: usize) -> Lay {
        Lay { nodes: Vec::with_capacity(128), kids: Vec::with_capacity(160), hash: 0xCBF2_9CE4_8422_2325, slot: slot.min(SLOTS_N - 1), origin: (0.0, 0.0) }
    }

    fn mix(&mut self, v: u32) {
        self.hash = (self.hash ^ v as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }

    fn push(&mut self, st: St, kids: &[N], words: Option<Words>, paint: P) -> N {
        let i = self.nodes.len() as u32;
        // the style, the shape and the words decide the layout (the paint does not: a hover costs nothing)
        let dm = |d: D| match d {
            D::Auto => (0u32, 0u32),
            D::Px(v) => (1, v.to_bits()),
            D::Pct(v) => (2, v.to_bits()),
        };
        let al = |a: Option<A>| a.map_or(0u32, |a| a as u32 + 1);
        let flags = st.col as u32 | (st.wrap as u32) << 1 | al(st.align) << 2 | al(st.justify) << 6 | al(st.this) << 10 | (st.abs.is_some() as u32) << 14 | (st.aspect.is_some() as u32) << 15;
        self.mix(flags);
        for v in [st.gap, st.pad[0], st.pad[1], st.pad[2], st.pad[3], st.grow, st.shrink, st.aspect.unwrap_or(0.0), st.abs.map_or(0.0, |a| a.0), st.abs.map_or(0.0, |a| a.1)] {
            self.mix(v.to_bits());
        }
        for d in [st.w, st.h, st.min_w, st.max_w, st.min_h] {
            let (k, v) = dm(d);
            self.mix(k);
            self.mix(v);
        }
        self.mix(kids.len() as u32);
        if let Some(w) = words.as_ref() {
            self.mix(w.px | (w.lines as u32) << 8 | (w.rich as u32) << 16 | 1 << 31);
            self.mix((w.role)(w.px) as u32);
            for b in w.s.as_bytes() {
                self.mix(*b as u32);
            }
        }
        let start = self.kids.len() as u32;
        for k in kids {
            self.kids.push(k.0);
            if let Some(c) = self.nodes.get_mut(k.0 as usize) {
                c.parent = i;
            }
        }
        self.nodes.push(Node { st, kids: (start, kids.len() as u32), parent: NO, words, paint });
        N(i)
    }

    /// A box with children.
    pub fn node(&mut self, st: St, kids: &[N]) -> N {
        self.push(st, kids, None, P::None)
    }
    /// A box with children that draws `paint` under them (a sticker, a fill).
    pub fn boxed(&mut self, st: St, paint: P, kids: &[N]) -> N {
        self.push(st, kids, None, paint)
    }
    /// A box with no children: a spacer, an icon's place, a bar (`St::boxed`, or any style), drawing `paint`.
    pub fn leaf(&mut self, st: St, paint: P) -> N {
        self.push(st, &[], None, paint)
    }
    /// A leaf that takes the room left over along its parent's direction.
    pub fn spring(&mut self) -> N {
        self.push(St::row().grow(1.0), &[], None, P::None)
    }
    /// One line of words in `rgba` (it keeps its width; set smaller, never cut, if its box is narrower).
    pub fn text(&mut self, s: &str, role: fn(u32) -> i32, px: u32, rgba: u32) -> N {
        self.words(s, role, px, 1, false, St::row().fixed(), P::Text(rgba, ALIGN_LEFT))
    }
    /// One line that gives way to its neighbours in a row (it is then set smaller to fit what it gets).
    pub fn text_soft(&mut self, s: &str, role: fn(u32) -> i32, px: u32, rgba: u32) -> N {
        self.words(s, role, px, 1, false, St::row().min_w(0.0), P::Text(rgba, ALIGN_LEFT))
    }
    /// Words that wrap inside the width their parent gives them, over at most `lines` lines.
    pub fn para(&mut self, s: &str, role: fn(u32) -> i32, px: u32, rgba: u32, lines: usize) -> N {
        self.words(s, role, px, lines.max(1), false, St::row().min_w(0.0), P::Text(rgba, ALIGN_LEFT))
    }
    /// A card's text in the item keyword language (kit::rich), wrapping over at most `lines` lines.
    pub fn rich(&mut self, s: &str, px: u32, rgba: u32, lines: usize) -> N {
        self.words(s, bs, px, lines.max(1), true, St::row().min_w(0.0), P::Text(rgba, ALIGN_LEFT))
    }
    /// Words with their own style and paint (a centred line, an edged title, a line that grows).
    #[allow(clippy::too_many_arguments)]
    pub fn words(&mut self, s: &str, role: fn(u32) -> i32, px: u32, lines: usize, rich: bool, st: St, paint: P) -> N {
        let lh = ((px as f32 * if rich { 1.23 } else { 1.3 }) + 0.5) as i32 as f32;
        self.push(st, &[], Some(Words { s: String::from(s), role, px, lh, lines, rich }), paint)
    }
    /// Change what a node draws.
    pub fn paint(&mut self, n: N, paint: P) {
        if let Some(c) = self.nodes.get_mut(n.0 as usize) {
            c.paint = paint;
        }
    }

    /// Lay the tree out under `root`, whose top-left corner goes to (x, y). `w` / `h`: the room it is given
    /// (None: as much as its content needs). The same tree in the same room as last present costs nothing: its
    /// rectangles are still in the slot.
    pub fn compute(&mut self, root: N, x: f32, y: f32, w: Option<f32>, h: Option<f32>) {
        self.origin = (x, y);
        // the key: the tree's hash with the root and the room it is given
        let mut key = self.hash;
        for v in [root.0, w.map_or(0x7FC0_0001, f32::to_bits), h.map_or(0x7FC0_0001, f32::to_bits)] {
            key = (key ^ v as u64).wrapping_mul(0x0000_0100_0000_01B3);
        }
        let n = self.nodes.len().min(MAX_NODES);
        let slot = unsafe { &mut *core::ptr::addr_of_mut!(CACHE[self.slot]) };
        if slot.key == key && slot.n == n as u32 {
            unsafe { STATS.1 += 1 };
            return;
        }
        unsafe { STATS.0 += 1 };
        // band units are not pixels (1.5 px at 720p): no rounding here, the host snaps straight edges itself
        let mut t: TaffyTree<u32> = TaffyTree::with_capacity(n);
        t.disable_rounding();
        // children are always made before their parents: one pass in creation order builds the tree
        let mut ids: Vec<NodeId> = Vec::with_capacity(n);
        let mut tmp: Vec<NodeId> = Vec::with_capacity(16);
        for (i, nd) in self.nodes.iter().enumerate().take(n) {
            let id = if nd.kids.1 == 0 {
                t.new_leaf_with_context(nd.st.style(), i as u32).unwrap()
            } else {
                tmp.clear();
                for k in &self.kids[nd.kids.0 as usize..(nd.kids.0 + nd.kids.1) as usize] {
                    if let Some(c) = ids.get(*k as usize) {
                        tmp.push(*c);
                    }
                }
                t.new_with_children(nd.st.style(), &tmp).unwrap()
            };
            ids.push(id);
        }
        let Some(rid) = ids.get(root.0 as usize).copied() else { return };
        let space = Size { width: w.map_or(AvailableSpace::MaxContent, AvailableSpace::Definite), height: h.map_or(AvailableSpace::MaxContent, AvailableSpace::Definite) };
        let nodes = &self.nodes;
        let _ = t.compute_layout_with_measure(rid, space, |inputs, _, ctx, style| {
            taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, |known, avail| match ctx.and_then(|i| nodes.get(*i as usize)).and_then(|nd| nd.words.as_ref()) {
                Some(wd) => measure(wd, known, avail),
                None => Size::ZERO,
            })
        });
        for (i, id) in ids.iter().enumerate() {
            if let Ok(l) = t.layout(*id) {
                slot.r[i] = [l.location.x, l.location.y, l.size.width, l.size.height];
            }
        }
        slot.key = key;
        slot.n = n as u32;
    }

    /// Move the laid-out tree: its root's top-left corner goes to (x, y) (no layout is done).
    pub fn place(&mut self, x: f32, y: f32) {
        self.origin = (x, y);
    }

    fn local(&self, i: u32) -> [f32; 4] {
        unsafe { CACHE[self.slot].r.get(i as usize).copied().unwrap_or([0.0; 4]) }
    }

    /// A node's rectangle (after `compute`).
    pub fn rect(&self, n: N) -> R {
        let (mut x, mut y) = self.origin;
        let l = self.local(n.0);
        // a node's location is relative to its parent: the sum up the chain is its place in the layout
        let mut at = n.0;
        while at != NO {
            let q = self.local(at);
            x += q[0];
            y += q[1];
            at = self.nodes.get(at as usize).map_or(NO, |nd| nd.parent);
        }
        R { x, y, w: l[2], h: l[3] }
    }

    /// Draw the tree under `root`: every node's paint in its rectangle, parents before their children, faded by
    /// `a`. `own` draws the `P::Own` nodes (tag, data, rectangle).
    pub fn draw(&self, e: &mut E, root: N, a: f32, own: &mut dyn FnMut(&mut E, u8, i32, R)) {
        let (ox, oy) = self.origin;
        self.walk(e, root.0, ox, oy, a, own);
    }

    fn walk(&self, e: &mut E, i: u32, px: f32, py: f32, a: f32, own: &mut dyn FnMut(&mut E, u8, i32, R)) {
        let Some(nd) = self.nodes.get(i as usize) else { return };
        let l = self.local(i);
        let r = R { x: px + l[0], y: py + l[1], w: l[2], h: l[3] };
        match nd.paint {
            P::None => {}
            P::Face(id) => face(e, id, r.x, r.y, r.w, r.h, alpha(WHITE, a)),
            P::Fill(col, rad) => {
                e.round_rect(r.x, r.y, r.w, r.h, rad, alpha(col, a));
            }
            P::Fill4(col, rad) => {
                e.shape4(r.x, r.y, r.w, r.h, rad, 0.0, alpha(col, a), 0);
            }
            P::Text(col, al) => put_text(e, nd, r, alpha(col, a), al),
            P::TextEdge(col, al, edge) => {
                e.outline(edge, alpha(INK, a));
                put_text(e, nd, r, alpha(col, a), al);
                e.outline(0.0, 0);
            }
            P::Spr(id) => spr(e, id, r.x, r.y, r.w, r.h, alpha(WHITE, a)),
            // the icon's frame cell is 4/3 of the size it is asked for: the cell fills the box
            P::Item(id) => item_icon(e, id, r.cx(), r.cy(), r.w.min(r.h) * 0.75, a),
            P::Disc(col) => disc(e, r.cx(), r.cy(), r.w.min(r.h), alpha(col, a), 0.0, 0),
            P::Own(tag, data) => own(e, tag, data, r),
        }
        for k in nd.kids.0..nd.kids.0 + nd.kids.1 {
            if let Some(c) = self.kids.get(k as usize) {
                self.walk(e, *c, r.x, r.y, a, own);
            }
        }
    }

    /// Nodes in the tree (the cost report).
    pub fn count(&self) -> usize {
        self.nodes.len()
    }
}

/// A text leaf in its rectangle: `align` anchors the lines inside it. A single line narrower than its words is set
/// smaller (down to two thirds); wrapped words take the lines the box has.
fn put_text(e: &mut E, nd: &Node, r: R, rgba: u32, align: i32) {
    let Some(wd) = nd.words.as_ref() else { return };
    let x = match align {
        ALIGN_CENTER => r.cx(),
        ALIGN_RIGHT => r.right(),
        _ => r.x,
    };
    let top = r.y + (wd.lh - wd.px as f32 * 1.25) * 0.5;
    if wd.rich {
        let fit = (((r.h + 0.5) / wd.lh) as i32).clamp(1, wd.lines as i32);
        rich(e, r.x, top, r.w + 0.5, wd.px, wd.lh, rgba, false, &wd.s, fit);
    } else if wd.lines <= 1 {
        text_fit(e, x, top, wd.role, wd.px, (wd.px * 2 / 3).max(7), rgba, align, r.w + 0.5, &wd.s);
    } else {
        let fit = ((r.h + 0.5) / wd.lh) as usize;
        wrap(e, x, top, r.w + 0.5, wd.role, wd.px, wd.lh, rgba, align, &wd.s, fit.clamp(1, wd.lines));
    }
}

/// A text leaf's size for the room it is offered: the game's own font metrics (`gd_ui_measure`).
fn measure(wd: &Words, known: Size<Option<f32>>, avail: Size<AvailableSpace>) -> Size<f32> {
    let role = (wd.role)(wd.px);
    // a rich line's keywords are in the display face, the wider one: its width is an upper bound
    let natural = if wd.rich { tw(&wd.s, d(wd.px)).max(tw(&wd.s, role)) } else { tw(&wd.s, role) };
    let room = known.width.or(match avail.width {
        AvailableSpace::Definite(w) => Some(w),
        // the narrowest it can be: wrapping words their longest word, a single line nothing (it is set smaller)
        AvailableSpace::MinContent => Some(if wd.lines > 1 { wd.s.split(' ').map(|w| tw(w, role)).fold(0.0, f32::max) } else { 0.0 }),
        AvailableSpace::MaxContent => None,
    });
    let w = match room {
        Some(r) if r < natural => r,
        _ => natural,
    };
    let lines = if wd.lines <= 1 || w >= natural {
        1
    } else if wd.rich {
        (rich_lines(w + 0.5, wd.px, &wd.s).max(1) as usize).min(wd.lines)
    } else {
        wrap_lines(&wd.s, w + 0.5, role).clamp(1, wd.lines)
    };
    Size { width: known.width.unwrap_or(w), height: known.height.unwrap_or(lines as f32 * wd.lh) }
}
