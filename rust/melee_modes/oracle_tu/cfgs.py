"""Per-TU oracle configs. Keys: src, module (Rust module under melee_modes::tu), cases (default per
function), funcs {name: {args: [spec...], cases, test}}, skip {name: reason}, mock {ext fn: "lo..hi"},
extsize {extern array: n}, local_types [names the port module defines itself], mockobj {fn: shared},
raw {object: nbytes} (compared as raw bytes), patch [(old, new)] applied to the TU text for the C side.
Arg specs: "lo..hi" | "N" | "null" | "buf[:N]" | "str" | "obj:<state object>"."""

# u8_bits overlays: PPC allocates bitfields MSB-first, x86 LSB-first; reverse the declaration so
# the native C sets the same bits the GameCube build does (bN is mask 1 << N).
U8_BITS = ("u8 b7 : 1, b6 : 1, b5 : 1, b4 : 1, b3 : 1, b2 : 1, b1 : 1, b0 : 1;",
           "u8 b0 : 1, b1 : 1, b2 : 1, b3 : 1, b4 : 1, b5 : 1, b6 : 1, b7 : 1;")

CFGS = {
    "gm_17eb": dict(
        src="src/melee/gm/gm_17EB.c", module="gm_17eb", cases=60,
        local_types=["AllstarStageEntry"],
        patch=[U8_BITS,
               # the storage is the record itself (u8[0x78] is its GC size; the native record is bigger)
               ("static u8 lbl_80472CB0[0x78];", "static UnkAllstarData lbl_80472CB0;"),
               ("return (UnkAllstarData*) lbl_80472CB0;", "return &lbl_80472CB0;"),
               ("allstar = (UnkAllstarData*) lbl_80472CB0;", "allstar = &lbl_80472CB0;"),
               # the C passes a GC pointer through an int; native code takes the pointer
               ("void fn_8017EE40(int arg0_int)\n{\n    MatchEnd* arg0 = (MatchEnd*) arg0_int;\n",
                "void fn_8017EE40_ptr(MatchEnd* arg0)\n{\n")],
        rust_names={"fn_8017EE40_ptr": "fn_8017EE40"},
        funcs={
            "gm_8017EB3C": dict(args=["0..4", "0..4"]),
            "gm_8017EB64": dict(args=["0..4", "0..4", "0..2"]),
            "gm_8017EB98": dict(args=["0..4", "0..4", "0..2"]),
            "gm_8017EBCC": dict(args=["0..4", "0..4", "0..2"]),
            "gm_8017EC00": dict(args=["0..4", "0..4"]),
            "gm_8017EC50": dict(args=["0..4", "0..4"]),
            "gm_8017ECA0": dict(args=["0..4", "0..4", "0..1"]),
            "gm_8017ECD4": dict(args=["0..4", "0..4", "0..1"]),
            "gm_8017ED08": dict(args=["0..4", "0..4", "0..1"]),
            "gm_8017ED3C": dict(args=["0..4", "0..4"]),
            "gm_8017ED8C": dict(args=["0..4", "0..4"]),
            "fn_8017EE40_ptr": dict(cases=400),
        },
        mock={"gm_GetCurrentGameMode": "2..4", "Player_GetPlayerSlotType": "0..3",
              "Player_GetPlayerCharacter": "28..32", "fn_8017E318": "-1..2"},
        fieldrange={"Unk1PData_xC.xD": "0..1", "Unk1PData_xC.xE": "0..1", "Unk1PData_xC.xF": "0..1",
                    "Unk1PData_xC.x1C": "0..2", "gmPlayerData.cpu_level": "0..4",
                    "MatchPlayerData.x44": "0..2", "MatchPlayerData.stocks": "0..4", "gmm_x0_528_t.stocks": "0..4"}),

    "gm_17ad": dict(
        src="src/melee/gm/gm_17AD.c", module="gm_17ad", cases=80,
        funcs={n: dict(args=["0..5"]) for n in (
            "fn_8017ADA8 fn_8017AE0C fn_8017B010 fn_8017B21C fn_8017B280 fn_8017B2E4 fn_8017B348 fn_8017B3AC "
            "fn_8017B410 fn_8017B4D0 fn_8017B534 fn_8017B598 fn_8017B5FC fn_8017B660 fn_8017B6C4 fn_8017B728 "
            "fn_8017B78C fn_8017B7F0 fn_8017B854 fn_8017B8B8 fn_8017B91C fn_8017B9F4").split()} | {
            n: dict(args=["0..3"]) for n in (
            "fn_8017AE70 fn_8017AED8 fn_8017AF40 fn_8017AFA8 fn_8017B07C fn_8017B0E4 fn_8017B14C fn_8017B1B4").split()},
        mock={"lbLang_IsSavedLanguageUS": "0..1"},
        fieldrange={"MatchPlayerData.pkind": "0..3"}),

    "gm_17ba": dict(
        src="src/melee/gm/gm_17BA.c", module="gm_17ba", cases=120,
        funcs={"fn_8017BACC": dict(args=["0..5"]), "fn_8017BB30": dict(args=["0..5"]),
               "fn_8017BB94": dict(args=["0..3"]), "fn_8017BC50": dict(args=["0..3"]),
               "fn_8017BD0C": dict(args=["0..3"]), "fn_8017BDC8": dict(args=["0..3"]),
               "fn_8017BB94_inline": dict(args=["0..3", "0..3"])},
        mock={"lbLang_IsSavedLanguageUS": "0..1"},
        fieldrange={"MatchPlayerData.pkind": "0..3", "MatchPlayerData.x4": "0x76..0x7A"}),

    "gm_17e4": dict(
        src="src/melee/gm/gm_17E4.c", module="gm_17e4", cases=100,
        local_types=["AdventureStageEntry"],
        patch=[U8_BITS,
               ("void fn_8017E8A4(int arg0_int)\n{\n    MatchEnd* arg0 = (MatchEnd*) arg0_int;\n",
                "void fn_8017E8A4_ptr(MatchEnd* arg0)\n{\n")],
        rust_names={"fn_8017E8A4_ptr": "fn_8017E8A4"},
        extsize={},
        fixups=[("gm_803DE650[45].x0 = 0xFF; gm_803DE650[0].x0 = 3; gm_803DE650[1].x0 = 0x11; gm_803DE650[2].x0 = 0x25; "
                 "gm_803DE650[3].x0 = 0x53; gm_803DE650[4].x0 = 0x5C;",
                 "{ let t = &mut *core::ptr::addr_of_mut!(m::ext::gm_803DE650); t[45].x0 = 0xFF; t[0].x0 = 3; t[1].x0 = 0x11; "
                 "t[2].x0 = 0x25; t[3].x0 = 0x53; t[4].x0 = 0x5C; }")],
        funcs={
            "gm_8017E48C": dict(args=["idx:gm_Mode_Adventure_States:0..48"]),
            "gm_8017E4C4": dict(args=["3|17|37|83|92|5|0|255"]),
            "gm_8017E500": dict(args=["0..21", "0..4"]),
            "gm_8017E528": dict(args=["0..21", "0..4"]),
            "gm_8017E578": dict(args=["0..21", "0..4"]),
            "gm_8017E5C8": dict(args=["0..21", "0..4", "0..4"]),
            "gm_8017E5FC": dict(args=["0..21", "0..4", "0..4"]),
            "gm_8017E630": dict(args=["0..21", "0..4", "0..4"]),
            "gm_8017E664": dict(args=["0..21", "0..4"]),
            "gm_8017E6B4": dict(args=["0..21", "0..4"]),
            "gm_8017E704": dict(args=["0..21", "0..4", "0..1"]),
            "gm_8017E738": dict(args=["0..21", "0..4", "0..1"]),
            "gm_8017E76C": dict(args=["0..21", "0..4", "0..1"]),
            "gm_8017E7A0": dict(args=["0..3"]),
            "fn_8017E8A4_ptr": dict(cases=400),
        },
        mock={"gm_GetCurrentSceneIndex": "3|17|37|83|92|3|17", "gm_GetCurrentGameMode": "3|4|5",
              "gm_RumbleEnabledForPlayer": "0..1", "Player_GetPlayerSlotType": "0..3",
              "Player_GetPlayerCharacter": "0|7|13|14|20|21|30", "fn_8017E318": "-1..2"},
        fieldrange={"Unk1PData_xC.xD": "0..1", "Unk1PData_xC.xE": "0..1", "Unk1PData_xC.xF": "0..1",
                    "Unk1PData_xC.x1C": "0..2", "gmPlayerData.cpu_level": "0..4", "VsSceneState.unk_C": "0..7",
                    "MatchPlayerData.x44": "0..2", "MatchPlayerData.stocks": "0..4", "gmm_x0_528_t.stocks": "0..4"}),

    "gm_1736": dict(
        src="src/melee/gm/gm_1736.c", module="gm_1736", cases=120,
        local_types=["lbl_803D6450_t", "lbl_803D646C_t"],
        funcs={
            "fn_80173834": dict(args=["0..30", "3..5", "0..1"]),
            "gm_8017390C": dict(args=["0..3", "0..1"]),
            "gm_80173BC4": dict(args=["0..30"]),
            "gm_80173B30": dict(cases=200),
            "gm_80173EEC": dict(cases=300),
            "gm_80173EEC_inline": dict(args=["0..30", "3..5"]),
        },
        mock={"Mex_Enabled": "0|0|0|0|0|0|0|1", "gm_801721EC": "0..1", "fn_80164B48": "0..1",
              "gmMainLib_8015EE90": "0..1", "un_803045A0": "0..1", "un_80304690": "0..1", "un_80304780": "0..1",
              "fn_80162CCC": "0..1", "gm_80162EC8": "0..1", "gm_80164ABC": "0..1", "gmMainLib_8015D640": "0..1",
              "gmMainLib_8015D94C": "0..1", "gm_80160474": "0..1",
              "gmMainLib_8015DADC": "|".join(["1"] * 60 + ["0"]),
              "gm_CKindToSelKind": "0..24", "gm_SelKindToCKind": "18|19|0|5|7|18|19",
              "gm_801BEBC0": "2|13|25|44|46|50|7", "lbLang_GetLanguageSetting": "0..1",
              "gm_GetCurrentGameMode": "34|36|37|38|3|4", "gm_GetPreviousGameMode": "0..40"},
        fieldrange={"MatchPlayerData.pkind": "0..3", "Unk1PData_xC.xD": "0..1", "gmPlayerData.cpu_level": "0..4"}),
}
