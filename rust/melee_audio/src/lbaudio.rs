//! The rest of `lbaudio_ax.c` the native sim needs: the retail bank tables, the sound-id ->
//! bank lookups the fighter code reaches through its hooks (`lbAudioAx_80023130`, `800230C8`,
//! `80023220`, `800233EC`), the bank manager that loads a match's fighter and stage SSMs
//! (`lbAudioAx_80026E84` / `80026EBC` / `80026F2C` / `8002702C` / `80027168` / `80027648` and
//! their statics, with the async ARQ chain run to completion synchronously), the boot-time bank
//! sizes (`fn_80023254` over `offsets_arr_803BC4E4`), the BGM by id (`lbAudioAx_80023F28`), the
//! crowd/heartbeat counters (`80024DC4`, `80024FDC`, `80024FF4`, `8002500C`, `80025038`) and the
//! per-frame `lbAudioAx_80027DF8` (except its `fn_80024654` volume ducking, not ported).
//!
//! Retail tables only (`LBAX_N` = 55, no mexData): the TARGET_PC m-ex branches are never taken
//! on a vanilla disc and are left out.

use crate::Sound;

/// `LBAX_N` (vanilla).
pub const LBAX_N: usize = 55;
/// `LBAX_CAP` (vanilla, 0x38).
pub const LBAX_CAP: usize = 0x38;
/// No sound / key-off sentinel ids.
pub const SFX_NONE: i32 = 0x83D60;

/// `s32_arr_803BB5D0`: {group, load priority, evict priority, x3}.
pub static BANK_INFO: [[i8; 4]; LBAX_CAP] = [
    [1, 5, 5, 0], [1, 5, 5, 0], [3, 0, 0, 0], [3, 0, 0, 0], [3, 4, 4, 0], [3, 4, 4, 0],
    [4, 2, 3, 4], [4, 2, 3, 7], [4, 2, 3, 7], [4, 2, 3, 6], [4, 2, 3, 6], [4, 2, 3, 4],
    [4, 2, 1, 0], [4, 2, 3, 9], [4, 2, 3, 1], [4, 2, 3, 8], [4, 2, 3, 7], [4, 2, 3, 2],
    [4, 2, 3, 1], [4, 2, 3, 7], [4, 2, 3, 1], [4, 2, 3, 6], [4, 2, 3, 1], [4, 2, 3, 1],
    [4, 2, 3, 7], [4, 2, 3, 1], [4, 2, 3, 6], [4, 2, 3, 2], [4, 2, 3, 1], [4, 2, 3, 6],
    [4, 2, 3, 7], [4, 2, 3, 6], [4, 2, 1, 0], [4, 2, 1, 0], [5, 2, 2, 0], [5, 2, 2, 0],
    [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0],
    [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0], [5, 2, 2, 0],
    [5, 2, 2, 0], [5, 2, 4, 0], [6, 1, 1, 0], [2, 5, 5, 0], [6, 1, 1, 0], [5, 2, 2, 0],
    [1, 5, 5, 0], [0, 0, 0, 0],
];

/// `s32_arr_803BB6B0` (retail 0x6F rows, GrKind-indexed): {ssm, echo, echo2}.
pub fn stage_audio(grkind: i32) -> [u8; 3] {
    const SPECIAL: [(usize, [u8; 3]); 26] = [
        (2, [0x22, 1, 1]), (4, [0x25, 1, 1]), (5, [0x29, 1, 1]), (6, [0x24, 1, 1]),
        (7, [0x37, 0x18, 0x18]), (8, [0x28, 0x40, 0x40]), (9, [0x2A, 0x80, 0x80]),
        (13, [0x2B, 1, 1]), (14, [0x23, 1, 1]), (15, [0x2C, 1, 0x88]), (16, [0x30, 1, 1]),
        (18, [0x26, 1, 0x40]), (19, [0x2D, 1, 1]), (20, [0x27, 1, 1]), (21, [0x2E, 1, 1]),
        (28, [0x2F, 1, 1]), (31, [0x31, 1, 1]), (32, [0x31, 0x48, 0x48]),
        (33, [0x31, 0x88, 0x88]), (34, [0x31, 1, 1]), (36, [0x37, 0x38, 0x38]),
        (37, [0x35, 0x38, 0x38]), (40, [0x31, 1, 1]), (110, [0, 0, 0]),
        (usize::MAX, [0; 3]), (usize::MAX, [0; 3]),
    ];
    if !(0..0x6F).contains(&grkind) {
        return [55, 1, 1]; // lbAudioAx_StageAudioByte's out-of-range answer
    }
    SPECIAL.iter().find(|(g, _)| *g == grkind as usize).map(|(_, r)| *r).unwrap_or([0x37, 1, 1])
}

