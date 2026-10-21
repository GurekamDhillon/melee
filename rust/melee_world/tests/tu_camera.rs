use melee_types::CmSubjectState_Auto;
use melee_world::tu::camera::*;

#[test]
fn subject_pool_alloc_free() {
    let mut w = CameraWorld::default();
    let mut pool = CmSubjectPool { free: 2 };
    let a = Camera_80029044(&mut w, &mut pool, CmSubjectState_Auto);
    assert_eq!(a, 0);
    w.subjects[0].pos.x = 5.0;
    Camera_80029020(&mut w, &mut pool);
    assert_eq!(w.subjects.len(), 2);
    assert_eq!(w.subjects[1].pos.x, 5.0); // older subject moved down the prev chain
    let s = &w.subjects[0];
    assert_eq!(s.state, 0);
    assert_eq!((s.ext.h.x, s.ext.h.y, s.ext.v.x, s.ext.v.y, s.ext.v.z), (-1.0, 1.0, 1.0, -1.0, 1.0));
    assert_eq!(s.target_ext.v.y, -1.0);
    assert_eq!(w.subjects[1].state, CmSubjectState_Auto);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut w2 = CameraWorld::default();
        Camera_80029020(&mut w2, &mut CmSubjectPool { free: 0 });
    }))
    .is_err());
    let freed = Camera_800290D4(&mut w, &mut pool, 1);
    assert_eq!(freed.pos.x, 5.0);
    assert_eq!((w.subjects.len(), pool.free), (1, 1));
    let mut s = w.subjects[0];
    s.set_was_framed(1);
    s.state_timer = 9;
    Camera_80028F5C(Some(&mut s), CmSubjectState_Auto);
    assert_eq!((s.was_framed(), s.state_timer), (0, 0));
    Camera_80028F5C(None, 0);
}
