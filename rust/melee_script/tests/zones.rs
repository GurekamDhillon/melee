//! Zone API and enter/exit dispatch over a mock game half backed by `zones_core::SzState`. Headless; no input sent.
use melee_script::zones_core::*;
use melee_script::{Config, Engine, MemHost, ScriptGame};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Z {
    s: SzState,
    mask: i32,
    pos: [(i32, f32, f32); 12],
    draft: SzDef,
    texts: [[Vec<u8>; 11]; 1],
}
struct G(Rc<RefCell<Z>>);
impl ScriptGame for G {
    fn zone_field(&self, slot: i32, field: i32) -> i32 {
        let z = &self.0.borrow().s.z[slot as usize];
        match field {
            SZF_HANDLE => z.handle,
            SZF_OWNER => z.owner,
            SZF_LIVE => z.live,
            SZF_MODEL => z.model,
            SZF_N => z.n as i32,
            SZF_TAGS => z.ntags as i32,
            SZF_AREA => z.area.to_bits() as i32,
            SZF_DX => z.dx.to_bits() as i32,
            SZF_DY => z.dy.to_bits() as i32,
            f if (SZF_X..SZF_X + 16).contains(&f) => z.x[(f - SZF_X) as usize].to_bits() as i32,
            f if (SZF_Y..SZF_Y + 16).contains(&f) => z.y[(f - SZF_Y) as usize].to_bits() as i32,
            _ => 0,
        }
    }
    fn zone_text(&self, slot: i32, text: i32, byte: i32) -> i32 {
        let z = &self.0.borrow().s.z[slot as usize];
        let s = match text {
            SZT_NAME => &z.name,
            SZT_LABEL => &z.label,
            SZT_KIND => &z.kind,
            t => &z.tags[(t - SZT_TAG) as usize],
        };
        s.as_bytes().get(byte as usize).copied().unwrap_or(0) as i32
    }
    fn zone_draft(&mut self, field: i32, value: i32) {
        let mut b = self.0.borrow_mut();
        if field == -1 {
            b.draft = SzDef::default();
            b.texts = Default::default();
            return;
        }
        let f = f32::from_bits(value as u32);
        match field {
            SZF_N => b.draft.n = value as usize,
            SZF_MODEL => b.draft.model = value,
            SZF_TAGS => b.draft.ntags = value as usize,
            x if (SZF_X..SZF_X + 16).contains(&x) => b.draft.x[(x - SZF_X) as usize] = f,
            y if (SZF_Y..SZF_Y + 16).contains(&y) => b.draft.y[(y - SZF_Y) as usize] = f,
            _ => {}
        }
    }
    fn zone_draft_text(&mut self, text: i32, byte: i32, value: i32) {
        let mut b = self.0.borrow_mut();
        let t = &mut b.texts[0][text as usize];
        if t.len() <= byte as usize {
            t.resize(byte as usize + 1, 0);
        }
        t[byte as usize] = value as u8;
    }
    fn zone_commit(&mut self, handle: i32, owner: i32, _floor: i32) -> i32 {
        let mut b = self.0.borrow_mut();
        let mut d = b.draft.clone();
        let tx = |b: &Z, i: usize| String::from_utf8_lossy(&b.texts[0][i]).to_string();
        d.name = tx(&b, 0);
        d.label = tx(&b, 1);
        d.kind = tx(&b, 2);
        for i in 0..d.ntags {
            d.tags[i] = tx(&b, 3 + i);
        }
        d.owner = owner;
        b.s.put(&d, handle)
    }
    fn zone_remove(&mut self, handle: i32, owner: i32) -> i32 {
        self.0.borrow_mut().s.remove(handle, owner) as i32
    }
    fn zones_arm(&mut self, mask: i32) {
        self.0.borrow_mut().mask = mask;
    }
    fn zones_frame(&mut self, _terminal: i32) {
        let mut b = self.0.borrow_mut();
        let mut inp = [SzInput::default(); SZ_ENTITIES];
        for i in 0..12 {
            inp[i] = SzInput { token: b.pos[i].0, x: b.pos[i].1, y: b.pos[i].2 };
        }
        let mask = b.mask;
        b.s.step(&inp, mask, true);
    }
    fn zone_entity(&self, entity: i32, field: i32, zone: i32) -> i32 {
        let b = self.0.borrow();
        let e = &b.s.e[entity as usize];
        match field {
            0 => e.id,
            1 => e.x.to_bits() as i32,
            2 => e.y.to_bits() as i32,
            3 => (e.handles[zone as usize] != 0) as i32,
            _ => e.frames[zone as usize],
        }
    }
    fn zone_event(&self, event: i32, field: i32) -> i32 {
        let b = self.0.borrow();
        let v = &b.s.ev[event as usize];
        match field {
            SZE_WHAT => v.what,
            SZE_ENTITY => v.entity,
            SZE_PORT => v.port,
            SZE_SUB => v.sub,
            SZE_X => v.x.to_bits() as i32,
            SZE_Y => v.y.to_bits() as i32,
            SZE_ZONE => v.zone_handle,
            f => v.from[(f - SZE_FROM) as usize],
        }
    }
    fn zone_event_text(&self, event: i32, text: i32, byte: i32) -> i32 {
        let b = self.0.borrow();
        let v = &b.s.ev[event as usize];
        let s = match text {
            SZT_NAME => &v.zone_name,
            SZT_LABEL => &v.zone_label,
            _ => &v.zone_kind,
        };
        s.as_bytes().get(byte as usize).copied().unwrap_or(0) as i32
    }
    fn zone_event_count(&self) -> i32 {
        self.0.borrow().s.ev.len() as i32
    }
}

fn engine(z: Rc<RefCell<Z>>) -> Engine {
    let host = Rc::new(RefCell::new(MemHost::default()));
    let dir = std::env::temp_dir().join(format!("melee_script_zones_{}", std::process::id()));
    let mut cfg = Config::from_env(dir.clone());
    cfg.data_dir = dir.join("data");
    cfg.scripts_dir = dir.join("scripts");
    Engine::new(host, Box::new(G(z)), cfg).unwrap()
}

#[test]
fn zone_api_and_hooks() {
    let z = Rc::new(RefCell::new(Z::default()));
    let eng = engine(z.clone());
    eng.st.borrow_mut().match_active = true;
    let src = "-- @gameplay: true\n\
        log = {}\n\
        function on_zone_enter(e) log[#log+1] = 'enter:'..e.name..':'..e.port..':'..e.sub end\n\
        function on_zone_exit(e) log[#log+1] = 'exit:'..e.name..':'..#e.from end\n\
        function on_zone_some(e) log[#log+1] = 'some:'..e.port end\n\
        function on_zone_none(e) log[#log+1] = 'none:'..e.port end\n\
        h = gd.zone_add{name='room', kind='room', x0=-10, y0=-10, x1=10, y1=10, tags={'a','b'}}\n\
        h2 = gd.zone_add{name='tri', points={{0,0},{20,0},{0,20}}}\n\
        pz = gd.point_zones(1, 1)\n\
        dup, m1 = pcall(gd.zone_add, {name='room', x0=0, y0=0, x1=1, y1=1})\n\
        bad, m2 = pcall(gd.zone_set, h, {x0=0})\n\
        bad2, m3 = pcall(gd.zone_set, h, {tags={bad='value'}})\n\
        bad3, m4 = pcall(gd.zone_add, {name='z', bogus=1})\n\
        gd.zone_set(h, {label='changed'})\n\
        lab = gd.point_zones(0, 0)[1].label\n\
        none7 = pcall(gd.zones_at, 7)\n";
    let i = eng.load_text("a", "a.lua", src.as_bytes(), None, "test").unwrap();
    let env = eng.st.borrow().scripts[i].env.clone().unwrap();
    assert_eq!(env.get::<i64>("h").unwrap(), 1, "{:?}", env.get::<String>("m1"));
    assert_eq!(env.get::<i64>("h2").unwrap(), 2);
    let pz: mlua::Table = env.get("pz").unwrap();
    assert_eq!(pz.raw_len(), 2); // room (area 400) and tri
    assert_eq!(pz.get::<mlua::Table>(1).unwrap().get::<String>("kind").unwrap(), "room");
    assert!(!env.get::<bool>("dup").unwrap());
    assert!(env.get::<String>("m1").unwrap().contains("zone refused (-3)"));
    assert!(env.get::<String>("m2").unwrap().contains("rectangle requires"));
    assert!(env.get::<String>("m3").unwrap().contains("consecutive integer keys"));
    assert!(env.get::<String>("m4").unwrap().contains("unknown zone definition field"));
    assert_eq!(env.get::<String>("lab").unwrap(), "changed");
    assert!(!env.get::<bool>("none7").unwrap());

    eng.update_want_events();
    assert_eq!(z.borrow().mask, 15);
    z.borrow_mut().pos[2] = (5, 1.0, 1.0); // port 2 enters both zones
    eng.frame_pre();
    eng.frame_post();
    z.borrow_mut().pos[2] = (5, 50.0, 50.0);
    eng.frame_pre();
    eng.frame_post();
    let log: mlua::Table = env.get("log").unwrap();
    let lines: Vec<String> = log.sequence_values::<String>().map(|v| v.unwrap()).collect();
    assert_eq!(lines, ["enter:room:2:0", "enter:tri:2:0", "some:2", "exit:room:2", "exit:tri:2", "none:2"].map(String::from));
}
