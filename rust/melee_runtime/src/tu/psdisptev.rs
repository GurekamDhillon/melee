//! Native particle TEV state selection from `psdisptev.c`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use melee_types::*;
use ext::*;
pub mod ext {
    use melee_types::*;
    pub static mut GXSetNumTexGens_HOOK: Option<unsafe extern "C" fn(u8)> = None;
    pub unsafe fn GXSetNumTexGens(nTexGens: u8) { (GXSetNumTexGens_HOOK.expect("TODO: GXSetNumTexGens"))(nTexGens); }
    pub static mut GXSetTevOp_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTevMode)> = None;
    pub unsafe fn GXSetTevOp(id: GXTevStageID, mode: GXTevMode) { (GXSetTevOp_HOOK.expect("TODO: GXSetTevOp"))(id, mode); }
    pub static mut GXSetTevColorIn_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTevColorArg, GXTevColorArg, GXTevColorArg, GXTevColorArg)> = None;
    pub unsafe fn GXSetTevColorIn(stage: GXTevStageID, a: GXTevColorArg, b: GXTevColorArg, c: GXTevColorArg, d: GXTevColorArg) { (GXSetTevColorIn_HOOK.expect("TODO: GXSetTevColorIn"))(stage, a, b, c, d); }
    pub static mut GXSetTevAlphaIn_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTevAlphaArg, GXTevAlphaArg, GXTevAlphaArg, GXTevAlphaArg)> = None;
    pub unsafe fn GXSetTevAlphaIn(stage: GXTevStageID, a: GXTevAlphaArg, b: GXTevAlphaArg, c: GXTevAlphaArg, d: GXTevAlphaArg) { (GXSetTevAlphaIn_HOOK.expect("TODO: GXSetTevAlphaIn"))(stage, a, b, c, d); }
    pub static mut GXSetTevColorOp_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTevOp, GXTevBias, GXTevScale, GXBool, GXTevRegID)> = None;
    pub unsafe fn GXSetTevColorOp(stage: GXTevStageID, op: GXTevOp, bias: GXTevBias, scale: GXTevScale, clamp: GXBool, out_reg: GXTevRegID) { (GXSetTevColorOp_HOOK.expect("TODO: GXSetTevColorOp"))(stage, op, bias, scale, clamp, out_reg); }
    pub static mut GXSetTevAlphaOp_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTevOp, GXTevBias, GXTevScale, GXBool, GXTevRegID)> = None;
    pub unsafe fn GXSetTevAlphaOp(stage: GXTevStageID, op: GXTevOp, bias: GXTevBias, scale: GXTevScale, clamp: GXBool, out_reg: GXTevRegID) { (GXSetTevAlphaOp_HOOK.expect("TODO: GXSetTevAlphaOp"))(stage, op, bias, scale, clamp, out_reg); }
    pub static mut GXSetTevSwapMode_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTevSwapSel, GXTevSwapSel)> = None;
    pub unsafe fn GXSetTevSwapMode(stage: GXTevStageID, ras_sel: GXTevSwapSel, tex_sel: GXTevSwapSel) { (GXSetTevSwapMode_HOOK.expect("TODO: GXSetTevSwapMode"))(stage, ras_sel, tex_sel); }
    pub static mut GXSetTevOrder_HOOK: Option<unsafe extern "C" fn(GXTevStageID, GXTexCoordID, GXTexMapID, GXChannelID)> = None;
    pub unsafe fn GXSetTevOrder(stage: GXTevStageID, coord: GXTexCoordID, map: GXTexMapID, color: GXChannelID) { (GXSetTevOrder_HOOK.expect("TODO: GXSetTevOrder"))(stage, coord, map, color); }
    pub static mut GXSetNumTevStages_HOOK: Option<unsafe extern "C" fn(u8)> = None;
    pub unsafe fn GXSetNumTevStages(nStages: u8) { (GXSetNumTevStages_HOOK.expect("TODO: GXSetNumTevStages"))(nStages); }
}

pub static mut prevTev: [u32; 2] = [0; 2];

