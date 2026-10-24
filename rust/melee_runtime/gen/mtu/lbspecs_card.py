"""lbcardnew spec (see lbspecs.py)."""
import lbspecs
from lbspecs import add

CRC = "CRC[fv() % 15]", "CRC[(fv() % 15) as usize]"
HSR = "HSR[fv() % 16]", "HSR[(fv() % 16) as usize]"
CRC_OR_RAND = ("((fv() & 1) ? CRC[fv() % 15] : (int) fv())", "if fv() & 1 != 0 { CRC[(fv() % 15) as usize] } else { fv() as i32 }")
HSR_OR_RAND = ("((fv() & 1) ? HSR[fv() % 16] : (int) fv())", "if fv() & 1 != 0 { HSR[(fv() % 16) as usize] } else { fv() as i32 }")
ENT = ("ENT", "(&raw mut ENT) as *mut LbCardEntry")
ENTV = ("(void*) ENT", "(&raw mut ENT) as *mut c_void")
IDX9 = ("(int) (fv() % 9)", "(fv() % 9) as i32")

# CARDGetStatus: fills the stat the way the card directory would (disk-id company/game, digit file names)
STAT_C = (
    "{ int q, m = (int) fv(); u32 k = fv() % 4; "
    "if (m & 1) { a2->company[0] = DID[0].company[0]; a2->company[1] = DID[0].company[1]; for (q = 0; q < 4; q++) a2->gameName[q] = DID[0].gameName[q]; } "
    "else if (m & 2) { a2->company[0] = state.company[0]; a2->company[1] = state.company[1]; for (q = 0; q < 4; q++) a2->gameName[q] = state.game_name[q]; } "
    "else { a2->company[0] = (u8) fv(); a2->company[1] = (u8) fv(); for (q = 0; q < 4; q++) a2->gameName[q] = (u8) fv(); } "
    "for (q = 0; q < 32; q++) a2->fileName[q] = 0; "
    "if (k == 3) { for (q = 0; STRS[m & 3][q]; q++) a2->fileName[q] = STRS[m & 3][q]; } "
    "else { int nd = 1 + (int) (fv() % 9); for (q = 0; q < nd; q++) a2->fileName[q] = (k == 1) ? (char) ('a' + fv() % 26) : (char) ('0' + fv() % 10); "
    "if (k == 2) a2->fileName[nd] = 'z'; } "
    "a2->length = fv(); }")
STAT_R = (
    "{ let m = fv() as i32; let k = fv() % 4; "
    "if m & 1 != 0 { (*a2).company[0] = DID[0].company[0] as u8; (*a2).company[1] = DID[0].company[1] as u8; for q in 0..4 { (*a2).gameName[q] = DID[0].gameName[q] as u8; } } "
    "else if m & 2 != 0 { (*a2).company[0] = m::state.company[0] as u8; (*a2).company[1] = m::state.company[1] as u8; for q in 0..4 { (*a2).gameName[q] = m::state.game_name[q] as u8; } } "
    "else { (*a2).company[0] = fv() as u8; (*a2).company[1] = fv() as u8; for q in 0..4 { (*a2).gameName[q] = fv() as u8; } } "
    "for q in 0..32 { (*a2).fileName[q] = 0; } "
    "if k == 3 { let mut q = 0; while STRS[(m & 3) as usize][q] != 0 { (*a2).fileName[q] = STRS[(m & 3) as usize][q] as i8; q += 1; } } "
    "else { let nd = 1 + (fv() % 9) as usize; for q in 0..nd { (*a2).fileName[q] = if k == 1 { (b'a' + (fv() % 26) as u8) as i8 } else { (b'0' + (fv() % 10) as u8) as i8 }; } "
    "if k == 2 { (*a2).fileName[nd] = b'z' as i8; } } "
    "(*a2).length = fv(); }")

SETUP_C = (
    "{ int q; state.work_area = (op == OP_lbCardNew_AllocWorkArea && (fv() & 1)) ? 0 : (void*) WRK; state.lib_area = (op == OP_lbCardNew_AllocWorkArea && (fv() & 1)) ? 0 : (void*) NODES; "
    "state.save_data = BUF; state.status = (fv() & 1) ? &IA[0] : 0; state.comment = (char*) BUF + 16; state.banner = BUF + 64; state.icons = BUF + 128; "
    "state.snapshot_entries = SNP; state.free_blocks = (fv() & 1) ? &IA[1] : 0; state.free_files = (fv() & 1) ? &IA[2] : 0; "
    "state.sectorsize = (fv() & 1) ? 0x2000 : (int) fv(); state.unused_bytes = (int) (fv() % 0x400000); state.unused_files = (int) (fv() % 4); "
    "state.mounted = (int) (fv() & 1); state.tasks_remaining = (int) (fv() % 4) - 1; state.saved_error = (int) (fv() % 18); state.chan = (int) (fv() % 3); "
    "for (q = 0; q < 11; q++) state.task_array[q].type = fv() % 16; state.task_array[fv() % 11].type = 14; "
    "for (q = 0; q < 9; q++) { ENT[q].file_size = (int) (fv() % 1000); ENT[q].data = BUF + 16 * q; } ENT[9].file_size = -1; "
    "for (q = 0; q < 10; q++) state.task_array[q].file_entries = ENT; }")
SETUP_R = (
    "{ m::state.work_area = if op == OP_lbCardNew_AllocWorkArea && fv() & 1 != 0 { null_mut() } else { (&raw mut WRK) as *mut c_void }; "
    "m::state.lib_area = if op == OP_lbCardNew_AllocWorkArea && fv() & 1 != 0 { null_mut() } else { (&raw mut NODES) as *mut c_void }; "
    "m::state.save_data = (&raw mut BUF) as *mut c_void; m::state.status = if fv() & 1 != 0 { &raw mut IA[0] } else { null_mut() }; "
    "m::state.comment = (&raw mut BUF as *mut u8).add(16) as *mut i8; m::state.banner = (&raw mut BUF as *mut u8).add(64) as *mut c_void; m::state.icons = (&raw mut BUF as *mut u8).add(128) as *mut c_void; "
    "m::state.snapshot_entries = (&raw mut SNP) as *mut lbCardNew_SnapshotEntry; m::state.free_blocks = if fv() & 1 != 0 { &raw mut IA[1] } else { null_mut() }; "
    "m::state.free_files = if fv() & 1 != 0 { &raw mut IA[2] } else { null_mut() }; "
    "m::state.sectorsize = if fv() & 1 != 0 { 0x2000 } else { fv() as i32 }; m::state.unused_bytes = (fv() % 0x400000) as i32; m::state.unused_files = (fv() % 4) as i32; "
    "m::state.mounted = (fv() & 1) as i32; m::state.tasks_remaining = (fv() % 4) as i32 - 1; m::state.saved_error = (fv() % 18) as i32; m::state.chan = (fv() % 3) as i32; "
    "for q in 0..11usize { m::state.task_array[q].r#type = fv() % 16; } m::state.task_array[(fv() % 11) as usize].r#type = 14; "
    "for q in 0..9usize { ENT[q].file_size = (fv() % 1000) as i32; ENT[q].data = (&raw mut BUF as *mut u8).add(16 * q) as *mut c_void; } ENT[9].file_size = -1; "
    "for q in 0..10usize { m::state.task_array[q].file_entries = (&raw mut ENT) as *mut c_void; } }")

crc_fns = ["CARDCheckAsync", "CARDDeleteAsync", "CARDFormatAsync", "CARDFreeBlocks", "CARDMountAsync", "CARDOpen",
           "CARDProbeEx", "CARDRenameAsync", "CARDClose", "CARDUnmount"]
