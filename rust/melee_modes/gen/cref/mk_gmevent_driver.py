#!/usr/bin/env python3
"""Writes gen/cref/tu/gmevent.c and tests/tu_gmevent.rs from one op table, so the C driver and
the Rust test make the same draws in the same order
(python rust/melee_modes/gen/cref/mk_gmevent_driver.py)."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
NL = chr(10)
PROCS = ["gm_801BC4F4", "gm_801BC670", "gm_801BC754", "gm_801BC9E8", "gm_801BCAF0", "gm_801BCC9C", "gm_801BCF20",
         "gm_801BCF40", "gm_801BD028", "gm_801BD164", "gm_801BD30C", "gm_801BD44C", "gm_801BD46C", "gm_801BD658",
         "gm_801BD7FC", "gm_801BD93C", "gm_801BDAD4", "gm_801BDAF4", "gm_801BDC08", "gm_801BDD44", "gm_801BDE94",
         "gm_801BE37C", "gm_801BE39C", "gm_801BE618", "gm_801BE638", "failEvent"]
SPECIAL = [4, 9, 12, 15, 19, 21, 23, 27, 29, 35, 39, 43, 44, 48, 36, 49]
NCASE = 300

# (C statement(s), Rust statement(s)); `ret` is the case's return word.
OPS = []
CHARLIST_OPS = []


def op(c, r, charlist=False):
    if charlist:
        CHARLIST_OPS.append(len(OPS))
    OPS.append((c, r))


op("gm_801BA8FC();", "m::gm_801BA8FC();")
op("{ int lo = (int) (fv() % 4); int hi = lo + (int) (fv() % (u32) (5 - lo)); int a3 = (int) (fv() & 1); "
   "gm_801BA938(&GM.vs.unk_530, lo, hi, a3); }",
   "{ let lo = (fv() % 4) as i32; let hi = lo + (fv() % (5 - lo) as u32) as i32; let a3 = (fv() & 1) as i32; "
   "m::gm_801BA938(&raw mut GM.vs.unk_530, lo, hi, a3); }")
op("ST.info.enter_data = &css_data; onEnterCss(&ST);",
   "ST.info.enter_data = &raw mut m::css_data as *mut _; m::onEnterCss(&raw mut ST);")
op("ST.info.exit_data = &css_data; onExitCss(&ST);",
   "ST.info.exit_data = &raw mut m::css_data as *mut _; m::onExitCss(&raw mut ST);")
op("gm_801BAB40(&PID, &SRC);", "m::gm_801BAB40(&raw mut PID, &raw mut SRC);")
op("{ int n = (int) (fv() % 6); ST.info.enter_data = &vs_enter_data; ret = (u32) gm_801BAC9C(&ST, n); }",
   "{ let n = (fv() % 6) as i32; ST.info.enter_data = &raw mut m::vs_enter_data as *mut _; "
   "ret = m::gm_801BAC9C(&raw mut ST, n) as u32; }", charlist=True)
op("ST.info.enter_data = &vs_enter_data; onEnterVs(&ST);",
   "ST.info.enter_data = &raw mut m::vs_enter_data as *mut _; m::onEnterVs(&raw mut ST);", charlist=True)
op("ST.info.exit_data = &vs_exit_data[1]; onExitVs(&ST);",
   "ST.info.exit_data = &raw mut m::vs_exit_data[1] as *mut _; m::onExitVs(&raw mut ST);")
op("gm_Mode_Event_OnInit();", "m::gm_Mode_Event_OnInit();")
op("gm_801BBB64();", "m::gm_801BBB64();")
op("gm_Mode_Event_OnLoad();", "m::gm_Mode_Event_OnLoad();")
op("gm_Mode_Event_OnUnload();", "m::gm_Mode_Event_OnUnload();")
op("fn_801BBFE8();", "m::fn_801BBFE8();")
op("gm_801BC00C();", "m::gm_801BC00C();")
op("ret = (u32) gm_801BC488();", "ret = m::gm_801BC488() as u32;")
for p in PROCS:
    op(f"{p}(&G1);", f"m::{p}(&raw mut G1);")
op("ret = PI(gm_GetEventData());", "ret = pi(m::gm_GetEventData());")
op("ret = (u32) gm_GetNextColor((u8) fv());", "ret = m::gm_GetNextColor(fv() as u8) as u32;")
op("gm_801BBB64_inline(&GM.vs.unk_530);", "m::gm_801BBB64_inline(&raw mut GM.vs.unk_530);")
op("gm_801BC00C_inline(&SRC);", "m::gm_801BC00C_inline(&raw mut SRC);")
op("ret = (u32) (int) gm_801BC00C_GetCharacter(&SRC);",
   "ret = m::gm_801BC00C_GetCharacter(&raw mut SRC) as i32 as u32;")
op("ret = (u32) gm_801BC00C_GetCharacterKind(&SRC);", "ret = m::gm_801BC00C_GetCharacterKind(&raw mut SRC);")
op("gm_801BEA10((int) fv());", "m::gm_801BEA10(fv() as i32);")
op("gm_801BEA4C((int) fv());", "m::gm_801BEA4C(fv() as i32);")
op("{ int a = (int) fv(); int b = (int) (fv() % 3); gm_801BEA88(a, b); }",
   "{ let a = fv() as i32; let b = (fv() % 3) as i32; m::gm_801BEA88(a, b); }")
op("gm_801BEAF0((int) fv());", "m::gm_801BEAF0(fv() as i32);")
op("gm_801BEB2C((int) fv());", "m::gm_801BEB2C(fv() as i32);")
op("gm_801BEB68((int) fv());", "m::gm_801BEB68(fv() as i32);")
op("gm_801BEB74((u8) fv());", "m::gm_801BEB74(fv() as u8);")
op("ret = (u32) gm_801BEB80();", "ret = m::gm_801BEB80() as u32;")
op("ret = (u32) gm_801BEB8C((u8) (fv() % 51));", "ret = m::gm_801BEB8C((fv() % 51) as u8) as u32;")
op("ret = (u32) gm_801BEBA8((u8) (fv() % 51));", "ret = m::gm_801BEBA8((fv() % 51) as u8) as u32;")
op("ret = (u32) gm_801BEBC0((u8) (fv() % 56));", "ret = m::gm_801BEBC0((fv() % 56) as u8) as u32;")
op("ret = (u32) gm_801BEBF8(gm_803DF918[fv() % 51]);",
   "ret = m::gm_801BEBF8(m::gm_803DF918[(fv() % 51) as usize]) as u32;")
op("ret = PI(gm_801BEC54());", "ret = pi(m::gm_801BEC54());")
op("{ int i; for (i = 0; i < 3; i++) hash_GameModeState(&gm_Mode_Event_States[i]); }",
   "{ for i in 0..3 { g::hash_GameModeState(&m::gm_Mode_Event_States[i]); } }")
op("{ int i; for (i = 0; i < 51; i++) { HM(FI((const void*) gm_803DF94C[i]->x0)); "
   "HM(FI((const void*) gm_803DF94C[i]->x4)); } }",
   "{ for i in 0..51 { hm(fi((*m::gm_803DF94C[i]).x0)); hm(fi((*m::gm_803DF94C[i]).x4)); } }")

NOPS = len(OPS)
CL_C = " || ".join(f"op == {i}" for i in CHARLIST_OPS)
CL_R = " || ".join(f"op == {i}" for i in CHARLIST_OPS)
# NULL player_init slots only where the C copes (onEnterVs skips them; the BBB64 users test them)
NULL_OK = [6, 7, 9, 10]
NU_C = " || ".join(f"op == {i}" for i in NULL_OK)
NU_R = NU_C
# ops that read a level's x4->x4 as a pointer to a player record (gm_801BE39C is op 37)
PTR_OPS = [7, 9, 10, 12, 13, 37]
PM_C = " || ".join(f"op == {i}" for i in PTR_OPS)
PM_R = PM_C
SPEC = ", ".join(map(str, SPECIAL))

C_HEAD = """/* TU: src/melee/gm/gmevent.c */
/* C oracle for src/tu/gmevent.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub. Written by mk_gmevent_driver.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#include "mtu_common.h"
#define NLV 51
static GameModeState ST;
static struct gmm_x0 GM;
static PreloadedGameModeState PCS;
static VsSceneController VSC;
static VsSceneState VSS;
static MatchEnd MEND;
static HSD_GObj GNEW, GOA[4], G1;
static PlayerInitData PID;
static struct gm_801BAB40_src SRC;
static struct gm_804D6900_t* LVP[NLV];
static struct gm_804D6900_t LV[NLV];
static struct gm_evinit EVI[NLV];
static struct gm_evbonus EVB[NLV];
static struct gm_evstage_table EVS[NLV];
static struct gm_801BAB40_src PIN[NLV][6];
static struct gm_804D6900_x4_t X4[NLV];
static struct gm_801BAB40_src X4SRC[NLV];
static u8 CHL[64];
static const int SPECIAL[16] = { @SPEC@ };
#include "gmevent_gen.h"

static void build_levels(int charlist, int nulls, int ptrmode)
{
    int l, j;
    for (l = 0; l < NLV; l++) {
        fill_gm_804D6900_t(&LV[l]);
        LV[l].kind = (u8) (fv() % 3);
        LV[l].flags = (u8) ((fv() & 0x1F) | ((fv() % 6) << 5));
        fill_gm_evinit(&EVI[l]);
        fill_gm_evbonus(&EVB[l]);
        fill_gm_evstage_table(&EVS[l]);
        EVS[l].count = (u8) (1 + fv() % 6);
        for (j = 0; j < 6; j++) {
            fill_gm_801BAB40_src(&PIN[l][j]);
            EVS[l].entries[j] = &PIN[l][j];
        }
        /* slot 4 with no character would make onEnterVs write the character into EventData byte 0xB,
         * which is a bitfield byte (native C and melee_types order bits differently): never in data */
        if (PIN[l][4].c_kind == ChKind_None) PIN[l][4].c_kind = 5;
        LV[l].evinit = &EVI[l];
        LV[l].evbonus = &EVB[l];
        LV[l].evstage_table = &EVS[l];
        for (j = 0; j < 5; j++) LV[l].player_init[j] = (j == 0 || j == 4 || !nulls || (fv() & 3)) ? &PIN[l][j] : 0;
        X4[l].x0 = (int) (fv() % 400);
        fill_gm_801BAB40_src(&X4SRC[l]);
        /* x4->x4 is data (coins, a frame count...) except where the code takes it as a player record */
        X4[l].x4 = ptrmode ? (intptr_t) &X4SRC[l] : (intptr_t) (int) fv();
        LV[l].x4 = charlist ? (struct gm_804D6900_x4_t*) CHL : &X4[l];
        LVP[l] = &LV[l];
    }
    gm_804D6900[0] = LVP;
}

