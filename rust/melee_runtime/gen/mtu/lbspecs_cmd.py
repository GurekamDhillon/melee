"""lbcommand spec (see lbspecs.py)."""
from lbspecs import add

C_EXTRA = r'''
static CommandInfo CI[3]; static CmdUnion CU[20]; static u32 CMD;
static u32 HP(const void* p) { unsigned long long v = (unsigned long long) p; return (v >> 32) == 0 ? (u32) v : PI(p); }
static void setcu(u32 cmd)
{
    int q;
    for (q = 0; q < 16; q++) {
        zero(&CU[q], sizeof CU[q]);
        switch (cmd) {
        case 1: CU[q].Command_00.value = fv(); break;
        case 2: CU[q].Command_02.value = fv(); break;
        case 3: CU[q].Command_03.value = fv() % 5; break;
        case 5: case 7: CU[q].Command_05.ptr = &CU[fv() % 16]; break;
        case 9: CU[q].Command_09.id = fv(); CU[q].Command_09.param_1 = fv(); CU[q].Command_09.param_2 = fv(); break;
        default: CU[q].Command_04.x = fv(); break;
        }
    }
}
static void setci(u32 cmd)
{
    int q;
    for (q = 0; q < 3; q++) {
        CommandInfo* c = &CI[q];
        zero(c, sizeof *c);
        c->timer = rw(); c->frame_count = rw(); c->u = &CU[fv() % 16];
        c->loop_count = cmd == 3 ? fv() % 2 : cmd == 4 ? 2 : cmd == 5 ? fv() % 3 : cmd == 6 ? 1 + fv() % 3 : fv() % 4;
        c->event_return[0] = &CU[fv() % 16]; c->event_return[1] = (CmdUnion*) (unsigned long long) (fv() % 4); c->event_return[2] = &CU[fv() % 16];
        c->loop_count_dup = fv(); c->unk_x18 = fv();
    }
}
'''
RS_EXTRA = r'''
static mut CI: [CommandInfo; 3] = unsafe { core::mem::zeroed() };
static mut CU: [CmdUnion; 20] = unsafe { core::mem::zeroed() };
static mut CMD: u32 = 0;
fn hp<T>(p: *const T) -> u32 { let v = p as usize as u64; if (v >> 32) == 0 { v as u32 } else { pi(p) } }
unsafe fn setcu(cmd: u32) {
    for q in 0..16usize {
        CU[q] = core::mem::zeroed();
        match cmd {
            1 => CU[q].Command_00.set_value(fv()),
            2 => CU[q].Command_02.set_value(fv()),
            3 => CU[q].Command_03.set_value(fv() % 5),
            5 | 7 => CU[q].Command_05.ptr = &raw mut CU[(fv() % 16) as usize],
            9 => { CU[q].Command_09.set_id(fv()); CU[q].Command_09.set_param_1(fv()); CU[q].Command_09.set_param_2(fv()); }
            _ => CU[q].Command_04.x = fv(),
        }
    }
}
unsafe fn setci(cmd: u32) {
    for q in 0..3usize {
        let c = &mut CI[q];
        *c = core::mem::zeroed();
        c.timer = rw(); c.frame_count = rw(); c.anon_8.u = &raw mut CU[(fv() % 16) as usize];
        c.loop_count = if cmd == 3 { fv() % 2 } else if cmd == 4 { 2 } else if cmd == 5 { fv() % 3 } else if cmd == 6 { 1 + fv() % 3 } else { fv() % 4 };
        c.event_return[0] = &raw mut CU[(fv() % 16) as usize]; c.event_return[1] = (fv() % 4) as usize as *mut CmdUnion; c.event_return[2] = &raw mut CU[(fv() % 16) as usize];
        c.loop_count_dup = fv(); c.unk_x18 = fv();
    }
}
'''
HASH_C = ("{ int q, k; for (q = 0; q < 3; q++) { CommandInfo* c = &CI[q]; HM(B(c->timer)); HM(B(c->frame_count)); HM(PI(c->u)); HM(c->loop_count); "
          "for (k = 0; k < 3; k++) HM(HP(c->event_return[k])); HM(c->loop_count_dup); HM(c->unk_x18); } }")
HASH_R = ("{ for q in 0..3usize { let c = &CI[q]; hm(c.timer.to_bits()); hm(c.frame_count.to_bits()); hm(pi(c.anon_8.u)); hm(c.loop_count); "
          "for k in 0..3usize { hm(hp(c.event_return[k])); } hm(c.loop_count_dup); hm(c.unk_x18); } }")
CIA = ("&CI[fv() % 3]", "&raw mut CI[(fv() % 3) as usize]")
ov = {f"Command_{i:02d}": {0: CIA} for i in range(10)}
ov["Command_Execute"] = {0: CIA, 1: ("CMD", "CMD")}

add("lbcommand", "src/melee/lb/lbcommand.c", dict(
    c_extra=C_EXTRA, rs_extra=RS_EXTRA,
    pregs=("preg(CI, sizeof CI); preg(CU, sizeof CU);", "preg(&raw const CI, core::mem::size_of_val(&CI)); preg(&raw const CU, core::mem::size_of_val(&CU));"),
    setup=("{ CMD = op < 10 ? (u32) op : fv() % 12; setcu(CMD); setci(CMD); }",
           "{ CMD = if op < 10 { op } else { fv() % 12 }; setcu(CMD); setci(CMD); }"),
    hash=(HASH_C, HASH_R), ov=ov),
    types=[], pools=[],
    patch=[("    u32* ptr = (u32*) info;\n    ptr[info->loop_count + 3] -= 1;",
            "    u32* ptr = (u32*) &info->event_return[info->loop_count - 1]; /* the 32-bit counter slot */\n    *ptr -= 1;")])
