#!/usr/bin/env python3
"""Write the per-record table of docs/rust-port/notes/layout-audit.md from the three checker outputs.

  python tools/rust-port/layout/make_table.py LA.json WALK.txt IDENTICAL.txt > table.md

LA.json: c_layout_check.py --json; WALK.txt: run_walk.sh output (VISITED lines; field values are not copied);
IDENTICAL.txt: layout_walk --identical. Triage notes for the part-A differences are in NOTES below.
"""
import json, sys

# Part A differences, triaged by reading the C and the Rust users (layout-audit.md explains the classes)
NOTES = {
    "Allocator": "native-only (lbmemory arena record; Rust keeps native pointers where C keeps u32 addresses)",
    "Camera": "upstream retyped a union member (x368 orbit); runtime struct, same bytes",
    "CardBlockHeader": "flexible array member (checker cannot size it); same layout",
    "CommandInfo": "TARGET_PC keeps both union spellings (upstream named it x8); same bytes",
    "Command_05": "STREAM_WORD_TYPES pin: jump target kept as the big-endian u32 word of the raw image (by design)",
    "Command_07": "STREAM_WORD_TYPES pin (as Command_05)",
    "CpuFighter": "upstream retyped u32 -> pointer; runtime struct",
    "Element_803F9628": "signedness of a static table field retyped upstream (s32 vs u32); values < 2^31",
    "FigaTrack": "upstream named the pad byte dummy0 (set in the data, unused); Rust keeps it as padding, unread",
    "Fighter": "upstream renames/retypes (x5A4/x5A8 u32 vs ptr, CommandInfo union); runtime struct, hash fields unchanged",
    "Fighter_MotionVars": "MasterHand pads split differently upstream; same size",
    "Ground": "grCastle vars retyped upstream (pointer vs u32 at +0xC4); runtime union",
    "GroundVars": "as Ground",
    "HSD_AObjDesc": "obj_id is a POINTER_KEY_FIELDS usize (the native joint the key names); disc walk checks it",
    "HSD_SynthVoiceEntry": "checker artifact: the record is extracted under a renamed tag (synth.c `foo`)",
    "IfStockUserData": "upstream retyped the HUD stock records (ifstock.c); runtime only",
    "Item": "upstream retypes (hold_kind enum u32 vs enum_t, xC78 vec int/float); runtime struct",
    "ItemAttr": "x8 f32 (C) vs i32 (Rust): no C or Rust reader; disc bits compared equal",
    "Item_ItemVars": "Fox blaster vars retyped upstream (pointer vs s32); runtime",
    "KinokoAttrs": "upstream: x8 is HSD_AnimJoint*[2]; Rust reads them through KinokoAnim (dat_custom it_80293660 path)",
    "MatchEnd": "upstream: player_standings kills is u16[6] (Rust u16[4] + x18 + x1A); same bytes; native layout identical",
    "MatchExitInfo": "as MatchEnd", "ResultsMatchInfo": "as MatchEnd", "MatchPlayerData": "as MatchEnd",
    "VsSceneController": "as MatchEnd", "VsSceneState": "as MatchEnd",
    "SpawnItem": "hold_kind enum signedness (u32 vs enum_t)",
    "StaleMoveTable": "upstream: xCBC is a union {s32; bitfields}; Rust keeps the bytes; native layout identical, hash unchanged",
    "_StaticPlayer": "as StaleMoveTable", "pl_StaleMoveTableExt_t": "as StaleMoveTable", "dmg": "as StaleMoveTable + x1894 retyped",
    "Struct2074": "upstream retyped int/float vectors; runtime fighter state, same bytes",
    "THPFrameBuffer": "flexible array member (checker cannot size it)",
    "Toy6E68": "NATIVE_FIELD_TYPES pin (native pointers in a pad)", "TyDisplayData": "NATIVE_FIELD_TYPES pin",
    "TyFiguponData": "NATIVE_FIELD_TYPES pin", "TrainingModeState": "upstream grew char_data 25 -> 27 into the pad; same size",
    "_ftDrMarioAttributes": "cape_kind ItemKind (u32) vs u32/s32 spelling; disc values compared equal",
    "_ftPurinAttributes": "upstream: xE8/xEC are floats; Rust has pointers there. Typed by extattr; disc walk equal (pointer slots null both sides)",
    "_ftSamusAttributes": "upstream: xD0 is s32; Rust pointer. ext_attr decoded typed (extattr); disc walk equal",
    "ftCommonData": "upstream: sub_colors[5] where Rust has x6DC[4] + x6EC; same bytes (PlCo walk: slot 0)",
    "ftCommon_MotionVars": "runtime union, pointer vs u32", "ftYoshi_GuardVars": "runtime, pointer vs u32", "ftYoshi_MotionVars": "runtime",
    "ftMasterHand_Damage_0": "pads split differently upstream; same size", "ftMasterHand_FingerBeamVars": "as Damage_0",
    "ftMasterHand_GrabVars": "as Damage_0", "ftMasterHand_MotionVars": "as Damage_0",
    "ft_80459B88_t": "upstream: copies[k+1] == Rust hats[k] (trap 2 of upstream-sync.md); same bytes",
    "grBigBlueRoute_GroundVars2": "runtime ground vars retyped upstream", "grCastle_GroundVars3": "runtime ground vars retyped upstream",
    "grZebes_GroundVars": "upstream shrank it to 0xC (stored_jobj); runtime union member, Rust keeps 0x18 (inside the union)",
    "ifMagnify": "upstream: GXColor records where Rust has s32 words; runtime HUD state",
    "ifStock_804A1378": "upstream retyped the stock HUD table (0x204 + per-player records); runtime only",
    "itFoodsAttributes": "flexible entries[] upstream; Rust reads the rows through FoodsRow (dat_custom)",
    "itFoxBlaster_ItemVars": "runtime item vars retyped (pointers)",
    "itGamewatchchefAttributes": "entries[5] upstream, Rust entries[1] indexed past (as the old C); word-swapped block, all 4-byte words",
    "itLeadeadAttributes": "upstream named the last pad byte x1F; Rust pad", "itSScopeAttributes": "upstream named x0; Rust pad, unread",
    "itTincleAttributes": "x0 is itSpecialAttrsHead* upstream, f32 in Rust; no reader of x0 (ittincle.c)",
    "itToolsAttributes": "flexible motions[] upstream (Rust motions[1] indexed past, ittools.rs)",
    "itUnkAttributes": "flexible array upstream", "itWhispyAppleAttributes": "x0 pointer upstream, pad in Rust; unread",
    "itWstarAttributes": "flexible x28_entries[] upstream (itwstar.rs indexes past)",
    "lbHeap_HeapOffsetView": "removed upstream; Rust-only view", "mnDiagram_AnimTable": "upstream retyped mndiagram (pc hacks dropped); runtime menu",
    "mnDiagram_Assets": "as mnDiagram_AnimTable", "mn_803ED1D0_t": "static menu table, pad vs field split",
    "unk_fx_0": "gfx_id is one 16-bit bitfield upstream, two 8-bit ones in Rust; same bits",
}


def main(la, walk, ident):
    r = json.load(open(la))["records"]
    visited = {l.split()[1] for l in open(walk) if l.startswith("VISITED ")}
    same = {}
    for l in open(ident):
        p = l.split()
        if len(p) >= 2:
            same[p[1]] = p[0] == "IDENTICAL"
    names = sorted(set(r) | set(same))
    print("| record | A: vs C layout | B: disc walk | native = GC layout | note |")
    print("|---|---|---|---|---|")
    for n in names:
        a = r.get(n, {}).get("status", "anonymous (checked inside its parent)")
        a = {"ok": "match", "MISMATCH": "differs"}.get(a, "no C record of that name" if a.startswith("no C") else a)
        b = "walked, 0 mismatch" if n in visited else ""
        s = "yes" if same.get(n) else "no"
        print(f"| `{n}` | {a} | {b} | {s} | {NOTES.get(n, '')} |")


if __name__ == "__main__":
    main(*sys.argv[1:4])
