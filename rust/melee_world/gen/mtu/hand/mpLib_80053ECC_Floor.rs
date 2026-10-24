/// `mpLib_80053ECC_Floor`: the start (left vertex) of the floor run `line_id` belongs to
pub unsafe fn mpLib_80053ECC_Floor(mut line_id: i32, mut vec: *mut Vec) {
    lineid_check(4448, line_id);

    // `goto skip; loop: line_id = prev_id; skip: prev_id = ...; if (...) goto loop;`
    loop {
        let prev_id: i16 = (*(*groundCollLine.offset(line_id as isize)).x0).prev_id0;
        if prev_id != -1 && ((*groundCollLine.offset(prev_id as isize)).flags & (CollLine_Floor as u32)) != 0 {
            line_id = prev_id as i32;
            continue;
        }
        break;
    }

    lineid_check(4453, line_id);

    let vtx: *mut CollVtx = groundCollVtx.offset((*(*groundCollLine.offset(line_id as isize)).x0).v0_idx as isize);
    (*vec).x = (*vtx).pos.x;
    (*vec).y = (*vtx).pos.y;
    (*vec).z = 0.0f32;
}
