/// `mpIsland_8005AE1C`: re-sort the floor (`arg0`) / ceiling (`arg1`) island lists against the vertex
/// range `[arg2, arg2 + arg3)`; `arg4` selects the side an overlapping island goes to. The C is a
/// goto state machine (`loop1_body` / `loop2_body`); each loop is a `while` here, with the
/// `add_to_prev_*` labels as flags.
pub unsafe fn mpIsland_8005AE1C(mut arg0: *mut *mut mp_UnkStruct0, mut arg1: *mut *mut mp_UnkStruct0, mut arg2: i32, mut arg3: i32, mut arg4: i32) {
    let mut prev_b: *mut mp_UnkStruct0 = null_mut();
    let mut prev_a: *mut mp_UnkStruct0 = null_mut();
    let vtx: *mut CollVtx = super::mplib::mpGetGroundCollVtx();
    let mut cur: *mut mp_UnkStruct0 = *arg0;
    let end: i32 = arg2 + arg3;
    let z_val: f32 = 0.0;

    while !cur.is_null() {
        let v0: u16 = (*cur).x4;
        let next: *mut mp_UnkStruct0 = (*cur).next;

        // add_to_prev_b_1: the island lies entirely below or entirely above the range
        let mut to_b = false;
        if (v0 as i32) < arg2 {
            if ((*cur).x6 as i32) < arg2 {
                to_b = true;
            }
        }
        if !to_b && !(end > v0 as i32) {
            if end <= (*cur).x6 as i32 {
                to_b = true;
            }
        }
        if !to_b {
            // do_assign_1
            (*cur).x8.x = (*vtx.offset(v0 as isize)).pos.x;
            (*cur).x8.y = (*vtx.offset((*cur).x4 as isize)).pos.y;
            (*cur).x8.z = z_val;
            (*cur).x14.x = (*vtx.offset((*cur).x6 as isize)).pos.x;
            (*cur).x14.y = (*vtx.offset((*cur).x6 as isize)).pos.y;
            (*cur).x14.z = z_val;

            if arg4 != 0 || ((*cur).x20 & 2) != 0 {
                to_b = true;
            } else {
                (*cur).x20 |= 2;
                (*cur).next = prev_a;
                prev_a = cur;
            }
        }
        if to_b {
            (*cur).next = prev_b;
            prev_b = cur;
        }
        cur = next;
    }

    cur = *arg1;
    while !cur.is_null() {
        let v0: u16 = (*cur).x4;
        let next: *mut mp_UnkStruct0 = (*cur).next;

        // add_to_prev_a_2
        let mut to_a = false;
        if (v0 as i32) < arg2 {
            if ((*cur).x6 as i32) < arg2 {
                to_a = true;
            }
        }
        if !to_a && !(end > v0 as i32) {
            if end <= (*cur).x6 as i32 {
                to_a = true;
            }
        }
        if !to_a {
            // do_assign_2
            (*cur).x8.x = (*vtx.offset(v0 as isize)).pos.x;
            (*cur).x8.y = (*vtx.offset((*cur).x4 as isize)).pos.y;
            (*cur).x8.z = z_val;
            (*cur).x14.x = (*vtx.offset((*cur).x6 as isize)).pos.x;
            (*cur).x14.y = (*vtx.offset((*cur).x6 as isize)).pos.y;
            (*cur).x14.z = z_val;

            if arg4 == 0 || ((*cur).x20 & 2) == 0 {
                to_a = true;
            } else {
                (*cur).x20 &= !2;
                (*cur).next = prev_b;
                prev_b = cur;
            }
        }
        if to_a {
            (*cur).next = prev_a;
            prev_a = cur;
        }
        cur = next;
    }

    *arg0 = prev_b;
    *arg1 = prev_a;
}
