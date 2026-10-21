//! Hand port, round 9: the ft/ftaction.c subaction-command handlers that only advance the
//! command pointer (`SKIP_CMD` / `NEXT_CMD`, lb/inlines.h) or set a fighter flag without reading
//! the command word. Handlers that decode `CmdUnion` bitfields are not ported: the command
//! stream's bitfield layout on the native build is not settled yet (see group-fighter.md).
//! `cmd->u` steps by `sizeof(CmdUnion)` (4) per command word, as the C pointer increment does.
//! Checked against the C (cut verbatim, shipping contraction) by tests/ftleaf_ref.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `SKIP_CMD(cmd, n)`: `n` times `++cmd->u`.
#[inline]
pub fn skip_cmd(cmd: &mut CommandInfo, n: usize) {
    unsafe { cmd.anon_8.u = cmd.anon_8.u.wrapping_add(n) }
}

macro_rules! skips {
    ($($name:ident = $n:expr;)*) => {$(
        #[doc = concat!("`", stringify!($name), "`: `SKIP_CMD(cmd, ", stringify!($n), ")`.")]
        pub fn $name(_g: G, cmd: &mut CommandInfo) { skip_cmd(cmd, $n) }
    )*};
}
skips! {
    ftaction_800711dc = 5;
    ftaction_800715ec = 5;
    ftaction_8007168c = 1;
    ftaction_800716f8 = 1;
    ftaction_80071774 = 1;
    ftaction_800717c8 = 1;
    ftaction_80071810 = 1;
    ftaction_80071ca4 = 3;
    ftaction_80071d30 = 1;
    ftaction_80071f0c = 3;
    ftaction_800722c8 = 7;
    ftaction_800726c0 = 4;
    ftaction_80072e24 = 3;
    ftaction_80072fe0 = 3;
    ftaction_8007309c = 2;
    ftaction_8007320c = 4;
    // NEXT_CMD only
    ftaction_8007296c = 1;
    ftaction_800729c4 = 1;
    ftaction_80072a4c = 1;
    ftaction_80072aac = 1;
    ftaction_80072b04 = 1;
    ftaction_80072b84 = 1;
    ftaction_80072be4 = 1;
    ftaction_80072c5c = 1;
    ftaction_80073108 = 1;
}

/// `ftAction_80071908`: `throw_flags_b1 = true`.
pub fn ftaction_80071908(g: G, cmd: &mut CommandInfo) {
    unsafe { fp(g).anon_2210.anon_0.set_throw_flags_b1(1) };
    skip_cmd(cmd, 1);
}
/// `ftAction_8007192C`: `throw_flags_b2 = true`.
pub fn ftaction_8007192c(g: G, cmd: &mut CommandInfo) {
    unsafe { fp(g).anon_2210.anon_0.set_throw_flags_b2(1) };
    skip_cmd(cmd, 1);
}
/// `ftAction_80071950`: `allow_interrupt = true`.
pub fn ftaction_80071950(g: G, cmd: &mut CommandInfo) {
    fp(g).set_allow_interrupt(1);
    skip_cmd(cmd, 1);
}
/// `ftAction_80071974`: `throw_flags_b0 = true`.
pub fn ftaction_80071974(g: G, cmd: &mut CommandInfo) {
    unsafe { fp(g).anon_2210.anon_0.set_throw_flags_b0(1) };
    skip_cmd(cmd, 1);
}
