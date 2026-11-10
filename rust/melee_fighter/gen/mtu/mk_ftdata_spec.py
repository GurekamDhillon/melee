#!/usr/bin/env python3
"""Writes gen/mtu/specs/ftdata.json: the state oracle of src/melee/ft/ftdata.c (ft_800852B0, ftData_800855C8, ftData_80085B10,
ftData_80085CD8) and of the Geno define functions it includes from pc/geno/geno_define_data.inc (GenoDefine_InitKinds,
CaptureBase, RestPoseTree, BindFighter).

  python rust/melee_fighter/gen/mtu/mk_ftdata_spec.py && python rust/melee_fighter/gen/mtu/mkdriver.py rust/melee_fighter/gen/mtu/specs/ftdata.json

The C and the Rust world are written from one description so the two cannot drift: every per-kind table of the TU is filled with
pointer-sized words drawn from the shared stream (0, a registered callback, or a pointer into the PADB pool) and hashed by pool id.
"""
import json
from pathlib import Path

FD = Path(__file__).resolve().parents[2]

# flat pointer-word tables: every element is pointer sized (function pointer, data pointer, or a pair of them)
FLAT = """ftData_OnLoad ftData_OnDeath ftData_OnUserDataRemove ftData_CharacterStateTables ftData_SpecialN ftData_SpecialAirN
ftData_SpecialS ftData_SpecialAirS ftData_SpecialHi ftData_SpecialAirHi ftData_SpecialLw ftData_SpecialAirLw ftData_OnAbsorb
ftData_OnItemPickupExt ftData_OnItemInvisible ftData_OnItemVisible ftData_OnItemDropExt ftData_OnItemPickup ftData_OnItemDrop
ftData_UnkMotionStates1 ftData_UnkMotionStates2 ftData_OnKnockbackEnter ftData_OnKnockbackExit ftData_UnkMotionStates3
ftData_UnkMotionStates4 ftKindCalcIndiviParamTable ftData_UnkMtxFunc0 ftData_803C24EC ftData_UnkDemoCallbacks0
ftData_UnkMotionStates0 ftData_803C1F40 ftData_803C23E4 ftData_803C2468 ftData_803C2360 ftData_UnkMotionStates5
ftData_UnkCallbackPairs0 ftData_Table_Unk1 gFtDataList ftData_ThrownRows ftData_AnimReloadData
GenoDefine_CommonRows GenoDefine_SpecificRows GenoDefine_RowBaseline""".split()
# tables whose non-NULL entries are written through: filled by hand with MEMBUF slices (or NULL), hashed with the flat ones
HAND = "GenoDefine_Costumes GenoDefine_CostumeStrings".split()
# plain integer words (size_t)
RAWW = ["ftData_AnimBankBytes"]

CPRE = r"""
#define KM Ft_Kind_Max
static UnkCostumeStruct UCS[KM][4];
static Fighter_CostumeStrings CSTR[KM][4];
static HSD_Joint JNT[8];
static u8 MEMBUF[8192];
static u8 MEMBUF2[16384];
static int REGD;
"""