static void hash_all(void)
{
    if (TRACEON) { int q; hash_StartMeleeRules(&vs_enter_data.rules); printf("H rules %08X%c", H, 10); for (q = 0; q < 6; q++) { hash_PlayerInitData(&vs_enter_data.players[q]); printf("H player%d %08X%c", q, H, 10); } }
    if (TRACEON) { struct EventData* e = &GM.vs.unk_530; printf("H ev x10 %d x18 %d x20 %d x24 %d x28 %d x2C %d x30 %d x34 %d x38 %d x3C %d x40 %d bits %d%d%d%d%d%d%d%d%c", e->x10, e->x18, e->x20, e->x24, e->x28, e->x2C, e->x30, e->x34, e->x38, e->x3C, e->x40, e->xB_0, e->xB_1, e->xB_2, e->xB_3, e->xB_4, e->xB_5, e->xB_6, e->xB_7, 10); }
    hash_EventData(&GM.vs.unk_530); if (TRACEON) printf("H EventData %08X%c", H, 10);
    hash_CSSData(&css_data); if (TRACEON) printf("H CSSData %08X%c", H, 10);
    hash_StartMeleeData(&vs_enter_data); if (TRACEON) printf("H StartMeleeData %08X%c", H, 10);
    hash_MatchExitInfo(&vs_exit_data[0]); if (TRACEON) printf("H MatchExitInfo %08X%c", H, 10);
    hash_MatchExitInfo(&vs_exit_data[1]); if (TRACEON) printf("H MatchExitInfo %08X%c", H, 10);
    hash_PreloadedGameModeState(&PCS); if (TRACEON) printf("H PreloadedGameModeState %08X%c", H, 10);
    hash_VsSceneController(&VSC); if (TRACEON) printf("H VsSceneController %08X%c", H, 10);
    HM(VSS.timer_seconds);
    HM(VSS.unk_2C); if (TRACEON) printf("H vss %08X%c", H, 10);
    hash_MatchEnd(&MEND); if (TRACEON) printf("H MatchEnd %08X%c", H, 10);
    hash_PlayerInitData(&PID); if (TRACEON) printf("H PlayerInitData %08X%c", H, 10);
    hash_gm_801BAB40_src(&SRC); if (TRACEON) printf("H gm_801BAB40_src %08X%c", H, 10);
}

