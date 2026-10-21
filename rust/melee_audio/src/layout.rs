//! GameCube offsets of every game-memory block this crate reads or writes. All values are the
//! SDK layouts (`libs/dolphin/include/dolphin/ax.h`, `axfx.h`); `tests` cross-checks each against
//! the generated `melee_types` GcLayout tables, so the m-ex importer and this crate agree.

/// `AXVPB` size reserved per voice (the game reads at voice+0x1B2 and +0x16E).
pub const VPB_SIZE: usize = 0x1F8;
pub const VPB_PRIORITY: usize = 0x0C;
pub const VPB_CALLBACK: usize = 0x10;
pub const VPB_USER_CONTEXT: usize = 0x14;
pub const VPB_INDEX: usize = 0x18;
/// `AXVPB.sync` (u32; synth.c clears bit 0x400 and tests 0x20).
pub const VPB_SYNC: usize = 0x1C;
/// `AXVPB.pb`.
pub const VPB_PB: usize = 0x138;
/// `pb.state` (u16).
pub const VPB_PB_STATE: usize = VPB_PB + 0x0E;
/// `pb.itd.flag` (u16).
pub const VPB_ITD_FLAG: usize = VPB_PB + 0x36;
/// `AXPBITD` fields relative to `pb.itd` (= [`VPB_ITD_FLAG`]).
pub const ITD_SHIFT_L: usize = 0x06;
pub const ITD_SHIFT_R: usize = 0x08;
pub const ITD_TARGET_L: usize = 0x0A;
pub const ITD_TARGET_R: usize = 0x0C;
/// `pb.ve.currentVolume` (u16).
pub const VPB_VE_VOL: usize = VPB_PB + 0x64;
/// `pb.addr.currentAddressHi/Lo` (u32).
pub const VPB_CURADDR: usize = VPB_PB + 0x6E + 0x0C;

/// `AXPBMIX` size, and the byte offset of each of the nine targets' volume in mixer order
/// (L, R, S, AuxA L/R/S, AuxB L/R/S); each target's s16 delta sits at +2.
pub const PBMIX_SIZE: usize = 0x24;
pub const PBMIX_OFF: [usize; 9] = [0x00, 0x04, 0x1C, 0x08, 0x0C, 0x20, 0x10, 0x14, 0x18];

/// `AXPBVE` (currentVolume, currentDelta).
pub const PBVE_SIZE: usize = 4;
/// `AXPBADDR`: loopFlag 0, format 2, loopAddress 4, endAddress 8, currentAddress 0xC.
pub const PBADDR_SIZE: usize = 0x10;
/// `AXPBADPCM`: a[8][2] 0, gain 0x20, pred_scale 0x22, yn1 0x24, yn2 0x26.
pub const PBADPCM_SIZE: usize = 0x28;
/// `AXPBSRC`: ratioHi 0, ratioLo 2.
pub const PBSRC_MIN: usize = 4;
/// `AXPBADPCMLOOP`: loop_pred_scale 0, loop_yn1 2, loop_yn2 4.
pub const PBADPCMLOOP_SIZE: usize = 6;

/// `AXFX_REVERBSTD` parameter fields.
pub const REVSTD_TEMPDISABLE: usize = 0x13C;
pub const REVSTD_COLORATION: usize = 0x140;
pub const REVSTD_MIX: usize = 0x144;
pub const REVSTD_TIME: usize = 0x148;
pub const REVSTD_DAMPING: usize = 0x14C;
pub const REVSTD_PREDELAY: usize = 0x150;
/// `AXFX_DELAY` parameter arrays (u32[3] each).
pub const DELAY_DELAY: usize = 0x3C;
pub const DELAY_FEEDBACK: usize = 0x48;
pub const DELAY_OUTPUT: usize = 0x54;