# c_head runs after the stub header: pw / fillw / hashw need cbev1 etc. (defined by the driver before c_pre)
CHEAD = r"""
static uintptr_t pw(void)
{
    u32 r = fv();
    if (r % 3 == 0) return 0;
    if (r % 3 == 1) return (uintptr_t) (((r >> 4) & 1) ? (const void*) cbev1 : (const void*) cbev2);
    { u32 row = fv() % 96; u32 off = (fv() % 4) * 8; return (uintptr_t) PADB[row] + off; }
}
static void fillw(void* t, unsigned bytes) { unsigned i; for (i = 0; i < bytes / sizeof(uintptr_t); i++) ((uintptr_t*) t)[i] = pw(); }
static void hashw(const void* t, unsigned bytes) { unsigned i; for (i = 0; i < bytes / sizeof(uintptr_t); i++) HM(PI((const void*) ((const uintptr_t*) t)[i])); }
static void fillr(void* t, unsigned bytes) { unsigned i; for (i = 0; i < bytes / sizeof(uintptr_t); i++) ((uintptr_t*) t)[i] = (uintptr_t) fv(); }
static void hashr(const void* t, unsigned bytes) { unsigned i; for (i = 0; i < bytes / sizeof(uintptr_t); i++) { uintptr_t w = ((const uintptr_t*) t)[i]; HM((u32) w); HM((u32) ((u64_t_) w >> 32)); } }
void* HSD_ObjAlloc(HSD_ObjAllocData* a0) { T(9002); T(PI((const void*) a0)); return 0; }
/* hand-written stub with out parameters (stubs.py cannot fill them): lbFile_800168A0(heap, name, &data, &size) */
bool lbFile_800168A0(int a0, const char* a1, void** a2, size_t* a3)
{
    T(9001); T((u32) a0); T(PI((const void*) a1));
    *a2 = (fv() & 3) ? (void*) PADB[fv() % 96] : 0;
    *a3 = (size_t) fv();
    return 0;
}
"""

TABLE_FILL_C = "\n".join(f"      fillw(&{n}, sizeof {n});" for n in FLAT)
RAW_FILL_C = "\n".join(f"      for (k_ = 0; k_ < KM; k_++) {n}[k_] = (size_t) fv();" for n in RAWW)
TABLE_HASH_C = "\n".join(f"    hashw(&{n}, sizeof {n});" for n in FLAT + HAND)
RAW_HASH_C = "\n".join(f"      for (k_ = 0; k_ < KM; k_++) HM((u32) {n}[k_]);" for n in RAWW)

