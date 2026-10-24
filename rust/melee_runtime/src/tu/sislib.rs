//! Native port of src/sysdolphin/baselib/sislib.c.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_variables, unused_assignments, static_mut_refs, clippy::all)]
use melee_types::*;
use ext::*;
// The C heap rounds to four bytes. Native SisBlock contains eight-byte pointers,
// so headers can be only four-byte aligned; avoid forming aligned Rust references.
macro_rules! sb_read { ($p:expr, $f:ident) => { core::ptr::addr_of!((*$p).$f).read_unaligned() }; }
macro_rules! sb_write { ($p:expr, $f:ident, $v:expr) => { core::ptr::addr_of_mut!((*$p).$f).write_unaligned($v) }; }

macro_rules! OSReport { ($fmt:expr $(,)?) => { ext::OSReport($fmt, core::ptr::null_mut()) }; ($fmt:expr, $arg:expr $(,)?) => { ext::OSReport($fmt, $arg) }; }
pub static mut HSD_SisLib_8040C490: HSD_WObjDesc = HSD_WObjDesc {
    class_name: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    pos: Vec3 {
        x: 0 as ::core::ffi::c_int as f32,
        y: 0 as ::core::ffi::c_int as f32,
        z: 1 as ::core::ffi::c_int as f32,
    },
    robjdesc: ::core::ptr::null::<HSD_RObjDesc>() as *mut HSD_RObjDesc,
};
pub static mut HSD_SisLib_8040C4A4: HSD_WObjDesc = HSD_WObjDesc {
    class_name: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    pos: Vec3 {
        x: 0 as ::core::ffi::c_int as f32,
        y: 0 as ::core::ffi::c_int as f32,
        z: 0 as ::core::ffi::c_int as f32,
    },
    robjdesc: ::core::ptr::null::<HSD_RObjDesc>() as *mut HSD_RObjDesc,
};
pub static mut HSD_SisLib_804D7968: u32 = 0;
pub static mut HSD_SisLib_8040C4B8: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
        class_name: ::core::ptr::null::<::core::ffi::c_char>()
            as *mut ::core::ffi::c_char,
        flags: 0 as u16,
        projection_type: PROJ_PERSPECTIVE as u16,
        viewport: _HSD_RectS16 {
            xmin: 0 as s16,
            xmax: 640 as s16,
            ymin: 0 as s16,
            ymax: 480 as s16,
        },
        scissor: _Scissor {
            left: 0 as u16,
            right: 640 as u16,
            top: 0 as u16,
            bottom: 480 as u16,
        },
        eyepos: &raw const HSD_SisLib_8040C490 as *mut HSD_WObjDesc,
        interest: &raw const HSD_SisLib_8040C4A4 as *mut HSD_WObjDesc,
        roll: 0.0f32,
        up_vector: ::core::ptr::null::<Vec3>() as *mut Vec3,
        nnear: 0.0f32,
        ffar: 0xffff as ::core::ffi::c_int as f32,
        fov: 30.0f32,
        aspect: 1.3333f32,
};
pub static mut HSD_SisLib_804D797C: *mut sislib_UnkAlloc3 = ::core::ptr::null::<
    sislib_UnkAlloc3,
>() as *mut sislib_UnkAlloc3;
pub static mut HSD_SisLib_804D7978: *mut HSD_Text = ::core::ptr::null::<HSD_Text>()
    as *mut HSD_Text;
pub static mut used_head: *mut SisBlock = ::core::ptr::null::<SisBlock>()
    as *mut SisBlock;
pub static mut free_head: *mut SisBlock = ::core::ptr::null::<SisBlock>()
    as *mut SisBlock;
pub static mut HSD_SisLib_804D1110: [*mut HSD_Archive; 5] = [::core::ptr::null::<
    HSD_Archive,
>() as *mut HSD_Archive; 5];
pub static mut HSD_SisLib_804D1124: [*mut SIS; 5] = [::core::ptr::null::<SIS>()
    as *mut SIS; 5];
