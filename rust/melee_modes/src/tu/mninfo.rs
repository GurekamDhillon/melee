//! src/melee/mn/mninfo.c: native port (c2rust-derived body; callees of unported TUs are `ext` TODO hooks).
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, unused_unsafe, unused_mut, unused_assignments, unused_variables, unused_parens, dead_code, static_mut_refs, clippy::all)]

use melee_types::*;
#[allow(unused_imports)] use self::ext::*;
#[allow(unused_imports)] use super::mnmain::{mn_80229624, mn_80229894, mn_8022EA78, mn_804A04F0, mn_804D4B64, mn_804D6BB8, mn_804D6BC8};

pub type u8_0 = u8;
pub type u16_0 = u16;
pub type u32_0 = u32;
pub type u64_0 = u64;
pub type f32_0 = f32;
pub type f64_0 = f64;
pub const MenuInput_Down: _MenuInput = 2;
pub const MenuInput_Up: _MenuInput = 1;
pub const MenuInput_Back: _MenuInput = 32;
pub type bool_0 = ::core::ffi::c_int;
pub type _MenuInput = ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MnInfoDataLayout {
    pub anim: AnimLoopSettings,
    pub sis_ids: [u32_0; 4],
    pub date_format: [::core::ffi::c_char; 12],
    pub time_format: [::core::ffi::c_char; 12],
    pub assert_report: [::core::ffi::c_char; 24],
    pub assert_file: [::core::ffi::c_char; 12],
    pub assert_expr: [::core::ffi::c_char; 12],
    pub top_joint: [::core::ffi::c_char; 24],
    pub top_animjoint: [::core::ffi::c_char; 28],
    pub top_matanim_joint: [::core::ffi::c_char; 32],
    pub top_shapeanim_joint: [::core::ffi::c_char; 40],
}
melee_core::gc_layout!(MnInfoDataLayout, "MnInfoDataLayout", size 0xE0, align 4, [
    anim @ 0x0: 12 "AnimLoopSettings",
    sis_ids @ 0xC: 16 "[u32_0; 4]",
    date_format @ 0x1C: 12 "[c_char; 12]",
    time_format @ 0x28: 12 "[c_char; 12]",
    assert_report @ 0x34: 24 "[c_char; 24]",
    assert_file @ 0x4C: 12 "[c_char; 12]",
    assert_expr @ 0x58: 12 "[c_char; 12]",
    top_joint @ 0x64: 24 "[c_char; 24]",
    top_animjoint @ 0x7C: 28 "[c_char; 28]",
    top_matanim_joint @ 0x98: 32 "[c_char; 32]",
    top_shapeanim_joint @ 0xB8: 40 "[c_char; 40]",
]);

#[inline]
pub unsafe extern "C" fn HSD_GObjGetUserData(
    mut gobj: *mut HSD_GObj,
) -> *mut ::core::ffi::c_void {
    return (*gobj).user_data;
}
pub const JOBJ_HIDDEN: ::core::ffi::c_int = (1 as ::core::ffi::c_int)
    << 4 as ::core::ffi::c_int;
#[inline]
pub unsafe extern "C" fn sfxBack() {
    lbAudioAx_80024030(0 as ::core::ffi::c_int);
}
#[inline]
pub unsafe extern "C" fn sfxMove() {
    lbAudioAx_80024030(2 as ::core::ffi::c_int);
}
pub static mut mnInfo_804A0958: StaticModelDesc = StaticModelDesc {
    joint: ::core::ptr::null::<HSD_Joint>() as *mut HSD_Joint,
    animjoint: ::core::ptr::null::<HSD_AnimJoint>() as *mut HSD_AnimJoint,
    matanim_joint: ::core::ptr::null::<HSD_MatAnimJoint>() as *mut HSD_MatAnimJoint,
    shapeanim_joint: ::core::ptr::null::<HSD_ShapeAnimJoint>() as *mut HSD_ShapeAnimJoint,
};
pub static mut mnInfo_804A0968: [u8_0; 72] = [0; 72];
pub static mut gap_10_804D6C7C_sbss: u32_0 = 0;
pub static mut mnInfo_804D6C78: *mut HSD_GObj = ::core::ptr::null::<HSD_GObj>()
    as *mut HSD_GObj;
