//! Native-pointer particle TU. Synthetic oracle compares every C definition and
//! seeded opcode/state transitions; no disc-derived data.
#![allow(non_snake_case, non_upper_case_globals, unused_assignments, unused_mut, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]
use melee_types::*;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use crate::gekko::{sinf, cosf, tanf, sqrtf};
use crate::hsd_mtx::fabsf_bitwise;
use crate::particle::{KIND_TORNADO, KIND_TEX_EDGE, KIND_MIRROR_S, KIND_MIRROR_T, KIND_PRIM_ENV, KIND_TEX_INTERP_NEAR, KIND_DISP_TEXTURE, KIND_TEX_FLIP_S, KIND_TEX_FLIP_T, KIND_TRAIL, KIND_DIR_VEC};
const M_PI:f64=core::f64::consts::PI;
unsafe fn atan2f(y: f32, x: f32) -> f32 { ext::atan2f(y, x) }
pub static mut hsd_804D78D0: u32 = 0;
pub static mut psCallback: *mut Option<
    unsafe extern "C" fn(*mut HSD_Particle) -> ::core::ffi::c_int,
> = ::core::ptr::null::<
    Option<unsafe extern "C" fn(*mut HSD_Particle) -> ::core::ffi::c_int>,
>() as *mut Option<unsafe extern "C" fn(*mut HSD_Particle) -> ::core::ffi::c_int>;
pub static mut hsd_804D78D8: u16 = 0 as u16;
pub static mut hsd_804D78DA: u16 = 0 as u16;
pub static mut numPeakParticles: u16 = 0;
pub static mut hsd_804D78DE: u16 = 0 as u16;
pub static mut hsd_804D78E0: u16 = 0 as u16;
pub static mut hsd_804D78E2: u16 = 0;
pub static mut hsd_804D78E8: Option<unsafe extern "C" fn(*mut HSD_Generator, *mut [f32; 4])> = None;
pub static mut hsd_804D78EC: Option<unsafe extern "C" fn(*mut HSD_Generator)> = None;
pub static mut psCamera: *mut HSD_CObj = ::core::ptr::null::<HSD_CObj>()
    as *mut HSD_CObj;
pub static mut hsd_804D78F4: *mut HSD_SList = ::core::ptr::null_mut();
pub static mut hsd_804D08E8: [*mut HSD_JObj; 8] = [::core::ptr::null::<HSD_JObj>()
    as *mut HSD_JObj; 8];
pub static mut hsd_804D0908: [*mut HSD_Particle; 16] = [::core::ptr::null::<
    HSD_Particle,
>() as *mut HSD_Particle; 16];
pub static mut hsd_804D0948: [*mut u32; 65] = [::core::ptr::null::<u32>()
    as *mut u32; 65];
pub static mut psFormGroupArray: [i32;65] = [0;65];
pub static mut psTexGroupArray: [*mut *mut HSD_PSTexGroup; 65] = [::core::ptr::null::<
    *mut HSD_PSTexGroup,
>() as *mut *mut HSD_PSTexGroup; 65];
pub static mut psNumCmdList: [*mut *mut HSD_PSFormGroup; 65] = [::core::ptr::null::<
    *mut HSD_PSFormGroup,
>() as *mut *mut HSD_PSFormGroup; 65];
pub static mut psCmdListArray: [::core::ffi::c_int; 65] = [0; 65];
pub static mut ptclref_804D0E5C: [*mut *mut HSD_PSCmdList; 65] = [::core::ptr::null::<
    *mut HSD_PSCmdList,
>() as *mut *mut HSD_PSCmdList; 65];
pub static mut hsd_804D0F60: hsd_804D0F60_t = unsafe { core::mem::zeroed() };
static mut particle_zero: f32 = 0.0f32;

unsafe extern "C" fn ref_INC(mut o: *mut ::core::ffi::c_void) {
    if !o.is_null() {
        let ref mut fresh0 = (*(o as *mut HSD_Obj)).ref_count;
        *fresh0 = (*fresh0).wrapping_add(1);
        if (*(o as *mut HSD_Obj)).ref_count as ::core::ffi::c_int
            != -(1 as ::core::ffi::c_int) as u16 as ::core::ffi::c_int
        {} else {
            __assert(
                b"src/sysdolphin/baselib/object.h\0" as *const u8
                    as *const ::core::ffi::c_char,
                87 as u32,
                b"HSD_OBJ(o)->ref_count != HSD_OBJ_NOREF\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
    }
}

unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> i32 {
    let mut result: i32 = 0;
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            228 as u32,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    result = 0;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0
        && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
    {
        result = 1;
    }
    return result;
}

unsafe extern "C" fn HSD_JObjSetupMatrix(mut jobj: *mut HSD_JObj) {
    if jobj.is_null() || HSD_JObjMtxIsDirty(jobj) == 0 {
        return;
    }
    HSD_JObjSetupMatrixSub(jobj);
}

unsafe extern "C" fn HSD_JObjAddTranslationX(
    mut jobj: *mut HSD_JObj,
    mut x: ::core::ffi::c_float,
) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            663 as u32,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.x += x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

unsafe extern "C" fn HSD_JObjAddTranslationY(
    mut jobj: *mut HSD_JObj,
    mut y: ::core::ffi::c_float,
) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            672 as u32,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.y += y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

unsafe extern "C" fn HSD_JObjAddTranslationZ(
    mut jobj: *mut HSD_JObj,
    mut z: ::core::ffi::c_float,
) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            690 as u32,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.z += z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}

pub unsafe extern "C" fn hsd_803983A4(mut gen: *mut HSD_Generator) {
    melee_core::ct!("hsd_803983A4");
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    if gen.is_null() {
        return;
    }
    if (*gen).kind & 0x800 as u32 != 0 {
        return;
    }
    if (*gen).r#type as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int == 0 {
        return;
    }
    jobj = (*gen).jobj;
    if jobj.is_null() {
        return;
    }
    HSD_JObjSetupMatrix(jobj);
    if (*gen).r#type as ::core::ffi::c_int & 0x200 as ::core::ffi::c_int != 0 {
        (*gen).pos.x = (*(*gen).jobj)
            .mtx[0 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize];
        (*gen).pos.y = (*(*gen).jobj)
            .mtx[1 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize];
        (*gen).pos.z = (*(*gen).jobj)
            .mtx[2 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize];
    }
    if (*gen).r#type as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int != 0 {
        if !(*gen).appsrt.is_null() && (*(*gen).appsrt).gp == gen {
            (*(*gen).appsrt).translate.x = (*(*gen).jobj)
                .mtx[0 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize];
            (*(*gen).appsrt).translate.y = (*(*gen).jobj)
                .mtx[1 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize];
            (*(*gen).appsrt).translate.z = (*(*gen).jobj)
                .mtx[2 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize];
        }
    }
    if (*gen).r#type as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int != 0 {
        if !(*gen).appsrt.is_null() && (*(*gen).appsrt).gp == gen {
            HSD_MtxGetScale(
                (&raw mut (*(*gen).jobj).mtx).cast(),
                &raw mut (*(*gen).appsrt).scale,
            );
        }
    }
}