CWORLD = r"""
    { int q_, k_, c_; unsigned j_;
      { static int sz_; if (!sz_) { sz_ = 1; if (getenv("TUSZ")) {
@SZ_C@
      } } }
      if (!REGD) { REGD = 1; preg(UCS, sizeof UCS); preg(CSTR, sizeof CSTR); preg(JNT, sizeof JNT); preg(MEMBUF, sizeof MEMBUF); preg(MEMBUF2, sizeof MEMBUF2); }
      zero(UCS, sizeof UCS); zero(CSTR, sizeof CSTR); zero(JNT, sizeof JNT); zero(MEMBUF, sizeof MEMBUF); zero(MEMBUF2, sizeof MEMBUF2);
      for (q_ = 0; q_ < 8; q_++) { JNT[q_].child = (fv() & 1) ? &JNT[fv() % 8] : 0; }
      for (q_ = 0; q_ < 96; q_++) zero(PADB[q_], sizeof PADB[q_]);
@FILL@
@RAWFILL@
      for (k_ = 0; k_ < KM; k_++) {
        gFtDataList[k_] = (k_ >= 27 && k_ % 4 == 0) ? 0 : &FDATA[k_ % NG];
        ftData_Table_Unk0[k_].data = (void*) pw();
        ftData_Table_Unk0[k_].count = (int) (fv() % 40);
        ftData_UnkIntPairs[k_].data = (void*) pw();
        ftData_UnkIntPairs[k_].count = (int) (fv() % 40);
        ftData_UnkBytePerCharacter[k_] = (u8) fv();
        ftData_803C2360[k_] = CSTR[k_];
        for (c_ = 0; c_ < 4; c_++) { CSTR[k_][c_].dat_filename = (char*) pw(); CSTR[k_][c_].joint_name = (char*) pw(); CSTR[k_][c_].matanim_joint_name = (char*) pw(); }
        CostumeListsForeachCharacter[k_].costume_list = UCS[k_];
        CostumeListsForeachCharacter[k_].numCostumes = (u8) (fv() % 5);
        for (c_ = 0; c_ < 4; c_++) {
          UCS[k_][c_].joint = &JNT[fv() % 8]; UCS[k_][c_].x4 = (void*) pw(); UCS[k_][c_].pad_x8 = fv();
          UCS[k_][c_].pad_xC = fv(); UCS[k_][c_].pad_x10 = fv(); UCS[k_][c_].x14_archive = (void*) pw();
        }
      }
      /* the tables the Geno functions index by a Mex kind keep their row pointers valid: costume_list is UCS[kind] */
      for (q_ = 0; q_ < 94; q_++) { GenoDefine_Costumes[q_] = (fv() & 1) ? (UnkCostumeStruct*) (MEMBUF + (fv() % 4) * 2048) : 0; GenoDefine_CostumeStrings[q_] = (fv() & 1) ? (Fighter_CostumeStrings*) (MEMBUF + (fv() % 4) * 2048) : 0; }
      for (q_ = 0; q_ < 6; q_++) { ft_8045993C[q_].pad_x0 = fv(); ft_8045993C[q_].pad_x4[0] = (u8) fv(); ft_8045993C[q_].pad_x4[1] = (u8) fv();
                                   ft_8045993C[q_].x6_b0 = fv() & 1; ft_8045993C[q_].x6_b1_b2 = fv() & 3; }
      for (q_ = 0; q_ < 6; q_++) { ftData_ThrowBanks[q_].kind = (int) (fv() % 10) - 1; ftData_ThrowBanks[q_].buf = (u8*) pw(); ftData_ThrowBanks[q_].cap = fv(); ftData_ThrowBanks[q_].sz = fv(); }
      ftData_ThrowBankLoads = fv(); ftData_ThrowBankEvicts = fv(); ftData_ThrowRegionCur = (u8*) pw();
      for (k_ = 0; k_ < KM; k_++) ftData_ThrownPrepared[k_] = (u8) fv();
      for (k_ = 0; k_ < KM; k_++) { GenoDefine_ContentIdentity[k_][0] = fv(); GenoDefine_ContentIdentity[k_][1] = fv(); GenoDefine_InterpreterAttempts[k_] = fv(); }
      for (q_ = 0; q_ < NG; q_++) {
        FT[q_].kind = (FighterKind) ((u32) FT[q_].kind % 27u); FT[q_].costume_id %= 4;
        FT[q_].x24 = (Fighter_WaitAnimData*) PADB[70 + q_]; FT[q_].x59C = (void*) PADB[80 + q_];
        FT[q_].x5A4 = (void*) (uintptr_t) (fv() % 8); FT[q_].x5A8 = (void*) pw(); FT[q_].x58C = (int) (fv() % 70);
        for (j_ = 0; j_ < 64; j_++) {
          Fighter_WaitAnimData* r_ = &((Fighter_WaitAnimData*) PADB[70 + q_])[j_];
          r_->x0 = (char*) pw(); r_->x4 = (s32) (fv() % 256); r_->x8 = (s32) (fv() % 1024); r_->xC = (CmdUnion*) pw(); r_->x10_animCurrFlags = (s32) fv(); r_->x14 = fv() % 8;
          r_ = &((Fighter_WaitAnimData*) PADB[3 + q_ * 24])[j_];
          r_->x0 = (char*) pw(); r_->x4 = (s32) (fv() % 256); r_->x8 = (s32) (fv() % 1024); r_->xC = (CmdUnion*) pw(); r_->x10_animCurrFlags = (s32) fv(); r_->x14 = fv() % 8;
        }
      }
    }
"""

