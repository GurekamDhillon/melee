//! Round 6 (night 3 round 2): gm leaf functions over game records, verified bit for bit against
//! the C by `tests/oracle_r6.rs`.
use melee_types::VsModeData;

/// gmmain_lib.c `gmMainLib_AdjustNameTags` (static inline): after name tag `tag` is deleted,
/// players holding it get GM_NAMETAG_COUNT (no tag) and higher tags shift down by one.
pub fn gmMainLib_AdjustNameTags(vmd: &mut VsModeData, tag: u8) {
    const GM_NAMETAG_COUNT: u8 = 120;
    for p in vmd.start.players.iter_mut() {
        if p.nametag == tag {
            p.nametag = GM_NAMETAG_COUNT;
        } else if p.nametag > tag && p.nametag != GM_NAMETAG_COUNT {
            p.nametag = p.nametag.wrapping_sub(1);
        }
    }
}

/// gmtou_0.c `fn_80194658_get_value` (static inline): `*++ptr`.
pub fn fn_80194658_get_value(ptr: &[i32]) -> i32 {
    ptr[1]
}
