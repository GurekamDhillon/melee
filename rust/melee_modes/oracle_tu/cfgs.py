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

    "gm_16f1": dict(
        src="src/melee/gm/gm_16F1.c", module="gm_16f1", cases=100, cstr_nocls=True,
        local_types=["lbl_803D5A4C_t", "lbl_803D5A4C_tbl", "lbl_803B7A60_t", "lbl_804D65A8_t", "lbl_8046DBC8_t",
                     "lbl_8046DBC8_x2", "lbl_804D65A0_t", "lbl_803B7AD0_t", "lbl_803D6300_t"],
        # PPC allocates bitfields MSB-first: the byte's bit 0 is x3_b7 (the native C allocates LSB-first)
        patch=[("(x58[i].x3 & 1))", "(x58[i].x3_b7 & 1))"),
               ("int fn_8016F39C(HSD_Text** arg0, void* arg1,", "int fn_8016F39C_ptr(HSD_Text** arg0, MatchEnd* arg1,"),
               ("int fn_8016F548(void* arg0,", "int fn_8016F548_ptr(MatchEnd* arg0,"),
               ("int fn_8016F740(void* arg0,", "int fn_8016F740_ptr(MatchEnd* arg0,"),
               ("int fn_8016F870(void* arg0,", "int fn_8016F870_ptr(MatchEnd* arg0,"),
               ("int fn_8016F9A8(void* arg0,", "int fn_8016F9A8_ptr(MatchEnd* arg0,"),
               ("int fn_80170110(void* arg0,", "int fn_80170110_ptr(MatchEnd* arg0,"),
               ("s32 scores[4];\n    u8 rankings[7] = { 0 };\n\n    if (lbl_804D65A0.x0", "s32 scores[6];\n    u8 rankings[7] = { 0 };\n\n    if (lbl_804D65A0.x0")],
        anon_vars={"lbl_803D5A4C": "lbl_803D5A4C_tbl", "lbl_8046DBC8": ("lbl_8046DBC8_t", {"x2": "lbl_8046DBC8_x2"}),
                   "lbl_804D65A0": "lbl_804D65A0_t"},
        rust_names={n+"_ptr": n for n in "fn_8016F39C fn_8016F548 fn_8016F740 fn_8016F870 fn_8016F9A8 fn_80170110".split()},
        mock={"fn_80163FA4": "0x1E|0x33|0|1|0x1E|0x33", "pl_80039418": "0|1", "fn_8016F160": "0|1",
              "Player_GetPlayerSlotType": "0..3", "Player_GetFalls": "0..4", "Player_GetSelfDestructs": "0..4",
              "Player_GetTeam": "0..2", "Player_GetKOsByPlayerIndex": "0..3", "gm_IsCurrently1PMode_inline": "0..1",
              "fn_8017E0E4": "-1..2", "lbLang_IsSettingUS": "0..1", "lbLang_GetLanguageSetting": "0..1",
              "Mex_Enabled": "0|0|0|0|0|0|0|1", "Toy_GetTrophyTotal": "0|49|50|99|100|149|150|199|200|249|250|300",
              "gm_GetVsPlayMatchTotal": "0|99|100|999|1000|4994|4995|6799|6800|9999|10000|49999|50000|99999|100000|999999|1000000|2000000",
              "Toy_803048C0": "-1..2", "gmMainLib_8015D94C": "0..1", "gmMainLib_8015D8D8": "0..1",
              "gmMainLib_8015DA1C": "0..1", "gmMainLib_8015D818": "0..1", "gm_8016B3D8": "0..1",
              "fn_8017E160": "0..1", "gm_80164ABC": "0..1", "gm_80164600": "0..1", "fn_80162CCC": "0..1",
              "gm_80162EC8": "0..1", "gmMainLib_8015D1AC": "|".join(["0"] * 40 + ["1"]), "gmMainLib_8015D2D4": "|".join(["0"] * 40 + ["1"]),
              "gmMainLib_8015D21C": "|".join(["1"] * 30 + ["0"]), "gmMainLib_8015D344": "0|1|4", "gmMainLib_8015D0F4": "|".join(["1"] * 30 + ["0"]),
              "gmMainLib_8015CFCC": "0|1|4", "gmMainLib_8015D4A8": "|".join(["1"] * 30 + ["0"]), "gmMainLib_8015CEFC": "|".join(["1"] * 60 + ["0"]),
              "gm_IsCKindUnlocked": "0..1", "gm_80164430": "0..1", "gmMainLib_8015D984": "0..1",
              "gmMainLib_8015DB6C": "0..4", "gm_CKindToUnlockIndex": "0..10", "gm_SelKindToUnlockIndex": "11|11|11|11|3",
              "gm_GetCKindByUnlockIndex": "0..32", "gm_801BEBC0": "0..32", "fn_80160710": "0|1", "fn_801607A8": "0|1",
              "fn_801607F4": "0|1", "pl_80040894": "0..5", "pl_800408B8": "0..5", "pl_800408DC": "0..5"},
        fieldrange={"MatchPlayerData.pkind": "0..3", "MatchPlayerData.x20": "0..5", "MatchPlayerData.x24": "-1..6",
                    "MatchPlayerData.x40": "0..4", "MatchPlayerData.self_destructs": "0..3",
                    "MatchEnd.match_kind": "0..4", "MatchEnd.sd_penalty": "-2..2", "MatchPlayerData.is_big_loser": "0..1",
                    "MatchEnd.x7": "0..2", "MatchPlayerData.x3": "0..1"},
        funcs={
            "fn_8016F180": dict(args=["0..0x2A0"]), "gmDecisionGetType": dict(args=["0..0x2A0"]),
            "gm_8016F208": dict(args=["0..0x2A0"]), "fn_8016F280": dict(args=["0..0x2A0"]),
            "gm_8016F2F8": dict(args=["0..0x2A0"]), "fn_8016F1F0": dict(args=["0..256"]),
            "fn_8016F39C_ptr": dict(args=["buf:4", "buf", "1..4", "0..0x110", "0..255", "0..5"], cases=200),
            "fn_8016F548_ptr": dict(args=["buf", "0..0x110", "0..255", "0..5"], cases=200),
            "fn_8016F740_ptr": dict(args=["buf", "0..0x110", "0..255", "0..5"], cases=200),
            "fn_8016F870_ptr": dict(args=["buf", "0..0x110", "0..255", "0..5"], cases=200),
            "fn_8016F9A8_ptr": dict(args=["buf", "0..0x110", "0..255", "0..5"], cases=200),
            "fn_8016FAD4": dict(args=["buf", "0..0x2A0", "0..255", "0..5"], cases=300),
            "fn_8016FFD4": dict(args=["buf", "0..255", "0..5"], cases=200),
            "fn_80170110_ptr": dict(args=["buf", "0..256", "0..255", "0..5"], cases=300),
            "fn_801701C0": dict(args=["buf", "0..5", "0xD0..0x102"], cases=1500),
            "fn_80171B00": dict(args=["0..5"]), "fn_80171B2C": dict(args=["0..5"]),
            "fn_8017279C": dict(args=["0x14..0x44", "0..0xFFFF"], cases=300),
            "gm_80172898": dict(args=["0..0xFFFF"]),
            "gm_8017219C": dict(args=["0..0x44"]),
            "gm_801721EC_1": dict(args=["0..0x44"]), "gm_801721EC_3": dict(args=["0..0x44"]),
            "inline3": dict(args=["0..0x44"]),
            "gm_80172C44": dict(args=["0..0x44"]), "fn_80172C78": dict(args=["0..30"]),
            "gm_DecideChallengerCpuLevel": dict(args=["0..30", "0..255"]),
            "inline2": dict(args=["0..30"]), "inline1": dict(args=["0..1100"]),
            "gm_80172DD4": dict(args=["0..1100"]), "gm_80172F00": dict(args=["0..250"]),
            "fn_80173098": dict(args=["0..30"]), "gm_80173224": dict(args=["0..30", "0..30"]),
            "gm_801732D8": dict(args=["0..30"]), "gm_80173460": dict(args=["0..30"]),
            "gm_801734D0": dict(args=["0..0xA000"]),
            "fn_8017232C": dict(cases=400), "fn_80172380": dict(cases=400), "fn_801723D4": dict(cases=400),
            "fn_80172428": dict(cases=300), "fn_80172478": dict(cases=300), "fn_80172FAC": dict(cases=400),
            "gm_8017335C": dict(cases=200), "gm_801733D8": dict(cases=200),
        }),
}