/// Native banks have pointer-sized header/table slots. The loader widens packed
/// 32-bit DAT slots before this boundary; command/texture/form records are the
/// generated native melee_types records, and all offsets remain byte offsets.
pub unsafe extern "C" fn psInitDataBankLoad(bank:i32,cmd:*const isize,tex:*const isize,refs:*const u32,form:*const isize) {
    melee_core::ct!("psInitDataBankLoad");
    if !form.is_null() { assert_eq!(*form,*tex,"particle form group count"); }
    let b=bank as usize;hsd_804D0948[b]=refs.cast_mut();psFormGroupArray[b]=*tex as i32;
    psTexGroupArray[b]=tex.add(1) as *mut *mut HSD_PSTexGroup;
    psNumCmdList[b]=if form.is_null(){core::ptr::null_mut()}else{form.add(1) as *mut *mut HSD_PSFormGroup};
    match *cmd.cast::<u16>() {
        0=>{psCmdListArray[b]=*cmd.add(1) as i32;ptclref_804D0E5C[b]=cmd.add(2) as *mut *mut HSD_PSCmdList;}
        0x40..=0x43=>{let count=*cmd.add(1) as i32;psCmdListArray[b]=(*cmd.add(2) as i32).wrapping_add(count);ptclref_804D0E5C[b]=cmd.offset(3-count as isize) as *mut *mut HSD_PSCmdList;}
        _=>panic!("psInitDataBanks: unknown version"),
    }
}
pub unsafe extern "C" fn psInitDataBankLocate(cmd:*mut isize,tex:*mut isize,form:*mut isize) {
    melee_core::ct!("psInitDataBankLocate");
    let version=*cmd.cast::<u16>();
    let (num,end,base)=match version {
        0=>{let n=*cmd.add(1) as i32;for i in 0..n {let p=cmd.offset(i as isize+2);*p=(*p).wrapping_add(cmd as isize);} (0,n,cmd.add(2))}
        0x40..=0x43=>{let n=*cmd.add(1) as i32;let count=*cmd.add(2) as i32;for i in 0..count{let p=cmd.offset(i as isize+3);if *p!=0{*p=(*p).wrapping_add(cmd as isize);}}(n,count.wrapping_add(n),cmd.offset(3-n as isize))}
        _=>(0,0,core::ptr::null_mut()),
    };
    for i in num..end {let cl=*base.offset(i as isize) as *mut HSD_PSCmdList;if !cl.is_null(){(*cl).kind=((*cl).kind&0xf1ff_ffff)|0x0800_0000;}}
    let groups=*tex as i32;
    for i in 1..=groups{let p=tex.offset(i as isize);if *p!=0{*p=(*p).wrapping_add(tex as isize);}}
    for i in 1..=groups{let tg=*tex.offset(i as isize) as *mut HSD_PSTexGroup;if tg.is_null(){continue;}
        let entries=(*tg).texTable.as_mut_ptr();
        for j in 0..(*tg).num {let e=entries.add(j as usize);if !(*e).is_null(){*e=(*e).wrapping_add(tex as usize);}}
        if (*tg).fmt!=8 && (*tg).fmt.wrapping_sub(9)>1 {continue;}
        let start=(*tg).num;let end=if (*tg).palflag&1!=0{start.wrapping_add(1)}else if (*tg).palnum!=0{start.wrapping_add((*tg).palnum as u32)}else{start.wrapping_mul(2)};
        for j in start..end{let e=entries.add(j as usize);if !(*e).is_null(){*e=(*e).wrapping_add(tex as usize);}}
    }
    if !form.is_null(){for i in 1..=groups{let p=form.offset(i as isize);if *p==0{continue;}*p=(*p).wrapping_add(form as isize);let fg=*p as *mut HSD_PSFormGroup;for j in 0..(*fg).num{let e=(*fg).formTable.as_mut_ptr().add(j as usize);if !(*e).is_null(){*e=(*e).wrapping_add(form as usize);}}}}
}
pub unsafe extern "C" fn psInitDataBank(bank:i32,cmd:*mut isize,tex:*mut isize,refs:*mut u32,form:*mut isize){ melee_core::ct!("psInitDataBank");if bank<65{psInitDataBankLocate(cmd,tex,form);psInitDataBankLoad(bank,cmd,tex,refs,form);}}