pub unsafe extern "C" fn HSD_SisLib_Alloc(mut size: s32) -> *mut ::core::ffi::c_void {
    let mut best: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut alloc_tail: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut remainder: s32 = 0;
    let mut best_size: s32 = 0;
    let mut free_cur: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut alloc_cur: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    best = ::core::ptr::null_mut::<SisBlock>();
    alloc_tail = ::core::ptr::null_mut::<SisBlock>();
    free_cur = free_head;
    alloc_cur = used_head;
    if size == 0 as ::core::ffi::c_int {
        OSReport!(
            b"ZERO byte alloc\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        );
        panic!("HSD_SisLib panic");
    }
    remainder = (size as ::core::ffi::c_int % 4 as ::core::ffi::c_int) as s32;
    if remainder != 0 as ::core::ffi::c_int {
        size += (4 as s32 - remainder) as ::core::ffi::c_int;
    }
    while !alloc_cur.is_null() {
        alloc_tail = alloc_cur;
        alloc_cur = sb_read!(alloc_cur, next);
    }
    loop {
        if sb_read!(free_cur, size) == size as u32 {
            best = free_cur;
            break;
        } else {
            if sb_read!(free_cur, size) > size as u32 {
                if !best.is_null() {
                    if sb_read!(best, size) > sb_read!(free_cur, size) {
                        best = free_cur;
                    }
                } else {
                    best = free_cur;
                }
            }
            free_cur = sb_read!(free_cur, next);
            if free_cur.is_null() {
                break;
            }
        }
    }
    if best.is_null() {
        OSReport!(
            b"Memory Empty\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        );
        panic!("HSD_SisLib panic");
    }
    free_cur = free_head;
    if free_cur == best {
        let mut data_ptr: *mut u8 = ::core::ptr::null_mut::<u8>();
        let mut next_free: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
        let mut remaining_size: ::core::ffi::c_int = 0;
        data_ptr = sb_read!(free_head, data) as *mut u8;
        next_free = sb_read!(free_cur, next);
        remaining_size = (sb_read!(free_cur, size).wrapping_sub(size as u32) as usize)
            .wrapping_sub(::core::mem::size_of::<SisBlock>() as usize)
            as ::core::ffi::c_int;
        if remaining_size < 0 as ::core::ffi::c_int {
            OSReport!(
                b"Memory Empty\n\0" as *const u8 as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            );
            panic!("HSD_SisLib panic");
        }
        free_head = data_ptr.offset(size as isize) as *mut SisBlock;
        sb_write!(free_head, next, next_free);
        sb_write!(free_head, data, free_head.offset(1 as ::core::ffi::c_int as isize)
            as *mut HSD_Text);
        sb_write!(free_head, size, remaining_size as u32);
        sb_write!(best, size, size as u32);
    } else {
        while sb_read!(free_cur, next) != best {
            free_cur = sb_read!(free_cur, next);
        }
        sb_write!(free_cur, next, sb_read!(best, next));
    }
    sb_write!(best, next, ::core::ptr::null_mut::<SisBlock>());
    if !alloc_tail.is_null() {
        sb_write!(alloc_tail, next, best);
    }
    if used_head.is_null() {
        used_head = best;
    }
    return sb_read!(best, data) as *mut ::core::ffi::c_void;
}
pub unsafe extern "C" fn HSD_SisLib_Free(mut ptr: *mut ::core::ffi::c_void) {
    let mut free_cur: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut free_tail: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut alloc_prev: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut alloc_cur: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    free_cur = free_head;
    free_tail = ::core::ptr::null_mut::<SisBlock>();
    alloc_cur = used_head;
    alloc_prev = ::core::ptr::null_mut::<SisBlock>();
    while !alloc_cur.is_null() {
        if sb_read!(alloc_cur, data) == ptr as *mut HSD_Text {
            break;
        }
        alloc_prev = alloc_cur;
        alloc_cur = sb_read!(alloc_cur, next);
    }
    if alloc_cur.is_null() {
        return;
    }
    if free_cur as *mut u8
        == (sb_read!(alloc_cur, data) as *mut u8).offset(sb_read!(alloc_cur, size) as isize)
    {
        let mut old_next: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
        let mut new_size: u32 = 0;
        old_next = sb_read!(free_cur, next);
        new_size = (sb_read!(free_cur, size).wrapping_add(sb_read!(alloc_cur, size)) as usize)
            .wrapping_add(::core::mem::size_of::<SisBlock>() as usize) as u32;
        if !alloc_prev.is_null() {
            sb_write!(alloc_prev, next, sb_read!(alloc_cur, next));
        } else {
            used_head = sb_read!(alloc_cur, next);
        }
        free_head = alloc_cur;
        sb_write!(alloc_cur, next, old_next);
        sb_write!(free_head, data, free_head.offset(1 as ::core::ffi::c_int as isize)
            as *mut HSD_Text);
        sb_write!(free_head, size, new_size);
        return;
    }
    while !free_cur.is_null() {
        free_tail = free_cur;
        free_cur = sb_read!(free_cur, next);
    }
    if !free_tail.is_null() {
        sb_write!(free_tail, next, alloc_cur);
    }
    if !alloc_prev.is_null() {
        sb_write!(alloc_prev, next, sb_read!(alloc_cur, next));
    } else {
        used_head = sb_read!(alloc_cur, next);
    }
    sb_write!(alloc_cur, next, ::core::ptr::null_mut::<SisBlock>());
}
pub unsafe extern "C" fn HSD_SisLib_803A5A2C(mut ptr: *mut ::core::ffi::c_void) {
    let mut next_text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut alloc: *mut SisBlock = ::core::ptr::null_mut::<SisBlock>();
    let mut curr: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut last: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    last = ::core::ptr::null_mut::<HSD_Text>();
    curr = HSD_SisLib_804D7978;
    while !curr.is_null() {
        next_text = (*curr).next;
        if curr == ptr as *mut HSD_Text {
            if !last.is_null() {
                (*last).next = next_text;
            } else {
                HSD_SisLib_804D7978 = next_text;
            }
            alloc = (*curr).alloc_data;
            if !alloc.is_null() {
                if !sb_read!(alloc, data).is_null() {
                    HSD_SisLib_Free(sb_read!(alloc, data) as *mut ::core::ffi::c_void);
                }
                HSD_SisLib_Free((*curr).alloc_data as *mut ::core::ffi::c_void);
            }
            if !(*curr).string_buffer.is_null() {
                HSD_SisLib_Free((*curr).string_buffer as *mut ::core::ffi::c_void);
            }
            HSD_SisLib_Free(curr as *mut ::core::ffi::c_void);
            return;
        }
        last = curr;
        curr = next_text;
    }
}
pub unsafe extern "C" fn HSD_SisLib_803A5ACC(
    mut font_idx: ::core::ffi::c_int,
    mut context_id: s32,
    mut pos_x: f32,
    mut pos_y: f32,
    mut pos_z: f32,
    mut box_w: f32,
    mut box_h: f32,
) -> *mut HSD_Text {
    let mut list_cur: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut list_tail: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    let mut cam_entry: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<
        sislib_UnkAlloc3,
    >();
    let mut gobj: *mut HSD_GObj = ::core::ptr::null_mut::<HSD_GObj>();
    let mut cam_idx: ::core::ffi::c_int = 0;
    let mut text: *mut HSD_Text = ::core::ptr::null_mut::<HSD_Text>();
    cam_idx = context_id as ::core::ffi::c_int;
    list_tail = ::core::ptr::null_mut::<HSD_Text>();
    cam_entry = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    gobj = ::core::ptr::null_mut::<HSD_GObj>();
    list_cur = HSD_SisLib_804D7978;
    if cam_idx >= 0 as ::core::ffi::c_int {
        cam_entry = HSD_SisLib_804D797C;
        while !((*cam_entry).xA as ::core::ffi::c_int == font_idx
            && {
                cam_idx -= 1;
                cam_idx < 0 as ::core::ffi::c_int
            })
        {
            if cam_entry.is_null() {
                OSReport!(
                    b"unknow camera\n\0" as *const u8 as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
                return ::core::ptr::null_mut::<HSD_Text>();
            }
            cam_entry = (*cam_entry).x0;
        }
        gobj = GObj_Create((*cam_entry).x8, (*cam_entry).xC, (*cam_entry).xD);
        GObj_SetupGXLink(
            gobj,
            Some(
                ext::cb::HSD_SisLib_803A84BC
                    as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
            ),
            (*cam_entry).xE,
            (*cam_entry).xF as u32,
        );
    }
    while !list_cur.is_null() {
        list_tail = list_cur;
        list_cur = (*list_cur).next;
    }
    text = HSD_SisLib_Alloc(::core::mem::size_of::<HSD_Text>() as s32) as *mut HSD_Text;
    if HSD_SisLib_804D7978.is_null() {
        HSD_SisLib_804D7978 = text;
    }
    if !list_tail.is_null() {
        (*list_tail).next = text;
    }
    if !gobj.is_null() {
        GObj_InitUserData(
            gobj,
            (*cam_entry).x8 as u8,
            Some(
                HSD_SisLib_803A5A2C
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            text as *mut ::core::ffi::c_void,
        );
    }
    (*text).next = ::core::ptr::null_mut::<HSD_Text>();
    (*text).entity = gobj as *mut HSD_GObj;
    (*text).pos_x = pos_x;
    (*text).pos_y = pos_y;
    (*text).pos_z = pos_z;
    (*text).box_size_x = box_w;
    (*text).box_size_y = box_h;
    (*text).x14.w = 0.0f32 as f32;
    (*text).x14.z = 0.0f32 as f32;
    (*text).x14.y = 0.0f32 as f32;
    (*text).x14.x = 0.0f32 as f32;
    (*text).font_size.y = 1.0f32 as f32;
    (*text).font_size.x = 1.0f32 as f32;
    (*text).alloc_data = ::core::ptr::null_mut::<SisBlock>();
    (*text).string_buffer = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*text).sis_buffer = ::core::ptr::null_mut::<SIS>();
    (*text).bg_color.a = 0 as u8;
    (*text).bg_color.b = 0 as u8;
    (*text).bg_color.g = 0 as u8;
    (*text).bg_color.r = 0 as u8;
    (*text).text_color.a = 0xff as u8;
    (*text).text_color.b = 0xff as u8;
    (*text).text_color.g = 0xff as u8;
    (*text).text_color.r = 0xff as u8;
    (*text).x34.y = 1.0f32 as f32;
    (*text).x34.x = 1.0f32 as f32;
    (*text).x3C.y = 0.0f32 as f32;
    (*text).x3C.x = 0.0f32 as f32;
    (*text).x46 = 0 as u16;
    (*text).x44 = 0 as u16;
    (*text).default_alignment = 0 as u8;
    (*text).default_fitting = 0 as u8;
    (*text).default_kerning = 0 as u8;
    (*text).x6E = 0 as u16;
    (*text).x6C = 0 as u16;
    (*text).x4E = 0 as u8;
    (*text).hidden = 0 as u8;
    (*text).x4C = 0 as u8;
    (*text).render_callback = None;
    (*text).font_idx = font_idx as u8;
    return text;
}
pub unsafe extern "C" fn HSD_SisLib_803A5CC4(mut text: *mut HSD_Text) {
    let mut curr: *mut HSD_Text = HSD_SisLib_804D7978;
    while !curr.is_null() {
        let mut next: *mut HSD_Text = (*curr).next;
        if curr == text {
            if !(*curr).entity.is_null() {
                HSD_GObjFree((*curr).entity as *mut HSD_GObj);
                (*curr).entity = ::core::ptr::null_mut::<HSD_GObj>();
            } else {
                HSD_SisLib_803A5A2C(curr as *mut ::core::ffi::c_void);
            }
            return;
        }
        curr = next;
    }
}
pub unsafe extern "C" fn HSD_SisLib_803A5D30() {
    let mut curr: *mut HSD_Text = HSD_SisLib_804D7978;
    while !curr.is_null() {
        let mut next: *mut HSD_Text = (*curr).next;
        if !(*curr).entity.is_null() {
            HSD_GObjFree((*curr).entity as *mut HSD_GObj);
            (*curr).entity = ::core::ptr::null_mut::<HSD_GObj>();
        } else {
            HSD_SisLib_803A5A2C(curr as *mut ::core::ffi::c_void);
        }
        curr = next;
    }
}
#[inline]
pub unsafe extern "C" fn HSD_SisLib_803A5DA0_inline0(mut font_idx: s32) {
    let mut curr: *mut HSD_Text = HSD_SisLib_804D7978;
    while !curr.is_null() {
        let mut next: *mut HSD_Text = (*curr).next;
        if (*curr).font_idx as ::core::ffi::c_int == font_idx {
            if !(*curr).entity.is_null() {
                HSD_GObjFree((*curr).entity as *mut HSD_GObj);
                (*curr).entity = ::core::ptr::null_mut::<HSD_GObj>();
            } else {
                HSD_SisLib_803A5A2C(curr as *mut ::core::ffi::c_void);
            }
        }
        curr = next;
    }
}
pub unsafe extern "C" fn HSD_SisLib_803A5DA0(mut font_idx: s32) {
    let mut last: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    let mut curr: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    HSD_SisLib_803A5DA0_inline0(font_idx);
    curr = HSD_SisLib_804D797C;
    while !curr.is_null() {
        let mut next: *mut sislib_UnkAlloc3 = (*curr).x0;
        if (*curr).xA as ::core::ffi::c_int == font_idx {
            if !(*curr).x4.is_null() {
                HSD_GObjFree((*curr).x4 as *mut HSD_GObj);
                (*curr).x4 = ::core::ptr::null_mut::<HSD_GObj>();
            }
            if !last.is_null() {
                (*last).x0 = next;
            } else {
                HSD_SisLib_804D797C = next;
            }
            HSD_SisLib_Free(curr as *mut ::core::ffi::c_void);
        }
        last = curr;
        curr = next;
    }
}
pub static mut HSD_SisLib_804D796C: *mut SisBlock = ::core::ptr::null::<SisBlock>()
    as *mut SisBlock;
pub unsafe extern "C" fn HSD_SisLib_803A5E70() {
    let mut curr: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    HSD_SisLib_803A5D30();
    curr = HSD_SisLib_804D797C;
    while !curr.is_null() {
        let mut next: *mut sislib_UnkAlloc3 = (*curr).x0;
        if !(*curr).x4.is_null() {
            HSD_GObjFree((*curr).x4 as *mut HSD_GObj);
            (*curr).x4 = ::core::ptr::null_mut::<HSD_GObj>();
        }
        HSD_SisLib_Free(curr as *mut ::core::ffi::c_void);
        curr = next;
    }
    HSD_SisLib_804D797C = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    free_head = HSD_SisLib_804D796C;
    used_head = ::core::ptr::null_mut::<SisBlock>();
    sb_write!(free_head, next, ::core::ptr::null_mut::<SisBlock>());
    sb_write!(free_head, data, free_head.offset(1 as ::core::ffi::c_int as isize)
        as *mut HSD_Text);
    sb_write!(free_head, size, (HSD_SisLib_804D7968 as usize)
        .wrapping_sub(::core::mem::size_of::<SisBlock>() as usize) as u32);
}
pub unsafe extern "C" fn HSD_SisLib_803A5F50(mut font_idx: s32) {
    let mut archive_ptr: *mut *mut HSD_Archive = ::core::ptr::null_mut::<
        *mut HSD_Archive,
    >();
    HSD_SisLib_803A5DA0(font_idx);
    archive_ptr = (&raw mut HSD_SisLib_804D1110 as *mut *mut HSD_Archive)
        .offset(font_idx as isize) as *mut *mut HSD_Archive;
    if !(*archive_ptr).is_null() {
        HSD_SisLib_803A947C(*archive_ptr);
        *archive_ptr = ::core::ptr::null_mut::<HSD_Archive>();
        HSD_SisLib_804D1124[font_idx as usize] = ::core::ptr::null_mut::<SIS>();
    }
}
pub unsafe extern "C" fn HSD_SisLib_803A5FBC() {
    let mut i: ::core::ffi::c_int = 0;
    HSD_SisLib_803A5E70();
    i = 0 as ::core::ffi::c_int;
    while i < 5 as ::core::ffi::c_int {
        if !HSD_SisLib_804D1110[i as usize].is_null() {
            HSD_SisLib_803A947C(HSD_SisLib_804D1110[i as usize]);
            HSD_SisLib_804D1110[i as usize] = ::core::ptr::null_mut::<HSD_Archive>();
            HSD_SisLib_804D1124[i as usize] = ::core::ptr::null_mut::<SIS>();
        }
        i += 1;
    }
    HSD_Free(HSD_SisLib_804D796C as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn HSD_SisLib_803A6048(mut size: u32) {
    let mut i: ::core::ffi::c_int = 0;
    HSD_SisLib_804D7968 = size as u32;
    used_head = ::core::ptr::null_mut::<SisBlock>();
    free_head = HSD_MemAlloc(HSD_SisLib_804D7968 as ssize_t) as *mut SisBlock;
    HSD_SisLib_804D796C = free_head;
    sb_write!(free_head, next, ::core::ptr::null_mut::<SisBlock>());
    sb_write!(free_head, data, free_head.offset(1 as ::core::ffi::c_int as isize)
        as *mut HSD_Text);
    sb_write!(free_head, size, (HSD_SisLib_804D7968 as usize)
        .wrapping_sub(::core::mem::size_of::<SisBlock>() as usize) as u32);
    HSD_SisLib_804D7978 = ::core::ptr::null_mut::<HSD_Text>();
    HSD_SisLib_804D797C = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    i = 0 as ::core::ffi::c_int;
    while i < 5 as ::core::ffi::c_int {
        HSD_SisLib_804D1110[i as usize] = ::core::ptr::null_mut::<HSD_Archive>();
        HSD_SisLib_804D1124[i as usize] = ::core::ptr::null_mut::<SIS>();
        i += 1;
    }
}
pub unsafe extern "C" fn fn_803A60EC(mut gobj: *mut ::core::ffi::c_void) {
    let mut curr: *mut sislib_UnkAlloc3 = HSD_SisLib_804D797C;
    while !curr.is_null() {
        if (*curr).x4 == gobj as *mut HSD_GObj {
            (*curr).x4 = ::core::ptr::null_mut::<HSD_GObj>();
            return;
        }
        curr = (*curr).x0;
    }
}
pub unsafe extern "C" fn sislib_WideCamera(
    mut gobj: *mut HSD_GObj,
    mut pass: ::core::ffi::c_int,
) {

    let aspect = f32::from_bits(View_AspectBits() as u32);
    let mut width: ::core::ffi::c_float = 0.;
    let mut left: ::core::ffi::c_float = 0.;
    width = 480.0f32 * aspect;
    left = (640.0f32 - width) * 0.5f32;
    HSD_CObjSetOrtho(
        (*gobj).hsd_obj as *mut HSD_CObj,
        0.0f32,
        -480.0f32,
        left as f32,
        left as f32 + width as f32,
    );
    HSD_GObj_803910D8(gobj, pass);
}
pub unsafe extern "C" fn HSD_SisLib_SetWideCanvas(
    mut font_idx: ::core::ffi::c_int,
    mut canvas: ::core::ffi::c_int,
) {
    let mut entry: *mut sislib_UnkAlloc3 = HSD_SisLib_804D797C;
    let mut index: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !entry.is_null() {
        if (*entry).xA as ::core::ffi::c_int == font_idx
            && {
                let fresh0 = index;
                index = index + 1;
                fresh0 == canvas
            }
        {
            if !(*entry).x4.is_null() {
                (*(*entry).x4).render_cb = Some(
                    sislib_WideCamera
                        as unsafe extern "C" fn(*mut HSD_GObj, ::core::ffi::c_int) -> (),
                ) as GObj_RenderFunc;
            }
            return;
        }
        entry = (*entry).x0;
    }
}
pub unsafe extern "C" fn HSD_SisLib_803A611C(
    mut font_idx: ::core::ffi::c_int,
    mut parent_gobj: *mut HSD_GObj,
    mut class_id: u16,
    mut p_link: u8,
    mut p_prio: u8,
    mut gx_link: u8,
    mut gx_prio: u8,
    mut render_prio: u32,
) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0;
    let mut list_tail: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<
        sislib_UnkAlloc3,
    >();
    let mut entry: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    let mut list_cur: *mut sislib_UnkAlloc3 = ::core::ptr::null_mut::<
        sislib_UnkAlloc3,
    >();
    list_tail = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    count = 0 as ::core::ffi::c_int;
    list_cur = HSD_SisLib_804D797C;
    while !list_cur.is_null() {
        list_tail = list_cur;
        if (*list_cur).xA as ::core::ffi::c_int == font_idx {
            count += 1 as ::core::ffi::c_int;
        }
        list_cur = (*list_cur).x0;
    }
    entry = HSD_SisLib_Alloc(::core::mem::size_of::<sislib_UnkAlloc3>() as s32)
        as *mut sislib_UnkAlloc3;
    if HSD_SisLib_804D797C.is_null() {
        HSD_SisLib_804D797C = entry;
    }
    if !list_tail.is_null() {
        (*list_tail).x0 = entry;
    }
    (*entry).x8 = class_id;
    (*entry).xC = p_link;
    (*entry).xD = p_prio;
    (*entry).xE = gx_link;
    (*entry).xF = gx_prio;
    (*entry).xA = font_idx as u16;
    (*entry).x0 = ::core::ptr::null_mut::<sislib_UnkAlloc3>();
    if !parent_gobj.is_null() {
        (*entry).x4 = ::core::ptr::null_mut::<HSD_GObj>();
    } else {
        (*entry).x4 = GObj_Create(class_id, p_link, p_prio) as *mut HSD_GObj;
        if !(*entry).x4.is_null() {
            let mut cobj: *mut HSD_CObj = HSD_CObjLoadDesc(
                &raw mut HSD_SisLib_8040C4B8 as *mut HSD_CObjDesc,
            );
            if !cobj.is_null() {
                HSD_CObjSetOrtho(cobj, 0.0f32, -480.0f32, 0.0f32, 640.0f32);
                let mut tmp: u8 = HSD_GObj_CameraKind;
                HSD_GObjObject_80390A70(
                    (*entry).x4 as *mut HSD_GObj,
                    tmp,
                    cobj as *mut ::core::ffi::c_void,
                );
                GObj_SetupGXLinkMax(
                    (*entry).x4 as *mut HSD_GObj,
                    Some(
                        ext::cb::HSD_GObj_803910D8
                            as unsafe extern "C" fn(
                                *mut HSD_GObj,
                                ::core::ffi::c_int,
                            ) -> (),
                    ),
                    render_prio,
                );
                (*(*entry).x4).gxlink_prios = 1u64.checked_shl(gx_link as u32).unwrap_or(0);
                GObj_InitUserData(
                    (*entry).x4 as *mut HSD_GObj,
                    class_id as u8,
                    Some(
                        fn_803A60EC
                            as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                    ),
                    (*entry).x4 as *mut ::core::ffi::c_void,
                );
            } else {
                HSD_GObjFree((*entry).x4 as *mut HSD_GObj);
                (*entry).x4 = ::core::ptr::null_mut::<HSD_GObj>();
            }
        }
    }
    return count;
}
pub unsafe extern "C" fn HSD_SisLib_803A62A0(
    mut font_idx: s32,
    mut archive_name: *mut ::core::ffi::c_char,
    mut symbol_name: *mut ::core::ffi::c_char,
) {
    let mut tmp: *mut HSD_Archive = HSD_SisLib_803A945C(archive_name);
    HSD_SisLib_804D1110[font_idx as usize] = tmp;
    if tmp.is_null() {
        OSReport!(
            b"Cannot open archive %s.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            archive_name,
        );
        panic!("HSD_SisLib panic");
    }
    let mut sis: *mut SIS = HSD_ArchiveGetPublicAddress(
        HSD_SisLib_804D1110[font_idx as usize],
        symbol_name,
    ) as *mut SIS;
    HSD_SisLib_804D1124[font_idx as usize] = sis;
    if sis.is_null() {
        OSReport!(
            b"Cannot find symbol %s.\n\0" as *const u8 as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
            symbol_name,
        );
        panic!("HSD_SisLib panic");
    }
}
pub unsafe extern "C" fn HSD_SisLib_803A6368(mut text: *mut HSD_Text, mut sis_idx: s32) {
    let mut sis_table: *mut *mut SIS = ::core::ptr::null_mut::<*mut SIS>();
    let mut i: s32 = 0;
    sis_table = HSD_SisLib_804D1124[(*text).font_idx as usize] as *mut *mut SIS;
    if !sis_table.is_null() {
        (*text).sis_buffer = *sis_table.offset(sis_idx as isize);
    }
    (*text).x60 = ::core::ptr::null_mut::<::core::ffi::c_void>();
    (*text).current_height = 0.0f32 as f32;
    (*text).current_width = 0.0f32 as f32;
    (*text).active_color = (*text).text_color;
    (*text).x80.x = (*text).x34.x;
    (*text).x80.y = (*text).x34.y;
    (*text).x78.x = (*text).x3C.x;
    (*text).x78.y = (*text).x3C.y;
    (*text).x90 = (*text).x44;
    (*text).x92 = (*text).x46;
    (*text).alignment = (*text).default_alignment;
    (*text).kerning = (*text).default_kerning;
    (*text).fitting = (*text).default_fitting;
    (*text).x6C = 0 as u16;
    (*text).x98 = 0 as u32;
    (*text).x94 = 0 as u32;
    (*text).x4B = 0 as u8;
    if !(*text).string_buffer.is_null() {
        HSD_SisLib_Free((*text).string_buffer as *mut ::core::ffi::c_void);
    }
    (*text).string_buffer = HSD_SisLib_Alloc(0x10 as s32) as *mut ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int as s32;
    (*text).x6E = 0x10 as u16;
    while i < (*text).x6E as ::core::ffi::c_int {
        *(*text).string_buffer.offset(i as isize) = 0 as ::core::ffi::c_char;
        i += 1 as ::core::ffi::c_int;
    }
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(a0: *mut HSD_GObj, a1: u8, a2: Option<unsafe extern "C" fn(*mut c_void)>, a3: *mut c_void);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void;
    fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj;
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32);
    fn HSD_Free(a0: *mut c_void);
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void;
    fn HSD_SisLib_803A84BC(a0: *mut HSD_GObj, a1: i32);
    fn HSD_SisLib_803A945C(a0: *mut i8) -> *mut HSD_Archive;
    fn HSD_SisLib_803A947C(a0: *mut HSD_Archive);
    fn OSReport(a0: *mut core::ffi::c_char, a1: *mut core::ffi::c_char);
    fn View_AspectBits() -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
        pub unsafe extern "C" fn HSD_SisLib_803A84BC(a0: *mut HSD_GObj, a1: i32) { super::HSD_SisLib_803A84BC(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT
