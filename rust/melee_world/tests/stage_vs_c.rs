//! melee_world::gr::stage against the C (cref/gen_stage_ref.sh -> tests/data/stage_ref.txt),
//! built with the shipping build's FMA contraction. Bit-exact on every output.
use melee_world::gr::stage::{stage_80224dc8, StageHooks, StageWorld, ST_MEX_EXT_MAX};
use melee_world::vec::Vec3;
use melee_types::{GrKind, StKind, StageIdPair};

const DATA: &str = include_str!("data/stage_ref.txt");

fn h(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

/// The oracle's fixed m-ex configuration (stage_ref.c).
struct MexOnly;
impl StageHooks for MexOnly {
    fn ground_801c2d24(&mut self, _: i32, _: &mut Vec3) -> bool { unreachable!() }
    fn hsd_randi(&mut self, _: i32) -> i32 { unreachable!() }
    fn gm_8016b238(&mut self) -> bool { unreachable!() }
    fn gm_8016b3a0(&mut self) -> bool { unreachable!() }
    fn ground_801c28ac(&mut self, _: StKind, _: i32, _: &mut i32) -> bool { unreachable!() }
    fn lbaudioax_80023f28(&mut self, _: i32) { unreachable!() }
    fn ground_801c5a84(&mut self, _: i32) { unreachable!() }
    fn ground_801c5aa4(&mut self, _: bool) { unreachable!() }
    fn ground_801c06b8(&mut self, _: GrKind) { unreachable!() }
    fn ground_801c0754(&mut self, _: &StageIdPair) { unreachable!() }
    fn ground_801c0800(&mut self, _: &StageIdPair) { unreachable!() }
    fn ground_on_load(&mut self, _: &StageIdPair) { unreachable!() }
    fn ground_801c0fb8(&mut self, _: &StageIdPair) { unreachable!() }
    fn ground_demo_init(&mut self, _: &StageIdPair, _: i32) { unreachable!() }
    fn mex_gr_external_count(&mut self) -> i32 { 300 }
    fn mex_gr_kind_for_ext(&mut self, e: i32) -> i32 {
        if e % 5 == 0 { -1 } else { (e * 7) % 80 }
    }
}

#[test]
fn stage_matches_c() {
    let mut w = StageWorld::default();
    let (mut g, mut d, mut m, mut s) = (0, 0, 0, 0);
    let mut mex_inited = false;
    for line in DATA.lines() {
        let mut it = line.split_whitespace();
        match it.next().unwrap() {
            "G" => {
                let (ins, outs) = line[2..].split_once('|').unwrap();
                let i: Vec<u32> = ins.split_whitespace().map(h).collect();
                let o: Vec<u32> = outs.split_whitespace().map(h).collect();
                assert_eq!(i.len(), 44);
                let f = |k: usize| f32::from_bits(i[k]);
                {
                    // StageCameraInfo: 29 consecutive f32 (GC +0x0..+0x74).
                    let ci = &mut w.stage_info.cam_info;
                    const _: () = assert!(core::mem::size_of::<melee_types::StageCameraInfo>() == 29 * 4);
                    // SAFETY: repr(C), 29 f32 fields (asserted above).
                    let arr: &mut [f32; 29] = unsafe { &mut *(ci as *mut _ as *mut [f32; 29]) };
                    for k in 0..29 {
                        arr[k] = f(k);
                    }
                    let bz = &mut w.stage_info.blast_zone;
                    bz.left = f(29);
                    bz.right = f(30);
                    bz.top = f(31);
                    bz.bottom = f(32);
                }
                w.arena.owner = i[33] as i32;
                w.arena.x = f(34);
                w.arena.y = f(35);
                for k in 0..4 {
                    w.arena.camera[k] = f(36 + k);
                    w.arena.blast[k] = f(40 + k);
                }
                let t = w.unk_set_vec3_tcam_offset();
                let fp = w.set_vec_to_fixed_cam_pos();
                let cac = w.stage_80224cac();
                let got: Vec<f32> = vec![
                    w.get_cam_bounds_left_offset(), w.get_cam_bounds_right_offset(),
                    w.get_cam_bounds_top_offset(), w.get_cam_bounds_bottom_offset(),
                    w.get_cam_pan_angle_radians(), w.get_cam_max_depth(), w.get_cam_zoom_rate(),
                    w.get_cam_info_x20(), w.get_cam_info_x24(), w.get_cam_fixed_zoom(),
                    w.get_cam_track_ratio(), w.get_cam_track_smooth(),
                    w.get_blast_zone_right_offset(), w.get_blast_zone_left_offset(),
                    w.get_blast_zone_top_offset(), w.get_blast_zone_bottom_offset(),
                    w.calc_unk_cam_y(), w.calc_unk_cam_y_bounds(), w.get_pause_cam_zpos_min(),
                    w.get_pause_cam_zpos_init(), w.get_pause_cam_zpos_max(),
                    w.get_cam_angle_radians_up(), w.get_cam_angle_radians_down(),
                    w.get_cam_angle_radians_left(), w.get_cam_angle_radians_right(),
                    w.get_cam_fixed_fov(), t.x, t.y, t.z, fp.x, fp.y, fp.z, cac.x, cac.y, cac.z,
                ];
                let got: Vec<u32> = got.iter().map(|x| x.to_bits()).collect();
                assert_eq!(got, o, "case {g}: {line}");
                g += 1;
            }
            "D" => {
                let a = h(it.next().unwrap()) as i32;
                assert_eq!(stage_80224dc8(a) as u32, h(it.next().unwrap()), "80224DC8({a})");
                d += 1;
            }
            "M" => {
                if !mex_inited {
                    w.stage_8022519c(&mut MexOnly, 0);
                    mex_inited = true;
                }
                let k = h(it.next().unwrap()) as usize;
                assert_eq!(w.stage_id_map[k], h(it.next().unwrap()), "stage_id_map[{k}]");
                m += 1;
            }
            "S" => {
                let kind = h(it.next().unwrap()) as i32;
                let r = w.stage_slot_select(kind);
                assert_eq!(r as u32, h(it.next().unwrap()), "slot select {kind}");
                assert_eq!(w.selected_stkind, h(it.next().unwrap()), "slot select {kind} stkind");
                s += 1;
            }
            x => panic!("bad tag {x}"),
        }
    }
    assert_eq!((g, d, m, s), (20000, 140, ST_MEX_EXT_MAX, 92));
}