CHASH = r"""
    { int k_, c_, q_;
@HASH@
@RAWHASH@
      for (k_ = 0; k_ < KM; k_++) {
        HM(PI(ftData_Table_Unk0[k_].data)); HM((u32) ftData_Table_Unk0[k_].count);
        HM(PI(ftData_UnkIntPairs[k_].data)); HM((u32) ftData_UnkIntPairs[k_].count);
        HM(ftData_UnkBytePerCharacter[k_]);
        HM(PI(CostumeListsForeachCharacter[k_].costume_list)); HM(CostumeListsForeachCharacter[k_].numCostumes);
        for (c_ = 0; c_ < 4; c_++) {
          HM(PI(CSTR[k_][c_].dat_filename)); HM(PI(CSTR[k_][c_].joint_name)); HM(PI(CSTR[k_][c_].matanim_joint_name));
          HM(PI(UCS[k_][c_].joint)); HM(PI(UCS[k_][c_].x4)); HM(UCS[k_][c_].pad_x8); HM(UCS[k_][c_].pad_xC);
          HM(UCS[k_][c_].pad_x10); HM(PI(UCS[k_][c_].x14_archive));
        }
        HM(ftData_ThrownPrepared[k_]); HM(GenoDefine_ContentIdentity[k_][0]); HM(GenoDefine_ContentIdentity[k_][1]); HM(GenoDefine_InterpreterAttempts[k_]);
      }
      for (q_ = 0; q_ < 6; q_++) { HM(ft_8045993C[q_].pad_x0); HM(ft_8045993C[q_].pad_x4[0]); HM(ft_8045993C[q_].pad_x4[1]); HM(ft_8045993C[q_].x6_b0); HM(ft_8045993C[q_].x6_b1_b2); }
      for (q_ = 0; q_ < 6; q_++) { HM((u32) ftData_ThrowBanks[q_].kind); HM(PI(ftData_ThrowBanks[q_].buf)); HM((u32) ftData_ThrowBanks[q_].cap); HM((u32) ftData_ThrowBanks[q_].sz); }
      HM(ftData_ThrowBankLoads); HM(ftData_ThrowBankEvicts); HM(PI(ftData_ThrowRegionCur));
      /* MEMBUF2 holds whatever HSD_MemAlloc callers put there (anim rows, captured motion rows): hash it word by word,
         a word above 4 GB as a native pointer (by region), any other word by value */
      for (k_ = 0; k_ < (int) (sizeof MEMBUF2 / 8); k_++) { unsigned long long w_ = ((const unsigned long long*) MEMBUF2)[k_]; if (w_ >> 32) HM(PI((const void*) (uintptr_t) w_)); else HM((u32) w_); }
    }
"""

# ------------------------------------------------------------------------------------------------------ the Rust half
RSHEAD = r"""
use melee_fighter::tu::ftdata as fdm;
static mut UCS: *mut [[UnkCostumeStruct; 4]; Ft_Kind_Max as usize] = null_mut();
static mut CSTR: *mut [[Fighter_CostumeStrings; 4]; Ft_Kind_Max as usize] = null_mut();
static mut JNT: *mut [HSD_Joint; 8] = null_mut();
static mut MEMBUF: *mut [u8; 8192] = null_mut();
static mut MEMBUF2: *mut [u8; 16384] = null_mut();
static mut REGD: bool = false;
const KM: usize = Ft_Kind_Max as usize;
unsafe fn pw() -> usize {
    let r = fv();
    if r % 3 == 0 { return 0; }
    if r % 3 == 1 { return if (r >> 4) & 1 != 0 { cbev1 as usize } else { cbev2 as usize }; }
    let row = (fv() % 96) as usize;
    let off = (fv() % 4) as usize * 8;
    (*PADB)[row].as_mut_ptr() as usize + off
}
unsafe fn fillw(t: *mut usize, bytes: usize) { for i in 0..bytes / 8 { *t.add(i) = pw(); } }
unsafe fn hashw(t: *const usize, bytes: usize) { for i in 0..bytes / 8 { hm(pi(*t.add(i) as *const u8)); } }
unsafe fn fillr(t: *mut usize, bytes: usize) { for i in 0..bytes / 8 { *t.add(i) = fv() as usize; } }
unsafe fn hashr(t: *const usize, bytes: usize) { for i in 0..bytes / 8 { let w = *t.add(i); hm(w as u32); hm((w as u64 >> 32) as u32); } }
macro_rules! tw { ($f:ident; $($t:ident),*) => { $( $f(&raw mut fdm::$t as *mut usize, core::mem::size_of_val(&*(&raw const fdm::$t))); )* } }
macro_rules! th { ($f:ident; $($t:ident),*) => { $( $f(&raw const fdm::$t as *const usize, core::mem::size_of_val(&*(&raw const fdm::$t))); )* } }
"""