def _gm_1798():
    """Results-screen display TU: the pointer graphs the C walks (gobj -> jobj -> dobj -> mobj -> tobj)
    are wired from typed state objects in `fixups` (C text, Rust text), so every write through them is
    hashed."""
    N = 4
    extra = {"gobArg": "HSD_GObj", "cobArg": "HSD_CObj", "scn": "SceneDesc", "fgt": "Fighter",
             "jrR": "HSD_JObj"}
    for i in range(5):
        extra[f"jr{i}"] = "HSD_JObj"
    for i in range(N):
        for a, t in (("gob", "HSD_GObj"), ("job", "HSD_JObj"), ("jch", "HSD_JObj"), ("dA", "HSD_DObj"),
                     ("mA", "HSD_MObj"), ("tA", "HSD_TObj"), ("jbb", "HSD_JObj"), ("dB", "HSD_DObj"),
                     ("dC", "HSD_DObj"), ("mC", "HSD_MObj"), ("tC", "HSD_TObj")):
            extra[f"{a}{i}"] = t
    fx = []

    def link(cexpr, rexpr, ctarget, rtarget, rcast=""):
        fx.append((f"{cexpr} = {ctarget};", f"{rexpr} = {rtarget}{rcast};"))
    O = lambda n: (f"&tuo_mo_{n}", f"core::ptr::addr_of_mut!(MO_{n})")
    V = " as *mut core::ffi::c_void"
    for i in range(N):
        c, r = O(f"gob{i}"); link(f"lbl_8046E38C[{i}]", f"m::lbl_8046E38C[{i}]", c, r)
        c, r = O(f"job{i}"); link(f"tuo_mo_gob{i}.hsd_obj", f"MO_gob{i}.hsd_obj", c, r, V)
        c, r = O(f"jch{i}"); link(f"tuo_mo_job{i}.child", f"MO_job{i}.child", c, r)
        c, r = O(f"dA{i}"); link(f"tuo_mo_jch{i}.u.dobj", f"MO_jch{i}.u.dobj", c, r)
        c, r = O(f"mA{i}"); link(f"tuo_mo_dA{i}.mobj", f"MO_dA{i}.mobj", c, r)
        c, r = O(f"tA{i}"); link(f"tuo_mo_mA{i}.tobj", f"MO_mA{i}.tobj", c, r)
        c, r = O(f"jbb{i}"); link(f"lbl_8046E39C[{i}]", f"m::lbl_8046E39C[{i}]", c, r)
        c, r = O(f"dB{i}"); link(f"tuo_mo_jbb{i}.u.dobj", f"MO_jbb{i}.u.dobj", c, r)
        c, r = O(f"dC{i}"); link(f"tuo_mo_dB{i}.next", f"MO_dB{i}.next", c, r)
        c, r = O(f"mC{i}"); link(f"tuo_mo_dC{i}.mobj", f"MO_dC{i}.mobj", c, r)
        c, r = O(f"tC{i}"); link(f"tuo_mo_mC{i}.tobj", f"MO_mC{i}.tobj", c, r)
    c, r = O("cobArg"); link("tuo_mo_gobArg.hsd_obj", "MO_gobArg.hsd_obj", c, r, V)
    c, r = O("fgt"); link("tuo_mo_gobE.user_data", "MO_gobE.user_data", c, r, V)
    c, r = O("scn"); link("lbl_8046DBE8.pnlsce", "m::ext::lbl_8046DBE8.pnlsce", c, r)
    c, r = O("jr0"); link("tuo_mo_jrR.child", "MO_jrR.child", c, r)
    for i in range(4):
        c, r = O(f"jr{i + 1}"); link(f"tuo_mo_jr{i}.next", f"MO_jr{i}.next", c, r)

    S = dict
    funcs = {
        "fn_80179990": S(args=["obj:gobArg", "0..3", "0..3"], cases=300),
        "fn_80179D3C": S(args=["obj:gobArg", "0..3"], cases=100), "fn_80179D60": S(args=["obj:gobArg", "0..3"], cases=100),
        "fn_80179D84": S(args=["obj:gobArg", "0..3"], cases=100), "fn_80179DA8": S(args=["obj:gobArg", "0..3"], cases=100),
        "fn_80179DCC": S(args=["obj:gobArg", "0..3"], cases=100), "fn_80179E34": S(args=["obj:gobArg", "0..3"], cases=100),
        "fn_80179E9C": S(args=["obj:gobArg", "0..3"], cases=100), "fn_80179F04": S(args=["obj:gobArg", "0..3"], cases=100),
        "fn_80179F6C": S(args=["0..3", "obj:gobArg"]),
        "fn_80179F84": S(args=["obj:jrR"]),
        "fn_80179854": S(cases=200),
        "fn_8017A078": S(args=["0..3"], cases=200),
        "fn_8017A318": S(args=["0..3"], cases=300),
        "fn_8017A67C": S(args=["0..40", "0..9", "0..3"], cases=400),
        "fn_8017A9B4": S(args=["0..3"], cases=100),
        "fn_8017AA78": S(args=["buf:4"], cases=200),
        "gm_1798_MexSetResultModels": S(args=["-1..5"], cases=100),
        "gmResult_TableKind": S(args=["0..40"]),
        "get_big_loser": S(args=["0..3", "buf"]),
        "fn_80179990_img_at": S(args=["buf:4", "0..3"]),
        "fn_80179990_set_erase_color": S(args=["buf", "0..3"], cases=100),
        "fn_80179990_copy_efb": S(args=["buf:4", "0..3", "buf", "buf"]),
        "fn_80179990_copy_efb_at": S(args=["buf:4", "0..3", "buf", "buf", "buf", "buf"]),
        "inline1": S(args=["buf:4", "0..3", "buf", "buf"]),
    }
    return dict(
        src="src/melee/gm/gm_1798.c", module="gm_1798", cases=60, cstr_nocls=True,
        local_types=["CharScaleEntry", "ResultsRenderFuncs", "ResultsPlayerConfig", "PackedS16x4", "U32Pair",
                     "lbl_8046E3AC_t", "ResultsDisplayData", "ResultsCharacterData"],
        extsize={"gmResultCharacterScaleData": 32},
        # the const config table is filled too: the C sees it through a non-const alias
        patch=[# fn_8017A318 falls off its end in the C (UB; the value is stored by gmresult.c): the port returns `gobj`
               ("        fn_8017A078(arg0);\n    }\n}\n\nFighter_GObj* fn_8017A67C",
                "        fn_8017A078(arg0);\n    }\n    return gobj;\n}\n\nFighter_GObj* fn_8017A67C"),
               ("#include <melee/ft/inlines.h>\n#endif\n",
                "#include <melee/ft/inlines.h>\n#endif\n#define lbl_803B7B68 (*(ResultsPlayerConfig*) &lbl_803B7B68_nc)\n"
                "extern ResultsPlayerConfig lbl_803B7B68_nc;\n")],
        rust_vars={"lbl_803B7B68_nc": "lbl_803B7B68"},
        mockobj={"Player_GetEntity": "gobE", "GObj_Create": "gobC"},
        extra_objs=extra,
        fixups=fx,
        funcs=funcs,
        nullable={"gm_80160438": 3, "fn_80174274": 0},
        # HSD_CObjLoadDesc takes a descriptor (a union bigger than the extern the C passes): neither filled
        # nor traced by content
        noout={"HSD_CObjLoadDesc": True}, no_arg_content=["HSD_CObjLoadDesc"],
        mock={"Mex_Enabled": "0|1", "fn_801795D4": "0..3", "fn_801796F0": "0..3", "HSD_CObjSetCurrent": "0|1",
              "ftLib_800876B4": "0|1", "gm_MexVanillaKind": "0..40", "HSD_Randi": "0..2"},
        fieldrange={"MatchPlayerData.pkind": "0..3", "MatchPlayerData.is_big_loser": "0..3",
                    "MatchPlayerData.team": "0..4", "MatchTeamData.is_big_loser": "0..3", "MatchEnd.is_teams": "0..1",
                    "MatchPlayerData.ftkind": "5..8", "MatchEnd.outcome": "5..8"})