pub unsafe extern "C" fn hsd_80398A08(mut unused: u32) {
    melee_core::ct!("hsd_80398A08");
    let mut i: ::core::ffi::c_int = 0;
    HSD_ObjAllocInit(
        &raw mut hsd_804D0F60.alloc_data,
        ::core::mem::size_of::<HSD_Particle>() as u32,
        4 as u32,
    );
    
    i = 0 as ::core::ffi::c_int;
    while i < 16 as ::core::ffi::c_int {
        hsd_804D0908[i as usize] = ::core::ptr::null_mut::<HSD_Particle>();
        i += 1;
    }
    hsd_804D78E2 = 0 as u16;
    numPeakParticles = 0 as u16;
    i = 0 as ::core::ffi::c_int;
    while i < 0x41 as ::core::ffi::c_int {
        psCmdListArray[i as usize] = 0 as ::core::ffi::c_int;
        psFormGroupArray[i as usize] = 0;
        ptclref_804D0E5C[i as usize] = ::core::ptr::null_mut::<*mut HSD_PSCmdList>();
        psTexGroupArray[i as usize] = ::core::ptr::null_mut::<*mut HSD_PSTexGroup>();
        psNumCmdList[i as usize] = ::core::ptr::null_mut::<*mut HSD_PSFormGroup>();
        hsd_804D0948[i as usize] = ::core::ptr::null_mut::<u32>();
        i += 1;
    }
    psCallback = ::core::ptr::null_mut::<
        Option<unsafe extern "C" fn(*mut HSD_Particle) -> ::core::ffi::c_int>,
    >();
    hsd_804D08E8[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[3 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[4 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[5 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[6 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
    hsd_804D08E8[7 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<HSD_JObj>();
}

pub unsafe extern "C" fn psGenerateParticle0(
    mut head: *mut *mut HSD_Particle,
    mut linkNo: ::core::ffi::c_int,
    mut bank: ::core::ffi::c_int,
    mut kind: u32,
    mut texGroup: u16,
    mut list: *mut u8,
    mut life: ::core::ffi::c_int,
    mut palflag: ::core::ffi::c_int,
    mut x: f32,
    mut y: f32,
    mut z: f32,
    mut vx: f32,
    mut vy: f32,
    mut vz: f32,
    mut size: f32,
    mut grav: f32,
    mut fric: f32,
    mut gp: *mut HSD_Generator,
    mut flgInterpret: ::core::ffi::c_int,
) -> *mut HSD_Particle {
    melee_core::ct!("psGenerateParticle0");
    let mut pp: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut slot: *mut *mut HSD_Particle = ::core::ptr::null_mut::<*mut HSD_Particle>();
    let mut cmd_wait: i16 = 0;
    pp = HSD_ObjAlloc(&raw mut hsd_804D0F60.alloc_data) as *mut HSD_Particle;
    if !pp.is_null() {
        memset(
            pp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HSD_Particle>() as u32,
        );
    }
    if pp.is_null() {
        return ::core::ptr::null_mut::<HSD_Particle>();
    }
    hsd_804D78E2 = (hsd_804D78E2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
        as u16;
    if hsd_804D78E2 as ::core::ffi::c_int > numPeakParticles as ::core::ffi::c_int {
        numPeakParticles = hsd_804D78E2;
    }
    if !gp.is_null() {
        (*pp).idnum = (*gp).idnum;
    } else {
        (*pp).idnum = hsd_8039D1EC();
    }
    (*pp).appsrt = ::core::ptr::null_mut::<HSD_psAppSRT>();
    if !gp.is_null() && !(*gp).appsrt.is_null() {
        psAttachParticleAppSRT(pp, (*gp).appsrt);
    }
    if head.is_null() {
        slot = (&raw mut hsd_804D0908 as *mut *mut HSD_Particle).offset(linkNo as isize)
            as *mut *mut HSD_Particle;
        (*pp).next = *slot;
        *slot = pp;
    } else {
        (*pp).next = *head;
        *head = pp;
    }
    (*pp).bank = bank as u8;
    (*pp).linkNo = linkNo as u8;
    (*pp).kind = kind;
    (*pp).texGroup = texGroup as u8;
    (*pp).pos.x = x;
    (*pp).pos.y = y;
    (*pp).pos.z = z;
    (*pp).vel.x = vx;
    (*pp).vel.y = vy;
    (*pp).vel.z = vz;
    (*pp).size = size as ::core::ffi::c_float;
    (*pp).grav = grav as ::core::ffi::c_float;
    (*pp).fric = fric as ::core::ffi::c_float;
    (*pp).life = (life + 1 as ::core::ffi::c_int) as u16;
    (*pp).cmdList = list;
    (*pp).cmdMarkPtr = 0 as u16;
    (*pp).cmdPtr = 0 as u16;
    if palflag != 0 as ::core::ffi::c_int {
        (*pp).kind = (*pp).kind | 0x10 as u32;
    }
    if !list.is_null() {
        cmd_wait = 1 as i16;
    } else {
        cmd_wait = 0 as i16;
    }
    (*pp).cmdWait = cmd_wait as u16;
    (*pp).poseNum = 0 as u8;
    (*pp).palNum = 0xff as u8;
    (*pp).primCol.a = 0xff as u8;
    (*pp).primCol.b = 0xff as u8;
    (*pp).primCol.g = 0xff as u8;
    (*pp).primCol.r = 0xff as u8;
    (*pp).envCol.a = 0 as u8;
    (*pp).envCol.b = 0 as u8;
    (*pp).envCol.g = 0 as u8;
    (*pp).envCol.r = 0 as u8;
    (*pp).envColCount = 0 as u16;
    (*pp).primColCount = 0 as u16;
    (*pp).sizeCount = 0 as u16;
    (*pp).envColRemain = 0 as u16;
    (*pp).primColRemain = 0 as u16;
    (*pp).aCmpMode = 0x33 as u8;
    (*pp).aCmpParam1 = 1 as u8;
    (*pp).aCmpParam2 = 0xff as u8;
    (*pp).aCmpRemain = 0 as u16;
    (*pp).aCmpCount = 0 as u16;
    (*pp).rotateCount = 0 as u16;
    (*pp).rotateTarget = 0.0f32;
    (*pp).rotate = 0.0f32;
    (*pp).r#gen = gp as *mut HSD_Generator;
    if !gp.is_null() {
        (*gp).numChild = (*gp).numChild.wrapping_add(1 as u32);
    }
    (*pp).pJObjOfs = 0 as u8;
    (*pp).matColRemain = 0 as u16;
    (*pp).matColCount = 0 as u16;
    (*pp).matRGB = 0xff as u8;
    (*pp).matA = 0xff as u8;
    (*pp).ambColRemain = 0 as u16;
    (*pp).ambColCount = 0 as u16;
    (*pp).ambRGB = 0xff as u8;
    (*pp).ambA = 0xff as u8;
    (*pp).trail = 1.0f32;
    (*pp).userdata = ::core::ptr::null_mut::<::core::ffi::c_float>();
    if !gp.is_null() {
        if !(*gp).userfunc.is_null() {
            if (*(*gp).userfunc).hookCreate.is_some() {
                (*(*gp).userfunc).hookCreate.expect("non-null function pointer")(pp);
            }
        }
    }
    (*pp).callback = None;
    if flgInterpret != 0 as ::core::ffi::c_int {
        hsd_8039930C(pp, ::core::ptr::null_mut::<HSD_Particle>());
    }
    return pp;
}

pub unsafe extern "C" fn hsd_80398F0C(
    mut linkNo: i32,
    mut bank: i32,
    mut kind: i32,
    mut texGroup: u16,
    mut cmdList: *mut u8,
    mut life: i32,
    mut zero: i32,
    mut gen: *mut HSD_Generator,
    mut pos_x: f32,
    mut pos_y: f32,
    mut pos_z: f32,
    mut vel_x: f32,
    mut vel_y: f32,
    mut vel_z: f32,
    mut fric: f32,
    mut rate: f32,
    mut angle3: f32,
) {
    melee_core::ct!("hsd_80398F0C");
    psGenerateParticle0(
        ::core::ptr::null_mut::<*mut HSD_Particle>(),
        linkNo as ::core::ffi::c_int,
        bank as ::core::ffi::c_int,
        kind as u32,
        texGroup,
        cmdList as *mut u8,
        life as ::core::ffi::c_int,
        zero as ::core::ffi::c_int,
        pos_x,
        pos_y,
        pos_z,
        vel_x,
        vel_y,
        vel_z,
        fric,
        rate,
        angle3,
        gen as *mut HSD_Generator,
        1 as ::core::ffi::c_int,
    );
}

pub unsafe extern "C" fn hsd_803991D8(
    mut gen: *mut HSD_Generator,
    mut jobj: *mut HSD_JObj,
    mut force: f32,
    mut range: f32,
) -> i32 {
    melee_core::ct!("hsd_803991D8");
    let mut dx: f32 = 0.;
    let mut dy: f32 = 0.;
    let mut dz: f32 = 0.;
    let mut dist_sq: f32 = 0.;
    let mut scale: f32 = 0.;
    if jobj.is_null() || range < 0.0f32 {
        return 0 as i32;
    }
    HSD_JObjSetupMatrix(jobj);
    dx = (*jobj).mtx[0 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize]
        - (*gen).fric;
    dy = (*jobj).mtx[1 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize]
        - (*gen).size;
    dz = (*jobj).mtx[2 as ::core::ffi::c_int as usize][3 as ::core::ffi::c_int as usize]
        - (*gen).radius;
    dist_sq = fmadds(dz,dz,fmadds(dx,dx,dy*dy));
    if dist_sq <= range * range {
        return 1 as i32;
    }
    if dist_sq == 0.0f32 {
        return 0 as i32;
    }
    scale = force / dist_sq;
    (*gen).pos.z = fmadds(scale,dx,(*gen).pos.z);
    (*gen).vel.x = fmadds(scale,dy,(*gen).vel.x);
    (*gen).vel.y = fmadds(scale,dz,(*gen).vel.y);
    return 0 as i32;
}

pub unsafe extern "C" fn psEnableTexture(
    mut pp: *mut HSD_Particle,
    mut textures: *const *mut u8,
) {
    melee_core::ct!("psEnableTexture");
    if !textures.is_null() && !(*textures.offset((*pp).poseNum as isize)).is_null() {
        (*pp).kind |= DispTexture as ::core::ffi::c_int as u32;
    }
}

pub unsafe extern "C" fn psSpawnChild(
    mut head: *mut *mut HSD_Particle,
    mut linkNo: ::core::ffi::c_int,
    mut bank: ::core::ffi::c_int,
    mut idx: ::core::ffi::c_int,
) -> *mut HSD_Particle {
    melee_core::ct!("psSpawnChild");
    let mut child: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut cl: *mut HSD_PSCmdList = ::core::ptr::null_mut::<HSD_PSCmdList>();
    let mut tg: *mut HSD_PSTexGroup = ::core::ptr::null_mut::<HSD_PSTexGroup>();
    let mut palflag: ::core::ffi::c_int = 0;
    if linkNo >= 8 as ::core::ffi::c_int {
        child = ::core::ptr::null_mut::<HSD_Particle>();
    } else if bank >= 65 as ::core::ffi::c_int {
        child = ::core::ptr::null_mut::<HSD_Particle>();
    } else if idx >= psCmdListArray[bank as usize] {
        child = ::core::ptr::null_mut::<HSD_Particle>();
    } else {
        cl = *ptclref_804D0E5C[bank as usize].offset(idx as isize);
        if cl.is_null() {
            child = ::core::ptr::null_mut::<HSD_Particle>();
        } else {
            tg = *psTexGroupArray[bank as usize].offset((*cl).texGroup as isize);
            if !tg.is_null() {
                palflag = (*tg).palflag as ::core::ffi::c_int;
            } else {
                palflag = 0 as ::core::ffi::c_int;
            }
            child = psGenerateParticle0(
                head,
                linkNo,
                bank,
                (*cl).kind,
                (*cl).texGroup,
                &raw mut (*cl).cmdList as *mut u8,
                (*cl).life as ::core::ffi::c_int,
                palflag,
                0.0f32,
                0.0f32,
                0.0f32,
                (*cl).vx as f32,
                (*cl).vy as f32,
                (*cl).vz as f32,
                (*cl).size as f32,
                (*cl).grav as f32,
                (*cl).fric as f32,
                ::core::ptr::null_mut::<HSD_Generator>(),
                0 as ::core::ffi::c_int,
            );
        }
    }
    return child;
}

pub unsafe extern "C" fn hsd_8039CEAC(mut mask: u32) {
    melee_core::ct!("hsd_8039CEAC");
    let mut cur: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >();
    let mut prev: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >();
    let mut i: i32 = 0;
    let mut bits: u32 = 0;
    let mut next: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >();
    bits = mask;
    i = 0 as ::core::ffi::c_int as i32;
    while i < 16 as ::core::ffi::c_int {
        if bits & 0x10000 as ::core::ffi::c_int as u32 == 0 {
            let mut owned: ::core::ffi::c_int = ext::ScriptGame_StageSlotParticleBegin(
                i as ::core::ffi::c_int,
            );
            cur = hsd_804D0908[i as usize] as *mut ::core::ffi::c_void;
            prev = ::core::ptr::null_mut::<::core::ffi::c_void>();
            while !cur.is_null() {
                next = hsd_8039930C(cur as *mut HSD_Particle, prev as *mut HSD_Particle);
                if prev.is_null() {
                    if hsd_804D0908[i as usize] != next as *mut HSD_Particle {
                        prev = cur;
                    }
                } else if next != *(prev as *mut *mut ::core::ffi::c_void) {
                    prev = cur;
                }
                cur = next;
            }
            if owned != 0 {
                ext::ScriptGame_StageSlotProcEnd();
            }
        }
        bits >>= 1 as ::core::ffi::c_int;
        i += 1;
    }
}

pub unsafe extern "C" fn hsd_8039CF4C(mut index: i32, mut jobj: *mut HSD_JObj) {
    melee_core::ct!("hsd_8039CF4C");
    if index < 0 as ::core::ffi::c_int || index > 8 as ::core::ffi::c_int {
        return;
    }
    if index != 0 as ::core::ffi::c_int {
        let mut p: *mut *mut HSD_JObj = &raw mut hsd_804D08E8 as *mut *mut HSD_JObj;
        let mut old: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        p = p.offset(index as isize);
        p = p.offset(-1);
        old = *p;
        if old != jobj {
            if !old.is_null() {
                HSD_JObjUnref(jobj);
            }
            *p = jobj;
            ref_INC(jobj as *mut ::core::ffi::c_void);
        }
    } else {
        let mut i: i32 = 0;
        i = 0 as ::core::ffi::c_int as i32;
        while i < 8 as ::core::ffi::c_int {
            if hsd_804D08E8[i as usize] == jobj {
                HSD_JObjUnref(hsd_804D08E8[i as usize]);
                hsd_804D08E8[i as usize] = ::core::ptr::null_mut::<HSD_JObj>();
            }
            i += 1;
        }
    };
}

pub unsafe extern "C" fn hsd_8039D048(mut particle: *mut ::core::ffi::c_void) {
    melee_core::ct!("hsd_8039D048");
    let mut flags: u32 = (*(particle as *mut HSD_Particle)).kind;
    if flags & 0x8000 as u32 != 0 {
        let mut p: *mut *mut HSD_JObj = (&raw mut hsd_804D08E8 as *mut *mut HSD_JObj)
            .offset((flags >> 12 as ::core::ffi::c_int & 7 as u32) as isize)
            as *mut *mut HSD_JObj;
        if !(*p).is_null() {
            HSD_JObjUnref(*p);
            *p = ::core::ptr::null_mut::<HSD_JObj>();
        }
    }
}

pub unsafe extern "C" fn HSD_StageSlotParticleLinkClear() {
    melee_core::ct!("HSD_StageSlotParticleLinkClear");
    let mut p: *mut HSD_Particle = hsd_804D0908[3 as ::core::ffi::c_int as usize];
    hsd_804D0908[3 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Particle,
    >();
    while !p.is_null() {
        let mut next: *mut HSD_Particle = (*p).next;
        if !(*p).r#gen.is_null() && !(*(*p).r#gen).userfunc.is_null()
            && (*(*(*p).r#gen).userfunc).hookDelete.is_some()
        {
            (*(*(*p).r#gen).userfunc).hookDelete.expect("non-null function pointer")(p);
        }
        if !(*p).r#gen.is_null() {
            (*(*p).r#gen).numChild = (*(*p).r#gen).numChild.wrapping_sub(1);
        }
        if !(*p).appsrt.is_null() {
            psRemoveParticleAppSRT(p);
        }
        hsd_8039D048(p as *mut ::core::ffi::c_void);
        HSD_ObjFree(&raw mut hsd_804D0F60.alloc_data, p as *mut ::core::ffi::c_void);
        hsd_804D78E2 = hsd_804D78E2.wrapping_sub(1);
        p = next;
    }
}

pub unsafe extern "C" fn hsd_8039D0A0(mut gen: *mut HSD_Generator) {
    melee_core::ct!("hsd_8039D0A0");
    let mut prev: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut prt: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut next: *mut HSD_Particle = ::core::ptr::null_mut::<HSD_Particle>();
    let mut head: *mut *mut HSD_Particle = ::core::ptr::null_mut::<*mut HSD_Particle>();
    let mut idnum: u16 = 0;
    prev = ::core::ptr::null_mut::<HSD_Particle>();
    idnum = (*gen).idnum;
    head = (&raw mut hsd_804D0908 as *mut *mut HSD_Particle)
        .offset((*gen).linkNo as isize) as *mut *mut HSD_Particle;
    prt = *head;
    while !prt.is_null() {
        next = (*prt).next;
        if (*prt).idnum as ::core::ffi::c_int == idnum as ::core::ffi::c_int
            && !(*prt).r#gen.is_null() && (*prt).r#gen == gen
        {
            if !(*prt).r#gen.is_null() && !(*(*prt).r#gen).userfunc.is_null()
                && (*(*(*prt).r#gen).userfunc).hookDelete.is_some()
            {
                (*(*(*prt).r#gen).userfunc)
                    .hookDelete
                    .expect("non-null function pointer")(prt);
            }
            if prev.is_null() {
                *head = (*prt).next;
            } else {
                (*prev).next = (*prt).next;
            }
            if !(*prt).r#gen.is_null() {
                (*(*prt).r#gen).numChild = (*(*prt).r#gen).numChild.wrapping_sub(1);
            }
            if !(*prt).appsrt.is_null() {
                psRemoveParticleAppSRT(prt);
            }
            if (*prt).kind & 0x8000 as u32 != 0 {
                let mut jidx: i32 = ((*prt).kind >> 12 as ::core::ffi::c_int
                    & 7 as u32) as i32;
                if !hsd_804D08E8[jidx as usize].is_null() {
                    HSD_JObjUnref(hsd_804D08E8[jidx as usize]);
                    hsd_804D08E8[jidx as usize] = ::core::ptr::null_mut::<HSD_JObj>();
                }
            }
            HSD_ObjFree(
                &raw mut hsd_804D0F60.alloc_data,
                prt as *mut ::core::ffi::c_void,
            );
            hsd_804D78E2 = hsd_804D78E2.wrapping_sub(1);
        } else {
            prev = prt;
        }
        prt = next;
    }
}

/// Byte order of the float operands in a command stream. The stream is raw game data: opcodes and operands are read
/// byte-wise, and the C (big-endian memory) assembles a float from its four bytes MSB first. The engine's banks keep
/// the DAT's stream bytes as they are, so it sets this; the synthetic oracle streams are native-endian (default).
pub static mut CMD_STREAM_BIG_ENDIAN: bool = false;
#[inline]
unsafe fn stream_word(b: [u8; 4]) -> u32 {
    if CMD_STREAM_BIG_ENDIAN { u32::from_be_bytes(b) } else { u32::from_ne_bytes(b) }
}

pub unsafe extern "C" fn psReadFloat(stream: *mut *mut u8) {
    melee_core::ct!("psReadFloat");
    hsd_804D78D0 = stream_word([**stream, *(*stream).add(1), *(*stream).add(2), *(*stream).add(3)]);
    *stream=(*stream).add(4);
}

pub unsafe extern "C" fn hsd_80398F8C(p: *mut HSD_Particle, angle: f32) {
        melee_core::ct!("hsd_80398F8C");
        let v = (&*p).vel;
        let (vx, vy, vz) = (v.x, v.y, v.z);
        let azimuth = if fabsf_bitwise(vz) < 1.175_494_4e-38 {
            if vy >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
        } else {
            atan2f(vy, vz)
        };
        let sin_a = sinf(azimuth);
        let cos_a = cosf(azimuth);
        // 556.
        let temp = fmadds(vy, sin_a, vz * cos_a);
        let az2 = if fabsf_bitwise(temp) < 1.175_494_4e-38 {
            if vx >= 0.0 { 1.570_796_4 } else { -1.570_796_4 }
        } else {
            atan2f(vx, temp)
        };
        let sin_e = sinf(az2);
        let cos_e = cosf(az2);
        // 573: (vx*vx + vy*vy) + vz*vz, both fused; then the inline sqrt (576-579).
        let mut mag = fmadds(vz, vz, fmadds(vx, vx, vy * vy));
        if mag > 0.0 {
            mag = sqrtf(mag);
        }
        let rand_angle = ((M_PI * ext::HSD_Randf() as f64) * 2.0) as f32;
        let sin_angle = mag * sinf(angle);
        let cos_rand = sin_angle * cosf(rand_angle);
        let sin_rand = sin_angle * sinf(rand_angle);
        let cos_angle = mag * cosf(angle);
        let pp = (&mut *p);
        // 591-595.
        pp.vel.x = fmadds(cos_rand, cos_e, cos_angle * sin_e);
        pp.vel.y = fmadds(cos_e, cos_angle * sin_a, fmadds(sin_e, -cos_rand * sin_a, sin_rand * cos_a));
        pp.vel.z = fmadds(cos_e, cos_angle * cos_a, fmadds(sin_e, -cos_rand * cos_a, -(sin_rand * sin_a)));
    }
fn col_step(cur: u8, target: u8, step: i32) -> u8 {
        (((target as i32) << 16).wrapping_add(step.wrapping_mul(cur as i32 - target as i32)) >> 16) as u8
    }
fn col_interp(cur: &mut _GXColor, target: _GXColor, remain: u16, count: u16) {
        if count != 0 {
            let step = ((remain as i32) << 16) / count as i32;
            cur.r = col_step(cur.r, target.r, step);
            cur.g = col_step(cur.g, target.g, step);
            cur.b = col_step(cur.b, target.b, step);
            cur.a = col_step(cur.a, target.a, step);
        }
    }
fn col_add(target: u8, delta: f32) -> u8 {
        let mut val = target as f32 + delta;
        if val < 0.0 {
            val = 0.0;
        }
        if val > 255.0 {
            val = 255.0;
        }
        val as i32 as u8
    }
pub unsafe fn interpret(p: *mut HSD_Particle, prev: *mut HSD_Particle) -> *mut HSD_Particle {
        melee_core::ct!("interpret");
        if (&*p).kind & 0x800 != 0 {
            return (&*p).next;
        }
        {
            let pp = (&mut *p);
            if pp.sizeCount != 0 {
                pp.size += (pp.sizeTarget - pp.size) / pp.sizeCount as f32;
                pp.sizeCount -= 1;
            }
            if pp.primColCount != 0 {
                pp.primColRemain = pp.primColRemain.wrapping_sub(1);
                if pp.primColRemain == 0 {
                    pp.primColCount = 0;
                    pp.primCol = pp.primColTarget;
                }
            }
            if pp.envColCount != 0 {
                pp.envColRemain = pp.envColRemain.wrapping_sub(1);
                if pp.envColRemain == 0 {
                    pp.envColCount = 0;
                    pp.envCol = pp.envColTarget;
                }
            }
            if pp.matColCount != 0 {
                pp.matColRemain = pp.matColRemain.wrapping_sub(1);
                if pp.matColRemain == 0 {
                    pp.matColCount = 0;
                    pp.matRGB = pp.matRGBTarget;
                    pp.matA = pp.matATarget;
                }
            }
            if pp.ambColCount != 0 {
                pp.ambColRemain = pp.ambColRemain.wrapping_sub(1);
                if pp.ambColRemain == 0 {
                    pp.ambColCount = 0;
                    pp.ambRGB = pp.ambRGBTarget;
                    pp.ambA = pp.ambATarget;
                }
            }
            if pp.aCmpCount != 0 {
                pp.aCmpRemain = pp.aCmpRemain.wrapping_sub(1);
                if pp.aCmpRemain == 0 {
                    pp.aCmpCount = 0;
                    pp.aCmpParam1 = pp.aCmpParam1Target;
                    pp.aCmpParam2 = pp.aCmpParam2Target;
                }
            }
            if pp.rotateCount != 0 {
                pp.rotate += (pp.rotateTarget - pp.rotate) / pp.rotateCount as f32;
                pp.rotateCount -= 1;
            }
        }

        let mut run = false;
        {
            let pp = (&mut *p);
            if pp.cmdWait != 0 {
                pp.cmdWait -= 1;
                run = pp.cmdWait == 0;
            }
        }
        if run {
            run_bytecode(p);
        }

        // do_life
        {
            let pp = (&mut *p);
            pp.life = pp.life.wrapping_sub(1);
        }
        if (&*p).life == 0 {
            return delete_particle(p, prev);
        }

        // Physics.
        if (&*p).kind & KIND_TORNADO != 0 {
            let g = (*p).r#gen;
            let gp = (&*g).clone();
            let pp = (&mut *p);
            let sin_a = sinf(pp.grav);
            let sin_b = sinf(pp.fric);
            let cos_a = cosf(pp.grav);
            let cos_b = cosf(pp.fric);
            pp.vel.z += gp.aux.tornado.vel;
            let mut r = if gp.radius < 0.0 { -gp.radius } else { gp.radius };
            let ang = if gp.angle < 0.0 { -gp.angle } else { gp.angle };
            // 2895: R += vel.z * tanf(ang).
            r = fmadds(pp.vel.z, tanf(ang), r);
            r *= pp.vel.y;
            pp.vel.x += gp.grav;
            let d = r * cosf(pp.vel.x);
            let e = r * sinf(pp.vel.x);
            let nd = -d;
            let vz = pp.vel.z;
            // 2905-2909.
            pp.pos.x = gp.pos.x + fmadds(d, cos_b, vz * sin_b);
            pp.pos.y = gp.pos.y + fmadds(cos_b, vz * sin_a, fmadds(sin_b, nd * sin_a, e * cos_a));
            pp.pos.z = gp.pos.z + fmadds(cos_b, vz * cos_a, fmadds(sin_b, nd * cos_a, -(e * sin_a)));
        } else {
            let pp = (&mut *p);
            if pp.kind & 1 != 0 {
                pp.vel.y -= pp.grav;
            }
            if pp.kind & 2 != 0 {
                pp.vel.x *= pp.fric;
                pp.vel.y *= pp.fric;
                pp.vel.z *= pp.fric;
            }
            pp.pos.x += pp.vel.x;
            pp.pos.y += pp.vel.y;
            pp.pos.z += pp.vel.z;
        }

        if (&*p).kind & 0x8000 != 0 {
            let slot = (((&*p).kind & 0x7000) >> 12) as usize;
            if hsd_804D08E8[slot].is_null() {
                if let Some(j) = ptr_option(ext::HSD_JObjAlloc()) {
                    hsd_8039CF4C(slot as i32 + 1, j);
                    ext::HSD_JObjUnref(j);
                }
            }
            if let Some(j) = ptr_option(hsd_804D08E8[slot]) {
                let m = { HSD_JObjSetupMatrix(j); (*j).mtx };
                let pos = (&*p).pos;
                { HSD_JObjAddTranslationX(j, pos.x-m[0][3]); HSD_JObjAddTranslationY(j, pos.y-m[1][3]); HSD_JObjAddTranslationZ(j, pos.z-m[2][3]); };
            }
        }

        if let Some(cb) = (&*p).callback {
            if cb(p) == -1 {
                return delete_particle(p, prev);
            }
        }
        (&*p).next
    }
unsafe fn run_bytecode(p: *mut HSD_Particle) {
        let list = (*p).cmdList;
        let mut pc: u32 = (&*p).cmdPtr as u32;
        let mut operand: u16;
        macro_rules! rd {
            () => {{
                let b = *list.add(pc as usize);
                pc += 1;
                b
            }};
        }
        // psReadFloat: four bytes in stream order into a float (big-endian on the console).
        macro_rules! rdf {
            () => {{
                let b = [*list.add(pc as usize), *list.add(pc as usize + 1), *list.add(pc as usize + 2), *list.add(pc as usize + 3)];
                pc += 4;
                hsd_804D78D0=stream_word(b); f32::from_bits(hsd_804D78D0)
            }};
        }
        // A count byte with the 0x80 two-byte extension.
        macro_rules! rdcount {
            () => {{
                let mut cnt = rd!() as u16;
                if cnt & 0x80 != 0 {
                    cnt = ((cnt & 0x7F) << 8) + rd!() as u16;
                }
                cnt
            }};
        }
        macro_rules! pp {
            () => {
                (&mut *p)
            };
        }
        'outer: loop {
            let opcode = rd!();
            if opcode < 0x80 {
                operand = (opcode & 0x1F) as u16;
                if opcode & 0x20 != 0 {
                    operand = (operand << 8) + rd!() as u16;
                }
                if opcode & 0xC0 == 0x40 {
                    let pn = rd!();
                    pp!().poseNum = pn;
                    enable_texture(p);
                }
            } else {
                operand = 0;
                let mut cls = opcode & 0xF8;
                if cls > 0x98 {
                    cls = opcode & 0xF0;
                    if cls != 0xC0 && cls != 0xD0 {
                        cls = opcode;
                    }
                }
                match cls {
                    0x80 | 0x88 | 0x90 | 0x98 => {
                        for (bit, axis) in [(1u8, 0), (2, 1), (4, 2)] {
                            if opcode & bit != 0 {
                                let f = rdf!();
                                let pp = pp!();
                                let v = match cls {
                                    0x80 | 0x88 => &mut pp.pos,
                                    _ => &mut pp.vel,
                                };
                                let c = match axis {
                                    0 => &mut v.x,
                                    1 => &mut v.y,
                                    _ => &mut v.z,
                                };
                                if cls == 0x80 || cls == 0x90 {
                                    *c = f;
                                } else {
                                    *c += f;
                                }
                            }
                        }
                    }
                    0xA0 => {
                        let cnt = rdcount!();
                        let f = rdf!();
                        let pp = pp!();
                        pp.sizeCount = cnt;
                        pp.sizeTarget = f;
                        if pp.sizeCount == 0 {
                            pp.size = pp.sizeTarget;
                        }
                    }
                    0xA1 => pp!().kind &= !KIND_DISP_TEXTURE,
                    0xA2 => {
                        let f = rdf!();
                        let pp = pp!();
                        pp.grav = f;
                        if pp.grav == 0.0 {
                            pp.kind &= !1;
                        } else {
                            pp.kind |= 1;
                        }
                    }
                    0xA3 => {
                        let f = rdf!();
                        let pp = pp!();
                        pp.fric = f;
                        if pp.fric == 1.0 {
                            pp.kind &= !2;
                        } else {
                            pp.kind |= 2;
                        }
                    }
                    0xA4 | 0xF1 | 0xB9 | 0xF2 => {
                        let mut idx = ((*list.add(pc as usize) as i32) << 8) + *list.add(pc as usize + 1) as i32;
                        pc += 2;
                        if cls == 0xF1 || cls == 0xF2 {
                            idx = remap((&*p).bank, idx);
                        }
                        if let Some(c) = ptr_option(psSpawnChild(&raw mut (*p).next, (*p).linkNo as i32, (*p).bank as i32, idx)) {
                            let src = (&*p).clone();
                            if cls == 0xB9 || cls == 0xF2 {
                                let cp = (&mut *c);
                                cp.pos = src.pos;
                                cp.vel = src.vel;
                                cp.idnum = src.idnum;
                                cp.r#gen = src.r#gen;
                            } else {
                                let cp = (&mut *c);
                                cp.idnum = src.idnum;
                                cp.r#gen = src.r#gen;
                            }
                            if let Some(g) = ptr_option(src.r#gen) {
                                let gg = (&mut *g);
                                gg.numChild = gg.numChild.wrapping_add(1);
                            }
                            if cls==0xA4 || cls==0xF1 || !src.appsrt.is_null() { ext::psAttachParticleAppSRT(c, src.appsrt); }
                            if cls == 0xA4 || cls == 0xF1 {
                                (&mut *c).pos = src.pos;
                            }
                            interpret(c, p);
                        }
                    }
                    0xA5 => {
                        let idx = ((*list.add(pc as usize) as i32) << 8) + *list.add(pc as usize + 1) as i32;
                        pc += 2;
                        spawn_child_generator(p, idx, None);
                    }
                    0xEF | 0xF0 => {
                        let mut idx = ((*list.add(pc as usize) as i32) << 8) + *list.add(pc as usize + 1) as i32;
                        let flags = *list.add(pc as usize + 2);
                        pc += 3;
                        if cls == 0xF0 {
                            idx = remap((&*p).bank, idx);
                        }
                        spawn_child_generator(p, idx, Some(flags));
                    }
                    0xA6 => {
                        let base = ((rd!() as i32) << 8) + rd!() as i32;
                        let range = ((rd!() as i32) << 8) + rd!() as i32;
                        let r = ext::HSD_Randf();
                        pp!().life = base.wrapping_add((range as f32 * r) as i32) as u16;
                    }
                    0xA7 => {
                        let threshold = rd!() as i32;
                        let r = ext::HSD_Randf();
                        if threshold >= (100.0f32 * r) as i32 {
                            pp!().life = 1;
                            break 'outer;
                        }
                    }
                    0xA8 => {
                        for axis in 0..3 {
                            let f = rdf!();
                            let r = ext::HSD_Randf();
                            // 1300/1306/1312: (2.0F * fval) * Randf() - fval, fused.
                            let d = fmadds(2.0 * f, r, -f);
                            let pp = pp!();
                            match axis {
                                0 => pp.pos.x += d,
                                1 => pp.pos.y += d,
                                _ => pp.pos.z += d,
                            }
                        }
                    }
                    0xA9 => {
                        let f = rdf!();
                        hsd_80398F8C(p, f);
                    }
                    0xAA => {
                        let mut idx = ((rd!() as i32) << 8) + rd!() as i32;
                        let range = ((rd!() as i32) << 8) + rd!() as i32;
                        let r = ext::HSD_Randf();
                        idx += (range as f32 * r) as i32;
                        idx = remap((&*p).bank, idx);
                        if let Some(c) = ptr_option(psSpawnChild(&raw mut (*p).next, (*p).linkNo as i32, (*p).bank as i32, idx)) {
                            let src = (&*p).clone();
                            let cp = (&mut *c);
                            cp.pos = src.pos;
                            cp.idnum = src.idnum;
                            cp.r#gen = src.r#gen;
                            if let Some(g) = ptr_option(src.r#gen) {
                                let gg = (&mut *g);
                                gg.numChild = gg.numChild.wrapping_add(1);
                            }
                            if src.appsrt.is_null() == false {
                                ext::psAttachParticleAppSRT(c, src.appsrt);
                            }
                            interpret(c, p);
                        }
                    }
                    0xAB => {
                        let s = rdf!();
                        let pp = pp!();
                        pp.vel.x *= s;
                        pp.vel.y *= s;
                        pp.vel.z *= s;
                    }
                    0xAC => {
                        let cnt = rdcount!();
                        let target = rdf!();
                        let range = rdf!();
                        let r = ext::HSD_Randf();
                        let pp = pp!();
                        pp.sizeCount = cnt;
                        // 1428: sizeTarget += range * Randf().
                        pp.sizeTarget = fmadds(range, r, target);
                        if pp.sizeCount == 0 {
                            pp.size = pp.sizeTarget;
                        }
                    }
                    0xAD => pp!().kind |= KIND_PRIM_ENV,
                    0xAE => pp!().kind &= !(KIND_MIRROR_S | KIND_MIRROR_T),
                    0xAF => {
                        let pp = pp!();
                        pp.kind &= !KIND_MIRROR_T;
                        pp.kind |= KIND_MIRROR_S;
                    }
                    0xB0 => {
                        let pp = pp!();
                        pp.kind &= !KIND_MIRROR_S;
                        pp.kind |= KIND_MIRROR_T;
                    }
                    0xB1 => pp!().kind |= KIND_MIRROR_S | KIND_MIRROR_T,
                    0xB2 => {
                        if let Some(s) = ptr_option((&*p).appsrt) {
                            if (&*s).xA2 == 0 {
                                let gp = (&*s).gp;
                                hsd_803983A4(gp);
                                ext::HSD_MtxSRT((&raw mut (*s).mmtx).cast(), &raw mut (*s).scale, (&raw mut (*s).rot).cast(), &raw mut (*s).translate, core::ptr::null_mut());
                                let m = (*s).mmtx;
                                let pp = (&mut *p);
                                // 1485-1496: each row two fused adds, then + m[i][3]; the
                                // rows read the already-updated components, as the C does.
                                pp.pos.x = fmadds(m[0][2], pp.pos.z, fmadds(m[0][0], pp.pos.x, m[0][1] * pp.pos.y)) + m[0][3];
                                pp.pos.y = fmadds(m[1][2], pp.pos.z, fmadds(m[1][0], pp.pos.x, m[1][1] * pp.pos.y)) + m[1][3];
                                pp.pos.z = fmadds(m[2][2], pp.pos.z, fmadds(m[2][0], pp.pos.x, m[2][1] * pp.pos.y)) + m[2][3];
                                ext::psRemoveParticleAppSRT(p);
                            }
                        }
                    }
                    0xB3 => {
                        {
                            let pp = pp!();
                            if pp.aCmpCount != 0 {
                                let step = ((pp.aCmpRemain as i32) << 16) / pp.aCmpCount as i32;
                                pp.aCmpParam1 = col_step(pp.aCmpParam1, pp.aCmpParam1Target, step);
                                pp.aCmpParam2 = col_step(pp.aCmpParam2, pp.aCmpParam2Target, step);
                            }
                        }
                        let cnt = rdcount!();
                        let mode = rd!();
                        let t1 = rd!();
                        let t2 = rd!();
                        let pp = pp!();
                        pp.aCmpCount = cnt;
                        pp.aCmpMode = mode;
                        pp.aCmpParam1Target = t1;
                        pp.aCmpParam2Target = t2;
                        if pp.aCmpCount == 0 {
                            pp.aCmpParam1 = pp.aCmpParam1Target;
                            pp.aCmpParam2 = pp.aCmpParam2Target;
                            pp.aCmpRemain = 0;
                            pp.aCmpCount = 0;
                        } else {
                            pp.aCmpRemain = pp.aCmpCount;
                        }
                    }
                    0xB4 => pp!().kind |= KIND_TEX_INTERP_NEAR,
                    0xB5 => pp!().kind &= !KIND_TEX_INTERP_NEAR,
                    0xB6 => {
                        let cnt = rdcount!();
                        let f = rdf!();
                        let pp = pp!();
                        pp.rotateCount = cnt;
                        pp.rotateTarget += f;
                        if pp.rotateCount == 0 {
                            pp.rotate = pp.rotateTarget;
                        }
                    }
                    0xB7 => {
                        // TARGET_PC masks the slot index to 3 bits.
                        let slot = ((rd!() as u32 + (&*p).pJObjOfs as u32) & 7) as usize;
                        if let Some(j) = ptr_option(hsd_804D08E8[slot]) {
                            let m = { HSD_JObjSetupMatrix(j); (*j).mtx };
                            let pp = pp!();
                            // 1602: vx*vx + vy*vy fused.
                            let val = fmadds(pp.vel.x, pp.vel.x, pp.vel.y * pp.vel.y);
                            let mut vel_mag = pp.vel.z * pp.vel.z;
                            vel_mag += val;
                            let dx = m[0][3] - pp.pos.x;
                            let mut dy = m[1][3];
                            dy -= pp.pos.y;
                            let dz = m[2][3] - pp.pos.z;
                            if vel_mag > 0.0 {
                                vel_mag = sqrtf(vel_mag);
                            }
                            // 1620-1621.
                            let val = fmadds(dx, dx, dy * dy);
                            let mut dist = fmadds(dz, dz, val);
                            if dist != 0.0 {
                                if dist > 0.0 {
                                    dist = sqrtf(dist);
                                }
                                let scale = vel_mag / dist;
                                pp.vel = Vec { x: dx * scale, y: dy * scale, z: dz * scale };
                            }
                        }
                    }
                    0xB8 => {
                        let idx = rd!() as u32 + (&*p).pJObjOfs as u32;
                        let force = rdf!();
                        let range = rdf!();
                        let j = hsd_804D08E8[(idx & 7) as usize];
                        if hsd_803991D8(p.cast(), j, force, range) != 0 {
                            pp!().life = 1;
                            break 'outer;
                        }
                    }
                    0xBA | 0xBB => {
                        let prim = cls == 0xBA;
                        {
                            let pp = pp!();
                            if prim {
                                let t = pp.primColTarget;
                                col_interp(&mut pp.primCol, t, pp.primColRemain, pp.primColCount);
                            } else {
                                let t = pp.envColTarget;
                                col_interp(&mut pp.envCol, t, pp.envColRemain, pp.envColCount);
                            }
                        }
                        for ch in 0..4 {
                            let r = ext::HSD_Randf();
                            let delta = rd!() as i8;
                            let rv = ((delta as i32) << 1) as f32 * r;
                            let pp = pp!();
                            let t = if prim { &mut pp.primColTarget } else { &mut pp.envColTarget };
                            let c = match ch {
                                0 => &mut t.r,
                                1 => &mut t.g,
                                2 => &mut t.b,
                                _ => &mut t.a,
                            };
                            *c = col_add(*c, rv);
                        }
                        let pp = pp!();
                        if prim {
                            if pp.primColCount == 0 {
                                pp.primCol = pp.primColTarget;
                            } else {
                                pp.primColRemain = pp.primColCount;
                            }
                        } else if pp.envColCount == 0 {
                            pp.envCol = pp.envColTarget;
                        } else {
                            pp.envColRemain = pp.envColCount;
                        }
                    }
                    0xBC => {
                        let pn = rd!();
                        let range = rd!() as f32;
                        let r = ext::HSD_Randf();
                        // 1978: randRange * Randf() + (f32) poseNum, fused.
                        pp!().poseNum = fmadds(range, r, pn as f32) as i32 as u8;
                        enable_texture(p);
                    }
                    0xBD => {
                        let base = rdf!();
                        let range = rdf!();
                        let r = ext::HSD_Randf();
                        // 2015.
                        let mut speed = fmadds(range, r, base);
                        let pp = pp!();
                        // 2016: vx*vx + vy*vy + vz*vz, two fused adds.
                        let mut mag = fmadds(pp.vel.z, pp.vel.z, fmadds(pp.vel.x, pp.vel.x, pp.vel.y * pp.vel.y));
                        if mag > 0.0 {
                            mag = sqrtf(mag);
                        }
                        if mag > 1e-10 {
                            speed /= mag;
                            pp.vel.x *= speed;
                            pp.vel.y *= speed;
                            pp.vel.z *= speed;
                        }
                    }
                    0xBE => {
                        let fx = rdf!();
                        pp!().vel.x *= fx;
                        let fy = rdf!();
                        pp!().vel.y *= fy;
                        let fz = rdf!();
                        pp!().vel.z *= fz;
                    }
                    0xBF => {
                        let idx = rd!();
                        let pp = pp!();
                        pp.kind |= ((((idx as u32) + pp.pJObjOfs as u32) & 7) << 12) | 0x8000;
                    }
                    0xC0 | 0xD0 => {
                        let prim = cls == 0xC0;
                        {
                            let pp = pp!();
                            if prim {
                                let t = pp.primColTarget;
                                col_interp(&mut pp.primCol, t, pp.primColRemain, pp.primColCount);
                            } else {
                                let t = pp.envColTarget;
                                col_interp(&mut pp.envCol, t, pp.envColRemain, pp.envColCount);
                            }
                        }
                        let cnt = rdcount!();
                        let mut t = if prim { (&*p).primCol } else { (&*p).envCol };
                        if opcode & 1 != 0 {
                            t.r = rd!();
                        }
                        if opcode & 2 != 0 {
                            t.g = rd!();
                        }
                        if opcode & 4 != 0 {
                            t.b = rd!();
                        }
                        if opcode & 8 != 0 {
                            t.a = rd!();
                        }
                        let pp = pp!();
                        if prim {
                            pp.primColCount = cnt;
                            pp.primColTarget = t;
                            if cnt == 0 {
                                pp.primCol = t;
                                pp.primColRemain = 0;
                            } else {
                                pp.primColRemain = cnt;
                            }
                        } else {
                            pp.envColCount = cnt;
                            pp.envColTarget = t;
                            if cnt == 0 {
                                pp.envCol = t;
                                pp.envColRemain = 0;
                            } else {
                                pp.envColRemain = cnt;
                            }
                        }
                    }
                    0xE0 => {
                        {
                            let pp = pp!();
                            let t = pp.primColTarget;
                            col_interp(&mut pp.primCol, t, pp.primColRemain, pp.primColCount);
                            let t = pp.envColTarget;
                            col_interp(&mut pp.envCol, t, pp.envColRemain, pp.envColCount);
                        }
                        for ch in 0..4 {
                            let r = ext::HSD_Randf();
                            let delta = rd!() as i8;
                            let rv = ((delta as i32) << 1) as f32 * r;
                            let pp = pp!();
                            let (pc_, ec_) = match ch {
                                0 => (&mut pp.primColTarget.r, &mut pp.envColTarget.r),
                                1 => (&mut pp.primColTarget.g, &mut pp.envColTarget.g),
                                2 => (&mut pp.primColTarget.b, &mut pp.envColTarget.b),
                                _ => (&mut pp.primColTarget.a, &mut pp.envColTarget.a),
                            };
                            *pc_ = col_add(*pc_, rv);
                            *ec_ = col_add(*ec_, rv);
                        }
                        let pp = pp!();
                        if pp.primColCount == 0 {
                            pp.primCol = pp.primColTarget;
                        }
                        pp.primColRemain = pp.primColCount;
                        if pp.envColCount == 0 {
                            pp.envCol = pp.envColTarget;
                        }
                        pp.envColRemain = pp.envColCount;
                    }
                    0xE9 => {
                        {
                            let pp = pp!();
                            let t = pp.primColTarget;
                            col_interp(&mut pp.primCol, t, pp.primColRemain, pp.primColCount);
                            let t = pp.envColTarget;
                            col_interp(&mut pp.envCol, t, pp.envColRemain, pp.envColCount);
                        }
                        let flags = rd!();
                        let timing = rd!() as i32;
                        let scale = if timing != 0 {
                            let r = ext::HSD_Randf();
                            ((timing + 1) as f32 * r) as i32 as f32 / timing as f32
                        } else {
                            ext::HSD_Randf()
                        };
                        for (bit, ch) in [(1u8, 0), (2, 1), (4, 2)] {
                            if flags & bit != 0 {
                                let delta = rd!() as i8;
                                let df = scale * ((delta as i32) << 1) as f32;
                                let pp = pp!();
                                let (pc_, ec_) = match ch {
                                    0 => (&mut pp.primColTarget.r, &mut pp.envColTarget.r),
                                    1 => (&mut pp.primColTarget.g, &mut pp.envColTarget.g),
                                    _ => (&mut pp.primColTarget.b, &mut pp.envColTarget.b),
                                };
                                if flags & 0x10 != 0 {
                                    *pc_ = col_add(*pc_, df);
                                }
                                if flags & 0x20 != 0 {
                                    *ec_ = col_add(*ec_, df);
                                }
                            }
                        }
                        if flags & 0x08 != 0 {
                            let a_rand = ext::HSD_Randf();
                            let delta = rd!() as i8;
                            let a_rand = ((timing + 1) as f32 * a_rand) as i32 as f32;
                            let df = (((delta as i32) << 1) as f32 * a_rand) / timing as f32;
                            let pp = pp!();
                            if flags & 0x10 != 0 {
                                pp.primColTarget.a = col_add(pp.primColTarget.a, df);
                            }
                            if flags & 0x20 != 0 {
                                pp.envColTarget.a = col_add(pp.envColTarget.a, df);
                            }
                        }
                        let pp = pp!();
                        if pp.primColCount == 0 {
                            pp.primCol = pp.primColTarget;
                        }
                        pp.primColRemain = pp.primColCount;
                        if pp.envColCount == 0 {
                            pp.envCol = pp.envColTarget;
                        }
                        pp.envColRemain = pp.envColCount;
                    }
                    0xE1 => {
                        let idx = rd!();
                        pp!().callback = if idx == 0 { None } else { *psCallback.add(idx as usize - 1) };
                    }
                    0xEC => {
                        let idx = rd!();
                        let v = rdf!();
                        let uf = (*(*p).r#gen).userfunc;
                        let handled = if !uf.is_null() {
                            if let Some(f) = (*uf).setUserData { f(p,idx,v); true } else { false }
                        } else { false };
                        if !handled && !(*p).userdata.is_null() { *(*p).userdata.add(idx as usize)=v; }
                    }
                    0xE2 => pp!().kind |= KIND_TEX_EDGE,
                    0xE3 => {
                        let n = rd!();
                        pp!().palNum = n;
                    }
                    0xE4 | 0xE5 => {
                        let bit = if cls == 0xE4 { KIND_TEX_FLIP_S } else { KIND_TEX_FLIP_T };
                        let mode = rd!() & 3;
                        match mode {
                            0 => pp!().kind &= !bit,
                            1 => pp!().kind |= bit,
                            2 => pp!().kind ^= bit,
                            _ => {
                                if ext::HSD_Randf() < 0.5 {
                                    pp!().kind &= !bit;
                                } else {
                                    pp!().kind |= bit;
                                }
                            }
                        }
                    }
                    0xE6 => pp!().kind |= KIND_DIR_VEC,
                    0xE7 => pp!().kind &= !KIND_DIR_VEC,
                    0xE8 => {
                        let trail = rdf!();
                        let pp = pp!();
                        if trail < 0.0 {
                            pp.kind &= !KIND_TRAIL;
                        } else {
                            pp.kind |= KIND_TRAIL;
                            pp.trail = trail;
                        }
                    }
                    0xEA | 0xEB => {
                        let mat = cls == 0xEA;
                        {
                            let pp = pp!();
                            let (count, remain) = if mat { (pp.matColCount, pp.matColRemain) } else { (pp.ambColCount, pp.ambColRemain) };
                            if count != 0 {
                                let step = ((remain as i32) << 16) / count as i32;
                                if mat {
                                    pp.matRGB = col_step(pp.matRGB, pp.matRGBTarget, step);
                                    pp.matA = col_step(pp.matA, pp.matATarget, step);
                                } else {
                                    pp.ambRGB = col_step(pp.ambRGB, pp.ambRGBTarget, step);
                                    pp.ambA = col_step(pp.ambA, pp.ambATarget, step);
                                }
                            }
                        }
                        let cnt = rdcount!();
                        let flags = rd!();
                        let rgb = if flags & 1 != 0 { Some(rd!()) } else { None };
                        let a = if flags & 8 != 0 { Some(rd!()) } else { None };
                        let pp = pp!();
                        if mat {
                            pp.matColCount = cnt;
                            pp.matRGBTarget = rgb.unwrap_or(pp.matRGB);
                            if let Some(a) = a {
                                pp.matATarget = a;
                            }
                            if cnt == 0 {
                                pp.matRGB = pp.matRGBTarget;
                                pp.matColRemain = 0;
                            } else {
                                pp.matColRemain = cnt;
                            }
                        } else {
                            pp.ambColCount = cnt;
                            pp.ambRGBTarget = rgb.unwrap_or(pp.ambRGB);
                            if let Some(a) = a {
                                pp.ambATarget = a;
                            }
                            if cnt == 0 {
                                pp.ambRGB = pp.ambRGBTarget;
                                pp.ambColRemain = 0;
                            } else {
                                pp.ambColRemain = cnt;
                            }
                        }
                    }
                    0xED => {
                        let mut base = rdf!();
                        let range = rdf!();
                        let timing = rd!() as i32;
                        if timing != 0 {
                            let r = ext::HSD_Randf();
                            let randi = ((timing + 1) as f32 * r) as i32;
                            base += range * randi as f32 / timing as f32;
                        } else {
                            let r = ext::HSD_Randf();
                            // 2779.
                            base = fmadds(range, r, base);
                        }
                        let pp = pp!();
                        pp.rotateTarget += base;
                        pp.rotate += base;
                    }
                    0xFA => {
                        let n = rd!();
                        let pp = pp!();
                        pp.loopCount = n;
                        pp.cmdLoopPtr = pc as u16;
                    }
                    0xFB => {
                        let pp = pp!();
                        pp.loopCount = pp.loopCount.wrapping_sub(1);
                        if pp.loopCount != 0 {
                            pc = pp.cmdLoopPtr as u32;
                        }
                    }
                    0xFC => pp!().cmdMarkPtr = pc as u16,
                    0xFD => pc = (&*p).cmdMarkPtr as u32,
                    0xFE | 0xFF => {
                        pp!().life = 1;
                        break 'outer;
                    }
                    _ => {}
                }
            }
            if operand != 0 {
                break;
            }
        }
        let pp = pp!();
        pp.cmdPtr = pc as u16;
        pp.cmdWait = operand;
    }
unsafe fn spawn_child_generator(p: *mut HSD_Particle, idx: i32, flags: Option<u8>) {
        let (link, bank) = ((&*p).linkNo as i32, (&*p).bank as i32);
        let gc = ext::hsd_8039F05C(link, bank, idx); if gc.is_null() { return; }
        let idnum = (&*p).idnum;
        (&mut *gc).idnum = idnum;
        if let Some(pg) = ptr_option((&*p).r#gen) {
            let j = (&*pg).jobj;
            (&mut *gc).jobj = j;
            if let Some(j) = ptr_option(j) {
                ref_INC(j.cast());
            }
        }
        (&mut *gc).r#type |= 0x100;
        if let Some(pg) = ptr_option((&*p).r#gen) {
            let t = (&*pg).r#type & 0x1E00;
            let gcp = (&mut *gc);
            gcp.r#type |= t;
            if gcp.kind & (1 << 17) != 0 {
                gcp.r#type &= !(1 << 9);
            }
        }
        if let Some(s) = ptr_option((&*p).appsrt) {
            ext::psAttachGeneratorAppSRT(gc, s);
        }
        if let Some(fl) = flags {
            let gcp = (&mut *gc);
            gcp.kind &= 0xF1FF_FFFF;
            gcp.kind |= ((fl & 7) as u32) << 25;
        }
        let ppos = (&*p).pos;
        match (ptr_option((*p).appsrt), ptr_option((*gc).appsrt)) {
            (Some(_), None) => {}
            (Some(ps), Some(gs)) => {
                if gs == ps {
                    (&mut *gc).pos = ppos;
                } else {
                    let t = (&*ps).translate;
                    (&mut *gs).translate = t;
                }
            }
            (None, Some(gs)) => (&mut *gs).translate = ppos,
            (None, None) => (&mut *gc).pos = ppos,
        }
    }
unsafe fn delete_particle(p:*mut HSD_Particle, prev:*mut HSD_Particle)->*mut HSD_Particle {
    if !(*p).r#gen.is_null() && !(*(*p).r#gen).userfunc.is_null() {
        if let Some(f)=(*(*(*p).r#gen).userfunc).hookDelete { f(p); }
    }
    if prev.is_null() { hsd_804D0908[(*p).linkNo as usize]=(*p).next; } else { (*prev).next=(*p).next; }
    let mut next=(*p).next;
    if !(*p).r#gen.is_null() { (*(*p).r#gen).numChild=(*(*p).r#gen).numChild.wrapping_sub(1); }
    if !(*p).appsrt.is_null() && ext::psRemoveParticleAppSRT(p)==0 && prev.is_null() {
        next=hsd_804D0908[(*p).linkNo as usize];
    }
    hsd_8039D048(p.cast());
    ext::HSD_ObjFree(&raw mut hsd_804D0F60.alloc_data,p.cast());
    hsd_804D78E2=hsd_804D78E2.wrapping_sub(1);
    next
}
fn ptr_option<T>(p:*mut T)->Option<*mut T> { if p.is_null(){None}else{Some(p)} }
unsafe fn remap(bank:u8,idx:i32)->i32 { let r=hsd_804D0948[bank as usize]; if r.is_null(){idx}else{*r.offset(idx as isize) as i32} }
unsafe fn enable_texture(p:*mut HSD_Particle) { let tg=*psTexGroupArray[(*p).bank as usize].add((*p).texGroup as usize); if !tg.is_null(){psEnableTexture(p,(*tg).texTable.as_ptr());} }
pub unsafe extern "C" fn hsd_8039930C(p:*mut HSD_Particle,prev:*mut HSD_Particle)->*mut c_void { melee_core::ct!("hsd_8039930C"); interpret(p,prev).cast() }
unsafe fn memset(p:*mut c_void,v:i32,n:u32)->*mut c_void { core::ptr::write_bytes(p,v as u8,n as usize);p }
fn OSPanic(_file:*const i8,line:i32,_msg:*const i8)->! { panic!("particle.c OSPanic line {line}") }
fn __assert(_file:*const i8,line:u32,_msg:*const i8)->! { panic!("particle.c assertion line {line}") }
use ext::*;
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjAlloc() -> *mut HSD_JObj = crate::tu::jobj::HSD_JObjAlloc();
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_JObjUnref(a0: *mut HSD_JObj) = crate::tu::jobj::HSD_JObjUnref(a0);
    fn HSD_MtxGetScale(a0: *mut c_void, a1: *mut Vec);
    fn HSD_MtxSRT(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec, a3: *mut Vec, a4: *mut Vec);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = crate::tu::objalloc::HSD_ObjAlloc(a0 as _);
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = crate::tu::objalloc::HSD_ObjAllocInit(a0 as _, a1 as _, a2);
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = crate::tu::objalloc::HSD_ObjFree(a0 as _, a1);
    fn HSD_Randf() -> f32 = crate::tu::random::HSD_Randf(crate::tu::globals::rand());
    fn ScriptGame_StageSlotParticleBegin(a0: i32) -> i32;
    fn ScriptGame_StageSlotProcEnd();
    fn atan2f(a0: f32, a1: f32) -> f32 = crate::lbtrigf::atan2f(a0, a1);
    fn hsd_8039D1EC() -> u16 = unsafe { crate::tu::generator::hsd_8039D1EC() };
    fn hsd_8039F05C(a0: i32, a1: i32, a2: i32) -> *mut HSD_Generator = unsafe { crate::tu::generator::hsd_8039F05C(a0, a1, a2) };
    fn psAttachGeneratorAppSRT(a0: *mut HSD_Generator, a1: *mut HSD_psAppSRT) -> i32 = unsafe { crate::tu::psappsrt::psAttachGeneratorAppSRT(a0, a1) };
    fn psAttachParticleAppSRT(a0: *mut HSD_Particle, a1: *mut HSD_psAppSRT) -> i32 = unsafe { crate::tu::psappsrt::psAttachParticleAppSRT(a0, a1) };
    fn psRemoveParticleAppSRT(a0: *mut HSD_Particle) -> i32 = unsafe { crate::tu::psappsrt::psRemoveParticleAppSRT(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT
