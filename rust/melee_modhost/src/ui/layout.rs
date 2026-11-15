//! gd-ui-v1.5 `gd_ui_layout`: flexbox layout for a mod's boxes, computed here, natively, by Taffy.
//!
//! A mod describes a tree of boxes (rows, columns, gaps, padding, sizes, text leaves) and gets each box's
//! rectangle back. The compute used to run inside the mod (Taffy compiled to wasm): in the interpreter that was
//! 2.8 ms for the Envoy pause panel, 3 to 4.5 ms on any present whose content changed, and needed four times the
//! presentation fuel. Natively the same tree is tens of microseconds, so the mod keeps only its builder and its
//! drawing. Text is measured with the host's own font tables, the ones the text is then drawn with.
//!
//! The request is little-endian u32 words:
//!   header  magic `LAY1`, node count, root index, available width, available height (f32 bits; NaN: unbounded)
//!   node    flags (bit 0 column, 1 wrap, 2..5 align-items + 1, 6..9 justify-content + 1, 10..13 align-self + 1),
//!           gap, padding l t r b, then five sizes as (kind, value) pairs: width, height, min width, max width,
//!           min height (kind 0 auto, 1 units, 2 fraction of the parent), grow, shrink, aspect (NaN: none),
//!           absolute x, y (NaN: in flow), child count, text kind (0 none, 1 plain, 2 runs)
//!           then the child indices (each lower than this node's: children come first)
//!           plain   role, max lines, line height, byte length, bytes (padded to a word)
//!           runs    max lines, line height, space width, run count, then per run: role, spaces before | starts
//!                   with punctuation << 16, byte length, bytes (padded). A run is one unbreakable word.
//! Alignment values: 1 start, 2 centre, 3 end, 4 stretch, 5 space-between. The answer is four f32 per node (x, y
//! relative to its parent, width, height), in node order.
//!
//! Presentation only: nothing here is read by a sim, and the result depends only on the request and the font.
use melee_gx::ui::Atlas;
use taffy::prelude::*;

pub const MAGIC: u32 = 0x3159_414C; // "LAY1"
/// The most nodes one request may carry.
pub const MAX_NODES: usize = 512;

