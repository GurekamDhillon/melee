//! Smoke test for tu/it_26B1.rs: the stage article table reset.
use melee_items::tu::it_26B1 as m;

#[test]
fn it_26b1_smoke() {
    unsafe {
        m::ext::it_804A0F60[3] = 1usize as *mut melee_types::Article;
        m::it_ResetStageArticleTable();
        assert!(m::ext::it_804A0F60.iter().all(|p| p.is_null()));
    }
}