/// `s32_arr_803BB8D4`: each bank's sound-id range.
pub static BANK_RANGE: [[i32; 2]; LBAX_CAP] = [
    [0, 0x20F], [0x2710, 0x2750], [0x4E20, 0x4E25], [0x7530, 0x7546], [0x9C40, 0x9C4A],
    [0xC350, 0xC356], [0xEA60, 0xEAB4], [0x11170, 0x111DF], [0x13880, 0x138C2],
    [0x15F90, 0x15FF5], [0x186A0, 0x1871C], [0x1ADB0, 0x1AE30], [0x1D4C0, 0x1D504],
    [0x1FBD0, 0x1FCC8], [0x222E0, 0x22379], [0x249F0, 0x24A3F], [0x27100, 0x2716C],
    [0x29810, 0x29877], [0x2BF20, 0x2BF7D], [0x2E630, 0x2E6B4], [0x30D40, 0x30DD3],
    [0x33450, 0x334B5], [0x35B60, 0x35BBA], [0x38270, 0x382D0], [0x3A980, 0x3A9E6],
    [0x3D090, 0x3D0DE], [0x3F7A0, 0x3F7EA], [0x41EB0, 0x41F77], [0x445C0, 0x44623],
    [0x46CD0, 0x46D17], [0x493E0, 0x4943A], [0x4BAF0, 0x4BB73], [0x4E200, 0x4E21D],
    [0x50910, 0x509ED], [0x53020, 0x53027], [0x55730, 0x55749], [0x57E40, 0x57E4A],
    [0x5A550, 0x5A551], [0x5CC60, 0x5CC6A], [0x5F370, 0x5F37D], [0x61A80, 0x61A8B],
    [0x64190, 0x64194], [0x668A0, 0x668A8], [0x68FB0, 0x68FB9], [0x6B6C0, 0x6B6D8],
    [0x6DDD0, 0x6DDD6], [0x704E0, 0x704E4], [0x72BF0, 0x72BF8], [0x75300, 0x75301],
    [0x77A10, 0x77A16], [0x7A120, 0x7A12F], [0x7C830, 0x7C865], [0x7EF40, 0x7EF41],
    [0x81650, 0x81654], [0x83D60, 0x83D60], [0x83D60, 0x83D60],
];

/// `ssm_files`.
pub static SSM_FILES: [&str; LBAX_N] = [
    "main.ssm", "pokemon.ssm", "nr_title.ssm", "nr_select.ssm", "nr_1p.ssm", "nr_vs.ssm",
    "captain.ssm", "clink.ssm", "dk.ssm", "drmario.ssm", "falco.ssm", "fox.ssm", "gkoopa.ssm",
    "ice.ssm", "kirby.ssm", "koopa.ssm", "link.ssm", "luigi.ssm", "mario.ssm", "mars.ssm",
    "mewtwo.ssm", "ness.ssm", "peach.ssm", "pichu.ssm", "pikachu.ssm", "purin.ssm",
    "samus.ssm", "zs.ssm", "yoshi.ssm", "gw.ssm", "ganon.ssm", "emblem.ssm", "mhands.ssm",
    "kirbytm.ssm", "castle.ssm", "corneria.ssm", "greatbay.ssm", "kongo.ssm", "mutecity.ssm",
    "onett.ssm", "zebes.ssm", "garden.ssm", "klaid.ssm", "greens.ssm", "venom.ssm",
    "bigblue.ssm", "fourside.ssm", "pupupu.ssm", "pstadium.ssm", "1padv.ssm", "ending.ssm",
    "nr_name.ssm", "1pend.ssm", "last.ssm", "end.ssm",
];

/// `hps_files` (retail 0x62).
pub static HPS_FILES: [&str; 0x62] = [
    "1p_qk.hps", "akaneia.hps", "baloon.hps", "bigblue.hps", "castle.hps", "continue.hps",
    "corneria.hps", "docmari.hps", "ending.hps", "famidemo.hps", "ff_1p01.hps", "ff_1p02.hps",
    "ff_bad.hps", "ff_dk.hps", "ff_emb.hps", "ff_flat.hps", "ff_fox.hps", "ff_fzero.hps",
    "ff_good.hps", "ff_ice.hps", "ff_kirby.hps", "ff_link.hps", "ff_mario.hps", "ff_nes.hps",
    "ff_poke.hps", "ff_samus.hps", "ff_step1.hps", "ff_step2.hps", "ff_step3.hps",
    "ff_yoshi.hps", "flatzone.hps", "fourside.hps", "gameover.hps", "garden.hps",
    "greatbay.hps", "greens.hps", "howto.hps", "howto_s.hps", "hyaku.hps", "hyaku2.hps",
    "icemt.hps", "inis1_01.hps", "inis1_02.hps", "inis2_01.hps", "inis2_02.hps",
    "intro_es.hps", "intro_nm.hps", "item_h.hps", "item_s.hps", "izumi.hps", "kongo.hps",
    "kraid.hps", "menu01.hps", "menu02.hps", "menu3.hps", "mrider.hps", "mutecity.hps",
    "old_dk.hps", "old_kb.hps", "old_ys.hps", "onetto.hps", "onetto2.hps", "opening.hps",
    "pokesta.hps", "pstadium.hps", "pura.hps", "rcruise.hps", "s_info1.hps", "s_info2.hps",
    "s_info3.hps", "s_new1.hps", "s_new2.hps", "s_newcom.hps", "s_select.hps", "saria.hps",
    "shrine.hps", "siren.hps", "smari3.hps", "sp_end.hps", "sp_giga.hps", "sp_metal.hps",
    "sp_zako.hps", "swm_15min.hps", "target.hps", "venom.hps", "vl_battle.hps",
    "vl_castle.hps", "vl_corneria.hps", "vl_cosmos.hps", "vl_figure1.hps", "vl_figure2.hps",
    "vl_fzero.hps", "vl_last_v2.hps", "vs_hyou1.hps", "vs_hyou2.hps", "yorster.hps",
    "ystory.hps", "zebes.hps",
];

