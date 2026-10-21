#![allow(non_snake_case, dead_code)]
//! Recording DevText host shared by the db TU smoke tests.
use melee_runtime::tu::dbcpu::{DevTextHost, DevTextId, Rgba};

#[derive(Default)]
pub struct Dt {
    pub log: Vec<String>,
    pub next: u32,
    pub null_create: bool,
}
impl DevTextHost for Dt {
    fn DevText_GetGObj(&mut self) -> Option<u32> { Some(99) }
    fn DevText_Create(&mut self, id: i32, x: i32, y: i32, w: i32, h: i32, n: usize) -> DevTextId {
        self.log.push(format!("create {id} {x} {y} {w} {h} {n:#x}"));
        if self.null_create { None } else { self.next += 1; Some(self.next) }
    }
    fn DevText_Show(&mut self, _: Option<u32>, t: DevTextId) { self.log.push(format!("show {t:?}")) }
    fn DevText_HideCursor(&mut self, _: DevTextId) { self.log.push("hidecursor".into()) }
    fn DevText_SetBGColor(&mut self, _: DevTextId, c: Rgba) { self.log.push(format!("bg {c:?}")) }
    fn DevText_SetTextColor(&mut self, _: DevTextId, c: Rgba) { self.log.push(format!("fg {c:?}")) }
    fn DevText_SetScale(&mut self, _: DevTextId, x: f32, y: f32) { self.log.push(format!("scale {x} {y}")) }
    fn DevText_Erase(&mut self, _: DevTextId) { self.log.push("erase".into()) }
    fn DevText_SetCursorXY(&mut self, _: DevTextId, x: i32, y: i32) { self.log.push(format!("xy {x} {y}")) }
    fn DevText_SetCursorX(&mut self, _: DevTextId, x: i32) { self.log.push(format!("x {x}")) }
    fn DevText_Printf(&mut self, _: DevTextId, s: &str) { self.log.push(format!("p {s}")) }
    fn DevText_ShowBackground(&mut self, _: DevTextId) { self.log.push("showbg".into()) }
    fn DevText_HideBackground(&mut self, _: DevTextId) { self.log.push("hidebg".into()) }
    fn DevText_ShowText(&mut self, _: DevTextId) { self.log.push("showtext".into()) }
    fn DevText_HideText(&mut self, _: DevTextId) { self.log.push("hidetext".into()) }
}

/// Implements DevTextHost for a struct with a `dt: Dt` field.
#[macro_export]
macro_rules! forward_devtext {
    ($t:ty) => {
        impl melee_runtime::tu::dbcpu::DevTextHost for $t {
            fn DevText_GetGObj(&mut self) -> Option<u32> { self.dt.DevText_GetGObj() }
            fn DevText_Create(&mut self, a: i32, b: i32, c: i32, d: i32, e: i32, n: usize) -> melee_runtime::tu::dbcpu::DevTextId { self.dt.DevText_Create(a, b, c, d, e, n) }
            fn DevText_Show(&mut self, g: Option<u32>, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_Show(g, t) }
            fn DevText_HideCursor(&mut self, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_HideCursor(t) }
            fn DevText_SetBGColor(&mut self, t: melee_runtime::tu::dbcpu::DevTextId, c: melee_runtime::tu::dbcpu::Rgba) { self.dt.DevText_SetBGColor(t, c) }
            fn DevText_SetTextColor(&mut self, t: melee_runtime::tu::dbcpu::DevTextId, c: melee_runtime::tu::dbcpu::Rgba) { self.dt.DevText_SetTextColor(t, c) }
            fn DevText_SetScale(&mut self, t: melee_runtime::tu::dbcpu::DevTextId, x: f32, y: f32) { self.dt.DevText_SetScale(t, x, y) }
            fn DevText_Erase(&mut self, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_Erase(t) }
            fn DevText_SetCursorXY(&mut self, t: melee_runtime::tu::dbcpu::DevTextId, x: i32, y: i32) { self.dt.DevText_SetCursorXY(t, x, y) }
            fn DevText_SetCursorX(&mut self, t: melee_runtime::tu::dbcpu::DevTextId, x: i32) { self.dt.DevText_SetCursorX(t, x) }
            fn DevText_Printf(&mut self, t: melee_runtime::tu::dbcpu::DevTextId, s: &str) { self.dt.DevText_Printf(t, s) }
            fn DevText_ShowBackground(&mut self, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_ShowBackground(t) }
            fn DevText_HideBackground(&mut self, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_HideBackground(t) }
            fn DevText_ShowText(&mut self, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_ShowText(t) }
            fn DevText_HideText(&mut self, t: melee_runtime::tu::dbcpu::DevTextId) { self.dt.DevText_HideText(t) }
        }
    };
}
