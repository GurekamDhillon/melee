//! src/melee/mn/mnname.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{mn_80229624, mn_80229894, mn_804A04F0, mn_804D6BB8, mn_804D6BC8};
#[allow(unused_imports)] use super::mnmainrule::{mn_80231634};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub type bool_0 = ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MnName_GObj {
    pub gobj: HSD_GObj,
    pub x38: *mut ::core::ffi::c_void,
    pub text: *mut HSD_Text,
    pub text2: *mut HSD_Text,
}
melee_core::gc_layout!(MnName_GObj, "MnName_GObj", size 0x48, align 8, [
    gobj @ 0x0: 56 "HSD_GObj",
    x38 @ 0x38: 4 "*mut c_void",
    text @ 0x3C: 4 "*mut HSD_Text",
    text2 @ 0x40: 4 "*mut HSD_Text",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub signed_characters: *mut ::core::ffi::c_char,
    pub unsigned_characters: *mut u8_0,
}
melee_core::gc_layout!(C2RustUnnamed_1, "C2RustUnnamed_1", size 0x4, align 4, [
    signed_characters @ 0x0: 4 "*mut c_char",
    unsigned_characters @ 0x0: 4 "*mut u8_0",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MnNameArchive {
    pub joint: *mut ::core::ffi::c_void,
    pub anim_joint: *mut ::core::ffi::c_void,
    pub matanim_joint: *mut ::core::ffi::c_void,
    pub shapeanim_joint: *mut ::core::ffi::c_void,
}
melee_core::gc_layout!(MnNameArchive, "MnNameArchive", size 0x10, align 4, [
    joint @ 0x0: 4 "*mut c_void",
    anim_joint @ 0x4: 4 "*mut c_void",
    matanim_joint @ 0x8: 4 "*mut c_void",
    shapeanim_joint @ 0xC: 4 "*mut c_void",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AnimTable {
    pub entries: [*mut AnimLoopSettings; 6],
}
melee_core::gc_layout!(AnimTable, "AnimTable", size 0x18, align 4, [
    entries @ 0x0: 24 "[*mut AnimLoopSettings; 6]",
]);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MnNameUserDataState {
    pub cur_menu: u8_0,
    pub prev_selection: u8_0,
}
melee_core::gc_layout!(MnNameUserDataState, "MnNameUserDataState", size 0x2, align 1, [
    cur_menu @ 0x0: 1 "u8_0",
    prev_selection @ 0x1: 1 "u8_0",
]);

#[allow(unused_imports)] use super::mnmain::{MenuFlow};
#[allow(unused_imports)] use super::mnmainrule::{mn_80231634_t};
#[inline]
pub unsafe extern "C" fn HSD_JObjMtxIsDirty(mut jobj: *mut HSD_JObj) -> bool_0 {
    let mut result: bool_0 = 0;
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            228 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    result = false_0 as bool_0;
    if (*jobj).flags & JOBJ_USER_DEF_MTX as u32_0 == 0
        && (*jobj).flags & JOBJ_MTX_DIRTY as u32_0 != 0
    {
        result = true_0 as bool_0;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetTranslateX(mut jobj: *mut HSD_JObj, mut x: f32_0) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            513 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjSetTranslateY(mut jobj: *mut HSD_JObj, mut y: f32_0) {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            531 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32_0 == 0 {
        if !jobj.is_null() && HSD_JObjMtxIsDirty(jobj) == 0 {
            HSD_JObjSetMtxDirtySub(jobj);
        }
    }
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetTranslationX(mut jobj: *mut HSD_JObj) -> f32_0 {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            582 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*jobj).translate.x;
}
#[inline]
pub unsafe extern "C" fn HSD_JObjGetTranslationY(mut jobj: *mut HSD_JObj) -> f32_0 {
    if !jobj.is_null() {} else {
        __assert(
            b"src/sysdolphin/baselib/jobj.h\0" as *const u8
                as *const ::core::ffi::c_char,
            588 as u32_0,
            b"jobj\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return (*jobj).translate.y;
}
#[inline]
pub unsafe extern "C" fn sfxBack() {
    lbAudioAx_80024030(0 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxForward() {
    lbAudioAx_80024030(1 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
pub static mut mnName_803ED538: [AnimLoopSettings; 4] = [
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 0.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 0.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 0.0f32,
        loop_frame: -0.1f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 0.0f32,
        loop_frame: -0.1f32,
    },
];
pub static mut mnName_803ED568: AnimLoopSettings = AnimLoopSettings {
    start_frame: 50.0f32,
    end_frame: 59.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnName_803ED574: AnimLoopSettings = AnimLoopSettings {
    start_frame: 40.0f32,
    end_frame: 49.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnName_803ED580: AnimLoopSettings = AnimLoopSettings {
    start_frame: 30.0f32,
    end_frame: 39.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnName_803ED58C: AnimLoopSettings = AnimLoopSettings {
    start_frame: 10.0f32,
    end_frame: 19.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnName_803ED598: AnimLoopSettings = AnimLoopSettings {
    start_frame: 20.0f32,
    end_frame: 29.0f32,
    loop_frame: -0.1f32,
};
pub static mut mnName_803ED5A4: [f32_0; 23] = [
    0.0f32,
    9.0f32,
    -0.1f32,
    0.0f32,
    0.0f32,
    0.0f32,
    0.0f32,
    0.0f32,
    0.0f32,
    0.0f32,
    0.0f32,
    0.0f32,
    1.0f32,
    0.0f32,
    0.0f32,
    2.0f32,
    3.0f32,
    4.0f32,
    0.0f32,
    5.0f32,
    6.0f32,
    7.0f32,
    8.0f32,
];
pub static mut mnName_803ED600: [f32_0; 6] = [
    0.0f32,
    9.0f32,
    -0.1f32,
    20.0f32,
    29.0f32,
    -0.1f32,
];
pub static mut mnName_803ED618: [Vec3; 2] = [
    Vec3 {
        x: -2.0f32,
        y: 0.5f32,
        z: 0.0f32,
    },
    Vec3 {
        x: -5.5f32,
        y: -2.9f32,
        z: 23.0f32,
    },
];
pub static mut mnName_803B8510: [*mut AnimLoopSettings; 6] = unsafe {
    [
        &raw const mnName_803ED568 as *mut AnimLoopSettings,
        &raw const mnName_803ED574 as *mut AnimLoopSettings,
        &raw const mnName_803ED580 as *mut AnimLoopSettings,
        &raw const mnName_803ED58C as *mut AnimLoopSettings,
        &raw const mnName_803ED598 as *mut AnimLoopSettings,
        &raw const mnName_803ED5A4 as *mut f32_0 as *mut AnimLoopSettings,
    ]
};
pub static mut mnName_NameDisplayOrder: [u8_0; 120] = [0; 120];
pub static mut mnName_804A06C0: MnNameArchive = MnNameArchive {
    joint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    anim_joint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    matanim_joint: ::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void,
    shapeanim_joint: ::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void,
};
pub static mut mnName_804A06D0: MnNameArchive = MnNameArchive {
    joint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    anim_joint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    matanim_joint: ::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void,
    shapeanim_joint: ::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void,
};
pub static mut mnName_804A06E0: MnNameArchive = MnNameArchive {
    joint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    anim_joint: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    matanim_joint: ::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void,
    shapeanim_joint: ::core::ptr::null::<::core::ffi::c_void>()
        as *mut ::core::ffi::c_void,
};
pub static mut mnNameNew_804A06F0: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut mnNameNew_804A0700: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut mnNameNew_804A0710: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut mnNameNew_804A0720: [StaticModelDesc; 2] = [StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
}; 2];
pub static mut mnName_804D4BD0: [f32_0; 2] = [39.0f32, 59.0f32];
pub static mut mnName_804D4BD8: [f32_0; 2] = [39.0f32, 19.0f32];
pub static mut mnName_804D4BE0: GXColor = _GXColor {
    r: 0 as u8_0,
    g: 0 as u8_0,
    b: 0 as u8_0,
    a: 0xff as u8_0,
};
pub static mut mnName_804D4BE4: GXColor = _GXColor {
    r: 0xa6 as u8_0,
    g: 0x81 as u8_0,
    b: 0x3d as u8_0,
    a: 0xff as u8_0,
};
pub static mut mnName_804D4BE8: [u8_0; 4] = [
    0x93 as ::core::ffi::c_int as u8_0,
    0x94 as ::core::ffi::c_int as u8_0,
    0x95 as ::core::ffi::c_int as u8_0,
    0 as ::core::ffi::c_int as u8_0,
];
pub static mut mnName_StringTerminator: [::core::ffi::c_char; 1] = unsafe {
    ::core::mem::transmute::<[u8; 1], [::core::ffi::c_char; 1]>(*b"\0")
};
pub unsafe extern "C" fn mnName_8023749C(
    mut slot: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut j: ::core::ffi::c_int = 0;
    let mut terminator: s8 = 0;
    let mut new_var: u16_0 = 0;
    let mut array: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        *mut ::core::ffi::c_char,
    >();
    let mut term: s8 = 0;
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    if lbLang_IsSavedLanguageUS() != 0 {
        array = &raw mut mnNameNew_803EE724 as *mut *mut ::core::ffi::c_char;
    } else {
        array = &raw mut mnNameNew_803EE720 as *mut *mut ::core::ffi::c_char;
    }
    new_var = slot as u8_0 as u16_0;
    j = 0 as ::core::ffi::c_int;
    while j != new_var as ::core::ffi::c_int
        && *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char)
            as ::core::ffi::c_int
            != *(*array.offset(j as isize)).offset(0 as ::core::ffi::c_int as isize)
                as s8 as ::core::ffi::c_int
    {
        j += 1;
    }
    str = *array.offset(j as isize);
    terminator = *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as s8;
    if terminator as ::core::ffi::c_int
        == *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        str = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return str;
}
pub unsafe extern "C" fn GetNameText(
    mut slot: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if IsNameValid(slot) == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return &raw mut (*(GetPersistentNameData
        as unsafe extern "C" fn(s32) -> *mut NameTagData)(slot as u8_0 as s32))
        .namedata as *mut ::core::ffi::c_char;
}
#[inline]
pub unsafe extern "C" fn GetNameCount_noinline() -> ::core::ffi::c_int {
    return GetNameCount();
}
pub unsafe extern "C" fn GetNameCount() -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 0x78 as ::core::ffi::c_int {
        if IsNameValid(i as u8_0 as ::core::ffi::c_int) != 0 {
            count += 1;
        }
        i += 1;
    }
    return count;
}
#[inline]
pub unsafe extern "C" fn GetNumNameList() -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while i < 0x78 as ::core::ffi::c_int {
        if IsNameValid(i as u8_0 as ::core::ffi::c_int) != 0 {
            count += 1;
        }
        i += 1;
    }
    return count;
}
pub unsafe extern "C" fn IsNameListFull() -> bool_0 {
    if GetNumNameList() < 0x78 as ::core::ffi::c_int {
        return false_0;
    }
    return true_0;
}
#[inline]
pub unsafe extern "C" fn readNameTerminator(
    mut terminator: *const ::core::ffi::c_char,
) -> s8 {
    return *terminator as s8;
}
#[inline]
pub unsafe extern "C" fn signedCharactersEqual(mut lhs: s8, mut rhs: u8_0) -> bool_0 {
    return (lhs as ::core::ffi::c_int == rhs as s8 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[inline]
pub unsafe extern "C" fn checkStringRest(
    mut ptr: *const ::core::ffi::c_char,
    mut terminator: s8,
) -> bool_0 {
    let mut c: ::core::ffi::c_char = ::core::mem::transmute::<
        [u8; 4],
        [::core::ffi::c_char; 4],
    >(*b"\xE3\x80\x80\0")[0 as ::core::ffi::c_int as usize];
    while terminator as ::core::ffi::c_int != *ptr as ::core::ffi::c_int {
        if c as ::core::ffi::c_int != *ptr as ::core::ffi::c_int
            || ::core::mem::transmute::<
                [u8; 4],
                [::core::ffi::c_char; 4],
            >(*b"\xE3\x80\x80\0")[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                != *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            return false_0;
        }
        ptr = ptr.offset(2 as ::core::ffi::c_int as isize);
    }
    return true_0;
}
#[inline]
pub unsafe extern "C" fn unsignedCharacter(mut character: s32) -> u8_0 {
    return character as u8_0;
}
pub unsafe extern "C" fn CompareNameStrings(
    mut str1: *mut ::core::ffi::c_char,
    mut str2: *mut ::core::ffi::c_char,
) -> s32 {
    let mut string1: C2RustUnnamed_1 = C2RustUnnamed_1 {
        signed_characters: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut unsigned_str1: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut terminator: s8 = *(&raw mut mnName_StringTerminator
        as *mut ::core::ffi::c_char) as s8;
    let mut i: s32 = 0 as s32;
    string1.signed_characters = str1;
    unsigned_str1 = string1.unsigned_characters;
    loop {
        let mut ch1: s8 = *str1.offset(i as isize) as s8;
        if signedCharactersEqual(terminator, ch1 as u8_0) != 0 {
            if checkStringRest(
                str2
                    .offset(
                        (i as ::core::ffi::c_ulonglong
                            & 0xffffffffffffffff as ::core::ffi::c_ulonglong) as isize,
                    ) as *mut ::core::ffi::c_char,
                readNameTerminator(
                    &raw mut mnName_StringTerminator as *mut ::core::ffi::c_char,
                ),
            ) != 0
            {
                return 0 as s32;
            }
            return 2 as s32;
        }
        let mut ch2: s8 = *str2.offset(i as isize) as s8;
        if signedCharactersEqual(
            *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as s8,
            unsignedCharacter(ch2 as s32),
        ) != 0
        {
            if checkStringRest(
                str1.offset(i as isize) as *mut ::core::ffi::c_char,
                terminator,
            ) != 0
            {
                return 0 as s32;
            }
            return 1 as s32;
        }
        if *unsigned_str1.offset(i as isize) as ::core::ffi::c_int
            > unsignedCharacter(ch2 as s32) as ::core::ffi::c_int
        {
            return 1 as s32;
        }
        if (*unsigned_str1.offset(i as isize) as ::core::ffi::c_int)
            < unsignedCharacter(ch2 as s32) as ::core::ffi::c_int
        {
            return 2 as s32;
        }
        i += 1;
    };
}
pub unsafe extern "C" fn IsNameUnique(mut name: *mut ::core::ffi::c_char) -> bool_0 {
    let mut i: ::core::ffi::c_int = 0;
    let mut namedata: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    i = 0 as ::core::ffi::c_int;
    while i < 0x78 as ::core::ffi::c_int {
        namedata = if IsNameValid(i & 0xff as ::core::ffi::c_int) == 0 {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            &raw mut (*(GetPersistentNameData
                as unsafe extern "C" fn(s32) -> *mut NameTagData)(i as u8_0 as s32))
                .namedata as *mut ::core::ffi::c_char
        };
        if namedata.is_null() {
            break;
        }
        if CompareNameStrings(namedata, name) == 0 as ::core::ffi::c_int {
            return true_0;
        }
        i += 1;
    }
    return false_0;
}
pub unsafe extern "C" fn DeleteName(mut arg0: u8_0) {
    let mut _2: [u8_0; 8] = [0; 8];
    let mut temp: NameTagData = NameTagData {
        vs_kos: [0; 120],
        stats: GmStats {
            sd_count: 0,
            pad_2: [0; 2],
            attacks_hit: 0,
            attacks_total: 0,
            damage_dealt: 0,
            damage_taken: 0,
            damage_recovered: 0,
            peak_damage: 0,
            match_count: 0,
            victories: 0,
            losses: 0,
            play_time: 0,
            total_player_count: 0,
            walk_distance: 0,
            run_distance: 0,
            fall_distance: 0,
            peak_height: 0,
            coins_collected: 0,
            coins_swiped: 0,
            coins_lost: 0,
        },
        play_time_by_fighter: [0; 25],
        namedata: [0; 8],
        x1A0: 0,
        rumble_enabled: 0,
        x1A2: 0,
        padding_x1A2: 0,
    };
    let mut i: s32 = 0;
    let mut longpos: u64_0 = 0;
    let mut pos: s32 = 0;
    let mut j: s32 = 0;
    let mut p: *mut u16_0 = ::core::ptr::null_mut::<u16_0>();
    let mut k: s32 = 0;
    let mut pos_u8: u8_0 = 0;
    pos = arg0 as s32;
    i = 0 as ::core::ffi::c_int as s32;
    loop {
        if IsNameValid(i as u8_0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
            p = (GetPersistentNameData(i) as *mut u8_0)
                .offset((pos as ::core::ffi::c_int * 2 as ::core::ffi::c_int) as isize)
                as *mut u16_0;
            k = pos;
            while k < 0x78 as ::core::ffi::c_int {
                if k == 0x77 as ::core::ffi::c_int {
                    *p = 0 as u16_0;
                } else {
                    *p = *p.offset(1 as ::core::ffi::c_int as isize);
                }
                p = p.offset(1);
                k += 1;
            }
        }
        i += 1;
        if !(i < 0x78 as ::core::ffi::c_int) {
            break;
        }
    }
    while pos < 0x78 as ::core::ffi::c_int {
        j = (pos as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as s32;
        while j < 0x78 as ::core::ffi::c_int {
            longpos = pos as u64_0;
            if IsNameValid(longpos as u8_0 as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
            {
                if IsNameValid(j as u8_0 as ::core::ffi::c_int)
                    != 0 as ::core::ffi::c_int
                {
                    temp = *GetPersistentNameData(pos as u8_0 as s32);
                    *GetPersistentNameData(pos as u8_0 as s32) = *GetPersistentNameData(
                        j as u8_0 as s32,
                    );
                    *GetPersistentNameData(j as u8_0 as s32) = temp;
                }
            }
            j += 1;
        }
        pos += 1;
    }
}
pub unsafe extern "C" fn IsNameValid(mut slot: ::core::ffi::c_int) -> bool_0 {
    if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char)
        as ::core::ffi::c_int
        == (*GetPersistentNameData(slot as u8_0 as s32))
            .namedata[0 as ::core::ffi::c_int as usize] as s8 as ::core::ffi::c_int
    {
        return false_0;
    }
    return true_0;
}
pub unsafe extern "C" fn CreateNameAtIndex(mut slot: s32) {
    let mut idx: s32 = slot & 0xff as s32;
    (*GetPersistentNameData(idx)).namedata[0 as ::core::ffi::c_int as usize] = *(&raw mut mnName_StringTerminator
        as *mut ::core::ffi::c_char);
    (*GetPersistentNameData(idx)).rumble_enabled = true_0 as u8_0;
    InitializePersistentNameData(slot);
}
pub unsafe extern "C" fn mnName_SortNames(mut arg0: *mut HSD_GObj) {
    let mut result: s32 = 0;
    let mut idx2: u8_0 = 0;
    let mut name2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut i: s32 = 0;
    let mut idx1: u8_0 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    if (*((*arg0).user_data as *mut HSD_GObj)).p_priority as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        i = 0 as ::core::ffi::c_int as s32;
        while 0x78 as ::core::ffi::c_int > i {
            mnName_NameDisplayOrder[i as usize] = i as u8_0;
            i += 1;
        }
        return;
    }
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x78 as ::core::ffi::c_int {
        let mut j: s32 = 0;
        j = (i as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as s32;
        while j < 0x78 as ::core::ffi::c_int {
            let mut name1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
                ::core::ffi::c_char,
            >();
            let mut index1: s32 = 0;
            idx1 = mnName_NameDisplayOrder[i as usize];
            idx2 = mnName_NameDisplayOrder[j as usize];
            index1 = idx1 as s32;
            name1 = &raw mut (*(GetPersistentNameData
                as unsafe extern "C" fn(s32) -> *mut NameTagData)(index1))
                .namedata as *mut ::core::ffi::c_char;
            name2 = &raw mut (*(GetPersistentNameData
                as unsafe extern "C" fn(s32) -> *mut NameTagData)(idx2 as s32))
                .namedata as *mut ::core::ffi::c_char;
            let mut e1: bool_0 = 0;
            let mut e2: bool_0 = 0;
            if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as s8
                as ::core::ffi::c_int
                == (*GetPersistentNameData(idx1 as s32))
                    .namedata[0 as ::core::ffi::c_int as usize] as s8
                    as ::core::ffi::c_int
            {
                e1 = 0 as ::core::ffi::c_int as bool_0;
            } else {
                e1 = 1 as ::core::ffi::c_int as bool_0;
            }
            let mut current_block_50: u64;
            if e1 != 0 as ::core::ffi::c_int {
                if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as s8
                    as ::core::ffi::c_int
                    == (*GetPersistentNameData(idx2 as s32))
                        .namedata[0 as ::core::ffi::c_int as usize] as s8
                        as ::core::ffi::c_int
                {
                    e2 = 0 as ::core::ffi::c_int as bool_0;
                } else {
                    e2 = 1 as ::core::ffi::c_int as bool_0;
                }
                if e2 != 0 as ::core::ffi::c_int {
                    result = CompareNameStrings(name1, name2);
                    current_block_50 = 2290177392965769716;
                } else {
                    current_block_50 = 12418479505543342242;
                }
            } else {
                current_block_50 = 12418479505543342242;
            }
            match current_block_50 {
                12418479505543342242 => {
                    if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char)
                        as s8 as ::core::ffi::c_int
                        == (*GetPersistentNameData(idx1 as s32))
                            .namedata[0 as ::core::ffi::c_int as usize] as s8
                            as ::core::ffi::c_int
                    {
                        e1 = 0 as ::core::ffi::c_int as bool_0;
                    } else {
                        e1 = 1 as ::core::ffi::c_int as bool_0;
                    }
                    let mut current_block_49: u64;
                    if e1 == 0 as ::core::ffi::c_int {
                        if *(&raw mut mnName_StringTerminator
                            as *mut ::core::ffi::c_char) as s8 as ::core::ffi::c_int
                            == (*GetPersistentNameData(idx2 as s32))
                                .namedata[0 as ::core::ffi::c_int as usize] as s8
                                as ::core::ffi::c_int
                        {
                            e2 = 0 as ::core::ffi::c_int as bool_0;
                        } else {
                            e2 = 1 as ::core::ffi::c_int as bool_0;
                        }
                        if e2 == 0 as ::core::ffi::c_int {
                            result = 0 as ::core::ffi::c_int as s32;
                            current_block_49 = 5892776923941496671;
                        } else {
                            current_block_49 = 11119259271652140274;
                        }
                    } else {
                        current_block_49 = 11119259271652140274;
                    }
                    match current_block_49 {
                        11119259271652140274 => {
                            if *(&raw mut mnName_StringTerminator
                                as *mut ::core::ffi::c_char) as s8 as ::core::ffi::c_int
                                == (*GetPersistentNameData(idx1 as s32))
                                    .namedata[0 as ::core::ffi::c_int as usize] as s8
                                    as ::core::ffi::c_int
                            {
                                e1 = 0 as ::core::ffi::c_int as bool_0;
                            } else {
                                e1 = 1 as ::core::ffi::c_int as bool_0;
                            }
                            if e1 == 0 as ::core::ffi::c_int {
                                result = 1 as ::core::ffi::c_int as s32;
                            } else {
                                result = 2 as ::core::ffi::c_int as s32;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            if result == 1 as ::core::ffi::c_int {
                result = mnName_NameDisplayOrder[i as usize] as s32;
                mnName_NameDisplayOrder[i as usize] = mnName_NameDisplayOrder[j
                    as usize];
                mnName_NameDisplayOrder[j as usize] = result as u8_0;
            }
            j += 1;
        }
        i += 1;
    }
}
pub static mut mnName_804D6BF8: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
pub unsafe extern "C" fn mnName_80237D94(mut arg0: s32, mut arg1: u8_0) -> u8_0 {
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    gobj = (*mnName_804D6BF8).user_data as *mut HSD_GObj;
    if (arg1 as ::core::ffi::c_uint) < 0x18 as ::core::ffi::c_uint {
        if arg0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            if arg1 as ::core::ffi::c_int % 6 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                return (arg1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u8_0;
            }
            return (arg1 as ::core::ffi::c_int + 5 as ::core::ffi::c_int) as u8_0;
        }
        if arg0 as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 {
            if arg1 as ::core::ffi::c_int % 6 as ::core::ffi::c_int
                != 5 as ::core::ffi::c_int
            {
                return (arg1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u8_0;
            }
            return (arg1 as ::core::ffi::c_int - 5 as ::core::ffi::c_int) as u8_0;
        }
        if arg0 as ::core::ffi::c_int & 8 as ::core::ffi::c_int != 0 {
            if arg1 as ::core::ffi::c_int / 6 as ::core::ffi::c_int
                == 3 as ::core::ffi::c_int
            {
                if mnName_GetColumnCount() > 4 as ::core::ffi::c_int {
                    sfxMove();
                    if mnName_GetColumnCount() as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int > (*gobj).gx_link as s32
                    {
                        (*gobj).gx_link = ((*gobj).gx_link as ::core::ffi::c_int
                            + 1 as ::core::ffi::c_int) as u8_0;
                    } else {
                        (*gobj).gx_link = 0 as u8_0;
                    }
                    mnName_802385A0(gobj);
                }
                return arg1;
            }
            return (arg1 as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as u8_0;
        }
        if arg0 as ::core::ffi::c_int & 4 as ::core::ffi::c_int != 0 {
            if arg1 as ::core::ffi::c_int / 6 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                if mnName_GetColumnCount() > 4 as ::core::ffi::c_int {
                    sfxMove();
                    if (*gobj).gx_link as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        (*gobj).gx_link = ((*gobj).gx_link as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as u8_0;
                    } else {
                        (*gobj).gx_link = (mnName_GetColumnCount() as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as u8_0;
                    }
                    mnName_802385A0(gobj);
                }
                return arg1;
            }
            return (arg1 as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as u8_0;
        }
    } else {
        if arg0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            if arg1 as ::core::ffi::c_int == 0x18 as ::core::ffi::c_int {
                return 0x1a as u8_0;
            }
            return (arg1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u8_0;
        }
        if arg0 as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 {
            if arg1 as ::core::ffi::c_int == 0x1a as ::core::ffi::c_int {
                return 0x18 as u8_0;
            }
            return (arg1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u8_0;
        }
    }
    return arg1;
}
pub unsafe extern "C" fn mnName_ConfirmNameDeleteInput(mut arg0: *mut HSD_GObj) {
    let mut col: s32 = 0;
    let mut nameIdx: s32 = 0;
    let mut nameIdxInt: s32 = 0;
    let mut colCount: s32 = 0;
    let mut idx: s32 = 0;
    let mut buttons: u32_0 = 0;
    let mut sel: u8_0 = 0;
    let mut flow: *mut MenuFlow = ::core::ptr::null_mut::<MenuFlow>();
    let mut gobj2: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    gobj2 = (*mnName_804D6BF8).user_data as *mut HSD_GObj;
    buttons = mn_80229624(4 as u32_0);
    flow = &raw mut mn_804A04F0;
    (*flow).buttons = buttons as u64_0;
    let mut _pad: [::core::ffi::c_uchar; 24] = [0; 24];
    if buttons & 0x10 as u32_0 != 0 {
        if (*flow).confirmed_selection as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            sfxBack();
            mn_804D6BC8.cooldown = 5 as u16_0;
            (*flow).x10 = 0 as u8_0;
            return;
        }
        sel = (*flow).hovered_selection as u8_0;
        let mut tmp: s32 = (*((*mnName_804D6BF8).user_data as *mut HSD_GObj)).gx_link
            as s32 + ((*flow).hovered_selection as s32 & 0xff as s32) / 6 as s32;
        col = tmp;
        colCount = mnName_GetColumnCount();
        if colCount > 4 as ::core::ffi::c_int && col >= colCount {
            col -= colCount;
        }
        idx = (sel as ::core::ffi::c_int % 6 as ::core::ffi::c_int
            + col as ::core::ffi::c_int * 6 as ::core::ffi::c_int) as s32;
        nameIdx = mnName_NameDisplayOrder[idx as usize] as s32;
        sfxForward();
        let mut term: u8_0 = *(&raw mut mnName_StringTerminator
            as *mut ::core::ffi::c_char) as u8_0;
        nameIdxInt = nameIdx as u8_0 as s32;
        mn_804D6BC8.cooldown = 5 as u16_0;
        (*GetPersistentNameData(nameIdxInt))
            .namedata[0 as ::core::ffi::c_int as usize] = term as ::core::ffi::c_char;
        (*GetPersistentNameData(nameIdxInt)).rumble_enabled = true_0 as u8_0;
        InitializePersistentNameData(nameIdx);
        if (*gobj2).gx_link as s32
            > mnName_GetColumnCount() as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            (*gobj2).gx_link = 0 as u8_0;
        }
        DeleteName(nameIdx as u8_0);
        mnName_SortNames(mnName_804D6BF8);
        if mnName_GetColumnCount() <= 4 as ::core::ffi::c_int {
            (*gobj2).gx_link = 0 as u8_0;
        }
        mnName_802385A0(gobj2);
        gmMainLib_8015DBF4(nameIdx);
        mn_804A04F0.x10 = 0 as u8_0;
        return;
    }
    if buttons & 0x20 as u32_0 != 0 {
        sfxBack();
        mn_804D6BC8.cooldown = 5 as u16_0;
        (*flow).x10 = 0 as u8_0;
        return;
    }
    if buttons & 8 as u32_0 != 0 || buttons & 4 as u32_0 != 0 {
        sfxMove();
        if mn_804A04F0.confirmed_selection as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            mn_804A04F0.confirmed_selection = 1 as u8_0;
            return;
        }
        mn_804A04F0.confirmed_selection = 0 as u8_0;
    }
}
#[inline]
pub unsafe extern "C" fn mnName_GetHoveredName() -> u8_0 {
    let mut colCount: s32 = 0;
    let mut sel: u8_0 = mn_804A04F0.hovered_selection as u8_0;
    let mut col: s32 = (*((*mnName_804D6BF8).user_data as *mut HSD_GObj)).gx_link as s32
        + sel as s32 / 6 as s32;
    let mut idx: s32 = 0;
    colCount = mnName_GetColumnCount();
    if colCount > 4 as ::core::ffi::c_int && col >= colCount {
        col -= colCount;
    }
    idx = (sel as ::core::ffi::c_int % 6 as ::core::ffi::c_int
        + col as ::core::ffi::c_int * 6 as ::core::ffi::c_int) as s32;
    return mnName_NameDisplayOrder[idx as usize];
}
#[inline]
pub unsafe extern "C" fn mnName_CountValid() -> u8_0 {
    let mut isValid: s32 = 0;
    let mut i: s32 = 0;
    let mut count: u8_0 = 0 as u8_0;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x78 as ::core::ffi::c_int {
        if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as s8
            as ::core::ffi::c_int
            == (*GetPersistentNameData(i as u8_0 as s32))
                .namedata[0 as ::core::ffi::c_int as usize] as s8 as ::core::ffi::c_int
        {
            isValid = 0 as ::core::ffi::c_int as s32;
        } else {
            isValid = 1 as ::core::ffi::c_int as s32;
        }
        if isValid != 0 as ::core::ffi::c_int {
            count = (count as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u8_0;
        }
        i += 1;
    }
    return count;
}
pub unsafe extern "C" fn mnName_MainInput(mut arg0: *mut HSD_GObj) {
    let mut gobj2: *mut MnName_GObj = (*mnName_804D6BF8).user_data as *mut MnName_GObj;
    let mut buttons: u32_0 = mn_80229624(4 as u32_0);
    let mut isFull: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 24] = [0; 24];
    mn_804A04F0.buttons = buttons as u64_0;
    if buttons & 0x20 as u32_0 != 0 {
        sfxBack();
        mn_804D6BC8.cooldown = 5 as u16_0;
        if (mn_804A04F0.hovered_selection as ::core::ffi::c_uint)
            < 0x18 as ::core::ffi::c_uint
        {
            mn_804A04F0.hovered_selection = 0x1a as u16_0;
            mnName_80238A04(gobj2 as *mut HSD_GObj, 0x1a as u8_0, 0 as u8_0);
            return;
        }
        mn_804A04F0.entering_menu = 0 as u8_0;
        HSD_SisLib_803A5D30();
        mn_80229894(2 as s32, 4 as u16_0, 3 as s32);
        return;
    }
    if buttons & 0x10 as u32_0 != 0 {
        mn_804D6BC8.cooldown = 5 as u16_0;
        match mn_804A04F0.hovered_selection as s32 {
            24 => {
                isFull = (if GetNameCount_noinline() < 0x78 as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as s32;
                if isFull == 0 as ::core::ffi::c_int {
                    let mut count: u8_0 = 0;
                    sfxForward();
                    mnName_80239878(1 as u8_0, gobj2 as *mut HSD_GObj);
                    mnName_80238A04(gobj2 as *mut HSD_GObj, 0x18 as u8_0, 1 as u8_0);
                    HSD_SisLib_803A5CC4((*gobj2).text2);
                    count = mnName_CountValid();
                    mnNameNew_EnterFromMnName(count as s32 as *mut ::core::ffi::c_void);
                    return;
                }
                lbAudioAx_80024030(3 as ::core::ffi::c_int);
                return;
            }
            25 => {
                if mnName_GetPageCount() != 0 as ::core::ffi::c_int {
                    sfxForward();
                    isFull = 0 as ::core::ffi::c_int as s32;
                    if (*gobj2).gobj.p_priority as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        isFull = 1 as ::core::ffi::c_int as s32;
                    }
                    (*gobj2).gobj.p_priority = isFull as u8_0;
                    mnName_SortNames(mnName_804D6BF8);
                    (*gobj2).gobj.gx_link = 0 as u8_0;
                    mnName_802385A0(gobj2 as *mut HSD_GObj);
                    return;
                }
                lbAudioAx_80024030(3 as ::core::ffi::c_int);
                return;
            }
            26 => {
                if mnName_GetPageCount() != 0 as ::core::ffi::c_int {
                    sfxForward();
                    mn_804A04F0.hovered_selection = 0 as u16_0;
                    mnName_80238A04(gobj2 as *mut HSD_GObj, 0x1a as u8_0, 1 as u8_0);
                    return;
                }
                lbAudioAx_80024030(3 as ::core::ffi::c_int);
                return;
            }
            _ => {
                if (mn_804A04F0.hovered_selection as ::core::ffi::c_uint)
                    < 0x18 as ::core::ffi::c_uint
                {
                    let mut isValid: s32 = 0;
                    let mut nameIdx: u8_0 = mnName_GetHoveredName();
                    if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char)
                        as s8 as ::core::ffi::c_int
                        == (*GetPersistentNameData(nameIdx as s32))
                            .namedata[0 as ::core::ffi::c_int as usize] as s8
                            as ::core::ffi::c_int
                    {
                        isValid = 0 as ::core::ffi::c_int as s32;
                    } else {
                        isValid = 1 as ::core::ffi::c_int as s32;
                    }
                    if isValid != 0 as ::core::ffi::c_int {
                        sfxForward();
                        mn_804A04F0.x10 = 2 as u8_0;
                        mn_804A04F0.confirmed_selection = 0 as u8_0;
                        mnName_8023A290();
                        return;
                    }
                    lbAudioAx_80024030(3 as ::core::ffi::c_int);
                    return;
                }
            }
        }
    } else {
        let mut isL: u32_0 = buttons & 0x40 as u32_0;
        if isL != 0 as u32_0 || buttons & 0x80 as u32_0 != 0 {
            if mnName_GetColumnCount() > 4 as ::core::ffi::c_int {
                sfxMove();
                if isL != 0 as u32_0 {
                    let mut scroll: u8_0 = (*gobj2).gobj.gx_link;
                    if scroll as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        (*gobj2).gobj.gx_link = (scroll as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int) as u8_0;
                    } else {
                        (*gobj2).gobj.gx_link = (mnName_GetColumnCount()
                            as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u8_0;
                    }
                } else if mnName_GetColumnCount() as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int > (*gobj2).gobj.gx_link as s32
                {
                    (*gobj2).gobj.gx_link = ((*gobj2).gobj.gx_link as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int) as u8_0;
                } else {
                    (*gobj2).gobj.gx_link = 0 as u8_0;
                }
                mnName_802385A0(gobj2 as *mut HSD_GObj);
            }
        } else if buttons & 0xf as u32_0 != 0 {
            let mut prev: u8_0 = mn_804A04F0.hovered_selection as u8_0;
            mn_804A04F0.hovered_selection = mnName_80237D94(buttons as s32, prev)
                as u16_0;
            if prev as s32 != mn_804A04F0.hovered_selection as s32 {
                sfxMove();
            }
        }
    };
}
pub unsafe extern "C" fn fn_80238540(mut gobj: *mut HSD_GObj) {
    match mn_804A04F0.x10 as s32 {
        0 => {
            mnName_MainInput(gobj);
        }
        1 => {
            mnNameNew_MainInput(gobj);
        }
        2 => {
            mnName_ConfirmNameDeleteInput(gobj);
        }
        _ => {}
    };
}
pub unsafe extern "C" fn mnName_802385A0(mut gobj: *mut HSD_GObj) {
    mnName_80238754(gobj);
    mnName_8023A058(gobj);
}
pub unsafe extern "C" fn mnName_GetPageCount() -> s32 {
    let mut count: s32 = 0 as s32;
    let mut i: s32 = 0;
    let mut extra: s32 = 0;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x78 as ::core::ffi::c_int {
        if IsNameValid(i as u8_0 as ::core::ffi::c_int) != 0 {
            count += 1;
        }
        i += 1;
    }
    if count as ::core::ffi::c_int % 24 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        extra = 1 as ::core::ffi::c_int as s32;
    } else {
        extra = 0 as ::core::ffi::c_int as s32;
    }
    return count / 24 as s32 + extra;
}
pub unsafe extern "C" fn mnName_GetColumnCount() -> s32 {
    let mut count: s32 = 0;
    let mut extra: s32 = 0;
    count = GetNumNameList() as s32;
    if count as ::core::ffi::c_int % 6 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        extra = 1 as ::core::ffi::c_int as s32;
    } else {
        extra = 0 as ::core::ffi::c_int as s32;
    }
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    return count / 6 as s32 + extra;
}
#[inline]
pub unsafe extern "C" fn mnName_80238754_noinline(mut gobj: *mut HSD_GObj) {
    mnName_80238754(gobj);
}
pub unsafe extern "C" fn mnName_80238754(mut gobj: *mut HSD_GObj) {
    let mut extra: s32 = 0;
    let mut jobj: *mut HSD_JObj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut count: s32 = GetNameCount_noinline() as s32;
    let mut col_count: s32 = 0;
    let mut f_col: f32_0 = 0.;
    if count as ::core::ffi::c_int % 6 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        extra = 1 as ::core::ffi::c_int as s32;
    } else {
        extra = 0 as ::core::ffi::c_int as s32;
    }
    col_count = count / 6 as s32 + extra;
    f_col = col_count as f32_0;
    if f_col > 4.0f32 {
        let mut scroll_pos: f32_0 = 0.;
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
        scroll_pos = ((*gobj).gx_link as u32_0 as ::core::ffi::c_float
            * (14.0f32 / (f_col as ::core::ffi::c_float - 1.0f32))) as f32_0;
        HSD_JObjSetTranslateX((*gobj).user_data as *mut HSD_JObj, scroll_pos);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
    };
}
#[inline]
pub unsafe extern "C" fn mnName_802388D4_noinline(
    mut gobj: *mut HSD_GObj,
    mut index: u8_0,
) -> *mut HSD_JObj {
    return mnName_802388D4(gobj, index);
}
pub unsafe extern "C" fn mnName_802388D4(
    mut gobj: *mut HSD_GObj,
    mut index: u8_0,
) -> *mut HSD_JObj {
    let mut p: *mut u8_0 = gobj as *mut u8_0;
    let mut result: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    if (index as ::core::ffi::c_int) < 0x18 as ::core::ffi::c_int {
        let mut jobj: *mut HSD_JObj = *(p.offset(0x30 as ::core::ffi::c_int as isize)
            as *mut *mut HSD_JObj);
        let mut i: s32 = 0;
        result = (if jobj.is_null() {
            ::core::ptr::null_mut::<HSD_JObj>()
        } else {
            (*jobj).child
        }) as *mut HSD_JObj;
        i = 0 as ::core::ffi::c_int as s32;
        while i < index as ::core::ffi::c_int {
            result = (if result.is_null() {
                ::core::ptr::null_mut::<HSD_JObj>()
            } else {
                (*result).next
            }) as *mut HSD_JObj;
            i += 1;
        }
        return result;
    }
    match index as ::core::ffi::c_int {
        24 => {
            return *(p.offset(0x24 as ::core::ffi::c_int as isize) as *mut *mut HSD_JObj);
        }
        25 => {
            return *(p.offset(0x18 as ::core::ffi::c_int as isize) as *mut *mut HSD_JObj);
        }
        26 => {
            return *(p.offset(0x1c as ::core::ffi::c_int as isize) as *mut *mut HSD_JObj);
        }
        _ => {}
    }
    return gobj as *mut HSD_JObj;
}
#[inline]
pub unsafe extern "C" fn mnName_80238964_noinline(
    mut index: u8_0,
    mut target: u8_0,
    mut flag: u8_0,
) -> f32_0 {
    return mnName_80238964(index, target, flag);
}
pub unsafe extern "C" fn mnName_80238964(
    mut index: u8_0,
    mut target: u8_0,
    mut flag: u8_0,
) -> f32_0 {
    let mut idx: s32 = 0;
    let mut base: *mut AnimLoopSettings = &raw mut mnName_803ED538
        as *mut AnimLoopSettings;
    if target as ::core::ffi::c_int == 0x18 as ::core::ffi::c_int {
        if flag != 0 {
            return (*if 5 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                &raw mut mnName_803ED568
            } else if 5 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                &raw mut mnName_803ED574
            } else if 5 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                &raw mut mnName_803ED580
            } else if 5 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                &raw mut mnName_803ED58C
            } else if 5 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                &raw mut mnName_803ED598
            } else {
                &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
            })
                .start_frame;
        }
        return (*if 4 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
            &raw mut mnName_803ED568
        } else if 4 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
            &raw mut mnName_803ED574
        } else if 4 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
            &raw mut mnName_803ED580
        } else if 4 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
            &raw mut mnName_803ED58C
        } else if 4 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
            &raw mut mnName_803ED598
        } else {
            &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
        })
            .start_frame;
    }
    idx = index as s32;
    match idx {
        24 => {
            if flag != 0 {
                return (*if 8 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                    &raw mut mnName_803ED568
                } else if 8 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                    &raw mut mnName_803ED574
                } else if 8 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                    &raw mut mnName_803ED580
                } else if 8 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                    &raw mut mnName_803ED58C
                } else if 8 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                    &raw mut mnName_803ED598
                } else {
                    &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
                })
                    .start_frame;
            }
            return (*if 6 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                &raw mut mnName_803ED568
            } else if 6 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                &raw mut mnName_803ED574
            } else if 6 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                &raw mut mnName_803ED580
            } else if 6 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                &raw mut mnName_803ED58C
            } else if 6 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                &raw mut mnName_803ED598
            } else {
                &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
            })
                .start_frame;
        }
        25 | 26 => {
            if flag != 0 {
                return (*if 8 as ::core::ffi::c_int
                    + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                        as ::core::ffi::c_int == 4 as ::core::ffi::c_int
                {
                    &raw mut mnName_803ED568
                } else if 8 as ::core::ffi::c_int
                    + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                        as ::core::ffi::c_int == 5 as ::core::ffi::c_int
                {
                    &raw mut mnName_803ED574
                } else if 8 as ::core::ffi::c_int
                    + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                        as ::core::ffi::c_int == 6 as ::core::ffi::c_int
                {
                    &raw mut mnName_803ED580
                } else if 8 as ::core::ffi::c_int
                    + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                        as ::core::ffi::c_int == 7 as ::core::ffi::c_int
                {
                    &raw mut mnName_803ED58C
                } else if 8 as ::core::ffi::c_int
                    + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                        as ::core::ffi::c_int == 8 as ::core::ffi::c_int
                {
                    &raw mut mnName_803ED598
                } else {
                    &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
                })
                    .start_frame;
            }
            return (*if 6 as ::core::ffi::c_int
                + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                    as ::core::ffi::c_int == 4 as ::core::ffi::c_int
            {
                &raw mut mnName_803ED568
            } else if 6 as ::core::ffi::c_int
                + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                    as ::core::ffi::c_int == 5 as ::core::ffi::c_int
            {
                &raw mut mnName_803ED574
            } else if 6 as ::core::ffi::c_int
                + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                    as ::core::ffi::c_int == 6 as ::core::ffi::c_int
            {
                &raw mut mnName_803ED580
            } else if 6 as ::core::ffi::c_int
                + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                    as ::core::ffi::c_int == 7 as ::core::ffi::c_int
            {
                &raw mut mnName_803ED58C
            } else if 6 as ::core::ffi::c_int
                + (index as ::core::ffi::c_int == target as ::core::ffi::c_int)
                    as ::core::ffi::c_int == 8 as ::core::ffi::c_int
            {
                &raw mut mnName_803ED598
            } else {
                &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
            })
                .start_frame;
        }
        _ => {}
    }
    panic!("Reached end of non-void function without returning");
}
pub unsafe extern "C" fn mnName_80238A04(
    mut gobj: *mut HSD_GObj,
    mut target: u8_0,
    mut flag: u8_0,
) {
    let mut base: *mut AnimLoopSettings = &raw mut mnName_803ED538
        as *mut AnimLoopSettings;
    let mut jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut i: ::core::ffi::c_int = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    i = 0x18 as ::core::ffi::c_int;
    while i < 0x1b as ::core::ffi::c_int {
        jobj = mnName_802388D4_noinline(gobj, i as u8_0);
        HSD_JObjReqAnimAll(jobj, mnName_80238964_noinline(i as u8_0, target, flag));
        HSD_JObjAnimAll(jobj);
        i += 1;
    }
    jobj2 = (*gobj).prev_gx as *mut HSD_JObj;
    if target as ::core::ffi::c_int == 0x18 as ::core::ffi::c_int {
        if flag != 0 {
            HSD_JObjReqAnimAll(
                jobj2,
                (*if 5 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                    &raw mut mnName_803ED568
                } else if 5 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                    &raw mut mnName_803ED574
                } else if 5 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                    &raw mut mnName_803ED580
                } else if 5 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                    &raw mut mnName_803ED58C
                } else if 5 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                    &raw mut mnName_803ED598
                } else {
                    &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
                })
                    .start_frame,
            );
        } else {
            HSD_JObjReqAnimAll(
                jobj2,
                (*if 4 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                    &raw mut mnName_803ED568
                } else if 4 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                    &raw mut mnName_803ED574
                } else if 4 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                    &raw mut mnName_803ED580
                } else if 4 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                    &raw mut mnName_803ED58C
                } else if 4 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                    &raw mut mnName_803ED598
                } else {
                    &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
                })
                    .start_frame,
            );
        }
    } else if flag != 0 {
        HSD_JObjReqAnimAll(
            jobj2,
            (*if 8 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                &raw mut mnName_803ED568
            } else if 8 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                &raw mut mnName_803ED574
            } else if 8 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                &raw mut mnName_803ED580
            } else if 8 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                &raw mut mnName_803ED58C
            } else if 8 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                &raw mut mnName_803ED598
            } else {
                &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
            })
                .start_frame,
        );
    } else {
        HSD_JObjReqAnimAll(
            jobj2,
            (*if 6 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                &raw mut mnName_803ED568
            } else if 6 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                &raw mut mnName_803ED574
            } else if 6 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                &raw mut mnName_803ED580
            } else if 6 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                &raw mut mnName_803ED58C
            } else if 6 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                &raw mut mnName_803ED598
            } else {
                &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
            })
                .start_frame,
        );
    }
    HSD_JObjAnimAll(jobj2);
}
pub unsafe extern "C" fn mnName_80238AE0(
    mut gobj: *mut HSD_GObj,
    mut index: u8_0,
    mut arg2: u8_0,
) {
    let mut color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut colorptr: *mut GXColor = ::core::ptr::null_mut::<GXColor>();
    if (index as ::core::ffi::c_int) < 0x18 as ::core::ffi::c_int {
        jobj = mnName_802388D4_noinline(gobj, index);
        HSD_JObjReqAnimAll(jobj, arg2 as f32_0);
        HSD_JObjAnimAll(jobj);
        if arg2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            colorptr = &raw mut mnName_804D4BE0;
        } else {
            colorptr = &raw mut mnName_804D4BE4;
        }
        color = *colorptr;
        HSD_SisLib_803A74F0(
            (*(gobj as *mut MnName_GObj)).text,
            index as s32,
            &raw mut color,
        );
        return;
    }
    jobj = mnName_802388D4_noinline(gobj, index);
    if (index as ::core::ffi::c_int) < 0x18 as ::core::ffi::c_int {
        HSD_JObjReqAnimAll(jobj, arg2 as f32_0);
    } else {
        match index as ::core::ffi::c_int {
            24 => {
                HSD_JObjReqAnimAll(jobj, mnName_804D4BD0[arg2 as usize]);
            }
            25 => {
                HSD_JObjReqAnimAll(jobj, mnName_804D4BD8[arg2 as usize]);
            }
            26 => {
                HSD_JObjReqAnimAll(jobj, mnName_804D4BD8[arg2 as usize]);
            }
            _ => {}
        }
    }
    HSD_JObjAnimAll(jobj);
}
#[inline]
pub unsafe extern "C" fn mnName_FindAnimLoop(
    mut tableBase: *const *mut AnimLoopSettings,
    mut frame: f32_0,
) -> *mut AnimLoopSettings {
    let mut i: s32 = 0;
    let mut table: AnimTable = AnimTable {
        entries: [::core::ptr::null_mut::<AnimLoopSettings>(); 6],
    };
    table = *(tableBase as *mut AnimTable) as AnimTable;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 6 as ::core::ffi::c_int {
        if (*table.entries[i as usize]).start_frame <= frame
            && frame <= (*table.entries[i as usize]).end_frame
        {
            return table.entries[i as usize];
        }
        i += 1;
    }
    if 0 as ::core::ffi::c_long != 0 {} else {
        OSReport(b"But AnimFrame!!!\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnname.c\0" as *const u8 as *const ::core::ffi::c_char,
            994 as u32_0,
            b"NULL\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    panic!("Reached end of non-void function without returning");
}
#[inline]
pub unsafe extern "C" fn mnName_80238C34_inline(mut anim: *mut AnimLoopSettings) -> f32_0 {
    return (*anim).end_frame;
}
#[inline]
pub unsafe extern "C" fn mnName_UpdateSelection(
    mut do_update: u8_0,
    mut data: *mut MnName_GObj,
) {
    if do_update != 0 {
        let mut prev: u32_0 = (*(data as *mut MnNameUserDataState)).prev_selection
            as u32_0;
        if prev != 0x1a as u32_0
            || mn_804A04F0.hovered_selection as ::core::ffi::c_uint
                >= 0x18 as ::core::ffi::c_uint
        {
            mnName_80238AE0(data as *mut HSD_GObj, prev as u8_0, 0 as u8_0);
        }
        mnName_80238AE0(
            data as *mut HSD_GObj,
            mn_804A04F0.hovered_selection as u8_0,
            1 as u8_0,
        );
    }
}
pub unsafe extern "C" fn mnName_80238C34(
    mut arg0: *mut HSD_GObj,
    mut arg1: u8_0,
    mut arg2: u8_0,
) {
    let mut tableBase: *const *mut AnimLoopSettings = &raw const mnName_803B8510
        as *const *mut AnimLoopSettings;
    let mut base: *mut AnimLoopSettings = &raw mut mnName_803ED538
        as *mut AnimLoopSettings;
    let mut data: *mut MnName_GObj = (*arg0).user_data as *mut MnName_GObj;
    let mut jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    mnName_UpdateSelection(arg1, data);
    let mut jobj: *mut HSD_JObj = mnName_802388D4_noinline(
        data as *mut HSD_GObj,
        0x18 as u8_0,
    );
    mn_8022ED6C(
        jobj,
        mnName_FindAnimLoop(
            &raw const mnName_803B8510 as *const *mut AnimLoopSettings,
            mn_8022F298(jobj),
        ),
    );
    jobj2 = mnName_802388D4_noinline(data as *mut HSD_GObj, 0x19 as u8_0);
    mn_8022ED6C(
        jobj2,
        mnName_FindAnimLoop(
            &raw const mnName_803B8510 as *const *mut AnimLoopSettings,
            mn_8022F298(jobj2),
        ),
    );
    jobj3 = mnName_802388D4_noinline(data as *mut HSD_GObj, 0x1a as u8_0);
    mn_8022ED6C(
        jobj3,
        mnName_FindAnimLoop(
            &raw const mnName_803B8510 as *const *mut AnimLoopSettings,
            mn_8022F298(jobj3),
        ),
    );
    let mut jobj_0: *mut HSD_JObj = (*data).gobj.prev_gx as *mut HSD_JObj;
    let mut result: f32_0 = 0.;
    result = mn_8022ED6C(
        jobj_0,
        mnName_FindAnimLoop(
            &raw const mnName_803B8510 as *const *mut AnimLoopSettings,
            mn_8022F298(jobj_0),
        ),
    ) as f32_0;
    if mnName_FindAnimLoop(tableBase, result)
        == (if 5 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
            &raw mut mnName_803ED568
        } else {
            (if 5 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                &raw mut mnName_803ED574
            } else {
                (if 5 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                    &raw mut mnName_803ED580
                } else {
                    (if 5 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                        &raw mut mnName_803ED58C
                    } else {
                        (if 5 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                            &raw mut mnName_803ED598
                        } else {
                            &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
                        })
                    })
                })
            })
        })
    {
        if result
            >= mnName_80238C34_inline(
                (if 5 as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                    &raw mut mnName_803ED568
                } else {
                    (if 5 as ::core::ffi::c_int == 5 as ::core::ffi::c_int {
                        &raw mut mnName_803ED574
                    } else {
                        (if 5 as ::core::ffi::c_int == 6 as ::core::ffi::c_int {
                            &raw mut mnName_803ED580
                        } else {
                            (if 5 as ::core::ffi::c_int == 7 as ::core::ffi::c_int {
                                &raw mut mnName_803ED58C
                            } else {
                                (if 5 as ::core::ffi::c_int == 8 as ::core::ffi::c_int {
                                    &raw mut mnName_803ED598
                                } else {
                                    &raw mut mnName_803ED5A4 as *mut AnimLoopSettings
                                })
                            })
                        })
                    })
                }),
            )
        {
            HSD_GObjFree(arg0);
        }
    }
}
pub unsafe extern "C" fn fn_80239574(mut arg0: *mut HSD_GObj) {
    let mut current_block: u64;
    let mut anim: *mut AnimLoopSettings = ::core::ptr::null_mut::<AnimLoopSettings>();
    let mut new_var: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut base: *mut AnimLoopSettings = &raw mut mnName_803ED538
        as *mut AnimLoopSettings;
    let mut doUpdate: s32 = 0 as s32;
    let mut doSel: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut doSelReset: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut data: *mut MnName_GObj = (*arg0).user_data as *mut MnName_GObj;
    let mut new_var2: s32 = 0;
    let mut state: u8_0 = (*data).gobj.p_link;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    if (state as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || (*data).gobj.p_link as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        || (*data).gobj.p_link as ::core::ffi::c_int == 3 as ::core::ffi::c_int)
        && *(data as *mut u8_0) as ::core::ffi::c_int
            != mn_804A04F0.cur_menu as ::core::ffi::c_int
    {
        if mn_804A04F0.buttons & 0x10 as u64_0 != 0 {
            (*data).gobj.p_link = 4 as u8_0;
        } else if mn_804A04F0.buttons & 0x20 as u64_0 != 0 {
            (*data).gobj.p_link = 2 as u8_0;
        }
        state = (*data).gobj.p_link;
        match state as s32 {
            1 => {
                anim = base.offset(0 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            2 => {
                anim = base.offset(2 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            3 => {
                anim = base.offset(1 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            4 => {
                anim = base.offset(3 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            _ => {}
        }
        let mut jobj: *mut HSD_JObj = (*data).gobj.next_gx as *mut HSD_JObj;
        HSD_JObjReqAnim(jobj, (*anim).start_frame);
        HSD_JObjAnim(jobj);
        state = (*data).gobj.p_link;
        if state as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            || state as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || state as ::core::ffi::c_int == 3 as ::core::ffi::c_int
        {
            doUpdate = 1 as ::core::ffi::c_int as s32;
            doSel = 1 as ::core::ffi::c_int;
            doSelReset = 1 as ::core::ffi::c_uint;
        }
    }
    state = (*data).gobj.p_link;
    if state as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        match state as s32 {
            1 => {
                anim = base.offset(0 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            2 => {
                anim = base.offset(2 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            3 => {
                anim = base.offset(1 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            4 => {
                anim = base.offset(3 as ::core::ffi::c_int as isize)
                    as *mut AnimLoopSettings;
            }
            _ => {}
        }
        let mut jobj_0: *mut HSD_JObj = (*data).gobj.next_gx as *mut HSD_JObj;
        new_var = jobj_0;
        if mn_8022F298(new_var) >= (*anim).end_frame {
            state = (*data).gobj.p_link;
            match state as s32 {
                3 | 1 => {
                    current_block = 5257379605652634929;
                    match current_block {
                        3430571300503193481 => {
                            HSD_GObjFree(arg0);
                            return;
                        }
                        _ => {
                            (*data).gobj.p_link = 0 as u8_0;
                        }
                    }
                }
                2 | 4 => {
                    current_block = 3430571300503193481;
                    match current_block {
                        3430571300503193481 => {
                            HSD_GObjFree(arg0);
                            return;
                        }
                        _ => {
                            (*data).gobj.p_link = 0 as u8_0;
                        }
                    }
                }
                _ => {}
            }
        }
        HSD_JObjAnim(new_var);
    }
    if mn_804A04F0.x10 as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        state = (*data).gobj.p_link;
        if (state as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            || state as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            || state as ::core::ffi::c_int == 3 as ::core::ffi::c_int)
            && *(data as *mut u8_0).offset(1 as ::core::ffi::c_int as isize) as s32
                != mn_804A04F0.hovered_selection as s32
        {
            doSel = 1 as ::core::ffi::c_int;
        }
    }
    mnName_80238C34(arg0, doSel as u8_0, doSelReset as u8_0);
    if doSel != 0 as ::core::ffi::c_int
        && mn_804A04F0.hovered_selection as ::core::ffi::c_uint
            >= 0x18 as ::core::ffi::c_uint
        && *(data as *mut u8_0).offset(1 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_uint >= 0x18 as ::core::ffi::c_uint
    {
        let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        let mut idx: s32 = mn_804A04F0.hovered_selection as s32 - 0x18 as s32;
        new_var2 = idx;
        if !(*data).text2.is_null() {
            HSD_SisLib_803A5CC4((*data).text2);
        }
        text = HSD_SisLib_803A5ACC(
            0 as ::core::ffi::c_int,
            0 as s32,
            -9.5f32,
            9.1f32,
            17.0f32,
            364.68332f32,
            38.38772f32,
        );
        (*data).text2 = text;
        (*text).font_size.x = 0.0521f32 as f32_0;
        (*text).font_size.y = 0.0521f32 as f32_0;
        HSD_SisLib_803A6368(text, mnName_804D4BE8[new_var2 as usize] as s32);
    }
    if doUpdate != 0 as ::core::ffi::c_int {
        *(data as *mut u8_0) = mn_804A04F0.cur_menu;
    }
    if doSel != 0 as ::core::ffi::c_int {
        *(data as *mut u8_0).offset(1 as ::core::ffi::c_int as isize) = mn_804A04F0
            .hovered_selection as u8_0;
    }
}
pub unsafe extern "C" fn mnName_80239878(mut arg0: u8_0, mut gobj: *mut HSD_GObj) {
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut count: s32 = 0;
    let mut extra: s32 = 0;
    let mut col_count: s32 = 0;
    let mut f_col: f32_0 = 0.;
    if arg0 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        mnName_80239FFC(gobj);
        HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32_0);
        return;
    }
    mnName_8023A058(gobj);
    jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    count = GetNameCount_noinline() as s32;
    if count as ::core::ffi::c_int % 6 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        extra = 1 as ::core::ffi::c_int as s32;
    } else {
        extra = 0 as ::core::ffi::c_int as s32;
    }
    col_count = count / 6 as s32 + extra;
    f_col = col_count as f32_0;
    if f_col > 4.0f32 {
        let mut scroll_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        let mut scroll_pos: f32_0 = 0.;
        HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
        scroll_pos = ((*gobj).gx_link as ::core::ffi::c_float
            * (14.0f32 / (f_col as ::core::ffi::c_float - 1.0f32))) as f32_0;
        HSD_JObjSetTranslateX((*gobj).user_data as *mut HSD_JObj, scroll_pos);
    } else {
        HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32_0);
    };
}
#[inline]
pub unsafe extern "C" fn mnName_TextWidth(
    mut start: f32_0,
    mut end: *mut HSD_JObj,
) -> f32_0 {
    return HSD_JObjGetTranslationX(end) - start;
}
#[inline]
pub unsafe extern "C" fn mnName_GetDisplayIndex(mut cell: s32) -> s32 {
    let mut index: u16_0 = mnName_NameDisplayOrder[cell as usize] as u16_0;
    return index as s32;
}
pub unsafe extern "C" fn mnName_80239A24(mut gobj: *mut HSD_GObj) {
    let mut text_row_height: f32_0 = 0.;
    let mut row: s32 = 0;
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut ref_jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut ref_jobj2: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text_position: Vec3 = Vec3 { x: 0., y: 0., z: 0. };
    let mut text_jobj0: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text_jobj1: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut col_width: f32_0 = 0.;
    let mut row_height: f32_0 = 0.;
    let mut text_col_width: f32_0 = 0.;
    let mut color_dst: *mut GXColor = ::core::ptr::null_mut::<GXColor>();
    let mut j: s32 = 0;
    let mut i: s32 = 0;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut archive: *mut MnNameArchive = &raw mut mnName_804A06C0;
    let mut data: *mut MnName_GObj = gobj as *mut MnName_GObj;
    let mut ref_jobj3: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut text_color: GXColor = _GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut count: s32 = 0;
    let mut extra: s32 = 0;
    let mut total_rows: s32 = 0;
    let mut is_valid: s32 = 0;
    let mut namedata: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<
        ::core::ffi::c_char,
    >();
    let mut is_valid2: s32 = 0;
    let mut color: *mut GXColor = ::core::ptr::null_mut::<GXColor>();
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut name_idx: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x18 as ::core::ffi::c_int {
        jobj = HSD_JObjLoadJoint((*archive).joint as *mut HSD_Joint);
        HSD_JObjAddAnimAll(
            jobj,
            (*archive).anim_joint as *mut HSD_AnimJoint,
            (*archive).matanim_joint as *mut HSD_MatAnimJoint,
            (*archive).shapeanim_joint as *mut HSD_ShapeAnimJoint,
        );
        HSD_JObjReqAnimAll(
            jobj,
            (i as u8_0 as ::core::ffi::c_int
                == mn_804A04F0.hovered_selection as ::core::ffi::c_int)
                as ::core::ffi::c_int as f32_0,
        );
        HSD_JObjAnimAll(jobj);
        ref_jobj = ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
            *mut HSD_JObj,
        >((*data).gobj.user_data_remove_func);
        HSD_JObjGetTranslationX(ref_jobj);
        col_width = (*ref_jobj).translate.x;
        col_width = HSD_JObjGetTranslationX((*data).gobj.x34_unk as *mut HSD_JObj)
            - col_width;
        row_height = HSD_JObjGetTranslationY(
            ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
                *mut HSD_JObj,
            >((*data).gobj.user_data_remove_func),
        );
        row_height = HSD_JObjGetTranslationY((*data).x38 as *mut HSD_JObj) - row_height;
        mnName_80239F5C(
            jobj,
            col_width
                * (i as u8_0 as ::core::ffi::c_int / 6 as ::core::ffi::c_int) as f32_0,
        );
        mnName_80239EBC(
            jobj,
            row_height
                * (i as u8_0 as ::core::ffi::c_int % 6 as ::core::ffi::c_int) as f32_0,
        );
        child = jobj;
        HSD_JObjAddChild(
            ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
                *mut HSD_JObj,
            >((*data).gobj.user_data_remove_func),
            child,
        );
        i += 1;
    }
    text = HSD_SisLib_803A6754(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*data).text = text;
    text_jobj0 = mnName_802388D4(gobj, 0 as u8_0);
    lb_8000B1CC(
        text_jobj0,
        &raw mut mnName_803ED618 as *mut Vec3,
        &raw mut text_position,
    );
    let mut pos_y: f32_0 = -text_position.y;
    let mut pos_z: f32_0 = text_position.z;
    let mut pos_x: f32_0 = text_position.x;
    (*text).pos_x = pos_x;
    (*text).pos_y = pos_y;
    (*text).pos_z = pos_z;
    (*text).font_size.x = 0.03f32 as f32_0;
    (*text).font_size.y = 0.03f32 as f32_0;
    (*text).text_color = mnName_804D4BE4;
    ref_jobj2 = mnName_802388D4_noinline(gobj, 6 as u8_0);
    HSD_JObjGetTranslationX(text_jobj0);
    text_col_width = (*text_jobj0).translate.x;
    text_col_width = mnName_TextWidth(text_col_width, ref_jobj2);
    text_jobj1 = mnName_802388D4_noinline(gobj, 1 as u8_0);
    text_row_height = HSD_JObjGetTranslationY(text_jobj0);
    text_row_height = -(HSD_JObjGetTranslationY(text_jobj1) - text_row_height);
    j = 0 as ::core::ffi::c_int as s32;
    while j < 0x18 as ::core::ffi::c_int {
        let mut global_data: *mut MnName_GObj = (*mnName_804D6BF8).user_data
            as *mut MnName_GObj;
        row = ((*global_data).gobj.gx_link as ::core::ffi::c_int
            + j as u8_0 as ::core::ffi::c_int / 6 as ::core::ffi::c_int) as s32;
        count = GetNameCount_noinline() as s32;
        if count as ::core::ffi::c_int % 6 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            extra = 1 as ::core::ffi::c_int as s32;
        } else {
            extra = 0 as ::core::ffi::c_int as s32;
        }
        total_rows = count / 6 as s32 + extra;
        if total_rows > 4 as ::core::ffi::c_int && row >= total_rows {
            row -= total_rows;
        }
        name_idx = mnName_GetDisplayIndex(j as u8_0 as s32 % 6 as s32 + row * 6 as s32);
        if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as u8_0 as s8
            as ::core::ffi::c_int
            == (*GetPersistentNameData(name_idx))
                .namedata[0 as ::core::ffi::c_int as usize] as s8 as ::core::ffi::c_int
        {
            is_valid = 0 as ::core::ffi::c_int as s32;
        } else {
            is_valid = 1 as ::core::ffi::c_int as s32;
        }
        if is_valid != 0 as ::core::ffi::c_int {
            if *(&raw mut mnName_StringTerminator as *mut ::core::ffi::c_char) as u8_0
                as s8 as ::core::ffi::c_int
                == (*GetPersistentNameData(name_idx))
                    .namedata[0 as ::core::ffi::c_int as usize] as s8
                    as ::core::ffi::c_int
            {
                is_valid2 = 0 as ::core::ffi::c_int as s32;
            } else {
                is_valid2 = 1 as ::core::ffi::c_int as s32;
            }
            if is_valid2 == 0 as ::core::ffi::c_int {
                namedata = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                namedata = (GetPersistentNameData(name_idx) as *mut ::core::ffi::c_char)
                    .offset(0x198 as ::core::ffi::c_int as isize);
            }
            let mut font_x: f32_0 = (*text).font_size.x;
            let mut x: f32_0 = (j as ::core::ffi::c_int / 6 as ::core::ffi::c_int)
                as f32_0 * text_col_width;
            HSD_SisLib_803A6B98(text, x as ::core::ffi::c_float / font_x as ::core::ffi::c_float, (j as ::core::ffi::c_int % 6 as ::core::ffi::c_int)
                    as ::core::ffi::c_float * text_row_height as ::core::ffi::c_float
                    / (*text).font_size.y as ::core::ffi::c_float, namedata, &[]);
            if j == mn_804A04F0.hovered_selection as s32 {
                color = &raw mut mnName_804D4BE0;
            } else {
                color = &raw mut mnName_804D4BE4;
            }
            text_color = *color;
            color_dst = &raw mut text_color;
            HSD_SisLib_803A74F0(text, j, color_dst);
        } else {
            HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, &raw mut mnName_StringTerminator as *mut ::core::ffi::c_char, &[]);
        }
        j += 1;
    }
}
pub unsafe extern "C" fn mnName_80239EBC(mut jobj: *mut HSD_JObj, mut y: f32_0) {
    HSD_JObjSetTranslateY(jobj, y);
}
pub unsafe extern "C" fn mnName_80239F5C(mut jobj: *mut HSD_JObj, mut x: f32_0) {
    HSD_JObjSetTranslateX(jobj, x);
}
pub unsafe extern "C" fn mnName_80239FFC(mut gobj: *mut HSD_GObj) {
    let mut mn: *mut MnName_GObj = gobj as *mut MnName_GObj;
    let mut jobj: *mut HSD_JObj = ::core::mem::transmute::<
        Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
        *mut HSD_JObj,
    >((*mn).gobj.user_data_remove_func);
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    if jobj.is_null() {
        child = ::core::ptr::null_mut::<HSD_JObj>();
    } else {
        child = (*jobj).child as *mut HSD_JObj;
    }
    HSD_JObjRemoveAll(child);
    if !(*mn).text.is_null() {
        HSD_SisLib_803A5CC4((*mn).text);
        (*mn).text = ::core::ptr::null_mut::<HSD_Text>();
    }
}
pub unsafe extern "C" fn mnName_8023A058(mut gobj: *mut HSD_GObj) {
    let mut mn: *mut MnName_GObj = gobj as *mut MnName_GObj;
    let mut jobj: *mut HSD_JObj = ::core::mem::transmute::<
        Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
        *mut HSD_JObj,
    >((*mn).gobj.user_data_remove_func);
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    if jobj.is_null() {
        child = ::core::ptr::null_mut::<HSD_JObj>();
    } else {
        child = (*jobj).child as *mut HSD_JObj;
    }
    HSD_JObjRemoveAll(child);
    if !(*mn).text.is_null() {
        HSD_SisLib_803A5CC4((*mn).text);
        (*mn).text = ::core::ptr::null_mut::<HSD_Text>();
    }
    mnName_80239A24(gobj);
}
pub static mut mnName_804D6BFC: *mut HSD_Text = ::core::ptr::null::<HSD_Text>()
    as *mut HSD_Text;
pub unsafe extern "C" fn fn_8023A0BC(mut gobj: *mut HSD_GObj) {
    let mut end_frame: *mut f32_0 = ::core::ptr::null_mut::<f32_0>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp2C: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp28: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut pad1: u32_0 = 0;
    let mut pad2: u32_0 = 0;
    let mut pad3: u32_0 = 0;
    let mut sp18: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp14: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut frame: f32_0 = 0.;
    let mut frame2: f32_0 = 0.;
    let mut new_var: *mut f32_0 = ::core::ptr::null_mut::<f32_0>();
    let mut sel: u8_0 = 0;
    let mut base: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut pad4: u32_0 = 0;
    let mut pad5: u32_0 = 0;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    base = &raw mut mnName_803ED538 as *mut AnimLoopSettings as *mut u8_0;
    lb_80011E24(jobj, &raw mut sp2C, &[(2 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    sel = mn_804A04F0.x10;
    if sel as ::core::ffi::c_int != 2 as ::core::ffi::c_int {
        HSD_SisLib_803A5CC4(mnName_804D6BFC);
        mnName_804D6BFC = ::core::ptr::null_mut::<HSD_Text>();
        HSD_GObjFree(gobj);
        return;
    }
    frame = mn_8022F298(sp2C);
    let mut current_block_36: u64;
    if mnName_803ED600[0 as ::core::ffi::c_int as usize] <= frame {
        end_frame = (&raw mut mnName_803ED600 as *mut f32_0)
            .offset(1 as ::core::ffi::c_int as isize) as *mut f32_0;
        new_var = (&raw mut mnName_803ED600 as *mut f32_0)
            .offset(1 as ::core::ffi::c_int as isize) as *mut f32_0;
        if frame < *new_var {
            frame2 = mn_8022EFD8(sp2C, &raw mut mnName_803ED600 as *mut AnimLoopSettings)
                as f32_0;
            lb_80011E24(jobj, &raw mut sp28, &[(8 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            mn_8022EFD8(sp28, &raw mut mnName_803ED600 as *mut AnimLoopSettings);
            if frame2 >= *end_frame {
                if !mnName_804D6BFC.is_null() {
                    HSD_SisLib_803A5CC4(mnName_804D6BFC);
                }
                text = HSD_SisLib_803A5ACC(
                    0 as ::core::ffi::c_int,
                    1 as s32,
                    mnName_803ED618[1 as ::core::ffi::c_int as usize].x,
                    mnName_803ED618[1 as ::core::ffi::c_int as usize].y,
                    mnName_803ED618[1 as ::core::ffi::c_int as usize].z,
                    416.6667f32,
                    33.333336f32,
                );
                mnName_804D6BFC = text;
                (*text).font_size.x = 0.03f32 as f32_0;
                (*text).font_size.y = 0.03f32 as f32_0;
                HSD_SisLib_803A6368(text, 0x49 as s32);
            }
            current_block_36 = 7245201122033322888;
        } else {
            current_block_36 = 6913004376374473933;
        }
    } else {
        current_block_36 = 6913004376374473933;
    }
    match current_block_36 {
        6913004376374473933 => {
            sel = mn_804A04F0.confirmed_selection;
            lb_80011E24(jobj, &raw mut sp14, &[(6 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            lb_80011E24(jobj, &raw mut sp18, &[(7 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
            if sel as s32 != 0 as ::core::ffi::c_int {
                HSD_JObjReqAnimAll(sp14, 1.0f32);
                HSD_JObjReqAnimAll(sp18, 0.0f32);
            } else {
                HSD_JObjReqAnimAll(sp14, 0.0f32);
                HSD_JObjReqAnimAll(sp18, 1.0f32);
            }
            HSD_JObjAnimAll(sp14);
            HSD_JObjAnimAll(sp18);
        }
        _ => {}
    };
}
#[inline]
pub unsafe extern "C" fn mnName_SetupDeleteCursor(mut sel: s32, mut jobj: *mut HSD_JObj) {
    let mut yes: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut no: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    lb_80011E24(jobj, &raw mut yes, &[(6 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    lb_80011E24(jobj, &raw mut no, &[(7 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    if sel != 0 as ::core::ffi::c_int {
        HSD_JObjReqAnimAll(yes, 1.0f32);
        HSD_JObjReqAnimAll(no, 0.0f32);
    } else {
        HSD_JObjReqAnimAll(yes, 0.0f32);
        HSD_JObjReqAnimAll(no, 1.0f32);
    }
    HSD_JObjAnimAll(yes);
    HSD_JObjAnimAll(no);
}
pub unsafe extern "C" fn mnName_8023A290() {
    let mut sp28: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp24: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut sp20: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut archive: *mut MnNameArchive = &raw mut mnName_804A06D0;
    gobj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    jobj = HSD_JObjLoadJoint((*archive).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj as *mut ::core::ffi::c_void);
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        6 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_8023A0BC as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*archive).anim_joint as *mut HSD_AnimJoint,
        (*archive).matanim_joint as *mut HSD_MatAnimJoint,
        (*archive).shapeanim_joint as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, mnName_803ED600[0 as ::core::ffi::c_int as usize]);
    HSD_JObjAnimAll(jobj);
    lb_80011E24(jobj, &raw mut sp28, &[(0xa as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    HSD_JObjSetFlagsAll(sp28, JOBJ_HIDDEN as u32_0);
    lb_80011E24(jobj, &raw mut sp28, &[(0xb as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    HSD_JObjSetFlagsAll(sp28, JOBJ_HIDDEN as u32_0);
    mnName_SetupDeleteCursor(mn_804A04F0.confirmed_selection as s32, jobj);
    if lbLang_IsSavedLanguageUS() != 0 {
        lb_80011E24(jobj, &raw mut sp24, &[(6 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        lb_80011E24(jobj, &raw mut sp20, &[(7 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        let mut x1: f32_0 = HSD_JObjGetTranslationX(sp24);
        HSD_JObjSetTranslateX(sp24, HSD_JObjGetTranslationX(sp20));
        HSD_JObjSetTranslateX(sp20, x1);
    }
}
#[inline]
pub unsafe extern "C" fn mnName_SetupScrollbarAndText(
    mut count: s32,
    mut scrollbar: *mut HSD_JObj,
    mut user_data: *mut MnName_GObj,
) {
    let mut extra: s32 = 0;
    let mut rows: f32_0 = 0.;
    let mut pos: f32_0 = 0.;
    let mut txt: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    if count as ::core::ffi::c_int % 6 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        extra = 1 as ::core::ffi::c_int as s32;
    } else {
        extra = 0 as ::core::ffi::c_int as s32;
    }
    rows = (count / 6 as s32 + extra) as f32_0;
    if rows > 4.0f32 {
        HSD_JObjClearFlagsAll(scrollbar, JOBJ_HIDDEN as u32_0);
        pos = ((*user_data).gobj.gx_link as ::core::ffi::c_float
            * (14.0f32 / (rows as ::core::ffi::c_float - 1.0f32))) as f32_0;
        HSD_JObjSetTranslateX((*user_data).gobj.user_data as *mut HSD_JObj, pos);
    } else {
        HSD_JObjSetFlagsAll(scrollbar, JOBJ_HIDDEN as u32_0);
    }
    let mut sel: s32 = mn_804A04F0.hovered_selection as s32 - 0x18 as s32;
    if !(*user_data).text2.is_null() {
        HSD_SisLib_803A5CC4((*user_data).text2);
    }
    txt = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        0 as s32,
        -9.5f32,
        9.1f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    (*user_data).text2 = txt;
    (*txt).font_size.x = 0.0521f32 as f32_0;
    (*txt).font_size.y = 0.0521f32 as f32_0;
    HSD_SisLib_803A6368(txt, mnName_804D4BE8[sel as usize] as s32);
}
pub unsafe extern "C" fn mnName_8023A59C(mut arg0: u8_0) -> *mut HSD_GObj {
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut count: s32 = 0;
    let mut jobj5: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut root_jobj: [*mut HSD_JObj; 1] = [::core::ptr::null_mut::<HSD_JObj>(); 1];
    let mut jobj7: [*mut HSD_JObj; 1] = [::core::ptr::null_mut::<HSD_JObj>(); 1];
    let mut archive: *mut MnNameArchive = &raw mut mnName_804A06E0;
    let mut slider: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut scrollbar_container: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut jobj4: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut user_data: *mut MnName_GObj = ::core::ptr::null_mut::<MnName_GObj>();
    let mut i: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    gobj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    mnName_804D6BF8 = gobj;
    root_jobj[0 as ::core::ffi::c_int as usize] = HSD_JObjLoadJoint(
        (*archive).joint as *mut HSD_Joint,
    );
    HSD_GObjObject_80390A70(
        gobj,
        HSD_GObj_JObjKind,
        root_jobj[0 as ::core::ffi::c_int as usize] as *mut ::core::ffi::c_void,
    );
    GObj_SetupGXLink(
        gobj,
        Some(
            HSD_GObj_JObjCallback
                as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
        ),
        4 as u8_0,
        0x80 as u32_0,
    );
    HSD_GObj_SetupProc(
        gobj,
        Some(fn_80239574 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    HSD_JObjAddAnimAll(
        root_jobj[0 as ::core::ffi::c_int as usize],
        (*archive).anim_joint as *mut HSD_AnimJoint,
        (*archive).matanim_joint as *mut HSD_MatAnimJoint,
        (*archive).shapeanim_joint as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(root_jobj[0 as ::core::ffi::c_int as usize], 0.0f32);
    HSD_JObjAnimAll(root_jobj[0 as ::core::ffi::c_int as usize]);
    user_data = HSD_MemAlloc(0x44 as ssize_t) as *mut MnName_GObj;
    if !user_data.is_null() {} else {
        OSReport(b"Can't get user_data.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char, &[]);
        __assert(
            b"src/melee/mn/mnname.c\0" as *const u8 as *const ::core::ffi::c_char,
            1604 as u32_0,
            b"user_data\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    *(&raw mut (*user_data).gobj.classifier as *mut u8_0) = mn_804A04F0.cur_menu;
    *(&raw mut (*user_data).gobj.classifier as *mut u8_0)
        .offset(1 as ::core::ffi::c_int as isize) = mn_804A04F0.hovered_selection
        as u8_0;
    (*user_data).gobj.gx_link = 0 as u8_0;
    (*user_data).gobj.p_priority = 0 as u8_0;
    (*user_data).gobj.p_link = arg0;
    (*user_data).text = ::core::ptr::null_mut::<HSD_Text>();
    (*user_data).text2 = ::core::ptr::null_mut::<HSD_Text>();
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0xd as ::core::ffi::c_int {
        lb_80011E24(root_jobj[0 as ::core::ffi::c_int as usize], (user_data as *mut u8_0)
                .offset((i << 2 as ::core::ffi::c_int) as isize)
                .offset(8 as ::core::ffi::c_int as isize) as *mut *mut HSD_JObj, &[(i) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        i += 1;
    }
    if mn_804A04F0.x10 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut p: *mut mn_80231634_t = ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
            *mut mn_80231634_t,
        >((*user_data).gobj.user_data_remove_func);
        let mut j: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        if p.is_null() {
            j = ::core::ptr::null_mut::<HSD_JObj>();
        } else {
            j = (*p).x10 as *mut HSD_JObj;
        }
        HSD_JObjRemoveAll(j);
        if !(*user_data).text.is_null() {
            HSD_SisLib_803A5CC4((*user_data).text);
            (*user_data).text = ::core::ptr::null_mut::<HSD_Text>();
        }
        HSD_JObjSetFlagsAll(
            (*user_data).gobj.hsd_obj as *mut HSD_JObj,
            JOBJ_HIDDEN as u32_0,
        );
    } else {
        HSD_JObjRemoveAll(
            mn_80231634(
                ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
                    *mut mn_80231634_t,
                >((*user_data).gobj.user_data_remove_func),
            ) as *mut HSD_JObj,
        );
        if !(*user_data).text.is_null() {
            HSD_SisLib_803A5CC4((*user_data).text);
            (*user_data).text = ::core::ptr::null_mut::<HSD_Text>();
        }
        mnName_80239A24(user_data as *mut HSD_GObj);
        mnName_80238754_noinline(user_data as *mut HSD_GObj);
    }
    jobj7[0 as ::core::ffi::c_int as usize] = *(user_data as *mut *mut HSD_JObj)
        .offset(9 as ::core::ffi::c_int as isize);
    HSD_JObjReqAnimAll(
        jobj7[0 as ::core::ffi::c_int as usize],
        mnName_804D4BD0[(mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == 0x18 as ::core::ffi::c_int) as ::core::ffi::c_int as usize],
    );
    HSD_JObjAnimAll(jobj7[0 as ::core::ffi::c_int as usize]);
    jobj4 = (*user_data).gobj.proc as *mut HSD_JObj;
    HSD_JObjReqAnimAll(
        jobj4,
        mnName_804D4BD8[(mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == 0x19 as ::core::ffi::c_int) as ::core::ffi::c_int as usize],
    );
    HSD_JObjAnimAll(jobj4);
    jobj5 = ::core::mem::transmute::<
        GObj_RenderFunc,
        *mut HSD_JObj,
    >((*user_data).gobj.render_cb);
    HSD_JObjReqAnimAll(
        jobj5,
        mnName_804D4BD8[(mn_804A04F0.hovered_selection as ::core::ffi::c_int
            == 0x1a as ::core::ffi::c_int) as ::core::ffi::c_int as usize],
    );
    HSD_JObjAnimAll(jobj5);
    scrollbar_container = (*user_data).gobj.hsd_obj as *mut HSD_JObj;
    count = GetNameCount_noinline() as s32;
    mnName_SetupScrollbarAndText(count, scrollbar_container, user_data);
    return gobj;
}
#[inline]
pub unsafe extern "C" fn mnName_8023A9B4_GetUserData(
    mut gobj2: *mut MnName_GObj,
) -> *mut mn_80231634_t {
    return ::core::mem::transmute::<
        Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
        *mut mn_80231634_t,
    >((*gobj2).gobj.user_data_remove_func);
}
#[inline]
pub unsafe extern "C" fn mnName_8023A9B4_ResetDisplayOrder() {
    let mut i: u32_0 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    i = 0 as u32_0;
    while i < 0x78 as u32_0 {
        mnName_NameDisplayOrder[i as usize] = i as u8_0;
        i = i.wrapping_add(1);
    }
    HSD_GObj_80390CD4(mnName_8023A59C(3 as u8_0));
}
#[inline]
pub unsafe extern "C" fn mnName_8023A9B4_GetGObj() -> *mut MnName_GObj {
    return (*mnName_804D6BF8).user_data as *mut MnName_GObj;
}
pub unsafe extern "C" fn mnName_8023A9B4(mut arg0: u8_0) {
    let mut i: u32_0 = 0;
    let mut gobj2: *mut MnName_GObj = ::core::ptr::null_mut::<MnName_GObj>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    mn_804A04F0.hovered_selection = 0x18 as u16_0;
    i = 0 as u32_0;
    mn_804A04F0.x10 = i as u8_0;
    mnName_8023A9B4_ResetDisplayOrder();
    gobj2 = mnName_8023A9B4_GetGObj();
    if mn_804A04F0.x10 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut p: *mut mn_80231634_t = mnName_8023A9B4_GetUserData(gobj2);
        if p.is_null() {
            jobj = ::core::ptr::null_mut::<HSD_JObj>();
        } else {
            jobj = (*p).x10 as *mut HSD_JObj;
        }
        HSD_JObjRemoveAll(jobj);
        if !(*gobj2).text.is_null() {
            HSD_SisLib_803A5CC4((*gobj2).text);
            (*gobj2).text = ::core::ptr::null_mut::<HSD_Text>();
        }
        HSD_JObjSetFlagsAll(
            (*gobj2).gobj.hsd_obj as *mut HSD_JObj,
            JOBJ_HIDDEN as u32_0,
        );
    } else {
        HSD_JObjRemoveAll(
            mn_80231634(
                ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
                    *mut mn_80231634_t,
                >((*gobj2).gobj.user_data_remove_func),
            ) as *mut HSD_JObj,
        );
        if !(*gobj2).text.is_null() {
            HSD_SisLib_803A5CC4((*gobj2).text);
            (*gobj2).text = ::core::ptr::null_mut::<HSD_Text>();
        }
        mnName_80239A24(gobj2 as *mut HSD_GObj);
        mnName_80238754_noinline(gobj2 as *mut HSD_GObj);
    }
    mnName_80238A04(
        gobj2 as *mut HSD_GObj,
        0x18 as u8_0,
        0 as ::core::ffi::c_uint as u8_0,
    );
    (*gobj2).gobj.gx_link = arg0;
    mnName_80238754_noinline(gobj2 as *mut HSD_GObj);
    HSD_JObjRemoveAll(
        mn_80231634(
            ::core::mem::transmute::<
                Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
                *mut mn_80231634_t,
            >((*gobj2).gobj.user_data_remove_func),
        ) as *mut HSD_JObj,
    );
    if !(*gobj2).text.is_null() {
        HSD_SisLib_803A5CC4((*gobj2).text);
        (*gobj2).text = ::core::ptr::null_mut::<HSD_Text>();
    }
    mnName_80239A24(gobj2 as *mut HSD_GObj);
}
pub static mut NotAllowedNamesList: *mut *mut ::core::ffi::c_char = ::core::ptr::null::<
    *mut ::core::ffi::c_char,
>() as *mut *mut ::core::ffi::c_char;
pub static mut AutoNamesList: *mut *mut u8_0 = ::core::ptr::null::<*mut u8_0>()
    as *mut *mut u8_0;
#[inline]
pub unsafe extern "C" fn mnName_InitNameDisplayOrder() {
    let mut i: u32_0 = 0;
    mn_804A04F0.hovered_selection = 0x18 as u16_0;
    mnName_804D6BFC = ::core::ptr::null_mut::<HSD_Text>();
    i = 0 as u32_0;
    while i < 0x78 as u32_0 {
        mnName_NameDisplayOrder[i as usize] = i as u8_0;
        i = i.wrapping_add(1);
    }
    mn_804A04F0.x10 = 0 as u8_0;
    HSD_GObj_80390CD4(mnName_8023A59C(1 as u8_0));
}
pub unsafe extern "C" fn mnName_8023AC40() -> s32 {
    let mut archive: *mut HSD_Archive = mn_804D6BB8;
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    lbArchive_LoadSections(archive, &raw mut mnName_804A06E0.joint, &[(b"MenMainConNmTp_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06E0.anim_joint) as usize, (b"MenMainConNmTp_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06E0.matanim_joint) as usize, (b"MenMainConNmTp_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06E0.shapeanim_joint) as usize, (b"MenMainConNmTp_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06C0.joint) as usize, (b"MenMainBaseNmTp_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06C0.anim_joint) as usize, (b"MenMainBaseNmTp_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06C0.matanim_joint) as usize, (b"MenMainBaseNmTp_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06C0.shapeanim_joint) as usize, (b"MenMainBaseNmTp_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06D0.joint) as usize, (b"MenMainWarCmn_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06D0.anim_joint) as usize, (b"MenMainWarCmn_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06D0.matanim_joint) as usize, (b"MenMainWarCmn_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnName_804A06D0.shapeanim_joint) as usize, (b"MenMainWarCmn_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A06F0.joint) as usize, (b"MenMainConEtNw_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A06F0.animjoint) as usize, (b"MenMainConEtNw_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A06F0.matanim_joint) as usize, (b"MenMainConEtNw_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A06F0.shapeanim_joint) as usize, (b"MenMainConEtNw_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0700.joint) as usize, (b"MenMainBaseEtNw_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0700.animjoint) as usize, (b"MenMainBaseEtNw_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0700.matanim_joint) as usize, (b"MenMainBaseEtNw_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0700.shapeanim_joint) as usize, (b"MenMainBaseEtNw_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0710.joint) as usize, (b"MenMainSubEtNw_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0710.animjoint) as usize, (b"MenMainSubEtNw_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0710.matanim_joint) as usize, (b"MenMainSubEtNw_Top_matanim_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut mnNameNew_804A0710.shapeanim_joint) as usize, (b"MenMainSubEtNw_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as ::core::ffi::c_int as isize))
            .joint) as usize, (b"MenMainSbaseEtNw_Top_joint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as ::core::ffi::c_int as isize))
            .animjoint) as usize, (b"MenMainSbaseEtNw_Top_animjoint\0" as *const u8 as *const ::core::ffi::c_char) as usize, (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as ::core::ffi::c_int as isize))
            .matanim_joint) as usize, (b"MenMainSbaseEtNw_Top_matanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (&raw mut (*(&raw mut mnNameNew_804A0720 as *mut StaticModelDesc)
            .offset(0 as ::core::ffi::c_int as isize))
            .shapeanim_joint) as usize, (b"MenMainSbaseEtNw_Top_shapeanim_joint\0" as *const u8
            as *const ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    if lbLang_IsSavedLanguageUS() != 0 {
        lbArchive_LoadSections(archive, &raw mut AutoNamesList as *mut *mut ::core::ffi::c_void, &[(&raw mut mnName_AutoNameUsName as *mut ::core::ffi::c_char) as usize, (&raw mut NotAllowedNamesList as *mut *mut ::core::ffi::c_void) as usize, (&raw mut mnName_RefuseNameUsName as *mut ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    } else {
        lbArchive_LoadSections(archive, &raw mut AutoNamesList as *mut *mut ::core::ffi::c_void, &[(&raw mut mnName_AutoNameName as *mut ::core::ffi::c_char) as usize, (&raw mut NotAllowedNamesList as *mut *mut ::core::ffi::c_void) as usize, (&raw mut mnName_RefuseNameName as *mut ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    }
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x12 as MenuKind8;
    mnName_InitNameDisplayOrder();
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        Some(fn_80238540 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u16_0 as u8_0 as u8_0);
    return proc_0 as s32;
}
pub static mut mnName_AutoNameUsName: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"mnNameAutoNameUs\0")
};
pub static mut mnName_RefuseNameUsName: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<
        [u8; 19],
        [::core::ffi::c_char; 19],
    >(*b"mnNameRefuseNameUs\0")
};
pub static mut mnName_AutoNameName: [::core::ffi::c_char; 15] = unsafe {
    ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"mnNameAutoName\0")
};
pub static mut mnName_RefuseNameName: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"mnNameRefuseName\0")
};
pub unsafe extern "C" fn IsNameNotAllowed(mut name: *mut ::core::ffi::c_char) -> bool_0 {
    let mut list: *mut *mut ::core::ffi::c_char = NotAllowedNamesList;
    while !(mnNameNew_NullCharacter as ::core::ffi::c_int
        == **list as ::core::ffi::c_int)
    {
        if CompareNameStrings(*list, name) == 0 {
            return true_0;
        }
        list = list.offset(1);
    }
    return false_0;
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

/// Callees in unported TUs: settable hooks named like the C function; unset they panic with a TODO.
pub mod ext {
    #![allow(non_snake_case, non_upper_case_globals, dead_code, unused_imports)]
    use melee_types::*;
    use super::*;
    macro_rules! hook {
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($h:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $h: Option<unsafe extern "C" fn($($t),*) $(-> $r)?> = None;
            pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                match $h {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(H_OSReport, OSReport, (a0: *mut ::core::ffi::c_char, va: &[usize]));
    hook!(H___assert, __assert, (a0: *const ::core::ffi::c_char, a1: u32_0, a2: *const ::core::ffi::c_char) -> !);
    hook!(H_GObj_Create, GObj_Create, (a0: u16_0, a1: u8_0, a2: u8_0) -> *mut HSD_GObj);
    pub static mut H_HSD_GObj_JObjCallback: Option<unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: ::core::ffi::c_int)  {
                match H_HSD_GObj_JObjCallback {
                    Some(f) => f(a0, a1),
                    None => { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } return; },
                }
            }
    hook!(H_HSD_GObj_80390CD4, HSD_GObj_80390CD4, (a0: *mut HSD_GObj));
    hook!(H_HSD_JObjReqAnimAll, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjSetMtxDirtySub, HSD_JObjSetMtxDirtySub, (a0: *mut HSD_JObj));
    hook!(H_HSD_JObjRemoveAll, HSD_JObjRemoveAll, (a0: *mut HSD_JObj));
    pub static mut H_HSD_JObjLoadJoint: Option<unsafe extern "C" fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            pub unsafe extern "C" fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj {
                match H_HSD_JObjLoadJoint {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ }; },
                }
            }
    hook!(H_HSD_JObjAddAnimAll, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(H_HSD_JObjAnimAll, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    hook!(H_HSD_JObjSetFlagsAll, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjClearFlagsAll, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjAddChild, HSD_JObjAddChild, (a0: *mut HSD_JObj, a1: *mut HSD_JObj));
    hook!(H_HSD_JObjReqAnim, HSD_JObjReqAnim, (a0: *mut HSD_JObj, a1: f32_0));
    hook!(H_HSD_JObjAnim, HSD_JObjAnim, (a0: *mut HSD_JObj));
    pub static mut H_lbAudioAx_80024030: Option<unsafe extern "C" fn(::core::ffi::c_int) > = None;
            pub unsafe extern "C" fn lbAudioAx_80024030(a0: ::core::ffi::c_int)  {
                match H_lbAudioAx_80024030 {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(a0 as _); } return; },
                }
            }
    hook!(H_lb_80011E24, lb_80011E24, (a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, va: &[usize]) -> ::core::ffi::c_int);
    pub static mut H_mn_8022ED6C: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut AnimLoopSettings) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn mn_8022ED6C(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> ::core::ffi::c_float {
                match H_mn_8022ED6C {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022ED6C(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022EFD8: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut AnimLoopSettings) -> ::core::ffi::c_float> = None;
            pub unsafe extern "C" fn mn_8022EFD8(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> ::core::ffi::c_float {
                match H_mn_8022EFD8 {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022EFD8(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_mn_8022F298: Option<unsafe extern "C" fn(*mut HSD_JObj) -> f32_0> = None;
            pub unsafe extern "C" fn mn_8022F298(a0: *mut HSD_JObj) -> f32_0 {
                match H_mn_8022F298 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022F298(a0 as _) as _ }; },
                }
            }
    pub static mut H_HSD_SisLib_803A74F0: Option<unsafe extern "C" fn(*mut HSD_Text, s32, *mut GXColor) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A74F0(a0: *mut HSD_Text, a1: s32, a2: *mut GXColor)  {
                match H_HSD_SisLib_803A74F0 {
                    Some(f) => f(a0, a1, a2),
                    None => { unsafe { let _ = melee_runtime::tu::hsd_3A64::HSD_SisLib_803A74F0(a0 as _, a1 as _, a2 as _); } return; },
                }
            }
    hook!(H_HSD_SisLib_803A6B98, HSD_SisLib_803A6B98, (a0: *mut HSD_Text, a1: ::core::ffi::c_float, a2: ::core::ffi::c_float, a3: *const ::core::ffi::c_char, va: &[usize]) -> ::core::ffi::c_int);
    pub static mut H_HSD_SisLib_803A6754: Option<unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int) -> *mut HSD_Text> = None;
            pub unsafe extern "C" fn HSD_SisLib_803A6754(a0: ::core::ffi::c_int, a1: ::core::ffi::c_int) -> *mut HSD_Text {
                match H_HSD_SisLib_803A6754 {
                    Some(f) => f(a0, a1),
                    None => { return unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0 as _, a1 as _) as _ }; },
                }
            }
    pub static mut H_HSD_SisLib_803A6368: Option<unsafe extern "C" fn(*mut HSD_Text, s32) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: s32)  {
                match H_HSD_SisLib_803A6368 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A5ACC: Option<unsafe extern "C" fn(::core::ffi::c_int, s32, f32_0, f32_0, f32_0, f32_0, f32_0) -> *mut HSD_Text> = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5ACC(a0: ::core::ffi::c_int, a1: s32, a2: f32_0, a3: f32_0, a4: f32_0, a5: f32_0, a6: f32_0) -> *mut HSD_Text {
                match H_HSD_SisLib_803A5ACC {
                    Some(f) => f(a0, a1, a2, a3, a4, a5, a6),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ }; },
                }
            }
    pub static mut H_HSD_SisLib_803A5CC4: Option<unsafe extern "C" fn(*mut HSD_Text) > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text)  {
                match H_HSD_SisLib_803A5CC4 {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0 as _); } return; },
                }
            }
    pub static mut H_HSD_SisLib_803A5D30: Option<unsafe extern "C" fn() > = None;
            pub unsafe extern "C" fn HSD_SisLib_803A5D30()  {
                match H_HSD_SisLib_803A5D30 {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A5D30(); } return; },
                }
            }
    pub static mut H_mnNameNew_MainInput: Option<unsafe extern "C" fn(*mut HSD_GObj) > = None;
            pub unsafe extern "C" fn mnNameNew_MainInput(a0: *mut HSD_GObj)  {
                match H_mnNameNew_MainInput {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = crate::tu::mnnamenew::mnNameNew_MainInput(a0 as _); } return; },
                }
            }
    pub static mut H_mnNameNew_EnterFromMnName: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) > = None;
            pub unsafe extern "C" fn mnNameNew_EnterFromMnName(a0: *mut ::core::ffi::c_void)  {
                match H_mnNameNew_EnterFromMnName {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = crate::tu::mnnamenew::mnNameNew_EnterFromMnName(a0 as _); } return; },
                }
            }
    pub static mut H_GetPersistentNameData: Option<unsafe extern "C" fn(s32) -> *mut NameTagData> = None;
            pub unsafe extern "C" fn GetPersistentNameData(a0: s32) -> *mut NameTagData {
                match H_GetPersistentNameData {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gmmain_lib::GetPersistentNameData(a0 as _) as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015DBF4: Option<unsafe extern "C" fn(s32) > = None;
            pub unsafe extern "C" fn gmMainLib_8015DBF4(a0: s32)  {
                match H_gmMainLib_8015DBF4 {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = crate::tu::gmmain_lib::gmMainLib_8015DBF4(a0 as _); } return; },
                }
            }
    pub static mut H_InitializePersistentNameData: Option<unsafe extern "C" fn(s32) > = None;
            pub unsafe extern "C" fn InitializePersistentNameData(a0: s32)  {
                match H_InitializePersistentNameData {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = crate::tu::gmmain_lib::InitializePersistentNameData(a0 as _); } return; },
                }
            }
    pub static mut H_lb_8000B1CC: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut Vec3, *mut Vec3) > = None;
            pub unsafe extern "C" fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec3, a2: *mut Vec3)  {
                match H_lb_8000B1CC {
                    Some(f) => f(a0, a1, a2),
                    None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } return; },
                }
            }
    hook!(H_lbArchive_LoadSections, lbArchive_LoadSections, (a0: *mut HSD_Archive, a1: *mut *mut ::core::ffi::c_void, va: &[usize]));
    pub static mut H_lbLang_IsSavedLanguageUS: Option<unsafe extern "C" fn() -> bool_0> = None;
            pub unsafe extern "C" fn lbLang_IsSavedLanguageUS() -> bool_0 {
                match H_lbLang_IsSavedLanguageUS {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ }; },
                }
            }
    hook!(H_GObj_SetupGXLink, GObj_SetupGXLink, (a0: *mut HSD_GObj, a1: GObj_RenderFunc, a2: u8_0, a3: u32_0));
    hook!(H_HSD_GObjObject_80390A70, HSD_GObjObject_80390A70, (a0: *mut HSD_GObj, a1: u8_0, a2: *mut ::core::ffi::c_void));
    hook!(H_HSD_GObjFree, HSD_GObjFree, (a0: *mut HSD_GObj));
    hook!(H_HSD_GObj_SetupProc, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8_0) -> *mut HSD_GObjProc);
    hook!(H_GObj_InitUserData, GObj_InitUserData, (a0: *mut HSD_GObj, a1: u8_0, a2: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>, a3: *mut ::core::ffi::c_void));
    pub static mut H_HSD_Free: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) > = None;
            pub unsafe extern "C" fn HSD_Free(a0: *mut ::core::ffi::c_void)  {
                match H_HSD_Free {
                    Some(f) => f(a0),
                    None => { unsafe { let _ = melee_runtime::tu::memory::HSD_Free(a0 as _); } return; },
                }
            }
    pub static mut H_HSD_MemAlloc: Option<unsafe extern "C" fn(ssize_t) -> *mut ::core::ffi::c_void> = None;
            pub unsafe extern "C" fn HSD_MemAlloc(a0: ssize_t) -> *mut ::core::ffi::c_void {
                match H_HSD_MemAlloc {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ }; },
                }
            }
    pub static mut HSD_GObj_804D783C: s32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8_0 = unsafe { core::mem::zeroed() };
    // the real tables (mnnamenew.c): a zero-length stand-in here would be read past its end
    pub use crate::tu::mnnamenew::{mnNameNew_803EE720, mnNameNew_803EE724};
    pub static mut mnNameNew_NullCharacter: ::core::ffi::c_char = unsafe { core::mem::zeroed() };
}