pub unsafe fn psSetupTevCommon() {
    GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1,
                    GX_TEVPREV);
    GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1,
                    GX_TEVPREV);
    GXSetTevOrder(GX_TEVSTAGE1, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    GXSetTevColorOp(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1,
                    GX_TEVPREV);
    GXSetTevAlphaOp(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1,
                    GX_TEVPREV);
    GXSetTevOrder(GX_TEVSTAGE2, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    GXSetTevColorOp(GX_TEVSTAGE2, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1,
                    GX_TEVPREV);
    GXSetTevAlphaOp(GX_TEVSTAGE2, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1,
                    GX_TEVPREV);
    GXSetTevSwapMode(GX_TEVSTAGE0, GX_TEV_SWAP0, GX_TEV_SWAP0);
    GXSetTevSwapMode(GX_TEVSTAGE1, GX_TEV_SWAP0, GX_TEV_SWAP0);
    GXSetTevSwapMode(GX_TEVSTAGE2, GX_TEV_SWAP0, GX_TEV_SWAP0);
}
pub unsafe fn psSetupTevInvalidState() { prevTev[0] = u32::MAX; }
pub unsafe fn psSetupTev(pp: *mut HSD_Particle) {
    let key = (*pp).kind & 0x80100480;
    if key == prevTev[0] { return; }
    prevTev[0] = key;
    // Untextured modes discard the environment-color bit before caching.
    if key & 0x480 == 0x80 { (*pp).kind &= !0x80; prevTev[0] &= !0x80; }
    match prevTev[0] {
        0x80000000 => {
GXSetNumTevStages(2);
        GXSetNumTexGens(0);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL,
                      GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A2, GX_CA_ZERO, GX_CA_ZERO,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_C0, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_A0, GX_CA_APREV,
                        GX_CA_ZERO);
        }
        0x0 => {
GXSetNumTevStages(1);
        GXSetNumTexGens(0);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL,
                      GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_C0, GX_CC_ZERO, GX_CC_ZERO,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A0, GX_CA_ZERO, GX_CA_ZERO,
                        GX_CA_ZERO);
        }
        0x80000480 => {
GXSetNumTevStages(3);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_C2, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_A2, GX_CA_APREV,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE2, GX_CC_ZERO, GX_CC_RASC, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE2, GX_CA_APREV, GX_CA_ZERO, GX_CA_ZERO,
                        GX_CA_ZERO);
        }
        0x80000400 => {
GXSetNumTevStages(2);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A2, GX_CA_ZERO, GX_CA_ZERO,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_TEXC, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_TEXA, GX_CA_APREV,
                        GX_CA_ZERO);
        }
        0x400 | 0x480 => {
GXSetNumTevStages(1);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA,
                        GX_CA_ZERO);
        }
        0x100400 => {
GXSetNumTevStages(1);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevOp(GX_TEVSTAGE0, GX_MODULATE);
        }
        0x100480 => {
GXSetNumTevStages(2);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_CPREV, GX_CC_ZERO, GX_CC_ZERO,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_RASA, GX_CA_APREV,
                        GX_CA_ZERO);
        }
        0x80100400 => {
GXSetNumTevStages(2);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_A2, GX_CA_RASA,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_TEXC, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_TEXA, GX_CA_APREV,
                        GX_CA_ZERO);
        }
        0x80100480 => {
GXSetNumTevStages(3);
        GXSetNumTexGens(1);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_C2, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_A2, GX_CA_APREV,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE2, GX_CC_ZERO, GX_CC_RASC, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE2, GX_CA_ZERO, GX_CA_RASA, GX_CA_APREV,
                        GX_CA_ZERO);
        }
        0x100000 => {
GXSetNumTevStages(1);
        GXSetNumTexGens(0);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL,
                      GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_RASC, GX_CC_ZERO, GX_CC_ZERO,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_RASA, GX_CA_ZERO, GX_CA_ZERO,
                        GX_CA_ZERO);
        }
        0x80100000 => {
GXSetNumTevStages(2);
        GXSetNumTexGens(0);
        GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL,
                      GX_COLOR0A0);
        GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_A2, GX_CA_RASA,
                        GX_CA_ZERO);
        GXSetTevColorIn(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_C0, GX_CC_CPREV,
                        GX_CC_ZERO);
        GXSetTevAlphaIn(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_A0, GX_CA_APREV,
                        GX_CA_ZERO);
        }
        _ => {}
    }
}