RSHOOKS = r"""
    UCS = bz(); CSTR = bz(); JNT = bz(); MEMBUF = bz(); MEMBUF2 = bz();
    m::ext::LBFILE_800168A0 = Some(|a0, a1, a2, a3| { t(9001); t(a0 as u32); t(pi(a1)); *a2 = if fv() & 3 != 0 { (*PADB)[(fv() % 96) as usize].as_mut_ptr() as *mut c_void } else { null_mut() }; *a3 = fv() as usize; });
    m::ext::HSD_OBJ_ALLOC_X59C = Some(|| { t(9002); t(0xFFFF_FFFF); null_mut() });
    m::ext::OS_REPORT = Some(|_| {});
    // guest addresses: the C keeps native pointers (Fighter.x5A8, the anim rows' x14) and hashes them by region, so the
    // port's gc_addr32 / gc_ptr must round-trip a 64-bit pointer and keep base + offset arithmetic: 256 MB segments,
    // token = (segment + 1) << 28 | low 28 bits
    static mut GC_SEG: [usize; 15] = [0; 15];
    fn gc_tok(p: *const c_void) -> u32 {
        if (p as usize) < 0x1000_0000 { return p as usize as u32; } // null and the small ARAM-style offsets stay as they are
        let (hi, lo) = ((p as usize) & !0x0FFF_FFFF, (p as usize) & 0x0FFF_FFFF);
        unsafe {
            for i in 0..15 { if GC_SEG[i] == hi { return ((i as u32 + 1) << 28) | lo as u32; } }
            for i in 0..15 { if GC_SEG[i] == 0 { GC_SEG[i] = hi; return ((i as u32 + 1) << 28) | lo as u32; } }
        }
        panic!("gc_tok: more than 15 segments")
    }
    fn gc_back(a: u32) -> *mut c_void {
        let s = (a >> 28) as usize;
        if s == 0 { return a as usize as *mut c_void; }
        unsafe { (GC_SEG[s - 1] | (a & 0x0FFF_FFFF) as usize) as *mut c_void }
    }
    m::ext::GC_ADDR32 = Some(|p| gc_tok(p));
    m::ext::GC_PTR = Some(|a| gc_back(a));
"""

