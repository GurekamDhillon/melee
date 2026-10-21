//! Tests for tu/ftanim.rs: the joint-tree walk (pre-order, via the module stack), the part-anim
//! blend step (no fused sites: `xC / (xC + (x4 - x8))`), and the slot reset.
use melee_fighter::tu::ftanim as m;
use melee_fighter::types::*;

#[test]
fn joint_tree_preorder() {
    // root -> (a -> (a1), b)
    let mut j: [HSD_Joint; 4] = unsafe { core::mem::zeroed() };
    let p: *mut HSD_Joint = j.as_mut_ptr();
    unsafe {
        (*p).child = p.add(1);
        (*p.add(1)).child = p.add(2);
        (*p.add(1)).next = p.add(3);
        let mut cur = p;
        let mut depth = 0;
        let mut order = vec![];
        while !cur.is_null() {
            order.push(cur.offset_from(p));
            m::ftAnim_GetNextJointInTree(&mut cur, &mut depth);
        }
        assert_eq!(order, vec![0, 1, 2, 3]);
        assert_eq!(depth, 0);
    }
}

#[test]
fn part_anim_step_bits() {
    let mut s: Fighter_x8B0_t = unsafe { core::mem::zeroed() };
    s.x4 = 10.0;
    s.xC = 0.3;
    let mut x8 = 0.0f32;
    for _ in 0..40 {
        let (t, ti) = m::part_anim_step(&mut s);
        x8 += 0.3f32;
        if 10.0 <= x8 {
            x8 = 10.0;
            assert_eq!((t, ti), (1.0, 0.0));
        } else {
            let e = 0.3f32 / (0.3f32 + (10.0f32 - x8));
            assert_eq!(t.to_bits(), e.to_bits());
            assert_eq!(ti.to_bits(), (1.0f32 - e).to_bits());
        }
        assert_eq!(s.x8.to_bits(), x8.to_bits());
    }
}

#[test]
fn slot_setters() {
    let mut fi: Fighter = unsafe { core::mem::zeroed() };
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = &mut fi as *mut Fighter as *mut _;
    m::ftAnim_8007077C(&mut g);
    for s in fi.x8B0.iter() {
        assert_eq!((s.x10, s.x11), (-1, -1));
    }
    m::ftAnim_80070FB4(&mut g, 2, 7);
    assert_eq!(fi.x8B0[2].x10, 7);
    unsafe { m::ftAnim_80070F28(&mut g) }; // every x11 is -1: no ft_data read
    fi.x2223_b0();
}