int main(void)
{
    int op, k, i;
    preg(&ST, sizeof ST);
    preg(&GM, sizeof GM);
    preg(&PCS, sizeof PCS);
    preg(&VSC, sizeof VSC);
    preg(&VSS, sizeof VSS);
    preg(&MEND, sizeof MEND);
    preg(&GNEW, sizeof GNEW);
    preg(GOA, sizeof GOA);
    preg(&G1, sizeof G1);
    preg(&PID, sizeof PID);
    preg(&SRC, sizeof SRC);
    preg(&css_data, sizeof css_data);
    preg(&vs_enter_data, sizeof vs_enter_data);
    preg(vs_exit_data, sizeof vs_exit_data);
    preg(gm_804D6900, sizeof gm_804D6900);
    preg(gm_Mode_Event_States, sizeof gm_Mode_Event_States);
    preg(CHL, sizeof CHL);
    freg_all();
    for (op = 0; op < @NOPS@; op++) {
        for (k = 0; k < @NCASE@; k++) {
            u32 ret = 0;
            tu_case(op, k);
            zero(&ST, sizeof ST);
            zero(&GM, sizeof GM);
            gmMainLib_804D3EE0 = &GM;
            fill_EventData(&GM.vs.unk_530);
            GM.vs.unk_530.unk_535 = (u8) (((fv() & 3) == 0) ? SPECIAL[fv() % 16] : fv() % 51);
            GM.vs.unk_530.x20 = (int) (fv() % 6);
            fill_CSSData(&css_data);
            fill_StartMeleeData(&vs_enter_data);
            fill_MatchExitInfo(&vs_exit_data[0]);
            fill_MatchExitInfo(&vs_exit_data[1]);
            fill_PreloadedGameModeState(&PCS);
            fill_VsSceneController(&VSC);
            VSS.timer_seconds = fv();
            VSS.unk_2C = (u16) fv();
            fill_MatchEnd(&MEND);
            fill_PlayerInitData(&PID);
            fill_gm_801BAB40_src(&SRC);
            for (i = 0; i < 20; i++) CHL[i] = (u8) (fv() % 25);
            CHL[20] = 33;
            ST.id = (u8) fv();
            build_levels(@CL@, @NU@, @PM@);
            switch (op) {
"""

C_TAIL = """            }
            hash_all();
            printf("%d %d %08X %08X %08X\\n", op, k, ret, H, TR); fflush(0);
        }
    }
    return 0;
}
"""

R_HEAD = """//! src/tu/gmevent.rs vs the C (gen/cref/tu/gmevent.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/mk_gmevent_driver.py` then
//! `python rust/melee_modes/gen/cref/run_tu.py gmevent`).
#![allow(static_mut_refs, non_snake_case, unused_unsafe, unused_assignments)]
mod mtu_harness;
use melee_modes::tu::gmevent as m;
use melee_modes::tu::gmevent::*;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmevent_gen.rs"]
mod g;