RSWORLD = r"""
    {
@SZ_R@
      if !REGD { REGD = true; preg(UCS, core::mem::size_of_val(&*UCS)); preg(CSTR, core::mem::size_of_val(&*CSTR)); preg(JNT, core::mem::size_of_val(&*JNT)); preg(MEMBUF, 8192); preg(MEMBUF2, 16384); }
      *UCS = core::mem::zeroed(); *CSTR = core::mem::zeroed(); *JNT = core::mem::zeroed(); *MEMBUF = [0; 8192]; *MEMBUF2 = [0; 16384];
      for q_ in 0..8 { (*JNT)[q_].child = if fv() & 1 != 0 { &raw mut (*JNT)[(fv() % 8) as usize] } else { null_mut() }; }
      for q_ in 0..96 { (*PADB)[q_] = [0; 2048]; }
@FILL@
@RAWFILL@
      for k_ in 0..KM {
        fdm::gFtDataList[k_] = if k_ >= 27 && k_ % 4 == 0 { null_mut() } else { &raw mut (*FDATA)[k_ % NG] };
        fdm::ftData_Table_Unk0[k_].data = pw() as *mut c_void;
        fdm::ftData_Table_Unk0[k_].count = (fv() % 40) as i32;
        fdm::ftData_UnkIntPairs[k_].data = pw() as *mut c_void;
        fdm::ftData_UnkIntPairs[k_].count = (fv() % 40) as i32;
        fdm::ftData_UnkBytePerCharacter[k_] = fv() as u8;
        fdm::ftData_803C2360[k_] = (*CSTR)[k_].as_mut_ptr();
        for c_ in 0..4 { (*CSTR)[k_][c_].dat_filename = pw() as *mut _; (*CSTR)[k_][c_].joint_name = pw() as *mut _; (*CSTR)[k_][c_].matanim_joint_name = pw() as *mut _; }
        fdm::CostumeListsForeachCharacter[k_].costume_list = (*UCS)[k_].as_mut_ptr();
        fdm::CostumeListsForeachCharacter[k_].numCostumes = (fv() % 5) as u8;
        for c_ in 0..4 {
          (*UCS)[k_][c_].joint = &raw mut (*JNT)[(fv() % 8) as usize]; (*UCS)[k_][c_].x4 = pw() as *mut _; (*UCS)[k_][c_].pad_x8 = fv();
          (*UCS)[k_][c_].pad_xC = fv(); (*UCS)[k_][c_].pad_x10 = fv(); (*UCS)[k_][c_].x14_archive = pw() as *mut _;
        }
      }
      for q_ in 0..(KM - 33) { fdm::GenoDefine_Costumes[q_] = if fv() & 1 != 0 { (*MEMBUF).as_mut_ptr().add((fv() % 4) as usize * 2048) as *mut UnkCostumeStruct } else { null_mut() }; fdm::GenoDefine_CostumeStrings[q_] = if fv() & 1 != 0 { (*MEMBUF).as_mut_ptr().add((fv() % 4) as usize * 2048) as *mut Fighter_CostumeStrings } else { null_mut() }; }
      for q_ in 0..6 { fdm::ft_8045993C[q_].pad_x0 = fv(); fdm::ft_8045993C[q_].pad_x4[0] = fv() as u8; fdm::ft_8045993C[q_].pad_x4[1] = fv() as u8;
                       fdm::ft_8045993C[q_].set_x6_b0((fv() & 1) as u8); fdm::ft_8045993C[q_].set_x6_b1_b2((fv() & 3) as u8); }
      { let ts = fdm::throw_state();
        for q_ in 0..6 { ts.banks[q_].kind = (fv() % 10) as i32 - 1; ts.banks[q_].buf = pw() as *mut u8; ts.banks[q_].cap = fv() as usize; ts.banks[q_].sz = fv() as usize; }
        ts.loads = fv(); ts.evicts = fv(); ts.region_cur = pw() as *mut u8;
        for k_ in 0..KM { ts.prepared[k_] = fv() as u8; } }
      for k_ in 0..KM { fdm::GenoDefine_ContentIdentity[k_][0] = fv(); fdm::GenoDefine_ContentIdentity[k_][1] = fv(); fdm::GenoDefine_InterpreterAttempts[k_] = fv(); }
      for q_ in 0..NG {
        (*FT)[q_].kind = ((*FT)[q_].kind as u32 % 27) as _; (*FT)[q_].costume_id %= 4;
        (*FT)[q_].x24 = (*PADB)[70 + q_].as_mut_ptr() as *mut Fighter_WaitAnimData; (*FT)[q_].x59C = (*PADB)[80 + q_].as_mut_ptr() as *mut _;
        (*FT)[q_].x5A4 = (fv() % 8) as usize; (*FT)[q_].x5A8 = pw() as usize; (*FT)[q_].x58C = fv() % 70;
        for j_ in 0..64usize {
          let r_ = (*PADB)[70 + q_].as_mut_ptr().cast::<Fighter_WaitAnimData>().add(j_);
          (*r_).x0 = pw() as *mut _; (*r_).x4 = (fv() % 256) as i32; (*r_).x8 = (fv() % 1024) as i32; (*r_).xC = pw() as *mut _; (*r_).x10_animCurrFlags = fv() as i32; (*r_).x14 = ((fv() % 8) as usize);
          let r_ = (*PADB)[3 + q_ * 24].as_mut_ptr().cast::<Fighter_WaitAnimData>().add(j_);
          (*r_).x0 = pw() as *mut _; (*r_).x4 = (fv() % 256) as i32; (*r_).x8 = (fv() % 1024) as i32; (*r_).xC = pw() as *mut _; (*r_).x10_animCurrFlags = fv() as i32; (*r_).x14 = ((fv() % 8) as usize);
        }
      }
    }
"""

