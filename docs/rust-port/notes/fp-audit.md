# Float-rounding audit (console vs Rust port)

Reference: retail DOL. Tools: `tools/rust-port/fp_audit/` (`scan_dol.py` -> DOL sites, `clang_sites.py` -> C front-end
fmuladd sites with square class, `compare.py` -> per-function match and per-site console prediction, `rust_sites.py` -> Rust index).
Re-run after upstream syncs:

    python tools/rust-port/fp_audit/scan_dol.py --csv dol.csv
    python tools/rust-port/fp_audit/clang_sites.py clang.csv
    python tools/rust-port/fp_audit/compare.py dol.csv clang.csv --out cmp.csv --sites sites.csv

## DOL site census

fused_s 2690, fused_d 993, frsp 1237, estimate 280, double 3847, f2i 1015, ps_arith 1486, ps_move 1294, i2f 1532.

## The fusion rule (measured)

MWCC (`-O4,p -fp_contract on`) does **not** fuse a square of a memory or computed expression (`p->x*p->x`,
`SQ(a-b)`, `VEC2_SQ_LEN(fp->...)`): 370 such squares are fmuls+fadds in the DOL, 5 fused. Squares of
variables, or fields of a register-resident local struct, fuse. Through an inline helper's pointer parameter
it is unfused unless the caller passes a register-resident local (lbVector_Len inside lbVector_8000E838;
it_802A3C98 inlined in the hookshot TU). With the rule, 757 functions' fused counts match the DOL, 71 do not.

The Rust port modelled every `a*b+c` as `fmadds` (single rounding of an f64 product+sum). Unfused squares now
use plain f32 `x*x + r`; helpers `sq_len2_mem`/`sq_len3_mem` (ftCo_Damage), `sq2_mem`/`sq3_mem` (lbvector),
`lbVector_Len_local`, `it_802A3C98_inl`, `hypot_mem`. Kept fused deliberately: ftCo_Ucf084Sdi (gecko asm is
fmadds), TARGET_PC-only mod code, CPU-AI variable squares.

## C PC build

Same mismatch, same fix (a shared console-semantics bug): `GW_NOFUSE(a * b)` / `GW_NOFUSE_SQ(x)` (src/Runtime/platform.h)
is an identity call under TARGET_PC, so clang cannot contract the product into an fmuladd; `(x)` for MWCC. Applied on the
133 DOL-unfused source lines (plus their continuation lines), `VEC2_SQ_LEN_NOFUSE`/`VEC3_SQ_LEN_NOFUSE` (mtx.h),
`VEC_XY_LENGTH_NOFUSE` (itlinkboomerang), `HYPOT_NOFUSE` (itnessyoyo). Fused contexts stay fused: `lbVector_Len_local`
inside lbVector_8000E838, `it_802A3C98_inl` for the hookshot TU's own callers (the out-of-line it_802A3C98 is unfused).
Check: rerun clang_sites.py; no f32 fmuladd with a sqmem/sqlocal/sqparam class remains on a DOL-unfused line.

`it_802A4BFC_sqrtf_offset`: the C's `*(&y + 6)` stack-slot trick is out of bounds on the PC (it returned x); the DOL runs
frsqrte + 3 Newton steps + frsp. Both builds now compute the MSL sqrtf.

## Result

slpcheck frame-exact over 125 replays: 86 before -> 102 after, on both the fast and the release profile (per-replay
results identical). The earlier release figure (53) came from a release exe in the shared `_build/cargo-shared` target
dir while other worktrees were building into it (clobbered crates); rebuilt in a private target dir, release equals fast.

## Open

- 71 functions whose fused count still differs (cross-inline fusion, e.g. ft_800CB6EC, ftColl_8007A06C,
  mpLib_80055E9C); listed below.
- f32 `.mul_add` uses (single rounding in f32) not yet swept to `fmadds`.
- PSMTXQuat / PSMTXRotAxisRad retail paired-single dataflow (C and Rust) not done.
- frsp / fctiw NaN / estimate sites not classified beyond inline sqrtf (already modelled).

## Unfused-square sites

Status "fixed/match" = the Rust function exists and its square is plain f32 after this branch;
"not reachable" = no Rust counterpart.