const NLV: usize = 51;
pub static mut ST: GameModeState = unsafe { core::mem::zeroed() };
pub static mut GM: gmm_x0 = unsafe { core::mem::zeroed() };
pub static mut PCS: PreloadedGameModeState = unsafe { core::mem::zeroed() };
pub static mut VSC: VsSceneController = unsafe { core::mem::zeroed() };
pub static mut VSS: VsSceneState = unsafe { core::mem::zeroed() };
pub static mut MEND: MatchEnd = unsafe { core::mem::zeroed() };
pub static mut GNEW: HSD_GObj = unsafe { core::mem::zeroed() };
pub static mut GOA: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
pub static mut G1: HSD_GObj = unsafe { core::mem::zeroed() };
pub static mut PID: PlayerInitData = unsafe { core::mem::zeroed() };
pub static mut SRC: gm_801BAB40_src = unsafe { core::mem::zeroed() };
static mut LVP: [*mut gm_804D6900_t; NLV] = [core::ptr::null_mut(); NLV];
static mut LV: [gm_804D6900_t; NLV] = unsafe { core::mem::zeroed() };
static mut EVI: [gm_evinit; NLV] = unsafe { core::mem::zeroed() };
static mut EVB: [gm_evbonus; NLV] = unsafe { core::mem::zeroed() };
static mut EVS: [gm_evstage_table; NLV] = unsafe { core::mem::zeroed() };
static mut PIN: [[gm_801BAB40_src; 6]; NLV] = unsafe { core::mem::zeroed() };
static mut X4: [gm_804D6900_x4_t; NLV] = unsafe { core::mem::zeroed() };
static mut X4SRC: [gm_801BAB40_src; NLV] = unsafe { core::mem::zeroed() };
pub static mut CHL: [u8; 64] = [0; 64];
const SPECIAL: [u32; 16] = [@SPEC@];

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