RSHASH = r"""
    {
@HASH@
@RAWHASH@
      for k_ in 0..KM {
        hm(pi(fdm::ftData_Table_Unk0[k_].data as *const u8)); hm(fdm::ftData_Table_Unk0[k_].count as u32);
        hm(pi(fdm::ftData_UnkIntPairs[k_].data as *const u8)); hm(fdm::ftData_UnkIntPairs[k_].count as u32);
        hm(fdm::ftData_UnkBytePerCharacter[k_] as u32);
        hm(pi(fdm::CostumeListsForeachCharacter[k_].costume_list as *const u8)); hm(fdm::CostumeListsForeachCharacter[k_].numCostumes as u32);
        for c_ in 0..4 {
          hm(pi((*CSTR)[k_][c_].dat_filename as *const u8)); hm(pi((*CSTR)[k_][c_].joint_name as *const u8)); hm(pi((*CSTR)[k_][c_].matanim_joint_name as *const u8));
          hm(pi((*UCS)[k_][c_].joint as *const u8)); hm(pi((*UCS)[k_][c_].x4 as *const u8)); hm((*UCS)[k_][c_].pad_x8); hm((*UCS)[k_][c_].pad_xC);
          hm((*UCS)[k_][c_].pad_x10); hm(pi((*UCS)[k_][c_].x14_archive as *const u8));
        }
        hm(fdm::throw_state().prepared[k_] as u32); hm(fdm::GenoDefine_ContentIdentity[k_][0]); hm(fdm::GenoDefine_ContentIdentity[k_][1]); hm(fdm::GenoDefine_InterpreterAttempts[k_]);
      }
      for q_ in 0..6 { hm(fdm::ft_8045993C[q_].pad_x0); hm(fdm::ft_8045993C[q_].pad_x4[0] as u32); hm(fdm::ft_8045993C[q_].pad_x4[1] as u32); hm(fdm::ft_8045993C[q_].x6_b0() as u32); hm(fdm::ft_8045993C[q_].x6_b1_b2() as u32); }
      { let ts = fdm::throw_state();
        for q_ in 0..6 { hm(ts.banks[q_].kind as u32); hm(pi(ts.banks[q_].buf as *const u8)); hm(ts.banks[q_].cap as u32); hm(ts.banks[q_].sz as u32); }
        hm(ts.loads); hm(ts.evicts); hm(pi(ts.region_cur as *const u8)); }
      for k_ in 0..(16384 / 8) { let w_ = *(*MEMBUF2).as_ptr().cast::<u64>().add(k_); if w_ >> 32 != 0 { hm(pi(w_ as usize as *const u8)); } else { hm(w_ as u32); } }
    }
"""