#[inline]
pub(crate) fn r16(p: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([p[o], p[o + 1]])
}
#[inline]
pub(crate) fn r32(p: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([p[o], p[o + 1], p[o + 2], p[o + 3]])
}
#[inline]
pub(crate) fn rf32(p: &[u8], o: usize) -> f32 {
    f32::from_bits(r32(p, o))
}
#[inline]
pub(crate) fn w16(p: &mut [u8], o: usize, v: u16) {
    p[o..o + 2].copy_from_slice(&v.to_be_bytes());
}
#[inline]
pub(crate) fn w32(p: &mut [u8], o: usize, v: u32) {
    p[o..o + 4].copy_from_slice(&v.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use melee_types::melee_core::gc::GcLayout;
    use melee_types::*;

    fn off<T: GcLayout>(f: &str) -> usize {
        T::gc_field(f).unwrap_or_else(|| panic!("{} has no {f}", T::GC_NAME)).offset as usize
    }

    #[test]
    fn offsets_match_generated_gc_layouts() {
        assert_eq!(off::<_AXVPB>("priority"), VPB_PRIORITY);
        assert_eq!(off::<_AXVPB>("callback"), VPB_CALLBACK);
        assert_eq!(off::<_AXVPB>("userContext"), VPB_USER_CONTEXT);
        assert_eq!(off::<_AXVPB>("index"), VPB_INDEX);
        assert_eq!(off::<_AXVPB>("pb"), VPB_PB);
        assert_eq!(off::<_AXVPB>("sync"), VPB_SYNC);
        assert_eq!(off::<_AXPBITD>("shiftL"), ITD_SHIFT_L);
        assert_eq!(off::<_AXPBITD>("shiftR"), ITD_SHIFT_R);
        assert_eq!(off::<_AXPBITD>("targetShiftL"), ITD_TARGET_L);
        assert_eq!(off::<_AXPBITD>("targetShiftR"), ITD_TARGET_R);
        assert!(_AXVPB::GC_SIZE as usize <= VPB_SIZE);
        assert_eq!(VPB_PB + off::<_AXPB>("state"), VPB_PB_STATE);
        assert_eq!(VPB_PB + off::<_AXPB>("itd") + off::<_AXPBITD>("flag"), VPB_ITD_FLAG);
        assert_eq!(VPB_PB + off::<_AXPB>("ve") + off::<_AXPBVE>("currentVolume"), VPB_VE_VOL);
        assert_eq!(VPB_PB + off::<_AXPB>("addr") + off::<_AXPBADDR>("currentAddressHi"), VPB_CURADDR);
        let mix = ["vL", "vR", "vS", "vAuxAL", "vAuxAR", "vAuxAS", "vAuxBL", "vAuxBR", "vAuxBS"];
        for (i, m) in mix.iter().enumerate() {
            assert_eq!(off::<_AXPBMIX>(m), PBMIX_OFF[i], "{m}");
        }
        assert_eq!(_AXPBMIX::GC_SIZE as usize, PBMIX_SIZE);
        assert_eq!(_AXPBVE::GC_SIZE as usize, PBVE_SIZE);
        assert_eq!(_AXPBADDR::GC_SIZE as usize, PBADDR_SIZE);
        assert_eq!(_AXPBADPCM::GC_SIZE as usize, PBADPCM_SIZE);
        assert_eq!(off::<_AXPBADPCM>("pred_scale"), 0x22);
        assert_eq!(off::<_AXPBADPCM>("yn1"), 0x24);
        assert_eq!(off::<_AXPBADPCM>("yn2"), 0x26);
        assert_eq!(off::<_AXPBSRC>("ratioLo"), 2);
        assert_eq!(_AXPBADPCMLOOP::GC_SIZE as usize, PBADPCMLOOP_SIZE);
        assert_eq!(off::<AXFX_REVERBSTD>("tempDisableFX"), REVSTD_TEMPDISABLE);
        assert_eq!(off::<AXFX_REVERBSTD>("coloration"), REVSTD_COLORATION);
        assert_eq!(off::<AXFX_REVERBSTD>("mix"), REVSTD_MIX);
        assert_eq!(off::<AXFX_REVERBSTD>("time"), REVSTD_TIME);
        assert_eq!(off::<AXFX_REVERBSTD>("damping"), REVSTD_DAMPING);
        assert_eq!(off::<AXFX_REVERBSTD>("preDelay"), REVSTD_PREDELAY);
        assert_eq!(off::<AXFX_DELAY>("delay"), DELAY_DELAY);
        assert_eq!(off::<AXFX_DELAY>("feedback"), DELAY_FEEDBACK);
        assert_eq!(off::<AXFX_DELAY>("output"), DELAY_OUTPUT);
    }
}
