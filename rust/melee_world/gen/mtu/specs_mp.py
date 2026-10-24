"""Spec of the combined mp oracle (mplib.c + mpcoll.c + mpisland.c compiled as one C TU, ported as
tu::{mplib,mpcoll,mpisland}); driven by gen/mtu/mk_mp.py."""
import os

SPECS = {}
# MTU_PATCHFILE=<file>: debugging only. The file holds blocks "old\n=====\nnew\n#####\n" of C text
# replaced everywhere in the combined TU (e.g. a printf in a function the Rust port also traces).
DBG_PATCH = []
if os.environ.get("MTU_PATCHFILE"):
    for blk in open(os.environ["MTU_PATCHFILE"], encoding="utf-8").read().split("#####\n"):
        if blk.strip():
            o, n = blk.split("=====\n")
            DBG_PATCH.append((o, n))

SPECS["mp"] = dict(
    src="src/melee/mp/mplib.c",
    srcs=["src/melee/mp/mplib.c", "src/melee/mp/mpcoll.c", "src/melee/mp/mpisland.c"],
    mods=["mplib", "mpcoll", "mpisland"],
    args=["-iquote", "src/melee/mp"],
    # the native build cannot keep 32-bit pointer <-> int casts: byte arithmetic on the char pointer instead
    patch_all=[("(CollLine*) ((int) groundCollLine + line_offset)", "(CollLine*) ((char*) groundCollLine + line_offset)"),
               # `flags_base[n * 2]`: the GameCube CollLine is 8 bytes (u32 pairs); natively it is 16
               ("flags_base[result * 2]", "flags_base[result * (int) (sizeof(CollLine) / 4)]"),
               ("flags_base[start_id * 2]", "flags_base[start_id * (int) (sizeof(CollLine) / 4)]")] + DBG_PATCH,
    skip_fields={"CollData": ["ecb_source"]},
    types=["CollData", "CollVtx", "CollLine", "CollJoint", "MapLine", "MapJoint", "MapCollData", "mp_UnkStruct0",
           "HSD_GObj", "HSD_JObj"],
    skip_data=["guess", "result", "same", "y"],
    # math, matrix and vector helpers the ports take from melee_runtime: the real C runs on the oracle side
    skip_stubs=["__HI", "__LO", "__frsqrte", "cosf", "sinf", "PSMTXMultVec", "PSVECNormalize", "lbVector_Add",
                "lbVector_Diff", "memset", "memzero"],
    ret={
        "HSD_MemAlloc": (
            "((a0 == 2048 * (int) sizeof(CollVtx)) ? (void*) VTXA : (a0 == 1536 * (int) sizeof(CollLine)) ? (void*) LINEA : "
            "(a0 == 8 * (int) sizeof(CollJoint)) ? (void*) JOINTA : (a0 == 0x2C && ISLN < 48) ? (void*) &ISL[ISLN++] : (void*) 0)",
            "if a0 == 2048 * core::mem::size_of::<CollVtx>() as i32 { &raw mut super::VTXA as *mut c_void } "
            "else if a0 == 1536 * core::mem::size_of::<CollLine>() as i32 { &raw mut super::LINEA as *mut c_void } "
            "else if a0 == 8 * core::mem::size_of::<CollJoint>() as i32 { &raw mut super::JOINTA as *mut c_void } "
            "else if a0 == 0x2C && super::ISLN < 48 { super::ISLN += 1; &raw mut super::ISL[(super::ISLN - 1) as usize] as *mut c_void } "
            "else { core::ptr::null_mut() }"),
        "ScriptGame_StageJointCapacity": ("8", "8"),
        "ScriptGame_StageJointActive": ("(int) ((fv() & 7) != 0)", "((fv() & 7) != 0) as i32"),
        "ScriptGame_StageLineActive": ("(int) ((fv() & 7) != 0)", "((fv() & 7) != 0) as i32"),
        "ScriptGame_StageJoint": ("(int) ((fv() & 7) == 0)", "((fv() & 7) == 0) as i32"),
        "ScriptGame_StageSlotSameJoint": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
        "Ground_801C0498": ("((rnd() & 1) ? 1.0f : 0.5f)", "if rnd() & 1 != 0 { 1.0 } else { 0.5 }"),
        "ftLib_80086984": ("&CD", "&raw mut super::CD"),
        "ftLib_80086960": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
        "Mex_GrLineTypeRow": ("(int) (fv() % 5) - 1", "(fv() % 5) as i32 - 1"),
    },
    effects={
        "lb_8000B1CC": ("if (a2) { a2->x = RNN(); a2->y = RNN(); a2->z = RNN(); }",
                        "if !a2.is_null() { (*a2).x = super::rnn(); (*a2).y = super::rnn(); (*a2).z = super::rnn(); }"),
    },
    null_ret_ok=["GObj_Create", "HSD_CObjGetCurrent", "HSD_GObj_SetupProc"],
)