| address | function | source line | instruction | Rust location | status |
|---|---|---|---|---|---|
| 800077A0 | lbColl_800077A0 | src/melee/lb/lbcollision.c:1508 | fmuls+fadds (square of f32sqlocal{lbVector_AngleXY}) | melee_runtime/src/tu/lbcollision.rs:892 | fixed/match |
| 80008FC8 | lbColl_80008FC8 | src/melee/lb/lbcollision.c:2072 | fmuls+fadds (square of f32sqlocal{PSVECCrossProduct|PSVECNormalize}) | melee_runtime/src/tu/lbcollision.rs:1429 | fixed/match |
| 800096B4 | lbColl_800096B4 | src/melee/lb/lbcollision.c:2204 | fmuls+fadds (square of f32sqlocal{PSVECCrossProduct|PSVECNormalize}) | melee_fighter/src/tu/ftCo_0A01.rs:71 | fixed/match |
| 8000D148 | lb_8000D148 | src/melee/lb/lb_00CE.c:182 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/fighter.rs:357 | fixed/match |
| 8000D148 | lb_8000D148 | src/melee/lb/lb_00CE.c:185 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/fighter.rs:357 | fixed/match |
| 8000D2EC | lbVector_Normalize | src/melee/lb/lbvector.h:12 | fmuls+fadds (square of f32sqparam0) | melee_fighter/src/tu/ftCo_0A01.rs:74 | fixed/match |
| 8000D3B0 | lbVector_NormalizeXY | src/melee/lb/lbvector.c:35 | fmuls+fadds (square of f32sqparam0) | melee_items/src/tu/it_2E5A.rs:282 | fixed/match |
| 8000D620 | lbVector_Angle | src/melee/lb/lbvector.h:12 | fmuls+fadds (square of f32sqparam0) | melee_fighter/src/tu/ftCo_0A01.rs:73 | fixed/match |
| 8000D790 | lbVector_AngleXY | src/melee/lb/lbvector.c:15 | fmuls+fadds (square of f32sqparam0) | melee_items/src/tu/itbox.rs:155 | fixed/match |
| 8000D8F4 | lbVector_RotateAboutUnitAxis | src/melee/lb/lbvector.c:164 | fmuls+fadds (square of f32sqparam1) | melee_items/src/tu/itcoll.rs:287 | fixed/match |
| 8000DCA8 | lbVector_CosAngle | src/melee/lb/lbvector.c:270 | fmuls+fadds (square of f32sqparam0) | melee_items/src/tu/itlinkboomerang.rs:283 | fixed/match |
| 8000DCA8 | lbVector_CosAngle | src/melee/lb/lbvector.c:271 | fmuls+fadds (square of f32sqparam1) | melee_items/src/tu/itlinkboomerang.rs:283 | fixed/match |
| 8000F38C | lbShadow_8000F38C | src/melee/lb/lbshadow.c:403 | fmuls+fadds (square of f32sqlocal{lbVector_Diff}) | melee_items/src/tu/vi0102.rs:80 | fixed/match |
| 8000F38C | lbShadow_8000F38C | src/melee/lb/lbshadow.c:408 | fmuls+fadds (square of f32sqlocal{lbVector_Diff}) | melee_items/src/tu/vi0102.rs:80 | fixed/match |
| 8000F38C | lbShadow_8000F38C | src/melee/lb/lbshadow.c:422 | fmuls+fadds (square of f32sqlocal{lbVector_Diff}) | melee_items/src/tu/vi0102.rs:80 | fixed/match |
| 8001044C | lb_8001044C | src/melee/lb/lb_00F9.c:654 | fmuls+fadds (square of f32sqlocal{PSVECCrossProduct|lbVector_Angle}) | melee_items/src/tu/item.rs:280 | fixed/match |
| 8001044C | lb_8001044C | src/melee/lb/lb_00F9.c:655 | fmuls+fadds (square of f32sqlocal{PSVECCrossProduct|lbVector_Angle}) | melee_items/src/tu/item.rs:280 | fixed/match |
| 8002BAA8 | Camera_8002BAA8 | src/melee/cm/camera.c:241 | fmuls+fadds (square of f32sqparam0) | melee_world/src/tu/camera.rs:1890 | fixed/match |
| 8002BAA8 | Camera_8002BAA8 | src/melee/cm/camera.c:241 | fmuls+fadds (square of f32sqparam0cmpd) | melee_world/src/tu/camera.rs:1890 | fixed/match |
| 8002C1A8 | Camera_8002C1A8 | src/melee/lb/lbvector.h:12 | fmuls+fadds (square of f32sqparam0) | melee_world/src/tu/camera.rs:2061 | fixed/match |
| 8002C5B4 | Camera_8002C5B4 | src/melee/cm/camera.c:2247 | fmuls+fadds (square of f32sqparam0) | melee_world/src/tu/camera.rs:2174 | fixed/match |
| 8002C5B4 | Camera_8002C5B4 | src/melee/cm/camera.c:2248 | fmuls+fadds (square of f32sqparam0) | melee_world/src/tu/camera.rs:2174 | fixed/match |
| 8002C5B4 | Camera_8002C5B4 | src/melee/cm/camera.c:2344 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/camera.rs:2174 | fixed/match |
| 8002DDC4 | Camera_8002DDC4 | src/melee/lb/lbvector.h:12 | fmuls+fadds (square of f32sqparam0) | melee_world/src/tu/camera.rs:2742 | fixed/match |
| 8002F0E4 | Camera_8002F0E4 | src/melee/cm/camera.c:3585 | fmuls+fadds (square of f32sqlocal{lbVector_Diff}) | melee_chara/src/adapt.rs:331 | fixed/match |
| 8002F0E4 | Camera_8002F0E4 | src/melee/cm/camera.c:3588 | fmuls+fadds (square of f32sqlocal{lbVector_Diff}) | melee_chara/src/adapt.rs:331 | fixed/match |
| 8002F0E4 | Camera_8002F0E4 | src/melee/cm/camera.c:3588 | fmuls+fadds (square of f32sqlocal{lbVector_Diff}cmpd) | melee_chara/src/adapt.rs:331 | fixed/match |
| 8002FEEC | Camera_8002FEEC | src/melee/cm/camera.c:4054 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/dbcamera.rs:65 | fixed/match |
| 8002FEEC | Camera_8002FEEC | src/melee/cm/camera.c:4055 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/dbcamera.rs:65 | fixed/match |
| 8003F294 | fn_8003F294 | src/melee/pl/plbonuslib.c:924 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/plbonuslib.rs:930 | fixed/match |
| 8003F654 | fn_8003F654 | src/melee/pl/plbonuslib.c:1006 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/plbonuslib.rs:1032 | fixed/match |
| 8004DB78 | mpLineGetNext | src/melee/mp/mplib.c:1052 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:552 | fixed/match |
| 8004DC04 | mpLineGetPrev | src/melee/mp/mplib.c:1081 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:573 | fixed/match |
| 8004ED5C | mpLib_8004ED5C | src/melee/mp/mplib.c:1052 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:1053 | fixed/match |
| 8004ED5C | mpLib_8004ED5C | src/melee/mp/mplib.c:1081 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:1053 | fixed/match |
| 8004ED5C | mpLib_8004ED5C | src/melee/mp/mplib.c:1609 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:1053 | fixed/match |
| 8004ED5C | mpLib_8004ED5C | src/melee/mp/mplib.c:1619 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:1053 | fixed/match |
| 8004F008 | mpCheckFloor | src/melee/mp/mplib.c:1710 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/fighter.rs:498 | fixed/match |
| 8004F008 | mpCheckFloor | src/melee/mp/mplib.c:1737 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/fighter.rs:498 | fixed/match |
| 8004F8A4 | mpCheckCeiling | src/melee/mp/mplib.c:2012 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_0A01.rs:76 | fixed/match |
| 8004F8A4 | mpCheckCeiling | src/melee/mp/mplib.c:2044 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_0A01.rs:76 | fixed/match |
| 800511A4 | mpLib_800511A4_RightWall | src/melee/mp/mplib.c:3014 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:2245 | fixed/match |
| 800511A4 | mpLib_800511A4_RightWall | src/melee/mp/mplib.c:3046 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:2245 | fixed/match |
| 800515A0 | mpLib_800515A0_LeftWall | src/melee/mp/mplib.c:3157 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:2370 | fixed/match |
| 800515A0 | mpLib_800515A0_LeftWall | src/melee/mp/mplib.c:3190 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/mplib.rs:2370 | fixed/match |
| 80056C54 | mpLib_80056C54 | src/melee/mp/mplib.c:1052 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftdonkey.rs:90 | fixed/match |
| 80056C54 | mpLib_80056C54 | src/melee/mp/mplib.c:1081 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftdonkey.rs:90 | fixed/match |
| 80087B34 | ft_80087B34 | src/melee/ft/ft_0877.c:198 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ft_0877.rs:123 | fixed/match |
| 80087B34 | ft_80087B34 | src/melee/ft/ft_0877.c:203 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ft_0877.rs:123 | fixed/match |
| 80087B34 | ft_80087B34 | src/melee/ft/ft_0877.c:215 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ft_0877.rs:123 | fixed/match |
| 8008E4F0 | ftCo_Damage_OnEveryHitlag | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:544 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:576 | fixed/match |
| 8008E4F0 | ftCo_Damage_OnEveryHitlag | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:559 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:576 | fixed/match |
| 8008E714 | ftCo_Damage_OnExitHitlag | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:618 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:623 | fixed/match |
| 8008FB64 | ftCo_Damage_Coll | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:1051 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:1131 | fixed/match |
| 8008FCB8 | ftCo_Damage_SetMv8FromKbThreshold | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:1071 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:1158 | fixed/match |
| 8008FF84 | ftCo_DamageFly_Phys | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:1169 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:1242 | fixed/match |
| 8009035C | ftCo_DamageFlyRoll_Phys | src/melee/ft/kinds/ftCommon/ftCo_Damage.c:1233 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Damage.rs:1312 | fixed/match |
| 80091620 | ftCo_DamageIce_Collide | src/melee/ft/kinds/ftCommon/ftCo_DamageIce.c:409 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_DamageIce.rs:401 | fixed/match |
| 80091620 | ftCo_DamageIce_Collide | src/melee/ft/kinds/ftCommon/ftCo_DamageIce.c:410 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_DamageIce.rs:401 | fixed/match |
| 80091BC4 | ftCo_80091BC4 | src/melee/ft/kinds/ftCommon/ftCo_Guard.c:187 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_Guard.rs:239 | fixed/match |
| 800BD19C | ftCo_800BD19C | src/melee/ft/kinds/ftCommon/ftCo_CaptureKirby.c:17 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_CaptureKirby.rs:73 | fixed/match |
| 800BDDCC | ftCo_ThrownKirbyStar_Phys | src/melee/ft/kinds/ftCommon/ftCo_ThrownKirby.c:168 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_ThrownKirby.rs:258 | fixed/match |
| 800BE260 | ftCo_ThrownCopyStar_Phys | src/melee/ft/kinds/ftCommon/ftCo_ThrownKirby.c:168 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ftCo_ThrownKirby.rs:280 | fixed/match |
| 800DF608 | ftCo_800DF608 | src/melee/ft/ft_0DF1.c:111 | fmuls+fadds (square of f32sqmem) | melee_fighter/src/tu/ft_0df1.rs:58 | fixed/match |
| 80110490 | ftSk_SpecialS_80110490 | src/melee/ft/kinds/ftSeak/ftseakspecials.c:72 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftseakspecials.rs:280 | fixed/match |
| 80119254 | ftNs_SpecialAirHi_Phys | src/melee/lb/lbvector.h:17 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftnessspecialhi.rs:748 | fixed/match |
| 80126C0C | ftPk_SpecialHi_80126C0C | src/melee/ft/kinds/ftPikachu/ftpikachuspecialhi.c:472 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftpikachuspecialhi.rs:465 | fixed/match |
| 80126E1C | ftPk_SpecialHi_80126E1C | src/melee/ft/kinds/ftPikachu/ftpikachuspecialhi.c:548 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftpikachuspecialhi.rs:519 | fixed/match |
| 80127064 | ftPk_SpecialHi_80127064 | src/melee/ft/kinds/ftPikachu/ftpikachuspecialhi.c:634 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftpikachuspecialhi.rs:585 | fixed/match |
| 80151F00 | ftMh_WalkLoop_Anim | src/melee/ft/kinds/ftMasterHand/ftmasterhandwalk.c:101 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftmasterhandwalk.rs:95 | fixed/match |
| 80152A50 | ftMh_Poke1_Phys | src/melee/ft/kinds/ftMasterHand/ftmasterhandpoke.c:94 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftmasterhandpoke.rs:100 | fixed/match |
| 80153254 | ftMh_FingerGun1_Phys | src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun.c:120 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftmasterhandfingergun.rs:119 | fixed/match |
| 8015442C | ftMh_Wait1_1_Phys | src/melee/ft/kinds/ftMasterHand/ftmasterhandbackdisappear.c:102 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftmasterhandbackdisappear.rs:160 | fixed/match |
| 80157FC8 | ftCh_RockCrushUp_Phys | src/melee/ft/kinds/ftCrazyHand/ftcrazyhandrockcrush0.c:40 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftcrazyhandrockcrush0.rs:88 | fixed/match |
| 80158C88 | ftCh_FingerBeamLoop_Phys | src/melee/ft/kinds/ftCrazyHand/ftcrazyhandfingerbeam.c:45 | fmuls+fadds (square of f32sqparam0) | melee_chara/src/tu/ftcrazyhandfingerbeam.rs:184 | fixed/match |
| 8015BE40 | ftBossLib_8015BE40 | src/melee/ft/ftbosslib.c:54 | fmuls+fadds (square of f32sqparam0) | melee_fighter/src/tu/ftbosslib.rs:100 | fixed/match |
| 801C4FAC | Ground_801C4FAC | src/melee/gr/ground.c:3915 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector}) | melee_world/src/tu/camera.rs:56 | fixed/match |
| 801D8134 | fn_801D8134 | src/melee/gr/grkongo.c:1405 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/grkongo.rs:1405 | fixed/match |
| 801D8134 | fn_801D8134 | src/melee/gr/grkongo.c:1406 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/grkongo.rs:1405 | fixed/match |
| 801D99E0 | grZebes_801D99E0 | src/melee/gr/grzebes.c:920 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/grzebes.rs:1438 | fixed/match |
| 801DBB60 | grZebes_801DBB60 | src/melee/gr/grzebes.c:2021 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/grzebes.rs:2425 | fixed/match |
| 801F1A34 | grMuteCity_801F1A34 | src/melee/gr/grmutecity.c:1616 | fmuls+fadds (square of f32sqlocal{HSD_JObjSetTranslate|grLib_801C96F8}) | melee_world/src/tu/grmutecity.rs:1220 | fixed/match |
| 801F1A34 | grMuteCity_801F1A34 | src/melee/gr/grmutecity.c:1617 | fmuls+fadds (square of f32sqlocal{HSD_JObjSetTranslate|grLib_801C96F8}) | melee_world/src/tu/grmutecity.rs:1220 | fixed/match |
| 80207C88 | grKinokoRoute_80207C88 | src/melee/gr/grkinokoroute.c:483 | fmuls+fadds (square of f32sqlocal{lbVector_Diff|lbVector_Normalize}) | melee_world/src/tu/grkinokoroute.rs:541 | fixed/match |
| 8020A21C | grShrineRoute_8020A21C | src/melee/gr/grshrineroute.c:1038 | fmuls+fadds (square of f32sqlocal{HSD_LObjSetPosition|lbVector_Add|lbVector_Normalize}) | melee_world/src/tu/grshrineroute.rs:728 | fixed/match |
| 80210454 | grOldKongo_80210454 | src/melee/gr/groldkongo.c:504 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/groldkongo.rs:624 | fixed/match |
| 80210454 | grOldKongo_80210454 | src/melee/gr/groldkongo.c:505 | fmuls+fadds (square of f32sqmem) | melee_world/src/tu/groldkongo.rs:624 | fixed/match |
| 80218888 | grPushOn_80218888 | src/melee/gr/grpushon.c:428 | fmuls+fadds (square of f32sqlocal{HSD_LObjSetPosition|lbVector_Add|lbVector_Normalize}) | melee_world/src/tu/grpushon.rs:322 | fixed/match |
| 8026B1D4 | it_8026B1D4 | src/melee/it/it_26B1.c:68 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/it_26B1.rs:297 | fixed/match |
| 8026B1D4 | it_8026B1D4 | src/melee/it/it_26B1.c:69 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/it_26B1.rs:297 | fixed/match |
| 8027737C | it_8027737C | src/melee/it/itmaplib.c:759 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/item.rs:262 | fixed/match |
| 8027737C | it_8027737C | src/melee/it/itmaplib.c:767 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/item.rs:262 | fixed/match |
| 80277544 | it_80277544 | src/melee/it/itmaplib.c:790 | fmuls+fadds (square of f32sqlocal{PSVECAdd}) | melee_items/src/tu/itgroundcoll.rs:51 | fixed/match |
| 802775F0 | it_802775F0 | src/melee/it/itmaplib.c:823 | fmuls+fadds (square of f32sqparam1) | melee_items/src/tu/itmaplib.rs:1051 | fixed/match |
| 802915E8 | itFlipper_DmgDealt | src/melee/lb/lbvector.h:17 | fmuls+fadds (square of f32sqparam0) | melee_items/src/tu/itflipper.rs:584 | fixed/match |
| 8029B8EC | itMariofireball_UnkMotion0_Coll | src/melee/it/kinds/itmariofireball.c:69 | fmuls+fadds (square of f32sqparam0) | melee_items/src/tu/itmariofireball.rs:148 | fixed/match |
| 802A0FD0 | itLinkboomerang_UnkMotion1_Phys | src/melee/it/kinds/itlinkboomerang.c:471 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkboomerang.rs:680 | fixed/match |
| 802A12FC | itLinkboomerang_UnkMotion2_Phys | src/melee/it/kinds/itlinkboomerang.c:529 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkboomerang.rs:725 | fixed/match |
| 802A1D60 | itLinkboomerang_UnkMotion3_Phys | src/melee/it/kinds/itlinkboomerang.c:787 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkboomerang.rs:948 | fixed/match |
| 802A20E8 | it_802A20E8 | src/melee/it/kinds/itlinkboomerang.c:874 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkboomerang.rs:993 | fixed/match |
| 802A3C98 | it_802A3C98 | src/melee/it/kinds/itlinkhookshot.c:875 | fmuls+fadds (square of f32sqparam2) | melee_items/src/tu/itclimbersstring.rs:133 | fixed/match |
| 802B51A0 | itSamusbomb_UnkMotion2_Phys | src/melee/it/kinds/itsamusbomb.c:177 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itsamusbomb.rs:333 | fixed/match |
| 802B67E4 | itSamusmissile_UnkMotion0_Phys | src/melee/it/kinds/itsamusmissile.c:248 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itsamusmissile.rs:317 | fixed/match |
| 802BFEC4 | it_802BFEC4 | src/melee/it/kinds/itnessyoyo.c:770 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftnessattackhi4.rs:119 | fixed/match |
| 802C0368 | itLuigifireball_UnkMotion0_Coll | src/melee/it/kinds/itluigifireball.c:73 | fmuls+fadds (square of f32sqparam0) | melee_items/src/tu/itluigifireball.rs:142 | fixed/match |
| 802C0CC4 | itDrmariopill_UnkMotion0_Coll | src/melee/it/kinds/itdrmariopill.c:235 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itdrmariopill.rs:281 | fixed/match |
| 802D472C | it_802D472C | src/melee/it/kinds/ithitodeman.c:206 | fmuls+fadds (square of f32sqlocal{lbVector_Diff|lbVector_Normalize}) | melee_items/src/tu/ithitodeman.rs:549 | fixed/match |
| 802E5AC4 | it_802E5AC4 | src/melee/it/kinds/it_2E5A.c:114 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/it_2E5A.rs:340 | fixed/match |
| 802E5AC4 | it_802E5AC4 | src/melee/it/kinds/it_2E5A.c:128 | fmuls+fadds (square of f32sqlocal{lbVector_Add_xy|lbVector_NormalizeXY}) | melee_items/src/tu/it_2E5A.rs:340 | fixed/match |
| 802F063C | it_802F063C | src/melee/it/kinds/itmasterhandlaser.c:180 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itmasterhandlaser.rs:252 | fixed/match |
| 802F063C | it_802F063C | src/melee/it/kinds/itmasterhandlaser.c:181 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itmasterhandlaser.rs:252 | fixed/match |
| 802F063C | it_802F063C | src/melee/it/kinds/itmasterhandlaser.c:184 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itmasterhandlaser.rs:252 | fixed/match |
| 802F063C | it_802F063C | src/melee/it/kinds/itmasterhandlaser.c:187 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itmasterhandlaser.rs:252 | fixed/match |
| 802F23AC | it_802F23AC | src/melee/it/kinds/itkirby_2F23.c:22 | fmuls+fadds (square of f32sqmem) | melee_chara/src/tu/ftkirbyspecialn.rs:116 | fixed/match |
| 802F2AE8 | it_2F28_UnkMotion0_Phys | src/melee/it/kinds/it_2F28.c:104 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/it_2F28.rs:174 | fixed/match |
| 80318CB4 | _tyDisplay_80318CB4 | src/melee/ty/tydisplay.c:832 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/tydisplay.rs:575 | fixed/match |
| 80368BC0 | roll2upvec | src/sysdolphin/baselib/cobj.c:704 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector|PSMTXRotAxisRad|vec_get_abs_y|vec_get_x}) | melee_runtime/src/tu/cobj_world.rs:241 | fixed/match |
| 80368BC0 | roll2upvec | src/sysdolphin/baselib/cobj.c:708 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector|PSMTXRotAxisRad|vec_get_abs_y|vec_get_x}) | melee_runtime/src/tu/cobj_world.rs:241 | fixed/match |
| 80368E70 | HSD_CObjGetUpVector | src/sysdolphin/baselib/cobj.c:704 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector|PSMTXRotAxisRad|vec_get_abs_y|vec_get_x}) | melee_items/src/tu/toy.rs:97 | fixed/match |
| 80368E70 | HSD_CObjGetUpVector | src/sysdolphin/baselib/cobj.c:708 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector|PSMTXRotAxisRad|vec_get_abs_y|vec_get_x}) | melee_items/src/tu/toy.rs:97 | fixed/match |
| 803699C0 | HSD_CObjSetRoll | src/sysdolphin/baselib/cobj.c:704 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector|PSMTXRotAxisRad|vec_get_abs_y|vec_get_x}) | melee_modes/src/tu/mngallery.rs:49 | fixed/match |
| 803699C0 | HSD_CObjSetRoll | src/sysdolphin/baselib/cobj.c:708 | fmuls+fadds (square of f32sqlocal{HSD_CObjGetEyeVector|PSMTXRotAxisRad|vec_get_abs_y|vec_get_x}) | melee_modes/src/tu/mngallery.rs:49 | fixed/match |
| 80373964 | mkVBillBoardMtx | src/sysdolphin/baselib/mtx.h:60 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/displayfunc.rs:205 | fixed/match |
| 80373B90 | mkHBillBoardMtx | src/sysdolphin/baselib/mtx.h:60 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/displayfunc.rs:234 | fixed/match |
| 80373E44 | mkBillBoardMtx | src/sysdolphin/baselib/mtx.h:60 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/displayfunc.rs:269 | fixed/match |
| 80379C24 | HSD_MtxGetRotation | src/sysdolphin/baselib/mtx.c:225 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkarrow.rs:98 | fixed/match |
| 80379C24 | HSD_MtxGetRotation | src/sysdolphin/baselib/mtx.c:228 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkarrow.rs:98 | fixed/match |
| 80379C24 | HSD_MtxGetRotation | src/sysdolphin/baselib/mtx.c:230 | fmuls+fadds (square of f32sqmem) | melee_items/src/tu/itlinkarrow.rs:98 | fixed/match |
| 8037B7B0 | resolveCnsOrientation | src/sysdolphin/baselib/mtx.h:60 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/robj.rs:339 | fixed/match |
| 8037E708 | MatToQuat | src/sysdolphin/baselib/quatlib.c:18 | fmuls+fadds (square of f32sqmem) | melee_modes/src/tu/lb_020A.rs:43 | fixed/match |
| 8037E708 | MatToQuat | src/sysdolphin/baselib/quatlib.c:20 | fmuls+fadds (square of f32sqmem) | melee_modes/src/tu/lb_020A.rs:43 | fixed/match |
| 8037E708 | MatToQuat | src/sysdolphin/baselib/quatlib.c:22 | fmuls+fadds (square of f32sqmem) | melee_modes/src/tu/lb_020A.rs:43 | fixed/match |
| 8037EB28 | MatToEuler | src/sysdolphin/baselib/quatlib.c:63 | fmuls+fadds (square of f32sqmem) | - | not reachable |
| 8037ECE0 | AngleAxisToQuat | src/sysdolphin/baselib/quatlib.c:105 | fmuls+fadds (square of f32sqparam0) | - | not reachable |
| 8039930C | hsd_8039930C | src/sysdolphin/baselib/particle.c:1578 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/particle.rs:1685 | fixed/match |
| 8039930C | hsd_8039930C | src/sysdolphin/baselib/particle.c:1994 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/particle.rs:1685 | fixed/match |
| 8039DAD4 | hsd_8039DAD4 | src/sysdolphin/baselib/generator.c:386 | fmuls+fadds (square of f32sqlocal{PSMTXMultVec}) | melee_runtime/src/tu/generator.rs:359 | fixed/match |
| 8039DAD4 | hsd_8039DAD4 | src/sysdolphin/baselib/generator.c:387 | fmuls+fadds (square of f32sqlocal{PSMTXMultVec}) | melee_runtime/src/tu/generator.rs:359 | fixed/match |
| 8039DAD4 | hsd_8039DAD4 | src/sysdolphin/baselib/generator.c:850 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/generator.rs:359 | fixed/match |
| 8039DAD4 | hsd_8039DAD4 | src/sysdolphin/baselib/generator.c:851 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/generator.rs:359 | fixed/match |
| 8039F05C | hsd_8039F05C | src/sysdolphin/baselib/generator.c:1181 | fmuls+fadds (square of f32sqmem) | melee_modes/src/tu/gmstaffroll.rs:110 | fixed/match |
| 8039F05C | hsd_8039F05C | src/sysdolphin/baselib/generator.c:1182 | fmuls+fadds (square of f32sqmem) | melee_modes/src/tu/gmstaffroll.rs:110 | fixed/match |
| 8039F05C | hsd_8039F05C | src/sysdolphin/baselib/generator.c:1184 | fmuls+fadds (square of f32sqmem) | melee_modes/src/tu/gmstaffroll.rs:110 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1282 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1283 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1287 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1288 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1453 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1454 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1458 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |
| 803A0088 | psDispParticles | src/sysdolphin/baselib/psdisp.c:1459 | fmuls+fadds (square of f32sqmem) | melee_runtime/src/tu/psdisp.rs:3082 | fixed/match |