def main():
    fill_c = TABLE_FILL_C
    fill_r = "      " + "\n      ".join(f"tw!(fillw; {n});" for n in FLAT)
    rawfill_r = "      " + "\n      ".join(f"for k_ in 0..KM {{ fdm::{n}[k_] = fv() as usize; }}" for n in RAWW)
    hash_r = "      " + "\n      ".join(f"th!(hashw; {n});" for n in FLAT + HAND)
    rawhash_r = "      " + "\n      ".join(f"for k_ in 0..KM {{ hm(fdm::{n}[k_] as u32); }}" for n in RAWW)
    nl_ = chr(10)
    sz_c = nl_.join(f'        printf("SZ {n} %u'+chr(92)+'n", (unsigned) sizeof '+n+');' for n in FLAT + HAND)
    sz_r = '      if std::env::var("TUSZ").is_ok() && !REGD {' + nl_ + nl_.join(f'        eprintln!("SZ {n} {{}}", core::mem::size_of_val(&*(&raw const fdm::{n})));' for n in FLAT + HAND) + nl_ + '      }'
    cworld = CWORLD.replace("@SZ_C@", sz_c).replace("@FILL@", fill_c).replace("@RAWFILL@", RAW_FILL_C)
    chash = CHASH.replace("@HASH@", TABLE_HASH_C).replace("@RAWHASH@", RAW_HASH_C)
    rsworld = RSWORLD.replace("@SZ_R@", sz_r).replace("@FILL@", fill_r).replace("@RAWFILL@", rawfill_r)
    rshash = RSHASH.replace("@HASH@", hash_r).replace("@RAWHASH@", rawhash_r)
    ops = [
        ["ft_800852B0()", "m::ft_800852B0()"],
        ["{ int k_ = (int) (fv() % KM); u8 c_ = (u8) fv(); ftData_800855C8((FighterKind) k_, c_); }",
         "{ let k_ = (fv() % KM as u32) as i32; let c_ = fv() as u8; m::ftData_800855C8(k_, c_); }"],
        ["ftData_80085B10(fp)", "m::ftData_80085B10(&mut *fp)"],
        ["{ int ms_ = (int) (fv() % 64); ftData_80085CD8(fp, &FT[(gi + 1) % NG], ms_); }",
         "{ let ms_ = (fv() % 64) as i32; m::ftData_80085CD8(fp, &raw mut (*FT)[(gi + 1) % NG], ms_); }"],
        ["GenoDefine_InitKinds()", "m::GenoDefine_InitKinds()"],
        ["{ int k_ = (int) (fv() % KM); GenoDefine_CaptureBase(k_); }",
         "{ let k_ = (fv() % KM as u32) as i32; m::GenoDefine_CaptureBase(k_); }"],
        ["ret = PI(GenoDefine_RestPoseTree(fp))", "ret = pi(m::GenoDefine_RestPoseTree(&mut *fp))"],
        ["GenoDefine_BindFighter(fp)", "m::GenoDefine_BindFighter(&mut *fp)"],
    ]
    stubs = (
        '--skip lbFile_800168A0 --skip HSD_ObjAlloc '
        '--ret "Mex_SlotInternal=-1|-1" '
        '--ret "Geno_DefineCount=(int) (fv() % 4)|(fv() % 4) as i32" '
        '--ret "Geno_DefineCKAt=(int) (fv() % 100) + 1|(fv() % 100) as i32 + 1" '
        '--ret "Geno_DefineBaseKind=(int) (fv() % 40) - 8|(fv() % 40) as i32 - 8" '
        '--ret "Geno_DefineCostumeCount=(int) (fv() % 6)|(fv() % 6) as i32" '
        '--ret "Geno_DefineThrownAs=((fv() & 1) ? -1 : a0)|if fv() & 1 != 0 { -1 } else { a0 }" '
        '--ret "Geno_DefineString=(const char*) PADB[40 + (fv() % 8)]|(*crate::PADB)[40 + (fv() % 8) as usize].as_mut_ptr() as *const core::ffi::c_char" '
        '--ret "PcCostumeAlloc=(void*) (MEMBUF + (fv() % 4) * 2048)|(*crate::MEMBUF).as_mut_ptr().add((fv() % 4) as usize * 2048) as *mut core::ffi::c_void" '
        '--ret "HSD_MemAlloc=((fv() % 8) == 0 ? (void*) 0 : (void*) (MEMBUF2 + (fv() % 8) * 2048))|if fv() % 8 == 0 { core::ptr::null_mut() } else { (*crate::MEMBUF2).as_mut_ptr().add((fv() % 8) as usize * 2048) as *mut core::ffi::c_void }" '
        ''
    )
    spec = {
        "stem": "ftdata", "csrc": "src/melee/ft/ftdata.c", "rmod": "melee_fighter::tu::ftdata", "cases": 400,
        "stubs": stubs, "ops": ops,
        "c_pre": CPRE, "c_head": CHEAD, "c_world": cworld, "c_hash": chash,
        "rs_head": RSHEAD, "rs_hooks": RSHOOKS, "rs_world": rsworld, "rs_hash": rshash,
    }
    out = FD / "gen/mtu/specs/ftdata.json"
    out.write_text(json.dumps(spec, indent=1), encoding="utf-8", newline="\n")
    print("wrote", out)


if __name__ == "__main__":
    main()