/// `offsets_arr_803BC4E4[i][0]`: each bank's ARAM size.
pub static BANK_SIZE: [i32; LBAX_CAP] = [
    2045824, 561760, 153024, 239264, 121888, 54176, 443328, 298400, 206368, 430560, 599712,
    573216, 487936, 477600, 586208, 526752, 328672, 372992, 372736, 513088, 562912, 509024,
    416736, 580128, 613088, 334528, 319040, 590176, 321472, 165344, 427872, 495520, 563712,
    572832, 176224, 441760, 490240, 360064, 150144, 54208, 387872, 428896, 304096, 144480,
    382272, 34880, 78592, 123104, 101504, 160576, 1168864, 348160, 363936, 663136, 1536, 0,
];

/// `offsets_arr_803BC6A4`: {normal id, Kirby-hat id} (the copy-ability voice remap), 0x4A rows.
pub static KIRBY_REMAP: [[i32; 2]; 0x4A] = [
    [150050, 330000], [150053, 330003], [150056, 330006], [150059, 330009], [260006, 330012],
    [260009, 330015], [260012, 330018], [260015, 330021], [260018, 330024], [260021, 330027],
    [80007, 330030], [80010, 330033], [210069, 330036], [210072, 330039], [210084, 330042],
    [240076, 330045], [240079, 330048], [60079, 330051], [110100, 330054], [110103, 330057],
    [110106, 330060], [110109, 330063], [110085, 330066], [180025, 330069], [180007, 330072],
    [160082, 330075], [160061, 330078], [160064, 330081], [220061, 330084], [130021, 330087],
    [130024, 330090], [280073, 330093], [280076, 330096], [280088, 330099], [90030, 330102],
    [90012, 330105], [100096, 330108], [100099, 330111], [100102, 330114], [100105, 330117],
    [100081, 330120], [300085, 330123], [290063, 330126], [310102, 330129], [310105, 330132],
    [310108, 330135], [250064, 330138], [250067, 330141], [250070, 330144], [250073, 330147],
    [70082, 330150], [70061, 330153], [70064, 330156], [170026, 330159], [170008, 330162],
    [190103, 330165], [190106, 330168], [190109, 330171], [200106, 330174], [200109, 330177],
    [200112, 330180], [200115, 330183], [200118, 330186], [200121, 330189], [200124, 330192],
    [200127, 330195], [200130, 330198], [270134, 330201], [270137, 330204], [270140, 330207],
    [270191, 330210], [270197, 330213], [230067, 330216], [230070, 330219],
];

/// `lbl_803BB3C0[ckind].x8`: a CharacterKind's bank mask (ChKind_Max = 33 rows).
pub static CKIND_BANK_MASK: [u64; 33] = [
    0x40, 0x100, 0x800, 0x20000000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000,
    0x100000, 0x200000, 0x400000, 0x1000000, 0x2000, 0x2000000, 0x4000000, 0x10000000,
    0x8000000, 0x8000000, 0x400, 0x80, 0x200, 0x80000000, 0x800000, 0x40000000, 0x100000000,
    0, 0, 0x1000, 0x100000000, 0, 0x2000,
];

/// `unk_arr_803BC4A0` (0x21 rows; the last row is zero-initialised).
pub static FIGHTER_BGM: [[u8; 2]; 0x21] = [
    [0x38, 0x03], [0x32, 0x21], [0x06, 0x54], [0x1E, 0x1E], [0x23, 0x31], [0x04, 0x04],
    [0x22, 0x4B], [0x2B, 0x2B], [0x4D, 0x4D], [0x01, 0x01], [0x41, 0x41], [0x3C, 0x1F],
    [0x42, 0x42], [0x40, 0x41], [0x28, 0x02], [0x40, 0x40], [0x61, 0x33], [0x60, 0x5F],
    [0x22, 0x4A], [0x22, 0x4A], [0x54, 0x06], [0x4A, 0x4A], [0x07, 0x07], [0x01, 0x01],
    [0x3F, 0x3F], [0x22, 0x22], [0x62, 0x62], [0x62, 0x62], [0x62, 0x62], [0x62, 0x62],
    [0x62, 0x62], [0x62, 0x62], [0, 0],
];