CFGS["gm_1798"] = _gm_1798()


def _gm_17c0():
    """1P-mode controller TU. Wiring: Player_GetEntity -> a gobj whose user_data is a Fighter (boss hook);
    the opponent-id table lbl_803D79F0 holds the game's ids (ends with ChKind_None)."""
    S = dict
    tab = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 24, 25, 33]
    fixups = [
        ("tuo_mo_gobE.user_data = &tuo_mo_fgt;", "MO_gobE.user_data = core::ptr::addr_of_mut!(MO_fgt) as *mut core::ffi::c_void;"),
        ("{ static const unsigned char t[24] = {" + ", ".join(map(str, tab)) + "}; int q; for (q = 0; q < 24; q++) lbl_803D79F0[q] = t[q]; }",
         "{ let t: [u8; 24] = [" + ", ".join(map(str, tab)) + "]; m::ext::lbl_803D79F0 = t; }"),
        ("DbLevel = (DbLKind) tuo_draw_range(&tuo_st, 0, 4);", "m::ext::DbLevel = rt::draw_range(&mut rt::ST, 0, 4) as _;"),
    ]
    u1 = "buf"
    funcs = {
        "BossHook_Kind": S(args=["26..29"]), "BossHook_AllSleeping": S(cases=200), "BossHook_DriveDeaths": S(cases=300),
        "BossHook_Poll": S(cases=300), "BossHook_Hold": S(args=["0..9"]), "BossHook_Release": S(),
        "fn_8017C0C8": S(cases=100),
        "fn_8017C1A4": S(args=["obj:gobArg"], cases=800),
        "fn_8017C71C": S(cases=100), "fn_8017C7A0": S(), "fn_8017C7EC": S(force={"gm_GetCurrentGameMode": "3|4|5"}),
        "gm_8017C838": S(cases=300),
        "gm_8017C984": S(args=["obj:gmClassic_80470708"]),
        "gm_8017C9A8": S(args=["buf", "buf", "0..2"]),
        "gm_8017CA38": S(args=["buf", "buf", "buf", "0..30"], cases=300),
        "gm_8017CBAC": S(args=["buf", "buf", "20..24"], cases=300),
        "gm_8017CD94": S(args=["buf", "0..30", "0..30", "0..3"], cases=100),
        "gm_8017CE34_CountEnemies": S(args=["list:3:4|13|27|28|33|1|2|26"]),
        "gm_8017CE34_SetupColors": S(args=["buf", "0..30", "list:3:4|13|27|28|33|1|2|26", "buf:16"]),
        "gm_8017CE34_GetCpuLevel": S(args=["buf"]),
        "gm_8017CE34": S(args=["buf", "buf", "list:3:4|13|27|28|33|1|2|26|33", "0..3", "0..3", "0..3", "0|0|600|3600",
                               "0x40..0x4F", "0..30", "0..3"], cases=1200),
        "gm_8017D7AC": S(args=["buf", "buf", "0..30"], cases=400),
        "fn_8017D9C0": S(args=["list:4:0|1|2|3|5|33", "list:3:0|1|2|4|5|33"], cases=200),
        "pick_random_ckind": S(test=False),
        "gm_8017DB58": S(args=["buf:3"]),
        "gm_8017DB6C": S(args=["buf:3", "0..2"]), "gm_8017DB78": S(args=["buf:3", "0..2"]),
        "gm_8017DB88": S(args=["buf", "0..255", "0..30", "0..30", "list:3:0|1|2|4|5|33", "0..30"], cases=300),
        "fn_8017DD7C": S(args=["buf:6", "buf:3", "0..3"], cases=200),
        "fn_8017DE54": S(args=["0..255", "list:3:4|13|27|28|33|1|2|26"], cases=200),
        "fn_8017DEC8": S(args=["0..3"]), "fn_8017DF28": S(), "fn_8017DF90": S(), "gm_8017DFF4": S(args=["0..3"]),
        "gm_8017E068": S(), "fn_8017E0E4": S(), "fn_8017E160": S(cases=200),
        "fn_8017E21C": S(), "gm_8017E280": S(args=["0..5", "0..300"], cases=300), "fn_8017E318": S(), "fn_8017E3C8": S(),
    }
    return dict(
        src="src/melee/gm/gm_17C0.c", module="gm_17c0", cases=100, cstr_nocls=True,
        local_types=["lbl_804706D8_t", "lbl_804706C0_t", "RegClearCharEntry", "boss_hook_t"],
        anon_vars={"boss_hook": "boss_hook_t"},
        extsize={"lbl_803D79F0": 24},
        extra_objs={"fgt": "Fighter", "gobArg": "HSD_GObj"},
        mockobj={"Player_GetEntity": "gobE", "GObj_Create": "gobC"},
        fixups=fixups,
        # the mocks must not scribble over the wired gobjs they are handed
        noout={"ftCo_800C8C84": True, "ft_80087C58": True, "ft_80087C64": True, "HSD_GObj_SetupProc": True},
        fnptr_nonnull=["Unk1PData.x48", "Unk1PData.x64", "Unk1PData.x68"],
        funcs=funcs,
        mock={"gm_GetCurrentGameMode": "3|4|5|1", "Script_OnePScripted": "0|1", "Script_OnePComplete": "0|1",
              "Script_BossHookEnabled": "0|1", "Player_GetRemainingHP": "-1|0|1|50|100|150|200|300",
              "ftBossLib_8015C74C": "10|20|30", "ftBossLib_8015C7EC": "5|10", "ftBossLib_8015C88C": "5|10|15",
              "ftBossLib_8015C9CC": "10|20", "ftBossLib_IsMasterHandEntry": "0|1", "ftBossLib_8015C2A8": "0|1",
              "fn_8017EDDC": "0|1", "ftBossLib_8015C530": "100|300", "gm_80173224": "0|5|33|33", "gm_80173754": "0|1",
              "gm_80164430": "0|1", "gmMainLib_8015D1AC": "0|1|2|3", "gmMainLib_8015D2D4": "0|1|2|3",
              "gmMainLib_8015D3FC": "0|1|2|3", "gm_CKindToSelKind": "0..24", "gm_IsCKindUnlocked": "0|1",
              "gm_GetNumCostumesForCKind": "0..4", "HSD_Randi": "0..22", "Ground_801C1DAC": "0|1",
              "fn_8016B510": "0|1", "fn_8016B4BC": "0|1", "gm_GetCurrentSceneIndex": "0..60"},
        fieldrange={"Fighter.kind": "26..29", "Fighter.motion_id": "9..13",
                    "lbl_804706C0_t.x0": "0..9", "lbl_804706C0_t.x4": "0..3", "lbl_804706C0_t.x8": "0..40",
                    "lbl_804706C0_t.xC": "0..300", "lbl_804706C0_t.x10": "0..5", "lbl_804706C0_t.x14": "0..5",
                    "boss_hook_t.sent_mask": "0..63", "boss_hook_t.pending": "0..1", "boss_hook_t.held": "0..1",
                    "boss_hook_t.restored": "0..1", "boss_hook_t.limit": "0..5", "boss_hook_t.elapsed": "0..5",
                    "boss_hook_t.sleep_wait": "295..302", "boss_hook_t.driven_mask": "0..7",
                    "lbl_804706D8_t.x0": "-1..4", "StartMeleeRules.stkind": "0x3B..0x4F",
                    "gmPlayerData.ckind": "2..20", "gmPlayerData.color": "0..2", "gmPlayerData.cpu_level": "0..4",
                    "Unk1PData.x9": "0..3", "Unk1PData_x24.ckind": "30..34", "DebugGameOverData.xC": "0..1",
                    "MatchEnd.outcome": "0..9", "MatchPlayerData.ftkind": "5..8", "MatchPlayerData.stocks": "0..3",
                    "MatchEnd.frame_count": "0..7200"})