hsr_fns = ["hsd_803B2550", "hsd_803B27F4", "hsd_803B286C", "hsd_803B2928", "hsd_803B29D8", "hsd_803B2A4C"]
ret = {n: CRC for n in crc_fns}
ret.update({n: HSR for n in hsr_fns})
ret.update({
    "hsd_803B2674": ("(int) (fv() % 12)", "(fv() % 12) as i32"),
    "CARDGetStatus": ("((fv() % 5 == 0) ? -3 : 0)", "if fv() % 5 == 0 { -3 } else { 0 }"),
    "DVDGetCurrentDiskID": ("&DID[0]", "&raw mut DID[0]"),
    "HSD_MemAlloc": ("((a0 >= 0xA000) ? (void*) WRK : (void*) NODES)", "if a0 >= 0xA000 { (&raw mut WRK) as *mut c_void } else { (&raw mut NODES) as *mut c_void }"),
})
add("lbcardnew", "src/melee/lb/lbcardnew.c", dict(
    pools=[("DVDDiskID", "DID", 1), ("LbCardEntry", "ENT", 12), ("lbCardNew_SnapshotEntry", "SNP", 127)],
    c_decl="static unsigned char WRK[16]; static unsigned char NODES[2100]; static const int CRC[15] = { 0, 0, 0, 0, -1, -2, -3, -4, -5, -6, -13, -14, -128, -7, -9 }; "
           "static const int HSR[16] = { 0, 0, 1, -258, -259, -260, -261, -262, -263, -257, -264, -265, -266, -267, -5, -3 };",
    rs_extra="use m::LbCardEntry; static mut WRK: [u8; 16] = [0; 16]; static mut NODES: [u8; 2100] = [0; 2100]; pub const CRC: [i32; 15] = [0, 0, 0, 0, -1, -2, -3, -4, -5, -6, -13, -14, -128, -7, -9]; "
             "pub const HSR: [i32; 16] = [0, 0, 1, -258, -259, -260, -261, -262, -263, -257, -264, -265, -266, -267, -5, -3];",
    pregs=("preg(WRK, sizeof WRK); preg(NODES, sizeof NODES);", "preg(&raw const WRK, 16); preg(&raw const NODES, 2100);"),
    setup=(SETUP_C, SETUP_R),
    ov={
        "convertSdkResult": {0: CRC_OR_RAND},
        "convertHsdResult": {0: HSR_OR_RAND},
        "fn_8001A0B0": {0: IDX9, 1: HSR_OR_RAND},
        "onCardComplete": {1: CRC_OR_RAND},
        "lb_8001B6E0": {0: IDX9},
        "executeNextTask": {0: ("(int) (fv() % 18)", "fv() % 18")},
        "lbCardNew_CompleteAllTasks": {0: ("(int) (fv() % 14)", "(fv() % 14) as i32")},
        "taskOpen": {1: ENT},
        "taskRead": {0: ENT},
        "taskWrite": {0: ENT},
        "setupCardEntries": {0: ("&state.card_state", "&raw mut m::state.card_state"), 2: ENT},
        "setTaskFilename": {1: ENTV},
        "lb_8001B7E0": {2: ENTV},
        "lb_8001BB48": {2: ENTV},
        "lb_8001BC18": {2: ("(void**) ENT", "(&raw mut ENT) as *mut *mut c_void")},
        "lb_8001BD34": {2: ENTV},
        "lb_8001BE30": {2: ENTV, 7: ("(void*) pick_cb(4)", "pick_cb(4) as *mut c_void")},
        "lb_8001BF04": {2: ENTV},
        "lb_8001BFD8": {1: ("SNP", "(&raw mut SNP) as *mut lbCardNew_SnapshotEntry")},
        "lb_8001C4A8": {0: ENTV},
        "lb_8001A4CC_dontinline": {1: ENTV},
    },
    ),
    types=["DVDDiskID", "LbCardEntry", "lbCardNew_SnapshotEntry"], pools=["DVDDiskID", "LbCardEntry", "lbCardNew_SnapshotEntry"],
    own_fill=["state"], skip_data=["result"],
    patch=[("isdigit(stat[0].fileName[0])", "(stat[0].fileName[0] >= '0' && stat[0].fileName[0] <= '9')")],
    skip_stubs=["memcpy", "strcmp", "strncmp", "strncpy", "strtoul"],
    null_ret_ok=[], ret=ret,
    effects={
        "CARDGetStatus": (STAT_C, STAT_R),
        "CARDProbeEx": ("*a1 = (int) fv(); *a2 = (fv() & 1) ? 0x2000 : (int) fv();", "*a1 = fv() as i32; *a2 = if fv() & 1 != 0 { 0x2000 } else { fv() as i32 };"),
        "CARDFreeBlocks": ("*a1 = (int) (fv() % 0x100000); *a2 = (int) (fv() % 4);", "*a1 = (fv() % 0x100000) as i32; *a2 = (fv() % 4) as i32;"),
        "hsd_803AAA48": ("if (fv() % 3 == 0) state.tasks_remaining = 0;", "if fv() % 3 == 0 { m::state.tasks_remaining = 0; }"),
        "wait_idle": ("if (state.tasks_remaining > 0 && (fv() & 1)) state.tasks_remaining = 0;", "if m::state.tasks_remaining > 0 && fv() & 1 != 0 { m::state.tasks_remaining = 0; }"),
    })