/// The file statics of lbaudio_ax.c the ported functions touch.
#[derive(Clone, Debug)]
pub struct LbState {
    /// lbl_804337C4: requested banks (1) for the next load.
    pub requested: [i32; LBAX_CAP],
    /// lbl_804338A4: wanted banks of the current load.
    pub wanted: [i32; LBAX_CAP],
    /// lbl_80433984: -1 none, 1 loaded, 2 loaded and committed.
    pub state: [i32; LBAX_CAP],
    /// lbl_80433A64: the HSD_SynthSFXLoad entrynum.
    pub entry: [i32; LBAX_CAP],
    /// lbl_804D6438 / 643C / 6440 / 6444: total and per-bank ARAM sizes.
    pub total_size: i32,
    pub bank_size: [i32; 3],
    /// lbl_804D6448 / 644C / 6450: loaded / wanted / pending sizes.
    pub loaded_size: i32,
    pub wanted_size: i32,
    pub pending_size: u32,
    /// lbl_8043373C / lbl_80433780: deferred sfx and their timers.
    pub deferred: [i32; 17],
    pub deferred_timer: [i32; 17],
    /// lbl_804D640C.
    pub muted: i32,
    /// lbl_804D641C.
    pub flush_deferred: i32,
    /// lbl_804D6420 / 6424 / 6428 / 642C / 6430 / 6434.
    pub hb1: i32,
    pub hb2: i32,
    pub hb1_vol: i32,
    pub hb2_vol: i32,
    pub hb1_timer: i32,
    pub hb2_timer: i32,
    /// lbl_804D38F0 / 38F4: the two loop voices.
    pub hb1_voice: i32,
    pub hb2_voice: i32,
    /// lbl_804D38EC.
    pub duck: f32,
    /// lbl_804D38D8: the stage echo byte.
    pub stage_echo: i32,
    /// cur_hps_stem.
    pub cur_hps: String,
    /// ssm_stem_pos 10 ("/audio/us/") vs 7 ("/audio/").
    pub ssm_dir: &'static str,
    /// The volume statics (lbvol.rs).
    pub vol: crate::lbvol::Vol,
    /// Live sound objects (posobj.rs), what they read from the game, and the handle counter.
    pub objs: Vec<crate::posobj::SoundObj>,
    pub env: crate::posobj::PosEnv,
    pub obj_seq: usize,
}

impl Default for LbState {
    fn default() -> Self {
        LbState {
            requested: [-1; LBAX_CAP],
            wanted: [-1; LBAX_CAP],
            state: [-1; LBAX_CAP],
            entry: [-1; LBAX_CAP],
            total_size: 0,
            bank_size: [0; 3],
            loaded_size: 0,
            wanted_size: 0,
            pending_size: 0,
            deferred: [SFX_NONE; 17],
            deferred_timer: [0; 17],
            muted: 0,
            flush_deferred: 0,
            hb1: 0,
            hb2: 0,
            hb1_vol: 0,
            hb2_vol: 0,
            hb1_timer: 0,
            hb2_timer: 0,
            hb1_voice: -1,
            hb2_voice: -1,
            duck: 1.0,
            stage_echo: 1,
            cur_hps: String::new(),
            ssm_dir: "/audio/us/",
            vol: crate::lbvol::Vol::default(),
            objs: Vec::new(),
            env: Default::default(),
            obj_seq: 0,
        }
    }
}

// ---- pure lookups ---------------------------------------------------------------------------

/// lbAudioAx_8002305C(ckind, which): a fighter's victory/series BGM id.
pub fn lb_8002305c(ckind: i32, which: i32) -> i32 {
    if (0..0x21).contains(&ckind) {
        FIGHTER_BGM[ckind as usize][which as usize] as i32
    } else {
        0x62
    }
}

/// lbAudioAx_80023090(bgm): the BGM's flags byte.
pub fn lb_80023090(idx: i32) -> i32 {
    // flags_arr_803BB800's first 0x62 bytes.
    const F: [i8; 0x62] = [
        1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0,
        0, 0, 0, 1, 1, 1, 1, 1,
    ];
    if (0..0x62).contains(&idx) { F[idx as usize] as i32 } else { 0 }
}

/// lbAudioAx_800230C8(bank, &lo, &hi): 1 when out of range.
pub fn lb_800230c8(i: i32, lo: Option<&mut i32>, hi: Option<&mut i32>) -> i32 {
    if !(0..LBAX_N as i32).contains(&i) {
        return 1;
    }
    if let Some(lo) = lo {
        *lo = BANK_RANGE[i as usize][0];
    }
    if let Some(hi) = hi {
        *hi = BANK_RANGE[i as usize][1];
    }
    0
}