## Mismatching functions

| address | function | DOL fused | predicted | status |
|---|---|---|---|---|
| 80391664 | DrawASCII | 9 | 10 | open |
| 801C4FAC | Ground_801C4FAC | 10 | 9 | open |
| 8038D5B4 | HSD_AudioSFXSetMix | 12 | 24 | open |
| 80376E90 | HSD_PadClampCheck3 | 6 | 8 | open |
| 803A4A68 | HSD_SObjLib_803A4A68 | 2 | 0 | open |
| 803A84BC | HSD_SisLib_803A84BC | 19 | 20 | open |
| 8037EF28 | QuatSlerp | 15 | 16 | open |
| 8035F0EC | TObjSetupMtx | 5 | 3 | open |
| 8030B530 | _Toy_8030B530 | 8 | 7 | open |
| 80319EF0 | _tyDisplay_80319EF0 | 9 | 1 | open |
| 8036D7E4 | drawShapeAnim | 30 | 14 | open |
| 801857C4 | fn_801857C4 | 3 | 4 | open |
| 80192BB0 | fn_80192BB0 | 1 | 3 | open |
| 800A229C | ftCo_800A229C | 7 | 15 | open |
| 800ABBA8 | ftCo_800ABBA8 | 13 | 15 | open |
| 800AF290 | ftCo_800AF290 | 6 | 3 | open |
| 800AF78C | ftCo_800AF78C | 5 | 2 | open |
| 800C0D0C | ftCo_800C0D0C | 2 | 3 | open |
| 800C2600 | ftCo_800C2600 | 26 | 28 | open |
| 800C318C | ftCo_800C318C | 2 | 3 | open |
| 800C4550 | ftCo_800C4550 | 2 | 3 | open |
| 800CF6E8 | ftCo_800CF6E8 | 37 | 0 | open |
| 800D0CBC | ftCo_800D0CBC | 1 | 0 | open |
| 800D0EC8 | ftCo_800D0EC8 | 1 | 0 | open |
| 800CF594 | ftCo_CalcYScaledKnockback | 2 | 1 | open |
| 800D65D8 | ftCo_SquatRv_CheckInput | 0 | 2 | open |
| 8007A06C | ftColl_8007A06C | 14 | 2 | open |
| 80119254 | ftNs_SpecialAirHi_Phys | 4 | 1 | open |
| 800CC3C4 | ftYs_JumpAerial_Anim_Cb | 1 | 0 | open |
| 800CBE98 | ftYs_JumpAerial_Enter | 1 | 0 | open |
| 80087B34 | ft_80087B34 | 0 | 2 | open |
| 800CB6EC | ft_800CB6EC | 1 | 0 | open |
| 801EA05C | grBigBlue_801EA05C | 4 | 0 | open |
| 801EACE8 | grBigBlue_801EACE8 | 2 | 0 | open |
| 801ED694 | grBigBlue_801ED694 | 14 | 15 | open |
| 801DFC98 | grCorneria_801DFC98 | 2 | 0 | open |
| 801DFF20 | grCorneria_801DFF20 | 2 | 0 | open |
| 801E01A8 | grCorneria_801E01A8 | 2 | 0 | open |
| 801E03C8 | grCorneria_801E03C8 | 3 | 0 | open |
| 801E0678 | grCorneria_801E0678 | 2 | 0 | open |
| 801E08CC | grCorneria_801E08CC | 6 | 0 | open |
| 801E0A74 | grCorneria_801E0A74 | 6 | 0 | open |
| 801E1060 | grCorneria_801E1060 | 4 | 0 | open |
| 8021483C | grGreens_8021483C | 8 | 4 | open |
| 8021D680 | grHomeRun_8021D680 | 7 | 9 | open |
| 8021DF50 | grHomeRun_8021DF50 | 1 | 3 | open |
| 8021E0D4 | grHomeRun_8021E0D4 | 1 | 3 | open |
| 8021E258 | grHomeRun_8021E258 | 1 | 3 | open |
| 8021E3DC | grHomeRun_8021E3DC | 1 | 3 | open |
| 801D577C | grKongo_801D577C | 1 | 3 | open |
| 801F1A34 | grMuteCity_801F1A34 | 11 | 10 | open |
| 801DC744 | grZebes_801DC744 | 11 | 19 | open |
| 803B3408 | hsd_803B3408 | 8 | 6 | open |
| 8027781C | it_8027781C | 12 | 14 | open |
| 8027C8D0 | it_8027C8D0 | 5 | 7 | open |
| 802CED54 | it_802CED54 | 2 | 0 | open |
| 802D208C | it_802D208C | 7 | 5 | open |
| 8000E9F0 | lbShadow_8000E9F0 | 36 | 37 | open |
| 800122F0 | lb_800122F0 | 2 | 3 | open |
| 802640A0 | mnCharSel_802640A0 | 1 | 3 | open |
| 802602A0 | mnCharSel_CursorThink | 10 | 12 | open |
| 80243038 | mnDiagram_CursorProc | 1 | 2 | open |
| 80252ADC | mnInfoBonus_80252ADC | 0 | 3 | open |
| 802351A0 | mnItemSw_802351A0 | 4 | 2 | open |
| 80236CBC | mnStageSw_80236CBC | 4 | 2 | open |
| 80055E9C | mpLib_80055E9C | 22 | 6 | open |
| 803A0088 | psDispParticles | 178 | 180 | open |
| 80378A94 | splGetSplinePoint | 47 | 48 | open |
| 8000D5BC | sqrtf__Ff | 3 | 0 | open |
| 80224990 | stageGObj1_GObjProc | 3 | 0 | open |
| 80224900 | stageGObj2_GObjProc | 2 | 0 | open |
