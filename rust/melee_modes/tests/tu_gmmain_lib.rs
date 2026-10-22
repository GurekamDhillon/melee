//! Smoke test for src/melee/gm/gmmain_lib.c (rust/melee_modes/src/tu/gmmain_lib.rs). This is NOT a
//! state oracle: it checks round-trips through the accessors and the `= {...}` initial values
//! against the literals in gmmain_lib.c. The per-function state oracle is still to be written.
use melee_modes::tu::gmmain_lib as g;

#[test]
fn gmmain_lib_smoke() {
    unsafe {
        g::gmMainLib_init_statics();

        // C initialisers (gmmain_lib.c).
        assert_eq!(g::gmMainLib_DefaultGameRules.bgm, 0x34);
        assert_eq!(g::gmMainLib_DefaultGameRules.time_limit, 2);
        assert_eq!(g::gmMainLib_DefaultGameRules.stock_count, 3);
        assert_eq!(g::gmMainLib_DefaultGameRules.damage_ratio, 10);
        assert_eq!(g::gmMainLib_DefaultGameRules.unk_14, -1);
        assert_eq!(g::gmMainLib_DefaultGamePrefs.item_freq, 2);
        assert_eq!(g::gmMainLib_DefaultGamePrefs.stage_mask, u32::MAX);
        assert_eq!(g::gmMainLib_DefaultGamePrefs.deflicker, 1);

        // Global record pointer points at the static record.
        assert_eq!(
            g::gmMainLib_804D3EE0 as usize,
            core::ptr::addr_of_mut!(g::gmMainLib_8045A6C0) as usize
        );

        // 64-bit save bit set/test (gmMainLib_8015CEB4 / 8015CEFC).
        g::gmMainLib_8015CEB4(3);
        assert!(g::gmMainLib_8015CEFC(3));
        assert!(!g::gmMainLib_8015CEFC(4));

        // Fighter stock count is a 3-bit field (gmMainLib_8015D1C8 / 8015D1AC).
        g::gmMainLib_8015D1C8(5, 6);
        assert_eq!(g::gmMainLib_8015D1AC(5), 6);

        // Name tag reset writes the default rank (InitializePersistentNameData sets x1A2 = 5).
        g::InitializePersistentNameData(0);
        assert_eq!((*g::GetPersistentNameData(0)).x1A2, 5);

        // gmMainLib_8015EF30 zeroes a stats block.
        let mut stats: melee_types::GmStats = core::mem::zeroed();
        stats.sd_count = 7;
        stats.coins_collected = 9;
        g::gmMainLib_8015EF30(&mut stats);
        assert_eq!(stats.sd_count, 0);
        assert_eq!(stats.coins_collected, 0);
    }
}