pub unsafe extern "C" fn mnInfo_80251A08(mut arg0: s32) -> s32 {
    match arg0 {
        62 => return 0 as s32,
        52 => {
            if lbLang_IsSettingUS() != 0 as ::core::ffi::c_int {
                return 0 as s32;
            }
            return gmMainLib_8015D94C(arg0 as u32_0) as s32;
        }
        53 => {
            if lbLang_IsSettingJP() != 0 as ::core::ffi::c_int {
                return 0 as s32;
            }
            return gmMainLib_8015D94C(arg0 as u32_0) as s32;
        }
        _ => return gmMainLib_8015D94C(arg0 as u32_0) as s32,
    };
}
pub unsafe extern "C" fn mnInfo_80251AA4() -> s32 {
    let mut i: s32 = 0;
    let mut var_r30: s32 = 0 as s32;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x42 as ::core::ffi::c_int {
        if mnInfo_80251A08(i) != 0 as ::core::ffi::c_int {
            var_r30 += 1;
        }
        i += 1;
    }
    return var_r30;
}
#[inline]
pub unsafe extern "C" fn mnInfo_80251AFC_inline(mut i: s32) -> bool_0 {
    let mut unlock_state: s32 = mnInfo_80251A08(mnInfo_804A0968[i as usize] as s32);
    return (unlock_state == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[inline]
pub unsafe extern "C" fn mnInfo_80251AFC_inline_2(mut j: s32) -> u32_0 {
    return *gmMainLib_8015D804(mnInfo_804A0968[j as usize] as s32);
}
#[inline]
pub unsafe extern "C" fn mnInfo_80251AFC_inline_3(mut ids: *mut u8_0, mut i: s32) -> s32 {
    return *ids.offset(i as isize) as s32;
}
pub unsafe extern "C" fn mnInfo_80251AFC() {
    let mut i: s32 = 0;
    let mut j: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    &raw mut mnInfo_804A0958;
    i = 0 as ::core::ffi::c_int as s32;
    while 0x42 as ::core::ffi::c_int > i {
        mnInfo_804A0968[i as usize] = i as u8_0;
        i += 1;
    }
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x42 as ::core::ffi::c_int {
        j = (i as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as s32;
        while j < 0x42 as ::core::ffi::c_int {
            if mnInfo_80251AFC_inline(i) != 0 {
                let mut tmp: u8_0 = mnInfo_804A0968[i as usize];
                mnInfo_804A0968[i as usize] = mnInfo_804A0968[j as usize];
                mnInfo_804A0968[j as usize] = tmp;
            }
            j += 1;
        }
        i += 1;
    }
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x42 as ::core::ffi::c_int {
        j = (i as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as s32;
        while j < 0x42 as ::core::ffi::c_int {
            if mnInfo_80251A08(mnInfo_804A0968[j as usize] as s32)
                != 0 as ::core::ffi::c_int
                && (mnInfo_80251A08(mnInfo_804A0968[i as usize] as s32)
                    == 0 as ::core::ffi::c_int
                    || *gmMainLib_8015D804(
                        mnInfo_80251AFC_inline_3(
                            &raw mut mnInfo_804A0968 as *mut u8_0,
                            i,
                        ),
                    ) > mnInfo_80251AFC_inline_2(j))
            {
                let mut tmp_0: u8_0 = mnInfo_804A0968[i as usize];
                mnInfo_804A0968[i as usize] = mnInfo_804A0968[j as usize];
                mnInfo_804A0968[j as usize] = tmp_0;
            }
            j += 1;
        }
        i += 1;
    }
}
pub static mut mnInfo_803EFC08: [AnimLoopSettings; 18] = [
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 199.0f32,
        loop_frame: 0.0f32,
    },
    AnimLoopSettings {
        start_frame: 1.8e-42f32,
        end_frame: 1.802e-42f32,
        loop_frame: 1.803e-42f32,
    },
    AnimLoopSettings {
        start_frame: 1.805e-42f32,
        end_frame: 2.1092525e-16f32,
        loop_frame: 1.379729e31f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 2.109659e-16f32,
        loop_frame: 1.4748028e31f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 225.43028f32,
        loop_frame: 5.083402e31f32,
    },
    AnimLoopSettings {
        start_frame: 5.085142e31f32,
        end_frame: 7.153577e22f32,
        loop_frame: 2.817505e20f32,
    },
    AnimLoopSettings {
        start_frame: 6.162976e-33f32,
        end_frame: 4.6115556e27f32,
        loop_frame: 2.8237532e23f32,
    },
    AnimLoopSettings {
        start_frame: 0.0f32,
        end_frame: 3.0854143e32f32,
        loop_frame: 1.6456562e19f32,
    },
    AnimLoopSettings {
        start_frame: 1.4757395e20f32,
        end_frame: 2.405757e8f32,
        loop_frame: 2.6912729e20f32,
    },
    AnimLoopSettings {
        start_frame: 7.3738955e28f32,
        end_frame: 1.5307577e19f32,
        loop_frame: 1.6892836e19f32,
    },
    AnimLoopSettings {
        start_frame: 1.8878586e28f32,
        end_frame: 2.405757e8f32,
        loop_frame: 2.6912729e20f32,
    },
    AnimLoopSettings {
        start_frame: 7.3738955e28f32,
        end_frame: 1.5307577e19f32,
        loop_frame: 1.6244036e19f32,
    },
    AnimLoopSettings {
        start_frame: 4.5346362e27f32,
        end_frame: 1.8878586e28f32,
        loop_frame: 2.405757e8f32,
    },
    AnimLoopSettings {
        start_frame: 2.6912729e20f32,
        end_frame: 7.3738955e28f32,
        loop_frame: 1.5307577e19f32,
    },
    AnimLoopSettings {
        start_frame: 1.710508e19f32,
        end_frame: 2.7487011e20f32,
        loop_frame: 1.6892836e19f32,
    },
    AnimLoopSettings {
        start_frame: 1.8878586e28f32,
        end_frame: 2.405757e8f32,
        loop_frame: 2.6912729e20f32,
    },
    AnimLoopSettings {
        start_frame: 7.3738955e28f32,
        end_frame: 1.5307577e19f32,
        loop_frame: 1.7539375e19f32,
    },
    AnimLoopSettings {
        start_frame: 2.8395941e29f32,
        end_frame: 1.7935375e25f32,
        loop_frame: 7.2243537e28f32,
    },
];
pub static mut gap_07_803EFCE0_data: [::core::ffi::c_char; 8] = unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"\0\0\0\0\0\0\0\0")
};
pub unsafe extern "C" fn mnInfo_80251D58(
    mut arg0: *mut mnInfo_GObj,
    mut arg1: s32,
    mut arg2: u32_0,
    mut arg3: u32_0,
) -> s32 {
    let mut sp34: [::core::ffi::c_char; 5] = [0; 5];
    let mut sp30: [::core::ffi::c_char; 3] = [0; 3];
    let mut sp2C: [::core::ffi::c_char; 3] = [0; 3];
    let mut sp28: [::core::ffi::c_char; 3] = [0; 3];
    let mut sp24: [::core::ffi::c_char; 3] = [0; 3];
    let mut sp20: [::core::ffi::c_char; 3] = [0; 3];
    let mut sp18: datetime = datetime {
        year: 0,
        month: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
    };
    let mut slot: *mut *mut HSD_Text = ::core::ptr::null_mut::<*mut HSD_Text>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut data: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    let mut layout: *mut MnInfoDataLayout = ::core::ptr::null_mut::<MnInfoDataLayout>();
    data = (*arg0).user_data as *mut MnInfoData;
    layout = &raw mut mnInfo_803EFC08 as *mut AnimLoopSettings as *mut MnInfoDataLayout;
    slot = (data as *mut u8_0)
        .offset((arg1 as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as isize)
        as *mut *mut HSD_Text;
    slot = slot.offset(2 as ::core::ffi::c_int as isize);
    if !(*slot).is_null() {
        HSD_SisLib_803A5CC4((*data).left_column[arg1 as usize]);
    }
    text = HSD_SisLib_803A6754(0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    *slot = text;
    (*text).pos_x = -7.5f32 as f32_0;
    (*text).pos_y = melee_core::fp::fmadds(3.45f32, arg1 as ::core::ffi::c_float, -5.8f32) as f32_0;
    (*text).pos_z = 17.0f32 as f32_0;
    (*text).text_color = mn_804D4B64;
    (*text).default_alignment = 2 as u8_0;
    (*text).font_size.x = 0.03f32 as f32_0;
    (*text).font_size.y = 0.03f32 as f32_0;
    gm_801692E8(arg3, &raw mut sp18);
    mn_8022EA78(
        &raw mut sp30 as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int,
        sp18.hour as u32_0,
    );
    mn_8022EA78(
        &raw mut sp2C as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int,
        sp18.minute as u32_0,
    );
    mn_8022EA78(
        &raw mut sp28 as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int,
        sp18.second as u32_0,
    );
    mn_8022EA78(
        &raw mut sp34 as *mut ::core::ffi::c_char,
        4 as ::core::ffi::c_int,
        sp18.year as u32_0,
    );
    mn_8022EA78(
        &raw mut sp24 as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int,
        sp18.month as u32_0,
    );
    mn_8022EA78(
        &raw mut sp20 as *mut ::core::ffi::c_char,
        2 as ::core::ffi::c_int,
        sp18.day as u32_0,
    );
    if lbLang_IsSavedLanguageUS() != 0 as ::core::ffi::c_int {
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, &raw mut (*layout).date_format as *mut ::core::ffi::c_char, &[(&raw mut sp24 as *mut ::core::ffi::c_char) as usize, (&raw mut sp20 as *mut ::core::ffi::c_char) as usize, (&raw mut sp34 as *mut ::core::ffi::c_char) as usize]);
    } else {
        HSD_SisLib_803A6B98(text, 0.0f32, 0.0f32, &raw mut (*layout).date_format as *mut ::core::ffi::c_char, &[(&raw mut sp34 as *mut ::core::ffi::c_char) as usize, (&raw mut sp24 as *mut ::core::ffi::c_char) as usize, (&raw mut sp20 as *mut ::core::ffi::c_char) as usize]);
    }
    return HSD_SisLib_803A6B98(text, 0.0f32, 40.0f32, &raw mut (*layout).time_format as *mut ::core::ffi::c_char, &[(&raw mut sp30 as *mut ::core::ffi::c_char) as usize, (&raw mut sp2C as *mut ::core::ffi::c_char) as usize, (&raw mut sp28 as *mut ::core::ffi::c_char) as usize]) as s32;
}
pub unsafe extern "C" fn mnInfo_80251F04(
    mut arg0: *mut mnInfo_GObj,
    mut arg1: s32,
    mut arg2: u32_0,
) {
    let mut sp16: s16 = 0;
    let mut unused: s16 = 0;
    let mut slot: *mut *mut HSD_Text = ::core::ptr::null_mut::<*mut HSD_Text>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut data: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    data = (*arg0).user_data as *mut MnInfoData;
    slot = (data as *mut u8_0)
        .offset((arg1 as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as isize)
        as *mut *mut HSD_Text;
    slot = slot.offset(6 as ::core::ffi::c_int as isize);
    if !(*slot).is_null() {
        HSD_SisLib_803A5CC4((*data).right_column[arg1 as usize]);
    }
    text = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        0 as s32,
        -5.0f32,
        melee_core::fp::fmadds(3.45f32, arg1 as f32_0, -5.9f32),
        17.0f32,
        514.2857f32,
        142.85715f32,
    );
    *slot = text;
    (*text).font_size.x = 0.035f32 as f32_0;
    (*text).font_size.y = 0.035f32 as f32_0;
    (*text).default_fitting = 1 as u8_0;
    un_802FE3F8(
        arg2 as ::core::ffi::c_int,
        0x4bd as ::core::ffi::c_int,
        &raw mut sp16,
        ::core::ptr::null_mut::<s16>(),
    );
    HSD_SisLib_803A6368(text, sp16 as u16_0 as s32);
}
#[inline]
pub unsafe extern "C" fn mnInfo_CountUnlocked() -> s32 {
    let mut i: s32 = 0;
    let mut count: s32 = 0 as s32;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 0x42 as ::core::ffi::c_int {
        if mnInfo_80251A08(i) != 0 as ::core::ffi::c_int {
            count += 1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return count;
}
#[inline]
pub unsafe extern "C" fn mnInfo_CreateEntries(mut id: u32_0) {
    let mut trophy: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
    let mut i: s32 = 0;
    let mut gobj: *mut mnInfo_GObj = ::core::ptr::null_mut::<mnInfo_GObj>();
    gobj = mnInfo_804D6C78 as *mut mnInfo_GObj;
    trophy = (&raw mut mnInfo_804A0968 as *mut u8_0).offset(id as isize) as *mut u8_0;
    i = 0 as ::core::ffi::c_int as s32;
    while i < 4 as ::core::ffi::c_int {
        if mnInfo_80251A08(*trophy as s32) != 0 as ::core::ffi::c_int {
            id = *trophy as u32_0;
            mnInfo_80251D58(gobj, i, id, *gmMainLib_8015D804(id as s32));
            mnInfo_80251F04(gobj, i, id);
        }
        trophy = trophy.offset(1);
        i += 1;
    }
}
#[inline]
pub unsafe extern "C" fn mnInfo_FreeEntries() {
    let mut j: s32 = 0;
    let mut data2: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    let mut data3: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    data2 = (*mnInfo_804D6C78).user_data as *mut MnInfoData;
    data3 = data2;
    j = 0 as ::core::ffi::c_int as s32;
    while j < 4 as ::core::ffi::c_int {
        if !(*data2).left_column[j as usize].is_null() {
            HSD_SisLib_803A5CC4((*data3).left_column[j as usize]);
            (*data2).left_column[j as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        if !(*data2).right_column[j as usize].is_null() {
            HSD_SisLib_803A5CC4((*data3).right_column[j as usize]);
            (*data2).right_column[j as usize] = ::core::ptr::null_mut::<HSD_Text>();
        }
        j += 1;
    }
}
pub unsafe extern "C" fn fn_80251FE4() {
    let mut data: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    let mut buttons: u64_0 = 0;
    let mut count: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 24] = [0; 24];
    data = (*mnInfo_804D6C78).user_data as *mut MnInfoData;
    if mn_804D6BC8.cooldown as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        mn_804D6BC8.cooldown = (mn_804D6BC8.cooldown as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as u16_0;
        mn_804D6BC8.x2 = 0 as u16_0;
        mn_804D6BC8.x4 = 0 as ::core::ffi::c_int as s32;
        return;
    }
    mn_804A04F0.buttons = mn_80229624(4 as u32_0) as u64_0;
    buttons = mn_804A04F0.buttons;
    if buttons & MenuInput_Back as ::core::ffi::c_int as u64_0 != 0 {
        sfxBack();
        mn_804A04F0.entering_menu = 0 as u8_0;
        mn_80229894(5 as s32, 4 as u16_0, 3 as s32);
        return;
    }
    if buttons & MenuInput_Up as ::core::ffi::c_int as u64_0 != 0 {
        if (*data).scroll_idx as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            (*data).scroll_idx = ((*data).scroll_idx as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as u8_0;
            sfxMove();
            mnInfo_FreeEntries();
            let mut other: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
            let mut trophy: *mut u8_0 = ::core::ptr::null_mut::<u8_0>();
            let mut i: s32 = 0;
            let mut gobj: *mut mnInfo_GObj = ::core::ptr::null_mut::<mnInfo_GObj>();
            gobj = mnInfo_804D6C78 as *mut mnInfo_GObj;
            trophy = (&raw mut mnInfo_804A0968 as *mut u8_0)
                .offset((*data).scroll_idx as isize) as *mut u8_0;
            other = trophy;
            i = 0 as ::core::ffi::c_int as s32;
            while i < 4 as ::core::ffi::c_int {
                if mnInfo_80251A08(*trophy as s32) != 0 as ::core::ffi::c_int {
                    let mut id: u32_0 = *trophy as u32_0;
                    mnInfo_80251D58(gobj, i, id, *gmMainLib_8015D804(id as s32));
                    mnInfo_80251F04(gobj, i, id);
                }
                trophy = trophy.offset(1);
                i += 1;
            }
        }
    } else if buttons & MenuInput_Down as ::core::ffi::c_int as u64_0 != 0 {
        count = mnInfo_CountUnlocked();
        if ((*data).scroll_idx as ::core::ffi::c_int + 4 as ::core::ffi::c_int) < count {
            sfxMove();
            (*data).scroll_idx = ((*data).scroll_idx as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as u8_0;
            mnInfo_FreeEntries();
            mnInfo_CreateEntries((*data).scroll_idx as u32_0);
        }
    }
}
pub unsafe extern "C" fn mnInfo_802522B8(mut gobj: *mut HSD_GObj) {
    let mut count: s32 = 0;
    let mut data: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    data = (*gobj).user_data as *mut MnInfoData;
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut child, &[(2 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    if (*data).scroll_idx as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32_0);
    } else {
        HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32_0);
    }
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut child, &[(1 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
    count = mnInfo_CountUnlocked();
    if ((*data).scroll_idx as ::core::ffi::c_int + 4 as ::core::ffi::c_int) < count {
        HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32_0);
    } else {
        HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32_0);
    }
    mn_8022ED6C(
        jobj as *mut HSD_JObj,
        &raw mut mnInfo_803EFC08 as *mut AnimLoopSettings,
    );
}
pub unsafe extern "C" fn fn_802523B8(mut gobj: *mut HSD_GObj) {
    HSD_GObjFree(gobj);
}
#[inline]
pub unsafe extern "C" fn fn_802523D8_inline(
    mut data: *mut MnInfoData,
    mut gobj: *mut HSD_GObj,
) {
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut _pad: [::core::ffi::c_uchar; 16] = [0; 16];
    if mn_804A04F0.cur_menu as ::core::ffi::c_int
        != MENU_KIND_DATA_SPECIAL as ::core::ffi::c_int
    {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_802523B8 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        let mut data2: *mut MnInfoData = HSD_GObjGetUserData(gobj) as *mut MnInfoData;
        let mut data3: *mut MnInfoData = data2;
        let mut i: ::core::ffi::c_int = 0;
        let mut left_null: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        let mut right_null: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        i = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            if !(*data2).left_column[i as usize].is_null() {
                HSD_SisLib_803A5CC4((*data3).left_column[i as usize]);
                (*data2).left_column[i as usize] = left_null;
            }
            if !(*data2).right_column[i as usize].is_null() {
                HSD_SisLib_803A5CC4((*data3).right_column[i as usize]);
                (*data2).right_column[i as usize] = right_null;
            }
            i += 1;
        }
        HSD_SisLib_803A5CC4((*data).description);
    } else {
        let mut child: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
        jobj = (*gobj).hsd_obj as *mut HSD_JObj;
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut child, &[(2 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        if (*data).scroll_idx as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32_0);
        } else {
            HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32_0);
        }
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut child, &[(1 as ::core::ffi::c_int) as usize, (-(1 as ::core::ffi::c_int)) as usize]);
        if ((*data).scroll_idx as ::core::ffi::c_int + 4 as ::core::ffi::c_int)
            < mnInfo_80251AA4()
        {
            HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32_0);
        } else {
            HSD_JObjSetFlagsAll(child, JOBJ_HIDDEN as u32_0);
        }
        mn_8022ED6C(
            jobj as *mut HSD_JObj,
            &raw mut mnInfo_803EFC08 as *mut AnimLoopSettings,
        );
    };
}
pub unsafe extern "C" fn fn_802523D8(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnInfoData = HSD_GObjGetUserData(gobj) as *mut MnInfoData;
    let mut _pad: [::core::ffi::c_uchar; 4] = [0; 4];
    fn_802523D8_inline(data, gobj);
}
#[inline]
pub unsafe extern "C" fn fn_80252548_inline(
    mut data: *mut MnInfoData,
    mut gobj: *mut HSD_GObj,
) {
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut jobj: *mut HSD_JObj = ::core::ptr::null_mut::<HSD_JObj>();
    let mut i: s32 = 0;
    let mut _pad: [::core::ffi::c_uchar; 16] = [0; 16];
    if mn_804A04F0.cur_menu as ::core::ffi::c_int
        != MENU_KIND_DATA_SPECIAL as ::core::ffi::c_int
    {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_802523B8 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
        let mut data2: *mut MnInfoData = HSD_GObjGetUserData(gobj) as *mut MnInfoData;
        let mut data3: *mut MnInfoData = data2;
        let mut j: ::core::ffi::c_int = 0;
        let mut left_null: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        let mut right_null: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
        j = 0 as ::core::ffi::c_int;
        while j < 4 as ::core::ffi::c_int {
            if !(*data2).left_column[j as usize].is_null() {
                HSD_SisLib_803A5CC4((*data3).left_column[j as usize]);
                (*data2).left_column[j as usize] = left_null;
            }
            if !(*data2).right_column[j as usize].is_null() {
                HSD_SisLib_803A5CC4((*data3).right_column[j as usize]);
                (*data2).right_column[j as usize] = right_null;
            }
            j += 1;
        }
        HSD_SisLib_803A5CC4((*data).description);
    } else {
        if (*data).anim_timer as s32 != 0 as ::core::ffi::c_int {
            (*data).anim_timer = (*data).anim_timer.wrapping_sub(1);
            return;
        }
        i = 0 as ::core::ffi::c_int as s32;
        while i < 4 as ::core::ffi::c_int {
            if mnInfo_80251A08(mnInfo_804A0968[i as usize] as s32)
                != 0 as ::core::ffi::c_int
            {
                let mut id: u32_0 = mnInfo_804A0968[i as usize] as u32_0;
                mnInfo_80251D58(
                    gobj as *mut mnInfo_GObj,
                    i,
                    id,
                    *gmMainLib_8015D804(id as s32),
                );
                mnInfo_80251F04(gobj as *mut mnInfo_GObj, i, id);
            }
            i += 1;
        }
        let mut model: *mut StaticModelDesc = &raw mut mnInfo_804A0958;
        jobj = HSD_JObjLoadJoint((*model).joint as *mut HSD_Joint);
        HSD_GObjObject_80390A70(
            gobj,
            HSD_GObj_JObjKind,
            jobj as *mut ::core::ffi::c_void,
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
        HSD_JObjAddAnimAll(
            jobj,
            (*model).animjoint,
            (*model).matanim_joint,
            (*model).shapeanim_joint,
        );
        HSD_JObjReqAnimAll(jobj, 0.0f32);
        mnInfo_802522B8(gobj);
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            gobj,
            Some(fn_802523D8 as unsafe extern "C" fn(*mut HSD_GObj) -> ()),
            0 as u8_0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    };
}
pub unsafe extern "C" fn fn_80252548(mut gobj: *mut HSD_GObj) {
    let mut data: *mut MnInfoData = HSD_GObjGetUserData(gobj) as *mut MnInfoData;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    fn_80252548_inline(data, gobj);
}
pub unsafe extern "C" fn mnInfo_80252720(mut data: *mut MnInfoData) {
    (*data).scroll_idx = 0 as u8_0;
    (*data).anim_timer = 10 as u32_0;
    (*data).description = ::core::ptr::null_mut::<HSD_Text>();
    (*data).left_column[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).right_column[0 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).left_column[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).right_column[1 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).left_column[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).right_column[2 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).left_column[3 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
    (*data).right_column[3 as ::core::ffi::c_int as usize] = ::core::ptr::null_mut::<
        HSD_Text,
    >();
}
pub unsafe extern "C" fn mnInfo_80252758() -> s32 {
    let mut user_data: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    let mut proc_0: *mut HSD_GObjProc = ::core::ptr::null_mut::<HSD_GObjProc>();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut archive: *mut HSD_Archive = ::core::ptr::null_mut::<HSD_Archive>();
    let mut model: *mut StaticModelDesc = &raw mut mnInfo_804A0958;
    let mut layout: *mut MnInfoDataLayout = &raw mut mnInfo_803EFC08
        as *mut AnimLoopSettings as *mut MnInfoDataLayout;
    let mut top_joint: *mut ::core::ffi::c_char = &raw mut (*layout).top_joint
        as *mut ::core::ffi::c_char;
    let mut animjoint: *mut *mut HSD_AnimJoint = &raw mut (*model).animjoint;
    let mut _pad: [::core::ffi::c_uchar; 8] = [0; 8];
    mn_804D6BC8.cooldown = 5 as u16_0;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x1d as MenuKind8;
    mn_804A04F0.hovered_selection = 0 as u16_0;
    archive = mn_804D6BB8;
    lbArchive_LoadSections(archive, &raw mut (*model).joint as *mut *mut ::core::ffi::c_void, &[(top_joint) as usize, (animjoint) as usize, (&raw mut (*layout).top_animjoint as *mut ::core::ffi::c_char) as usize, (&raw mut (*model).matanim_joint) as usize, (&raw mut (*layout).top_matanim_joint as *mut ::core::ffi::c_char) as usize, (&raw mut (*model).shapeanim_joint) as usize, (&raw mut (*layout).top_shapeanim_joint as *mut ::core::ffi::c_char) as usize, (0 as ::core::ffi::c_int) as usize]);
    mnInfo_80251AFC();
    gobj = GObj_Create(6 as u16_0, 7 as u8_0, 0x80 as u8_0);
    mnInfo_804D6C78 = gobj;
    user_data = HSD_MemAlloc(::core::mem::size_of::<MnInfoData>() as ssize_t)
        as *mut MnInfoData;
    if user_data.is_null() {
        OSReport(&raw mut (*layout).assert_report as *mut ::core::ffi::c_char, &[]);
        __assert(
            &raw mut (*layout).assert_file as *mut ::core::ffi::c_char,
            0x267 as u32_0,
            &raw mut (*layout).assert_expr as *mut ::core::ffi::c_char,
        );
    }
    mnInfo_80252720(user_data);
    GObj_InitUserData(
        gobj,
        0 as u8_0,
        Some(HSD_Free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        user_data as *mut ::core::ffi::c_void,
    );
    proc_0 = HSD_GObj_SetupProc(
        gobj,
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut HSD_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(fn_80252548 as unsafe extern "C" fn(*mut HSD_GObj) -> ())),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8_0 as u8_0);
    let mut data: *mut MnInfoData = ::core::ptr::null_mut::<MnInfoData>();
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    data = (*gobj).user_data as *mut MnInfoData;
    if !(*data).description.is_null() {
        HSD_SisLib_803A5CC4((*data).description);
    }
    text = HSD_SisLib_803A5ACC(
        0 as ::core::ffi::c_int,
        1 as s32,
        -9.5f32,
        9.1f32,
        17.0f32,
        364.68332f32,
        38.38772f32,
    );
    (*data).description = text;
    (*text).font_size.x = 0.0521f32 as f32_0;
    (*text).font_size.y = 0.0521f32 as f32_0;
    HSD_SisLib_803A6368(text, 0xa3 as s32);
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0 as u16_0, 1 as u8_0, 0x80 as u8_0),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn() -> ()>,
            HSD_GObjEvent,
        >(Some(fn_80251FE4 as unsafe extern "C" fn() -> ())),
        0 as u8_0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u16_0 as u8_0 as u8_0);
    return proc_0 as s32;
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
    hook!(H_HSD_JObjReqAnimAll, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32_0));
    pub static mut H_HSD_JObjLoadJoint: Option<unsafe extern "C" fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            pub unsafe extern "C" fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj {
                match H_HSD_JObjLoadJoint {
                    Some(f) => f(a0),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ }; },
                }
            }
    hook!(H_HSD_JObjAddAnimAll, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(H_HSD_JObjSetFlagsAll, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
    hook!(H_HSD_JObjClearFlagsAll, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, a1: u32_0));
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
    pub static mut H_gm_801692E8: Option<unsafe extern "C" fn(u32_0, *mut datetime) > = None;
            pub unsafe extern "C" fn gm_801692E8(a0: u32_0, a1: *mut datetime)  {
                match H_gm_801692E8 {
                    Some(f) => f(a0, a1),
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_801692E8(a0 as _, a1 as _); } return; },
                }
            }
    pub static mut H_gmMainLib_8015D804: Option<unsafe extern "C" fn(s32) -> *mut u32_0> = None;
            pub unsafe extern "C" fn gmMainLib_8015D804(a0: s32) -> *mut u32_0 {
                match H_gmMainLib_8015D804 {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015D804(a0 as _) as _ }; },
                }
            }
    pub static mut H_gmMainLib_8015D94C: Option<unsafe extern "C" fn(u32_0) -> ::core::ffi::c_int> = None;
            pub unsafe extern "C" fn gmMainLib_8015D94C(a0: u32_0) -> ::core::ffi::c_int {
                match H_gmMainLib_8015D94C {
                    Some(f) => f(a0),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015D94C(a0 as _) as _ }; },
                }
            }
    pub static mut H_un_802FE3F8: Option<unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_int, *mut s16, *mut s16) > = None;
            pub unsafe extern "C" fn un_802FE3F8(a0: ::core::ffi::c_int, a1: ::core::ffi::c_int, a2: *mut s16, a3: *mut s16)  {
                match H_un_802FE3F8 {
                    Some(f) => f(a0, a1, a2, a3),
                    None => { unsafe { let _ = melee_world::tu::ifprize::un_802FE3F8(a0 as _, a1 as _, a2 as _, a3 as _); } return; },
                }
            }
    hook!(H_lbArchive_LoadSections, lbArchive_LoadSections, (a0: *mut HSD_Archive, a1: *mut *mut ::core::ffi::c_void, va: &[usize]));
    pub static mut H_lbLang_IsSettingJP: Option<unsafe extern "C" fn() -> bool_0> = None;
            pub unsafe extern "C" fn lbLang_IsSettingJP() -> bool_0 {
                match H_lbLang_IsSettingJP {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingJP() as _ }; },
                }
            }
    pub static mut H_lbLang_IsSettingUS: Option<unsafe extern "C" fn() -> bool_0> = None;
            pub unsafe extern "C" fn lbLang_IsSettingUS() -> bool_0 {
                match H_lbLang_IsSettingUS {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_IsSettingUS() as _ }; },
                }
            }
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
    hook!(H_HSD_GObjProc_RemoveProc, HSD_GObjProc_RemoveProc, (a0: *mut HSD_GObjProc));
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
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_804D783C: s32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8_0 = unsafe { core::mem::zeroed() };
}