/// lbAudioAx_80023130(id): the bank a sound id belongs to, 55 for none.
pub fn lb_80023130(id: i32) -> i32 {
    if (0..SFX_NONE).contains(&id) {
        for (i, r) in BANK_RANGE.iter().take(LBAX_N).enumerate() {
            if r[0] <= id && id <= r[1] {
                return i as i32;
            }
        }
    }
    55
}

/// lbAudioAx_80023220(bank): the bank's x3 byte.
pub fn lb_80023220(idx: i32) -> i32 {
    if (0..LBAX_N as i32).contains(&idx) { BANK_INFO[idx as usize][3] as i32 } else { 0 }
}

/// fn_80023254(group): the bank list of a group, by size, largest first (lbl_80433B44).
pub fn fn_80023254(group: i32) -> [usize; LBAX_CAP] {
    let mut list = [55usize; LBAX_CAP];
    let mut used = [false; LBAX_CAP];
    for count in 0..=LBAX_N {
        for index in 0..=LBAX_N {
            if group == BANK_INFO[index][0] as i32 && !used[index] && (BANK_SIZE[list[count]] as u32) < (BANK_SIZE[index] as u32) {
                let mut i = LBAX_N;
                while i > count {
                    list[i] = list[i - 1];
                    i -= 1;
                }
                list[count] = index;
                used[index] = true;
            }
        }
    }
    list
}

/// The boot's three SFX bank sizes (lbl_804D643C / 6440 / 6444), as lbaudio_ax.c:2393 sums them.
pub fn boot_bank_sizes() -> [i32; 3] {
    let b0 = BANK_SIZE[0];
    let b1 = BANK_SIZE[0x33] + BANK_SIZE[1] + BANK_SIZE[0x36];
    let l3 = fn_80023254(3);
    let mut b2 = BANK_SIZE[l3[0]];
    let l4 = fn_80023254(4);
    b2 += BANK_SIZE[l4[0]] + BANK_SIZE[l4[1]] + BANK_SIZE[l4[2]] + BANK_SIZE[l4[3]];
    let l5 = fn_80023254(5);
    b2 += BANK_SIZE[l5[0]];
    [b0, b1, b2]
}

/// lbAudioAx_80026E84(ckind): the bank mask of a CharacterKind.
pub fn lb_80026e84(ckind: i32) -> u64 {
    if (0..33).contains(&ckind) { CKIND_BANK_MASK[ckind as usize] } else { 0 }
}

/// lbAudioAx_80026EBC on a GrKind (Stage_8022519C already applied).
pub fn lb_80026ebc_grkind(grkind: i32) -> u64 {
    if !(0..0x6F).contains(&grkind) {
        return 0;
    }
    let shift = stage_audio(grkind)[0] as u32;
    if shift == 55 || shift as usize >= LBAX_N {
        return 0;
    }
    <u64 as melee_core::ppc::Ppc>::pshl(1u64, (shift) as i64)
}

fn flags_mask(flags: u32) -> u64 {
    let mut m = 0u64;
    if flags & 1 != 0 {
        m = m.wrapping_add(0x0048000000000003);
    }
    if flags & 2 != 0 {
        m = m.wrapping_add(0x3C);
    }
    if flags & 4 != 0 {
        m = m.wrapping_add(0x00800003FFFFFFC0);
    }
    if flags & 8 != 0 {
        m = m.wrapping_add(0x0023FFFC00000000);
    }
    if flags & 16 != 0 {
        m = m.wrapping_add(0x0014000000000000);
    }
    m
}

// ---- stateful ------------------------------------------------------------------------------

impl Sound {
    fn bank_loaded_committed(&self, bank: usize) -> bool {
        self.lb.state[bank] == 2 // fn_80026E58
    }

    /// lbAudioAx_800233EC(id): Kirby's copy-ability voice remap, by whether kirbytm.ssm (0x21)
    /// is the committed bank.
    pub fn lb_audio_ax_800233ec(&self, id: i32) -> i32 {
        let slot = lb_80023130(id);
        if self.bank_loaded_committed(0x21) {
            if matches!(slot, 6..=11 | 13 | 15..=31) {
                if let Some(r) = KIRBY_REMAP.iter().find(|r| r[0] == id) {
                    return r[1];
                }
            }
        } else if slot == 0x21 {
            if let Some(r) = KIRBY_REMAP.iter().find(|r| r[1] == id) {
                return r[0];
            }
        }
        id
    }

    /// lbAudioAx_80026F2C(flags): clear the requests of the flagged groups.
    pub fn lb_audio_ax_80026f2c(&mut self, flags: u32) {
        let mut mask = flags_mask(flags);
        for i in 0..LBAX_CAP - 1 {
            if mask & 1 != 0 {
                self.lb.requested[i] = -1;
            }
            mask >>= 1;
        }
    }