unsafe fn zero<T>(p: *mut T) {
    core::ptr::write_bytes(p, 0, 1)
}

unsafe fn build_levels(charlist: bool, nulls: bool, ptrmode: bool) {
    for l in 0..NLV {
        g::fill_gm_804D6900_t(&mut LV[l]);
        LV[l].kind = (fv() % 3) as u8;
        LV[l].flags = ((fv() & 0x1F) | ((fv() % 6) << 5)) as u8;
        g::fill_gm_evinit(&mut EVI[l]);
        g::fill_gm_evbonus(&mut EVB[l]);
        g::fill_gm_evstage_table(&mut EVS[l]);
        EVS[l].count = (1 + fv() % 6) as u8;
        for j in 0..6 {
            g::fill_gm_801BAB40_src(&mut PIN[l][j]);
            EVS[l].entries[j] = &raw mut PIN[l][j];
        }
        if PIN[l][4].c_kind as i32 == ChKind_None as i32 {
            PIN[l][4].c_kind = 5;
        }
        LV[l].evinit = &raw mut EVI[l];
        LV[l].evbonus = &raw mut EVB[l];
        LV[l].evstage_table = &raw mut EVS[l];
        for j in 0..5 {
            LV[l].player_init[j] =
                if j == 0 || j == 4 || !nulls || (fv() & 3) != 0 { &raw mut PIN[l][j] } else { core::ptr::null_mut() };
        }
        X4[l].x0 = (fv() % 400) as i32;
        g::fill_gm_801BAB40_src(&mut X4SRC[l]);
        X4[l].x4 = if ptrmode { &raw mut X4SRC[l] as isize } else { fv() as i32 as isize };
        LV[l].x4 = if charlist { &raw mut CHL as *mut gm_804D6900_x4_t } else { &raw mut X4[l] };
        LVP[l] = &raw mut LV[l];
    }
    m::gm_804D6900[0] = &raw mut LVP as *mut *mut gm_804D6900_t;
}

unsafe fn hash_all() {
    if TRACEON { g::hash_StartMeleeRules(&m::vs_enter_data.rules); println!("H rules {:08X}", H); for q in 0..6 { g::hash_PlayerInitData(&m::vs_enter_data.players[q]); println!("H player{} {:08X}", q, H); } }
    if TRACEON {
        let e = &GM.vs.unk_530;
        println!("H ev x10 {} x18 {} x20 {} x24 {} x28 {} x2C {} x30 {} x34 {} x38 {} x3C {} x40 {} bits {}{}{}{}{}{}{}{}", e.x10, e.x18, e.x20, e.x24, e.x28, e.x2C, e.x30, e.x34, e.x38, e.x3C, e.x40, e.xB_0(), e.xB_1(), e.xB_2(), e.xB_3(), e.xB_4(), e.xB_5(), e.xB_6(), e.xB_7());
    }
    g::hash_EventData(&GM.vs.unk_530); if TRACEON { println!("H EventData {:08X}", H); }
    g::hash_CSSData(&m::css_data); if TRACEON { println!("H CSSData {:08X}", H); }
    g::hash_StartMeleeData(&m::vs_enter_data); if TRACEON { println!("H StartMeleeData {:08X}", H); }
    g::hash_MatchExitInfo(&m::vs_exit_data[0]); if TRACEON { println!("H MatchExitInfo {:08X}", H); }
    g::hash_MatchExitInfo(&m::vs_exit_data[1]); if TRACEON { println!("H MatchExitInfo {:08X}", H); }
    g::hash_PreloadedGameModeState(&PCS); if TRACEON { println!("H PreloadedGameModeState {:08X}", H); }
    g::hash_VsSceneController(&VSC); if TRACEON { println!("H VsSceneController {:08X}", H); }
    hm(VSS.timer_seconds);
    hm(VSS.unk_2C as u32);
    g::hash_MatchEnd(&MEND); if TRACEON { println!("H MatchEnd {:08X}", H); }
    g::hash_PlayerInitData(&PID); if TRACEON { println!("H PlayerInitData {:08X}", H); }
    g::hash_gm_801BAB40_src(&SRC); if TRACEON { println!("H gm_801BAB40_src {:08X}", H); }
}

