/// `mpLib_80053DA4_Floor`: the end (right vertex) of the floor run `line_id` belongs to
pub unsafe fn mpLib_80053DA4_Floor(mut line_id: i32, mut pos_out: *mut Vec3) {
    lineid_check(4428, line_id);

    // `goto skip; loop: line_id = next_id; skip: next_id = ...; if (...) goto loop;`
    loop {
        let next_id: i16 = (*(*groundCollLine.offset(line_id as isize)).x0).next_id0;
        if next_id != -1 && ((*groundCollLine.offset(next_id as isize)).flags & (CollLine_Floor as u32)) != 0 {
            line_id = next_id as i32;
            continue;
        }
        break;
    }

    lineid_check(4433, line_id);

    let vtx: *mut CollVtx = groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v1_idx as isize);
    (*pos_out).x = (*vtx).pos.x;
    (*pos_out).y = (*vtx).pos.y;
    (*pos_out).z = 0.0f32;
}