    /// lbAudioAx_8002702C(flags, mask): request the masked banks of the flagged groups.
    pub fn lb_audio_ax_8002702c(&mut self, flags: u32, mask: u64) {
        let mut mask = mask.wrapping_add(0x48000000000003);
        let mut result = flags_mask(flags);
        for i in 0..LBAX_CAP - 1 {
            if result & 1 != 0 && mask & 1 != 0 {
                self.lb.requested[i] = 1;
            }
            result >>= 1;
            mask >>= 1;
        }
    }

    fn fn_80026650(&self) -> Option<usize> {
        for pri in (0..=4).rev() {
            for i in 0..LBAX_N {
                if pri == BANK_INFO[i][1] as i32 && self.lb.wanted[i] == 1 && self.lb.state[i] == -1 {
                    return Some(i);
                }
            }
        }
        None
    }

    fn fn_800268b4(&mut self) {
        let lb = &mut self.lb;
        lb.pending_size = 0;
        lb.wanted_size = 0;
        lb.loaded_size = 0;
        for i in 0..LBAX_N {
            let f = (lb.wanted[i] != -1) as i32 | if lb.state[i] != -1 { 2 } else { 0 };
            let s = BANK_SIZE[i];
            match f {
                1 => {
                    lb.wanted_size += s;
                    lb.pending_size = lb.pending_size.wrapping_add(s as u32);
                }
                2 => lb.loaded_size += s,
                3 => {
                    lb.wanted_size += s;
                    lb.loaded_size += s;
                }
                _ => {}
            }
        }
    }

    fn fn_800267b0(&mut self) {
        if self.lb.pending_size == 0 {
            return;
        }
        for pri in 0..5 {
            let mut j = 0;
            while ((self.lb.total_size as u32) < (self.lb.loaded_size as u32).wrapping_add(self.lb.pending_size)) && j < LBAX_N {
                if self.lb.state[j] != -1 && pri == BANK_INFO[j][2] as i32 && self.lb.wanted[j] == -1 {
                    let e = self.lb.entry[j];
                    self.sfx_group_data_remove(e);
                    self.lb.entry[j] = -1;
                    self.lb.state[j] = -1;
                    self.lb.loaded_size -= BANK_SIZE[j];
                }
                j += 1;
            }
        }
        self.sfx_bank_deflag(2);
    }

    /// The fn_80026C04 load chain, run to completion: load every wanted bank into SFX bank 2,
    /// highest load priority first.
    fn load_chain(&mut self) {
        while let Some(slot) = self.fn_80026650() {
            let path = format!("{}{}", self.lb.ssm_dir, SSM_FILES[slot]);
            match self.load_ssm(&path, 2) {
                Ok(e) => {
                    self.lb.entry[slot] = e;
                    self.lb.state[slot] = 1;
                    self.lb.wanted_size -= BANK_SIZE[slot];
                    self.lb.pending_size = self.lb.pending_size.wrapping_sub(BANK_SIZE[slot] as u32);
                    self.lb.loaded_size += BANK_SIZE[slot];
                }
                Err(_) => {
                    // The C waits forever on a failed DVD read; mark it so the chain moves on.
                    self.lb.entry[slot] = -1;
                    self.lb.state[slot] = 1;
                }
            }
        }
    }

    /// lbAudioAx_80027168 + lbAudioAx_80027648: commit the requested banks and load them.
    pub fn lb_audio_ax_80027168(&mut self) {
        for i in 0..LBAX_N {
            if BANK_INFO[i][1] != 5 && self.lb.state[i] == 2 {
                self.lb.state[i] = 1;
            }
        }
        let count = (0..LBAX_N).filter(|&i| BANK_INFO[i][1] != 5 && self.lb.requested[i] != -1).count();
        if count == 0 {
            return;
        }
        self.lb.wanted = self.lb.requested;
        self.fn_800268b4();
        self.fn_800267b0();
        self.load_chain();
        // lbAudioAx_80027648 / fn_80027488
        for i in 0..LBAX_N {
            if self.lb.wanted[i] == 1 && self.lb.state[i] == 1 {
                self.lb.state[i] = 2;
            }
        }
    }

    /// `gm_LoadAnnouncer` (gm_1601.c): drop the scene-group requests and request nr_vs.ssm (the
    /// announcer, countdown and VS-scene sounds). The VS scene runs it before 8002785C.
    pub fn lb_load_announcer(&mut self) {
        self.lb_audio_ax_80026f2c(0x12);
        self.lb_audio_ax_8002702c(2, 0x20);
        self.lb_audio_ax_80027168();
    }