CFGS["gm_17c0"] = _gm_17c0()

# the trace functions of pad_falcon_fox_fd (docs/rust-port/trace) that were ported but not checked
SCRIPT_GAME_FUNCS = ['ScriptGame_ArmorRelease', 'ScriptGame_BuildClear', 'ScriptGame_CameraParamsRelease', 'ScriptGame_CapsEffectsRelease', 'ScriptGame_CapsMovementRelease', 'ScriptGame_CapsStatusRelease', 'ScriptGame_CritRelease', 'ScriptGame_EchoClear', 'ScriptGame_EchoReset', 'ScriptGame_FighterBenchRelease', 'ScriptGame_FighterI', 'ScriptGame_FighterModsRelease', 'ScriptGame_GameMode', 'ScriptGame_IntrWinRelease', 'ScriptGame_ItemCount', 'ScriptGame_LabSetPreviewRow', 'ScriptGame_OnePTemplatesClear', 'ScriptGame_ShockRelease', 'ScriptGame_SimRelease', 'ScriptGame_StageIsolationClear', 'ScriptGame_StageIsolationWatch', 'ScriptGame_StageMusicTick', 'ScriptGame_StageSlotCanCreate', 'ScriptGame_StageSlotCreated', 'ScriptGame_StageSlotProcBegin', 'ScriptGame_ZoneEventCount', 'ScriptGame_ZonesArm', 'ScriptGame_ZonesFrame', 'ScriptGame_ZonesRelease', 'script_camera_params_release', 'ScriptGame_CameraFov', 'ScriptGame_CpuModeInfo', 'ScriptGame_CpuModeReset', 'ScriptGame_FighterBenchTick', 'ScriptGame_HitRulePercentClear', 'ScriptGame_IntrWinTick', 'ScriptGame_LabPreviewRow', 'ScriptGame_MapWide', 'ScriptGame_NoteMotion', 'ScriptGame_OnePSpawn', 'ScriptGame_StageGameplayScene', 'ScriptGame_StageJoint', 'ScriptGame_StageJointActive', 'ScriptGame_StageJointCapacity', 'ScriptGame_StageLineActive', 'ScriptGame_StageMusicReset', 'ScriptGame_StageSlotDestroyed', 'ScriptGame_StageSlotJointUpdate', 'ScriptGame_StageSlotParticleBegin', 'ScriptGame_StageSlotProcEnd', 'ScriptGame_VirtualPadApply', 'ScriptGame_VirtualPadObserve', 'ScriptMods_GameEvent', 'ScriptParts_Draw', 'ScriptParts_Tint', 'script_background_render', 'script_crit_launch_clear', 'script_mesh_order_bucket', 'script_retail_forget_devices', 'script_stage_draw_init', 'script_stage_render', 'ScriptGame_AreaDrain', 'ScriptGame_ArmorFrame', 'ScriptGame_CapsEffectsFrame', 'ScriptGame_EchoCapture', 'ScriptGame_EchoFrame', 'ScriptGame_SkillFrame', 'ScriptGame_StageFrame', 'ScriptGame_StageKind', 'ScriptGame_StageSlotsFrame', 'ScriptGame_VirtualPadConsume', 'ScriptMods_StageFrame', 'script_caps_status_frame', 'script_crit_flush', 'script_hit_family_move', 'script_shock_frame', 'script_skill_one', 'ScriptGame_EchoCapsule', 'ScriptGame_EchoCapsuleCount', 'ScriptGame_EchoPrepare', 'ScriptParts_Forget', 'ScriptGame_NoteSpecialInput', 'ScriptGame_EchoBlocked', 'ScriptGame_EchoIsCapsule', 'ScriptGame_EchoSameGroup', 'ScriptMods_Skill', 'script_skill_emit', 'ScriptGame_ItemI', 'ScriptGame_EntityField', 'ScriptGame_FighterF', 'ScriptGame_HitI', 'ScriptGame_LabAnimEnd', 'ScriptGame_LabAnimSymbol', 'ScriptGame_LabF', 'ScriptGame_LabI', 'script_lab_mission_floor_valid', 'script_lab_mission_floor_height']
CFGS["script_game"] = dict(src="pc/gameworld/script_game.c", module="script_game", cases=40, all=False,
                          funcs={n: {} for n in SCRIPT_GAME_FUNCS},
                          # (functions whose C side faults on the synthetic state are dropped by tuoracle.py --build and reported)
                          noext=["memcpy", "memset", "memmove", "strcpy", "strlen", "strcmp", "strncpy", "strncmp", "strcat"],
                          # the owned-object lists hold object pointers a seeded fill leaves NULL; a random count walks past the 512 entries
                          fixups=[("script_slot_owned.count = 0; script_retail_owned.count = 0;",
                                   "(*core::ptr::addr_of_mut!(m::script_slot_owned)).count = 0; (*core::ptr::addr_of_mut!(m::script_retail_owned)).count = 0;"),
                                  # the skill ring is indexed by head % 256: a negative head would write before the ring (UB in C)
                                  ("script_skill_head = (int) ((unsigned) script_skill_head % 100000u);",
                                   "(*core::ptr::addr_of_mut!(m::script_skill_head)) = ((*core::ptr::addr_of_mut!(m::script_skill_head)) as u32 % 100000) as _;"),
                                  # size_t is 32 bits in the C compile (gc_size_t) and usize in the native port: keep the filled byte count in 32 bits
                                  ("{ int i; for (i = 0; i < SCRIPT_STAGE_ARCHIVES; i++) script_stage.archives[i].bytes = (unsigned) script_stage.archives[i].bytes; }",
                                   "for i in 0..8 { let a = &mut (*core::ptr::addr_of_mut!(m::script_stage)).archives[i]; a.bytes = (a.bytes as u32) as usize; }")])
