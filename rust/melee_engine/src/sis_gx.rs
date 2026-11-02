//! Points the HSD_SisLib text draw (melee_runtime tu::hsd_3A76) GX calls at the bridge; hand-written because that TU's
//! ext module is hand-written (not in seams_gen.rs, which gen_gx_bridge.py reads).
#![allow(non_snake_case, unused_unsafe, clippy::all)]

pub fn install() {
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetZMode = Some(|a0, a1, a2| unsafe { crate::gxbridge::GXSetZMode(a0 as u8, a1, a2 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetViewport = Some(|a0, a1, a2, a3, a4, a5| unsafe { crate::gxbridge::GXSetViewport(a0, a1, a2, a3, a4, a5) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetScissor = Some(|a0, a1, a2, a3| unsafe { crate::gxbridge::GXSetScissor(a0, a1, a2, a3) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetProjection = Some(|a0, a1| unsafe { crate::gxbridge::GXSetProjection(a0 as _, a1) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetCurrentMtx = Some(|a0| unsafe { crate::gxbridge::GXSetCurrentMtx(a0) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTexCoordGen2 = Some(|a0, a1, a2, a3, a4, a5| unsafe { crate::gxbridge::GXSetTexCoordGen2(a0, a1, a2, a3, a4 as u8, a5) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetNumChans = Some(|a0| unsafe { crate::gxbridge::GXSetNumChans(a0 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXClearVtxDesc = Some(|| unsafe { crate::gxbridge::GXClearVtxDesc() }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetCullMode = Some(|a0| unsafe { crate::gxbridge::GXSetCullMode(a0) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetNumTexGens = Some(|a0| unsafe { crate::gxbridge::GXSetNumTexGens(a0 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetNumTevStages = Some(|a0| unsafe { crate::gxbridge::GXSetNumTevStages(a0 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetVtxAttrFmt = Some(|a0, a1, a2, a3, a4| unsafe { crate::gxbridge::GXSetVtxAttrFmt(a0, a1, a2, a3, a4 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetVtxDesc = Some(|a0, a1| unsafe { crate::gxbridge::GXSetVtxDesc(a0, a1) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevOrder = Some(|a0, a1, a2, a3| unsafe { crate::gxbridge::GXSetTevOrder(a0, a1, a2, a3) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetBlendMode = Some(|a0, a1, a2, a3| unsafe { crate::gxbridge::GXSetBlendMode(a0, a1, a2, a3) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetAlphaCompare = Some(|a0, a1, a2, a3, a4| unsafe { crate::gxbridge::GXSetAlphaCompare(a0, a1 as u8, a2, a3, a4 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetColorUpdate = Some(|a0| unsafe { crate::gxbridge::GXSetColorUpdate(a0 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetAlphaUpdate = Some(|a0| unsafe { crate::gxbridge::GXSetAlphaUpdate(a0 as u8) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevColorIn = Some(|a0, a1, a2, a3, a4| unsafe { crate::gxbridge::GXSetTevColorIn(a0, a1, a2, a3, a4) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevColorOp = Some(|a0, a1, a2, a3, a4, a5| unsafe { crate::gxbridge::GXSetTevColorOp(a0, a1, a2, a3, a4 as u8, a5) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevAlphaOp = Some(|a0, a1, a2, a3, a4, a5| unsafe { crate::gxbridge::GXSetTevAlphaOp(a0, a1, a2, a3, a4 as u8, a5) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevAlphaIn = Some(|a0, a1, a2, a3, a4| unsafe { crate::gxbridge::GXSetTevAlphaIn(a0, a1, a2, a3, a4) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevColor = Some(|a0, a1| unsafe { crate::gxbridge::GXSetTevColor(a0, core::mem::transmute(a1)) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevKAlphaSel = Some(|a0, a1| unsafe { crate::gxbridge::GXSetTevKAlphaSel(a0, a1) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXSetTevKColor = Some(|a0, a1| unsafe { crate::gxbridge::GXSetTevKColor(a0, core::mem::transmute(a1)) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXBegin = Some(|a0, a1, a2| unsafe { crate::gxbridge::GXBegin(a0, a1, a2) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXPosition3f32 = Some(|a0, a1, a2| unsafe { crate::gxbridge::GXPosition3f32(a0, a1, a2) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXTexCoord2f32 = Some(|a0, a1| unsafe { crate::gxbridge::GXTexCoord2f32(a0, a1) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXInitTexObj = Some(|a0, a1, a2, a3, a4, a5, a6, a7| unsafe { crate::gxbridge::GXInitTexObj(a0 as _, a1 as _, a2, a3, a4, a5, a6, a7) }); }
    unsafe { melee_runtime::tu::hsd_3A76::ext::hook::GXLoadTexObj = Some(|a0, a1| unsafe { crate::gxbridge::GXLoadTexObj(a0 as _, a1) }); }
}