    /// lbAudioAx_8002785C's VS branch: the players' CharacterKinds, the stage's StKind and its
    /// GrKind (Stage_8022519C). Loads the banks and sets the stage echo byte.
    pub fn lb_load_match_banks(&mut self, ckinds: &[i32], stkind: i32, grkind: i32) {
        let mut result = 0u64;
        for &ck in ckinds {
            result |= lb_80026e84(ck);
        }
        if stkind == 0xD9 || stkind == 0xE5 {
            result |= 0x200004000;
        }
        if stkind == 0x46 || stkind == 0x47 {
            result |= 0xC00;
        }
        self.lb.stage_echo = stage_audio(grkind)[1] as i32;
        result |= lb_80026ebc_grkind(grkind);
        if result != 0 {
            self.lb_audio_ax_80026f2c(0xC);
            self.lb_audio_ax_8002702c(0xC, result);
            self.lb_audio_ax_80027168();
        }
    }

    /// lbAudioAx_800236DC: stop the BGM.
    pub fn lb_audio_ax_800236dc(&mut self) -> i32 {
        self.stream_stop();
        self.lb.cur_hps.clear();
        -1
    }

    /// lbAudioAx_80023F28(bgm): start BGM `id` unless it already plays. 1 = nothing started.
    pub fn lb_audio_ax_80023f28(&mut self, id: i32) -> i32 {
        if !(0..0x62).contains(&id) {
            return 1;
        }
        let file = HPS_FILES[id as usize];
        let path = format!("/audio/{file}");
        if self.lb.cur_hps == path {
            return 1;
        }
        self.lb_audio_ax_800236dc();
        self.lb.cur_hps = path;
        self.lb_play_bgm_file(file, 0x7F, 1);
        0
    }

    /// lbAudioAx_80024B58(voice, vol).
    pub fn lb_audio_ax_80024b58(&mut self, voice: i32, vol: i32) {
        self.audio_sfx_set_volume_ex(voice, (vol.clamp(0, 0x7F) * 2) as u8);
    }

    /// lbAudioAx_80024DC4(id): defer a sound effect (10 frames).
    pub fn lb_audio_ax_80024dc4(&mut self, id: i32) {
        let lb = &mut self.lb;
        for i in 0..0x10 {
            if lb.deferred[i] == id {
                lb.deferred_timer[i] = 0xA;
                return;
            }
        }
        for i in 0..0x10 {
            if lb.deferred[i] == SFX_NONE {
                lb.deferred[i] = id;
                lb.deferred_timer[i] = 0xA;
                return;
            }
        }
    }

    pub fn lb_audio_ax_80024fdc(&mut self) {
        self.lb.hb1_timer = 0x1FE;
        self.lb.hb1 += 1;
    }
    pub fn lb_audio_ax_80024ff4(&mut self) {
        self.lb.hb2_timer = 0x1E0;
        self.lb.hb2 += 1;
    }
    pub fn lb_audio_ax_8002500c(&mut self, n: i32) {
        if n > 0 {
            self.lb.hb1 -= n;
            if self.lb.hb1 <= 0 {
                self.lb.hb1 = 0;
            }
        }
    }
    pub fn lb_audio_ax_80025038(&mut self, n: i32) {
        if n > 0 {
            self.lb.hb2 -= n;
            if self.lb.hb2 < 0 {
                self.lb.hb2 = 0;
            }
        }
    }

