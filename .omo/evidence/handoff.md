# Custom fighters, round 2 handoff (lead stopped at session limit)

## Done (round 1, merged by main session as game d3eca5609 / ws 236b2b0)
- R1 runtime provider: pc/platform/gw_mex_modfighter.inc (fighter half of mexData from mods/<id>/mexfighter.txt; gw_Mex_MexData()/CssIconCount() stay 0).
- R2 installer: pc/platform/gw_nucleus_fighter.h (verdict works/partial/needs-base), routed from gw_nucleus_engine.inc for NC_T_GAMEPLAY zips.
- Proof 1: ACE B. Falcon repackaged (ws tools/nucleus/make_synth_fighter.py), vanilla disc, FD: idle/jab/ftilt/dtilt/utilt/side-B frame-identical to ACE (action, anim frame, hitboxes, damage). In-game suite 383/383.
- Proof 2: Giga Bowser installs but PANIC ftparts.c:453 (parts num mismatch, 76 bones vs Bowser's row).

## Half-done (uncommitted-before-WIP, NOT built, NOT run): round 2 item (1), FighterPartsTable from bone lookup
- gw_nucleus_fighter.h: nf_bone_names[53] (MexManager lookup key -> engine FtPart index, by NAME; MexManager has extra waistB and lHave/rHave before thumbs), nf_parse_json reads boneDefinitions.lookup + boneCount, installer writes `bonecount=` and `bones=` (53 joint indices in FtPart order, 255 = none) to mexfighter.txt.
- gw_mex_modfighter.inc: mf_parse reads bonecount/bones; gw_mf_build allocates a PlCo-shaped struct {joint_to_part*, part_to_joint*, parts_num} per fighter in the persistent image (gw_log "own parts table"); new export gw_Mex_FtPartsTable(internal) returns its guest address or 0.
- src/melee/ft/fighter.c ftCommonData_ExtendKindTable: a mod kind with a parts table uses it (slot 0) and NULL for the hidden-part list (slot 1); otherwise clone base's row as in round 1.
- UNVERIFIED RISKS: bones array is 64 ints in mf_fighter but nf bones[64] and 53 roles used; part_to_joint is 128 bytes (FtPart_56/109 stay 0xFF); the installer unit test (pc/tests/nucleus_fighter_tests.h) has no boneDefinitions yet; first compile may have typos.

## Planned next, in order
1. Build (command below), run `build.sh --native-test nucleus-core` (expect 304+ checks), then reinstall Giga Bowser (NF_INSTALL_ZIP=C:/Users/Gurek/AppData/Local/SSBM Nucleus/storage/builtin_seed/giga-bowser/fighter.zip NF_INSTALL_MODS=<dir> NF_INSTALL_POST=4002 ... --native-test nucleus-core) and run the proof scene; if ftparts.c:453 persists compare the model's DFS joint count with boneCount.
2. Until Giga Bowser plays: installer downgrade to needs-base when boneDefinitions are absent AND the zip's skeleton cannot be assumed (not done).
3. R4: installer counts existing fighter mods (slot cap 31 message), two-mod conflict messages; two-client loopback netplay refusal run (MELEE_NETPLAY_BIND=127.0.0.1, local server, guest joins by code via gd.netplay_act("code", ...)); coexistence run with Goku define + a custom stage mod; gd.rewind_test with a mod fighter (0 bytes).
4. Parity N-B / down-B / up-B / nair for B. Falcon with MELEE_LOG=all, alone on a quiet machine (the ACE log dropped rows without it; it is slow, ~10 min per run).
5. Narrow install path to posts tagged Moveset/Fighter (gw_nucleus_catalog.h nc_why_not_files, NC_T_GAMEPLAY branch) and say "download is an external link" for posts whose only file is a ~0-byte "*google drive*.txt" (Tails 17327); never follow the link.
6. Screenshots (realtime, offscreen) of CSS + match for B. Falcon and Giga Bowser.
7. Own effect banks (cheap?), sound banks, announcer, Kirby hats: each needs the m-ex ssm/effect/music tables, which the synthetic image deliberately leaves empty.

## Commands
export GW_ROOT="E:/Projects/Melee Workspace" GW_MELEE="E:/Projects/Melee Workspace/worktrees/nucfighters" GW_BUILD_ROOT="E:/Projects/Melee Workspace/_build/agents/nucfighters"; . "$GW_ROOT/.env"
Build: cd $GW_MELEE && bash "E:/Projects/Melee Workspace/worktrees/nucfighters-ws/tools/port/build.sh" > $GW_BUILD_ROOT/buildN.log 2>&1   (do not edit sources during a build: provenance fails)
Native: build.sh --native-test nucleus-core ; installer dev mode: NF_INSTALL_ZIP/NF_INSTALL_MODS/NF_INSTALL_POST env on that command.
Run: MELEE_MODS_DIR=<mods> MELEE_SCENE="mode=vs;p1=ck:34;p2=fox/cpu;stage=fd" MELEE_SCRIPT=$GW_BUILD_ROOT/proof/fighter_proof.lua MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000 bash <ws>/tools/port/run.sh --idle-cpus --max-seconds 300 NAME --iso "$GW_ISO_VANILLA"   (ACE: p1=ck:51 --iso $GW_ISO_ACE, empty MELEE_MODS_DIR)
Mod dirs: _build/agents/nucfighters/modsA (B. Falcon, post 4501), modsB (Giga Bowser, post 4002). Proof script: _build/agents/nucfighters/proof/fighter_proof.lua (copy in the ws scratchpad).
Suite: run.sh --test --max-seconds 900 suite --iso $GW_ISO_VANILLA   (383 registered).
Tails: _build/nucleus-tails in the ws worktree holds the one fetched file (screenshots + 0-byte Drive pointer); not installable from Nucleus.

## Processes
None of mine left running (killed proofA2/proofACE2).
