/// `mpLib_80056C54`: walk `var_f25` units along the floor run from `pos` (`arg7` is the budget
/// spent on wall / ceiling lines). The two `goto block_30/55` jumps into the else arm become the
/// `run` flag. `sqrt_tmp - 4` (a stack-layout hack in the C) is an ordinary scratch float here.
pub unsafe fn mpLib_80056C54(mut line_id: i32, mut pos: *mut Vec3, mut line_id_out: *mut i32, mut vec_out: *mut Vec3, mut flags_out: *mut u32, mut normal_out: *mut Vec3, mut var_f25: f32, mut arg7: f32) -> i32 {
    let mut sp64: f32 = 0.0;
    let mut sp58: Vec3 = core::mem::zeroed();
    let mut sp4C: Vec3 = core::mem::zeroed();
    let mut sqrt_tmp: f32 = 0.0;
    let mut dist_f28: f32 = 0.0;
    let mut total_dist_f27: f32 = 0.0;
    let mut x_f2: f32;
    let mut y_f0: f32;
    let mut result_r30: i32 = 1;
    let mut new_id_r5: i32;
    let mut flags_r0: u32;

    if mpLib_80054ED8(line_id) == 0 {
        return 0;
    }

    if mpLib_8004DD90_Floor(line_id, pos, &mut sp64, null_mut(), null_mut()) == -1 {
        return 0;
    }
    sp58 = *pos;
    sp58.y += sp64;
    if var_f25 > 0.0 {
        loop {
            mpLineGetV1Pos(line_id, &mut sp4C);
            x_f2 = (sp58.x - sp4C.x) * (sp58.x - sp4C.x);
            y_f0 = (sp58.y - sp4C.y) * (sp58.y - sp4C.y);
            dist_f28 = sqrtf_store(x_f2 + y_f0, &mut sqrt_tmp);
            flags_r0 = mpLineGetKind(line_id) as u32;
            let mut run = false;
            if flags_r0 & 0xC != 0 {
                total_dist_f27 += dist_f28;
                if total_dist_f27 > arg7 {
                    result_r30 = 0;
                } else {
                    run = true; // goto block_30
                }
            } else if flags_r0 & 2 != 0 {
                result_r30 = 0;
            } else {
                total_dist_f27 = 0.0;
                run = true;
            }
            if run && !(dist_f28 > var_f25) {
                new_id_r5 = mpLineGetNext(line_id);
                if new_id_r5 == -1 {
                    result_r30 = 0;
                } else {
                    var_f25 -= dist_f28;
                    line_id = new_id_r5;
                    sp58 = sp4C;
                    continue;
                }
            }
            break;
        }
    } else {
        var_f25 = -var_f25;
        loop {
            mpLineGetV0Pos(line_id, &mut sp4C);
            x_f2 = (sp58.x - sp4C.x) * (sp58.x - sp4C.x);
            y_f0 = (sp58.y - sp4C.y) * (sp58.y - sp4C.y);
            dist_f28 = sqrtf_store(x_f2 + y_f0, &mut sqrt_tmp);
            flags_r0 = mpLineGetKind(line_id) as u32;
            let mut run = false;
            if flags_r0 & 0xC != 0 {
                total_dist_f27 += dist_f28;
                if total_dist_f27 > arg7 {
                    result_r30 = 0;
                } else {
                    run = true; // goto block_55
                }
            } else if flags_r0 & 2 != 0 {
                result_r30 = 0;
            } else {
                total_dist_f27 = 0.0;
                run = true;
            }
            if run && !(dist_f28 > var_f25) {
                new_id_r5 = mpLineGetPrev(line_id);
                if new_id_r5 == -1 {
                    result_r30 = 0;
                } else {
                    var_f25 -= dist_f28;
                    line_id = new_id_r5;
                    sp58 = sp4C;
                    continue;
                }
            }
            break;
        }
    }
    if line_id != -1 {
        if (mpLineGetKind(line_id) as u32) & (CollLine_Floor as u32) == 0 {
            line_id = -1;
        }
    }
    if !line_id_out.is_null() {
        *line_id_out = line_id;
    }
    if line_id != -1 {
        if !flags_out.is_null() {
            *flags_out = mpLineGetFlags(line_id);
        }
        if !normal_out.is_null() {
            mpLineGetNormal(line_id, normal_out);
        }
    }
    if !vec_out.is_null() {
        if result_r30 != 0 {
            if (dist_f28 as f64) < 0.0001 {
                *vec_out = sp58;
            } else {
                let temp_f2_5: f32 = var_f25 / dist_f28;
                (*vec_out).x = fmadds(temp_f2_5, sp4C.x - sp58.x, sp58.x);
                (*vec_out).y = fmadds(temp_f2_5, sp4C.y - sp58.y, sp58.y);
                (*vec_out).z = fmadds(temp_f2_5, sp4C.z - sp58.z, sp58.z);
            }
        } else {
            *vec_out = sp4C;
        }
    }
    result_r30
}