    /// lbAudioAx_80027DF8: the per-frame audio update (without fn_80024654's ducking).
    pub fn lb_audio_ax_80027df8(&mut self) {
        if self.lb.muted == 0 {
            self.lb.hb1_timer -= 1;
            if self.lb.hb1_timer <= 0 {
                self.lb.hb1 = 0;
                self.lb.hb1_timer = 0;
            }
            self.lb.hb2_timer -= 1;
            if self.lb.hb2_timer <= 0 {
                self.lb.hb2 = 0;
                self.lb.hb2_timer = 0;
            }
        }
        if self.lb.hb1 != 0 {
            if self.lb.hb1_voice != -1 {
                if !self.audio_sfx_check(self.lb.hb1_voice) {
                    self.lb.hb1_voice = self.lb_start(0x84, self.lb.hb1_vol, 0x40, 5, 4);
                } else {
                    self.lb_audio_ax_80024b58(self.lb.hb1_voice, self.lb.hb1_vol);
                }
            } else {
                self.lb.hb1_vol = 0x7F;
                self.lb.hb1_voice = self.lb_start(0x84, 0x7F, 0x40, 5, 4);
                self.lb.hb2_vol = 0;
            }
        } else if self.lb.hb1_voice != -1 {
            self.lb_start(SFX_NONE, self.lb.hb1_vol, 0x40, 5, 4);
            self.lb.hb1_vol = 0;
            self.lb.hb1_voice = -1;
            self.lb.hb2_vol = if self.lb.hb2_voice == -1 { 0 } else { 0x7F };
        }
        if self.lb.hb2 != 0 {
            if self.lb.hb2_voice != -1 {
                if !self.audio_sfx_check(self.lb.hb2_voice) {
                    self.lb.hb2_voice = self.lb_start(0x85, self.lb.hb2_vol, 0x40, 6, 4);
                } else {
                    self.lb_audio_ax_80024b58(self.lb.hb2_voice, self.lb.hb2_vol);
                }
            } else {
                self.lb.hb2_vol = 0x7F;
                self.lb.hb2_voice = self.lb_start(0x85, 0x7F, 0x40, 6, 4);
                self.lb.hb1_vol = 0;
            }
        } else if self.lb.hb2_voice != -1 {
            self.lb_start(SFX_NONE, self.lb.hb2_vol, 0x40, 6, 4);
            self.lb.hb2_vol = 0;
            self.lb.hb2_voice = -1;
            self.lb.hb1_vol = if self.lb.hb1_voice == -1 { 0 } else { 0x7F };
        }
        self.lb.duck = if self.lb.hb1_voice == -1 && self.lb.hb2_voice == -1 { 1.0 } else { 0.2 };
        self.lb_fn_80024654(false);
        if self.lb.flush_deferred == 1 {
            for i in 0..0x10 {
                if self.lb.deferred[i] != SFX_NONE {
                    let id = self.lb.deferred[i];
                    self.lb_audio_ax_800237a8(id, 0x7F, 0x40);
                    self.lb.deferred[i] = SFX_NONE;
                    self.lb.deferred_timer[i] = 0;
                }
            }
            self.lb.flush_deferred = 0;
            return;
        }
        if self.lb.muted == 0 {
            for i in 0..16 {
                if self.lb.deferred_timer[i] > 0 {
                    self.lb.deferred_timer[i] -= 1;
                }
                if self.lb.deferred_timer[i] <= 0 {
                    self.lb.deferred[i] = SFX_NONE;
                    self.lb.deferred_timer[i] = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_lookups() {
        assert_eq!(lb_80023130(0), 0);
        assert_eq!(lb_80023130(0x2750), 1);
        assert_eq!(lb_80023130(0xEA60), 6);
        assert_eq!(lb_80023130(0x83D60), 55);
        assert_eq!(lb_80023130(-1), 55);
        let (mut lo, mut hi) = (0, 0);
        assert_eq!(lb_800230c8(11, Some(&mut lo), Some(&mut hi)), 0);
        assert_eq!((lo, hi), (0x1ADB0, 0x1AE30));
        assert_eq!(lb_800230c8(55, None, None), 1);
        assert_eq!(lb_80023220(13), 9);
        assert_eq!(stage_audio(37), [0x35, 0x38, 0x38]);
        assert_eq!(lb_80026ebc_grkind(37), 1 << 0x35);
        assert_eq!(lb_80026e84(0), 1 << 6); // Captain -> captain.ssm
    }

    #[test]
    fn kirby_remap_follows_kirbytm() {
        let mut s = Sound::new();
        assert_eq!(s.lb_audio_ax_800233ec(150050), 150050);
        assert_eq!(s.lb_audio_ax_800233ec(330000), 150050);
        s.lb.state[0x21] = 2;
        assert_eq!(s.lb_audio_ax_800233ec(150050), 330000);
        assert_eq!(s.lb_audio_ax_800233ec(330000), 330000);
    }

    #[test]
    fn boot_sizes_from_table() {
        // group 3 = nr_title(153024)/nr_select(239264)/nr_1p/nr_vs: biggest 239264; group 4's
        // four biggest fighters; group 5's biggest stage.
        let s = boot_bank_sizes();
        assert_eq!(s[0], 2045824);
        assert_eq!(s[1], 348160 + 561760 + 1536);
        let l3 = fn_80023254(3);
        assert_eq!(l3[0], 3);
        let l4 = fn_80023254(4);
        assert_eq!(&l4[..4], &[24, 10, 27, 14]); // pikachu, falco, zs, kirby: the four largest
        assert_eq!(s[2], 239264 + 613088 + 599712 + 590176 + 586208 + BANK_SIZE[fn_80023254(5)[0]]);
    }

    #[test]
    fn deferred_and_heartbeat_counters() {
        let mut s = Sound::new();
        s.lb_audio_ax_80024dc4(5);
        s.lb_audio_ax_80024dc4(5);
        assert_eq!(s.lb.deferred[0], 5);
        assert_eq!(s.lb.deferred[1], SFX_NONE);
        for _ in 0..10 {
            s.lb_audio_ax_80027df8();
        }
        assert_eq!(s.lb.deferred[0], SFX_NONE);
        s.lb_audio_ax_80024fdc();
        assert_eq!((s.lb.hb1, s.lb.hb1_timer), (1, 0x1FE));
        s.lb_audio_ax_8002500c(3);
        assert_eq!(s.lb.hb1, 0);
    }
}