#[test]
fn gmevent_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const GM);
        reg(&raw const PCS);
        reg(&raw const VSC);
        reg(&raw const VSS);
        reg(&raw const MEND);
        reg(&raw const GNEW);
        reg(&raw const GOA);
        reg(&raw const G1);
        reg(&raw const PID);
        reg(&raw const SRC);
        reg(&raw const m::css_data);
        reg(&raw const m::vs_enter_data);
        reg(&raw const m::vs_exit_data);
        reg(&raw const m::gm_804D6900);
        reg(&raw const m::gm_Mode_Event_States);
        reg(&raw const CHL);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmevent.txt"));
        for op in 0..@NOPS@_u32 {
            for k in 0..@NCASE@_u32 {
                case(op, k);
                let mut ret = 0u32;
                zero(&raw mut ST);
                zero(&raw mut GM);
                m::ext::gmMainLib_804D3EE0 = &raw mut GM;
                g::fill_EventData(&mut GM.vs.unk_530);
                GM.vs.unk_530.unk_535 =
                    if (fv() & 3) == 0 { SPECIAL[(fv() % 16) as usize] as u8 } else { (fv() % 51) as u8 };
                GM.vs.unk_530.x20 = (fv() % 6) as i32;
                g::fill_CSSData(&mut m::css_data);
                g::fill_StartMeleeData(&mut m::vs_enter_data);
                g::fill_MatchExitInfo(&mut m::vs_exit_data[0]);
                g::fill_MatchExitInfo(&mut m::vs_exit_data[1]);
                g::fill_PreloadedGameModeState(&mut PCS);
                g::fill_VsSceneController(&mut VSC);
                VSS.timer_seconds = fv();
                VSS.unk_2C = fv() as u16;
                g::fill_MatchEnd(&mut MEND);
                g::fill_PlayerInitData(&mut PID);
                g::fill_gm_801BAB40_src(&mut SRC);
                for i in 0..20 {
                    CHL[i] = (fv() % 25) as u8;
                }
                CHL[20] = 33;
                ST.id = fv() as u8;
                build_levels(@CLR@, @NUR@, @PMR@);
                match op {
"""

R_TAIL = """                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), @NOPS@ * @NCASE@);
    }
}
"""


def fill(s):
    return (s.replace("@SPEC@", SPEC).replace("@NOPS@", str(NOPS)).replace("@NCASE@", str(NCASE))
            .replace("@CL@", CL_C).replace("@CLR@", CL_R).replace("@NU@", NU_C).replace("@NUR@", NU_R).replace("@PM@", PM_C).replace("@PMR@", PM_R))


out = [fill(C_HEAD)]
for i, (cs, rs) in enumerate(OPS):
    out.append(f"            case {i}: {cs} break;")
out.append(fill(C_TAIL))
(ROOT / "rust/melee_modes/gen/cref/tu/gmevent.c").write_text(NL.join(out), encoding="utf-8", newline=NL)

out = [fill(R_HEAD)]
for i, (cs, rs) in enumerate(OPS):
    out.append(f"                    {i} => {{ {rs} }}")
out.append(fill(R_TAIL))
(ROOT / "rust/melee_modes/tests/tu_gmevent.rs").write_text(NL.join(out), encoding="utf-8", newline=NL)
print("ops", NOPS)