enum Words<'a> {
    Plain { role: i32, lines: usize, lh: f32, s: &'a str },
    Runs { lines: usize, lh: f32, space: f32, runs: Vec<(i32, u32, bool, &'a str)> },
}

struct Rd<'a> {
    b: &'a [u8],
    o: usize,
}
impl<'a> Rd<'a> {
    fn u(&mut self) -> Option<u32> {
        let v = self.b.get(self.o..self.o + 4)?;
        self.o += 4;
        Some(u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
    }
    fn f(&mut self) -> Option<f32> {
        self.u().map(f32::from_bits)
    }
    fn s(&mut self, len: usize) -> Option<&'a str> {
        let v = self.b.get(self.o..self.o.checked_add(len)?)?;
        self.o += len.div_ceil(4) * 4;
        core::str::from_utf8(v).ok()
    }
}

fn dim(kind: u32, v: f32) -> Dimension {
    match kind {
        1 if v.is_finite() => Dimension::length(v),
        2 if v.is_finite() => Dimension::percent(v),
        _ => Dimension::auto(),
    }
}
fn lpa(kind: u32, v: f32) -> LengthPercentageAuto {
    match kind {
        1 if v.is_finite() => LengthPercentageAuto::length(v),
        2 if v.is_finite() => LengthPercentageAuto::percent(v),
        _ => LengthPercentageAuto::auto(),
    }
}
fn lp(v: f32) -> LengthPercentage {
    LengthPercentage::length(if v.is_finite() { v.max(0.0) } else { 0.0 })
}

/// Where the first line of `s` ends at width `w` (the mod's own rule, kit::line_end: the longest run of whole
/// words that fits; a single word wider than the line stays whole).
fn line_end(a: &Atlas, s: &str, w: f32, role: i32) -> usize {
    let st = super::cmd::style(role);
    if a.measure(s, st) <= w {
        return s.len();
    }
    let mut cut = 0;
    for (i, c) in s.char_indices() {
        if c == ' ' {
            if a.measure(&s[..i], st) <= w {
                cut = i;
            } else {
                break;
            }
        }
    }
    if cut == 0 { s.find(' ').unwrap_or(s.len()) } else { cut }
}

fn plain_lines(a: &Atlas, s: &str, w: f32, role: i32) -> usize {
    let mut rest = s.trim();
    let mut n = 0;
    while !rest.is_empty() && n < 64 {
        let cut = line_end(a, rest, w, role);
        rest = rest[cut..].trim_start();
        n += 1;
    }
    n.max(1)
}

/// The lines a run list needs at width `w` (the mod's kit::rich rule) and its width on one line.
fn run_lines(a: &Atlas, runs: &[(i32, u32, bool, &str)], space: f32, w: f32) -> (usize, f32) {
    let (mut cx, mut line, mut total) = (0.0f32, 0usize, 0.0f32);
    for &(role, spaces, punct, s) in runs {
        cx += space * spaces as f32;
        total += space * spaces as f32;
        let ww = a.measure(s, super::cmd::style(role));
        if cx > 0.0 && cx + ww > w && !punct {
            line += 1;
            cx = 0.0;
        }
        cx += ww;
        total += ww;
    }
    (line + 1, total)
}

fn measure(a: &Atlas, wd: &Words, known: Size<Option<f32>>, avail: Size<AvailableSpace>) -> Size<f32> {
    let (natural, lines_max, lh) = match wd {
        Words::Plain { role, lines, lh, s } => (a.measure(s, super::cmd::style(*role)), *lines, *lh),
        Words::Runs { lines, lh, space, runs } => (run_lines(a, runs, *space, f32::MAX).1, *lines, *lh),
    };
    let room = known.width.or(match avail.width {
        AvailableSpace::Definite(w) => Some(w),
        // the narrowest it can be: wrapping words their longest word, a single line nothing (it is set smaller)
        AvailableSpace::MinContent => Some(if lines_max > 1 {
            match wd {
                Words::Plain { role, s, .. } => s.split(' ').map(|w| a.measure(w, super::cmd::style(*role))).fold(0.0, f32::max),
                Words::Runs { runs, .. } => runs.iter().map(|r| a.measure(r.3, super::cmd::style(r.0))).fold(0.0, f32::max),
            }
        } else {
            0.0
        }),
        AvailableSpace::MaxContent => None,
    });
    let w = match room {
        Some(r) if r < natural => r,
        _ => natural,
    };
    let lines = if lines_max <= 1 || w >= natural {
        1
    } else {
        match wd {
            Words::Plain { role, s, .. } => plain_lines(a, s, w + 0.5, *role),
            Words::Runs { space, runs, .. } => run_lines(a, runs, *space, w + 0.5).0,
        }
        .clamp(1, lines_max)
    };
    Size { width: known.width.unwrap_or(w), height: known.height.unwrap_or(lines as f32 * lh) }
}

/// Lay a request out. Returns four f32 per node, or what is wrong with the request.
pub fn compute(req: &[u8], atlas: &Atlas) -> Result<Vec<f32>, &'static str> {
    let mut r = Rd { b: req, o: 0 };
    let bad = "truncated request";
    if r.u().ok_or(bad)? != MAGIC {
        return Err("bad magic");
    }
    let n = r.u().ok_or(bad)? as usize;
    let root = r.u().ok_or(bad)? as usize;
    let (aw, ah) = (r.f().ok_or(bad)?, r.f().ok_or(bad)?);
    if n == 0 || n > MAX_NODES || root >= n {
        return Err("node count or root");
    }
    let mut t: TaffyTree<usize> = TaffyTree::with_capacity(n);
    // the mod's units are not pixels: no rounding here, the renderer snaps straight edges itself
    t.disable_rounding();
    let mut ids: Vec<NodeId> = Vec::with_capacity(n);
    let mut words: Vec<Option<Words>> = Vec::with_capacity(n);
    let mut kids: Vec<NodeId> = Vec::new();
    for i in 0..n {
        let flags = r.u().ok_or(bad)?;
        let gap = r.f().ok_or(bad)?;
        let pad = [r.f().ok_or(bad)?, r.f().ok_or(bad)?, r.f().ok_or(bad)?, r.f().ok_or(bad)?];
        let mut sz = [(0u32, 0f32); 5];
        for s in sz.iter_mut() {
            *s = (r.u().ok_or(bad)?, r.f().ok_or(bad)?);
        }
        let (grow, shrink, aspect) = (r.f().ok_or(bad)?, r.f().ok_or(bad)?, r.f().ok_or(bad)?);
        let (ax, ay) = (r.f().ok_or(bad)?, r.f().ok_or(bad)?);
        let nk = r.u().ok_or(bad)? as usize;
        let tk = r.u().ok_or(bad)?;
        let items = |v: u32| match v {
            1 => Some(AlignItems::FLEX_START),
            2 => Some(AlignItems::CENTER),
            3 => Some(AlignItems::FLEX_END),
            4 => Some(AlignItems::STRETCH),
            _ => None,
        };
        let mut st = Style {
            display: Display::Flex,
            flex_direction: if flags & 1 != 0 { FlexDirection::Column } else { FlexDirection::Row },
            flex_wrap: if flags & 2 != 0 { FlexWrap::Wrap } else { FlexWrap::NoWrap },
            gap: Size { width: lp(gap), height: lp(gap) },
            padding: Rect { left: lp(pad[0]), top: lp(pad[1]), right: lp(pad[2]), bottom: lp(pad[3]) },
            size: Size { width: dim(sz[0].0, sz[0].1), height: dim(sz[1].0, sz[1].1) },
            min_size: Size { width: lpa(sz[2].0, sz[2].1), height: lpa(sz[4].0, sz[4].1) },
            max_size: Size { width: lpa(sz[3].0, sz[3].1), height: LengthPercentageAuto::auto() },
            flex_grow: if grow.is_finite() { grow.max(0.0) } else { 0.0 },
            flex_shrink: if shrink.is_finite() { shrink.max(0.0) } else { 1.0 },
            aspect_ratio: (aspect.is_finite() && aspect > 0.0).then_some(aspect),
            ..Style::DEFAULT
        };
        if let Some(a) = items((flags >> 2) & 15) {
            st.align_items = a;
        }
        if let Some(a) = items((flags >> 10) & 15) {
            st.align_self = Some(a);
        }
        match (flags >> 6) & 15 {
            1 => st.justify_content = JustifyContent::FLEX_START,
            2 => st.justify_content = JustifyContent::CENTER,
            3 => st.justify_content = JustifyContent::FLEX_END,
            4 => st.justify_content = JustifyContent::STRETCH,
            5 => st.justify_content = JustifyContent::SPACE_BETWEEN,
            _ => {}
        }
        if ax.is_finite() && ay.is_finite() {
            st.position = Position::Absolute;
            st.inset = Rect { left: LengthPercentageAuto::length(ax), top: LengthPercentageAuto::length(ay), right: LengthPercentageAuto::auto(), bottom: LengthPercentageAuto::auto() };
        }
        kids.clear();
        for _ in 0..nk {
            let k = r.u().ok_or(bad)? as usize;
            if k >= i {
                return Err("a child is not before its parent");
            }
            kids.push(ids[k]);
        }
        let wd = match tk {
            0 => None,
            1 => {
                let role = r.u().ok_or(bad)? as i32;
                let lines = r.u().ok_or(bad)? as usize;
                let lh = r.f().ok_or(bad)?;
                let len = r.u().ok_or(bad)? as usize;
                Some(Words::Plain { role, lines, lh, s: r.s(len).ok_or("text is not UTF-8 or runs past the request")? })
            }
            2 => {
                let lines = r.u().ok_or(bad)? as usize;
                let (lh, space) = (r.f().ok_or(bad)?, r.f().ok_or(bad)?);
                let nr = r.u().ok_or(bad)? as usize;
                if nr > 4096 {
                    return Err("too many runs");
                }
                let mut runs = Vec::with_capacity(nr);
                for _ in 0..nr {
                    let role = r.u().ok_or(bad)? as i32;
                    let sp = r.u().ok_or(bad)?;
                    let len = r.u().ok_or(bad)? as usize;
                    runs.push((role, sp & 0xFFFF, sp >> 16 != 0, r.s(len).ok_or("text is not UTF-8 or runs past the request")?));
                }
                Some(Words::Runs { lines, lh, space, runs })
            }
            _ => return Err("text kind"),
        };
        // a role the text op would refuse is refused here too (the same check as OP_TEXT)
        let role_ok = |role: i32| (0..super::cmd::ROLES.len() as i32).contains(&(role & 0xFF)) && role as u32 & !0x00FF_1FFF == 0 && (role >> 8) & 3 != 3;
        match wd.as_ref() {
            Some(Words::Plain { role, .. }) if !role_ok(*role) => return Err("text role"),
            Some(Words::Runs { runs, .. }) if runs.iter().any(|x| !role_ok(x.0)) => return Err("text role"),
            _ => {}
        }
        let id = if nk == 0 { t.new_leaf_with_context(st, i) } else { t.new_with_children(st, &kids) }.map_err(|_| "taffy refused a node")?;
        ids.push(id);
        words.push(wd);
    }
    let av = |v: f32| if v.is_finite() { AvailableSpace::Definite(v) } else { AvailableSpace::MaxContent };
    t.compute_layout_with_measure(ids[root], Size { width: av(aw), height: av(ah) }, |inputs, _, ctx, style| {
        taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, |known, avail| match ctx.and_then(|i| words.get(*i)).and_then(|w| w.as_ref()) {
            Some(wd) => measure(atlas, wd, known, avail),
            None => Size::ZERO,
        })
    })
    .map_err(|_| "taffy failed")?;
    let mut out = Vec::with_capacity(n * 4);
    for id in &ids {
        let l = t.layout(*id).map_err(|_| "no layout")?;
        out.extend_from_slice(&[l.location.x, l.location.y, l.size.width, l.size.height]);
    }
    Ok(out)
}
