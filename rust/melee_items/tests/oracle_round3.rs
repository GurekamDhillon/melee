//! Round 3 (night 3): src/itlib3.rs against tests/c_oracle/round3.c (contraction on + gwtool
//! lowering, as the shipping build). Same harness as round 2 (the infrastructure below is a
//! copy of oracle_round2.rs's): both sides fill the round-2 Item field list from the same LCG,
//! draw the function's own inputs, run it, and hash every filled field + `ret` + the output
//! Vec3; `ret` also folds in the fields the function touches outside that list. 48 cases per
//! table, 32 tables for 30 functions.
#![allow(non_upper_case_globals, dead_code)]

#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;
#[path = "round2/fields.rs"]
mod fields;

use expected as e;
use melee_items::itlib3::*;
use melee_items::layout::*;
use std::ptr::null_mut;

pub const KINDS: [i32; 29] = [It_Kind_Heart, It_Kind_Tomato, It_Kind_Foods, It_Kind_Coin,
    It_Kind_Lucky_Egg, It_Kind_WhispyHealApple, It_Kind_L_Gun, It_Kind_S_Scope,
    It_Kind_StarRod, It_Kind_LipStick, It_Kind_F_Flower, It_Kind_Harisen, It_Kind_Freeze,
    It_Kind_MSBomb, It_Kind_Flipper, It_Kind_BombHei, It_Kind_Link_Bomb, It_Kind_Kuriboh,
    It_Kind_Octarock_Stone - 1, It_Kind_Octarock_Stone, It_Kind_Old_Kuri - 1,
    It_Kind_Old_Kuri, It_Kind_Arwing_Laser - 1, It_Kind_Arwing_Laser, It_Kind_Kyasarin_Egg,
    It_Kind_WhispyApple, 0, 100, 230];
const POOL: [u32; 10] = [0x00000000, 0x80000000, 0x3F800000, 0xBF800000, 0x3F000000,
    0xBF000000, 0x40000000, 0x00000001, 0x7F7FFFFF, 0x42C80000];

const DMG: [u32; 4] = [0x00000000, 0x80000000, 0x3F800000, 0x3F000000];

pub fn hmix(h: u32, v: u32) -> u32 {
    (h ^ v).wrapping_mul(0x01000193)
}

/// round2.c's LCG, `RF()` and `PTR()`/`PIDX()`.
pub struct Gen {
    s: u32,
    ga: *mut HSD_GObj,
    gb: *mut HSD_GObj,
}
impl Gen {
    pub fn r(&mut self) -> u32 {
        self.s = self.s.wrapping_mul(1103515245).wrapping_add(12345);
        self.s
    }
    pub fn rf(&mut self) -> f32 {
        let mut b = self.r();
        let sel = self.r() % 8;
        if sel == 0 {
            return f32::from_bits(POOL[(b % 10) as usize]);
        }
        if sel < 4 {
            return ((b >> 8) as i32 - 0x7FFFFF) as f32 / 65536.0;
        }
        if (b >> 23) & 0xFF == 0xFF {
            b &= !0x40000000;
        }
        f32::from_bits(b)
    }
    /// round2.c `RM()`: |x| < 128 with 16 fraction bits.
    pub fn rm(&mut self) -> f32 {
        ((self.r() >> 8) as i32 - 0x7FFFFF) as f32 / 65536.0
    }
    pub fn ptr(&mut self) -> *mut HSD_GObj {
        let r = self.r();
        match r % 3 {
            0 => null_mut(),
            1 => self.ga,
            _ => self.gb,
        }
    }
    pub fn pidx<T>(&self, p: *mut T) -> u32 {
        let p = p as *mut HSD_GObj;
        if p.is_null() {
            0
        } else if p == self.ga {
            1
        } else if p == self.gb {
            2
        } else {
            3
        }
    }
}

pub struct Ext {
    attr: ItemAttr,
    art: Article,
    unk: itUnkAttributes_legacy,
    common: ItemCommonData,
}

type Case = fn(&mut Item, &mut Gen, &mut Vec, &mut Ext) -> u32;

fn run(id: u32, name: &str, want: &[u32; 48], case: Case) {
    let mut ga = Box::new(HSD_GObj::zeroed());
    let mut gb = Box::new(HSD_GObj::zeroed());
    let mut ip = Box::new(Item::zeroed());
    let mut ext = Box::new(Ext {
        attr: ItemAttr::zeroed(),
        art: Article::zeroed(),
        unk: itUnkAttributes_legacy::zeroed(),
        common: ItemCommonData::zeroed(),
    });
    for c in 0..48u32 {
        let mut g = Gen {
            s: 0x2545F491 ^ id.wrapping_mul(0x9E3779B9) ^ c.wrapping_mul(0x85EBCA6B),
            ga: &mut *ga,
            gb: &mut *gb,
        };
        fields::fill(&mut ip, &mut g);
        // round2.c fill_ext()
        let x = &mut *ext;
        x.attr = ItemAttr::zeroed();
        x.art = Article::zeroed();
        x.unk = itUnkAttributes_legacy::zeroed();
        x.common = ItemCommonData::zeroed();
        x.attr.set_x0_is_heavy(g.r() as u8);
        x.attr.set_x0_78(g.r() as u8);
        x.attr.set_x0_hold_kind(g.r() as u8);
        x.attr.x1C_damage_mul = g.rf();
        x.unk.x0_float = g.rf();
        x.unk.x4_float = g.rf();
        x.art.x4_specialAttributes = &mut x.unk as *mut itUnkAttributes_legacy as *mut _;
        x.art.x8_hurtbones = if g.r() & 1 != 0 { g.ga as *mut _ } else { null_mut() };
        x.common.xB8 = g.rf();
        x.common.xBC = g.rf();
        x.common.xDC = g.r() as i32;
        ip.xCC_item_attr = &mut x.attr;
        ip.xC4_article_data = &mut x.art;
        let mut ov = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        let ret = case(&mut ip, &mut g, &mut ov, x);
        let h = hmix(
            hmix(hmix(hmix(fields::snap(&ip, &g), ret), ov.x.to_bits()), ov.y.to_bits()),
            ov.z.to_bits(),
        );
        assert_eq!(h, want[c as usize], "{name} case {c}");
    }
}

fn hurt_in(g: &mut Gen) -> HurtCapsule {
    let mut hc = HurtCapsule::zeroed();
    hc.a_offset = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    hc.b_offset = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    hc.scale = g.rf();
    hc
}
fn hurt_h(hc: &HurtCapsule) -> u32 {
    [hc.a_offset.x, hc.a_offset.y, hc.a_offset.z, hc.b_offset.x, hc.b_offset.y, hc.b_offset.z,
        hc.scale]
        .iter()
        .fold(0, |h, v| hmix(h, v.to_bits()))
}


// ---- round3.c helpers -------------------------------------------------------------------

fn kind_pick(g: &mut Gen, own: i32) -> i32 {
    if g.r() & 1 != 0 {
        own
    } else {
        KINDS[(g.r() % KINDS.len() as u32) as usize]
    }
}

/// round3.c `r3_list`: n (0..3) GObjs, the case Item first, then two side Items.
struct List {
    gl: Box<[HSD_GObj; 3]>,
    li: Box<[Item; 2]>,
    head: *mut HSD_GObj,
}
fn r3_list(ip: *mut Item, g: &mut Gen) -> List {
    let n = g.r() % 4;
    let mut l = List {
        gl: Box::new([HSD_GObj::zeroed(), HSD_GObj::zeroed(), HSD_GObj::zeroed()]),
        li: Box::new([Item::zeroed(), Item::zeroed()]),
        head: null_mut(),
    };
    let own = unsafe { (*ip).kind };
    for k in 0..2 {
        l.li[k].kind = kind_pick(g, own);
        l.li[k].xDC8_word.flags.bf_0 = g.r().to_be_bytes();
    }
    let base = l.gl.as_mut_ptr();
    unsafe {
        (*base).user_data = ip as *mut _;
        (*base.add(1)).user_data = &mut l.li[0] as *mut Item as *mut _;
        (*base.add(2)).user_data = &mut l.li[1] as *mut Item as *mut _;
        let mut k = 0;
        while k + 1 < n as usize {
            (*base.add(k)).next = base.add(k + 1);
            k += 1;
        }
    }
    l.head = if n != 0 { base } else { null_mut() };
    l
}
fn r3_list_h(l: &List) -> u32 {
    let h = hmix(0, u32::from_be_bytes(l.li[0].xDC8_word.flags.bf_0));
    let h = hmix(h, u32::from_be_bytes(l.li[1].xDC8_word.flags.bf_0));
    hmix(hmix(h, l.li[0].kind as u32), l.li[1].kind as u32)
}
fn r3_gidx(l: &List, p: *mut HSD_GObj) -> u32 {
    if p.is_null() {
        0
    } else {
        (unsafe { p.offset_from(l.gl.as_ptr()) } + 1) as u32
    }
}
fn r3_pk_h(v: &itPokemonSpawn_ItemVars) -> u32 {
    let mut h = hmix(hmix(hmix(0, v.x0 as u32), v.x4.x.to_bits()), v.x4.y.to_bits());
    for f in [v.x4.z, v.x10.x, v.x10.y, v.x10.z, v.x1C, v.x20, v.x24, v.x28] {
        h = hmix(h, f.to_bits());
    }
    for f in v.x2C {
        h = hmix(h, f.to_bits());
    }
    for f in [v.x44, v.x48, v.x4C, v.x50] {
        h = hmix(h, f.to_bits());
    }
    hmix(hmix(hmix(h, v.x54 as u32), v.x58 as u32), v.x5C as u32)
}
fn r3_pk_fill(v: &mut itPokemonSpawn_ItemVars, g: &mut Gen) {
    v.x0 = g.r() as i32;
    v.x4 = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    v.x10 = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    v.x1C = g.rf();
    v.x20 = g.rf();
    v.x24 = g.rf();
    v.x28 = g.rf();
    for k in 0..6 {
        v.x2C[k] = g.rf();
    }
    v.x44 = g.rf();
    v.x48 = g.rf();
    v.x4C = g.rf();
    v.x50 = g.rf();
    v.x54 = g.r() as i32;
    v.x58 = g.r() as i32;
    v.x5C = g.r() as i32;
}
fn r3_pk_attr(g: &mut Gen) -> Box<itPokemonSpawn_DatAttrs> {
    let mut a = Box::new(itPokemonSpawn_DatAttrs::zeroed());
    a.xC = g.rf();
    a.x10 = g.rf();
    a.x14 = g.rf();
    a.x18 = g.rf();
    for k in 0..6 {
        a.x1C[k] = g.rf();
    }
    a.x34 = g.rf();
    a.x38 = (g.r() % 7) as i32;
    for k in 0..30 {
        a.pokemon_spawn_weights[k] =
            if g.r() & 3 != 0 { (g.r() % 1000) as i32 } else { g.r() as i32 };
    }
    a
}
fn r3_pkind(g: &mut Gen) -> i32 {
    if g.r() & 3 != 0 {
        It_PKind_Start + (g.r() % 30) as i32
    } else {
        (g.r() % 300) as i32
    }
}
fn r3_fill_extra(ip: &mut Item, g: &mut Gen) {
    ip.xCC8_knockback = g.rf();
    ip.xCCC_incDamageDirection = g.rf();
    ip.xCB8_outDamageDirection = g.rf();
    ip.xC68 = g.rf();
    ip.xCD0 = g.rf();
    ip.xCC0 = g.rf();
    ip.xDCF_flag.byte = g.r() as u8;
    ip.xCC4 = g.r() as i32;
    ip.xCF0_itemGObj = g.ptr();
    ip.xCFC = g.ptr();
    ip.xCB4 = g.r() as i32;
    ip.xC38 = g.r() as i32;
    ip.xCE0 = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    ip.xCD4 = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
    ip.xC6C = g.rf();
    ip.xCA8 = g.r() as i32;
}
fn r3_hdr_extra(ip: &Item, g: &Gen) -> u32 {
    let mut h = 0;
    for f in [ip.xCC8_knockback, ip.xCCC_incDamageDirection, ip.xCB8_outDamageDirection, ip.xC68,
        ip.xCD0, ip.xCC0]
    {
        h = hmix(h, f.to_bits());
    }
    h = hmix(h, unsafe { ip.xDCF_flag.byte } as u32);
    h = hmix(hmix(hmix(hmix(h, ip.xCC4 as u32), g.pidx(ip.xCF0_itemGObj)), g.pidx(ip.xCFC)),
        ip.xCB4 as u32);
    h = hmix(h, ip.xC38 as u32);
    for f in [ip.xCE0.x, ip.xCE0.y, ip.xCE0.z, ip.xCD4.x, ip.xCD4.y, ip.xCD4.z] {
        h = hmix(h, f.to_bits());
    }
    hmix(hmix(h, ip.xC6C.to_bits()), ip.xCA8 as u32)
}
unsafe extern "C" fn r3_pick(_: *mut HSD_GObj) {}

macro_rules! cases {
    ($($id:literal $want:ident |$ip:ident, $g:ident, $ov:ident, $x:ident| $body:expr;)*) => {
        #[test]
        fn itlib3_matches_c() {
            let mut n = 0;
            $(
                {
                    #[allow(unused_variables, unused_unsafe, unused_mut)]
                    fn case($ip: &mut Item, $g: &mut Gen, $ov: &mut Vec, $x: &mut Ext) -> u32 {
                        unsafe { $body }
                    }
                    run($id, stringify!($want), &e::$want, case);
                    n += 1;
                }
            )*
            assert_eq!(n, 32);
        }
    };
}

cases! {
    201 R3_80272818 |ip, g, ov, x| {
        let mut a = Box::new(it_2728_DatAttrs::zeroed());
        a.fighter = g.ptr() as *mut Fighter;
        x.art.x4_specialAttributes = &mut *a as *mut it_2728_DatAttrs as *mut _;
        g.pidx(it_80272818(ip))
    };
    202 R3_80272828 |ip, g, ov, x| {
        let ipp = ip as *mut Item;
        let l = r3_list(ipp, g);
        let k = kind_pick(g, (*ipp).kind);
        hmix(r3_gidx(&l, it_80272828(l.head, k)), r3_list_h(&l))
    };
    203 R3_80272860 |ip, g, ov, x| { let a1 = g.rf(); let a2 = g.rf(); it_80272860(ip, a1, a2); 0 };
    204 R3_80272D1C |ip, g, ov, x| {
        let mut gg = Box::new(HSD_GObj::zeroed());
        let use_g = g.r() & 3 != 0;
        gg.classifier = if g.r() & 1 != 0 { HSD_GOBJ_CLASS_ITEM as u16 } else { g.r() as u16 };
        let p: *const HSD_GObj = if use_g { &*gg } else { std::ptr::null() };
        it_80272d1c(p) as u32
    };
    205 R3_BOUNCEVICTIM |ip, g, ov, x| {
        x.common.x58_float = g.rm();
        x.common.x5C_float = g.rm();
        x.common.x60_float = g.rm();
        if g.r() & 1 != 0 {
            ip.x40_vel.y = g.rm();
        }
        it_coll_bounce_off_victim(ip, &x.common);
        0
    };
    206 R3_80273500 |ip, g, ov, x| {
        let v = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        it_80273500(ip, &v);
        0
    };
    207 R3_80274658 |ip, g, ov, x| {
        ip.spin_spd = if g.r() & 3 != 0 { g.rf() } else { 0.0 };
        ip.xD3C_spinSpeed = g.rf();
        let a = g.rf();
        let r = it_80274658(ip, a) as u32;
        hmix(hmix(r, ip.xD3C_spinSpeed.to_bits()), ip.spin_spd.to_bits())
    };
    208 R3_80274D04 |ip, g, ov, x| {
        let mut o = lb_80014638_arg1_t::zeroed();
        o.unk_x = g.rf();
        o.unk_y = g.rf();
        it_80274d04(ip, &mut o);
        hmix(o.unk_x.to_bits(), o.unk_y.to_bits())
    };
    209 R3_80275174 |ip, g, ov, x| {
        x.common.x4C_float = g.rf();
        let lt = g.rf();
        it_80275174(ip, lt, &x.common);
        0
    };
    210 R3_80275328 |ip, g, ov, x| {
        let b = g.r() as u8;
        let mut lk = Box::new(ItemLink::zeroed());
        lk.prev = g.ptr() as *mut ItemLink;
        lk.next = g.ptr() as *mut ItemLink;
        lk.vel = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        lk.pos = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        lk.x20 = Vec { x: g.rf(), y: g.rf(), z: g.rf() };
        lk.bf_2c[0] = b;
        lk.x1CC = g.r() as i32;
        lk.gobj = g.ptr();
        lk.jobj = g.ptr() as *mut HSD_JObj;
        let own = g.ptr();
        it_80275328(own, &mut lk);
        let mut r = hmix(hmix(g.pidx(lk.prev), g.pidx(lk.next)), lk.vel.x.to_bits());
        for f in [lk.vel.y, lk.vel.z, lk.pos.x, lk.pos.y, lk.pos.z, lk.x20.x, lk.x20.y, lk.x20.z] {
            r = hmix(r, f.to_bits());
        }
        r = hmix(r, lk.bf_2c[0] as u32);
        hmix(hmix(hmix(r, lk.x1CC as u32), g.pidx(lk.gobj)), g.pidx(lk.jobj))
    };
    211 R3_80275390 |ip, g, ov, x| { it_80275390(ip); 0 };
    212 R3_8026B3C0 |ip, g, ov, x| {
        let ipp = ip as *mut Item;
        let l = r3_list(ipp, g);
        let k = kind_pick(g, (*ipp).kind);
        hmix(it_8026b3c0(l.head, k) as u32, r3_list_h(&l))
    };
    213 R3_8026C3FC |ip, g, ov, x| {
        let l = r3_list(ip as *mut Item, g);
        it_8026c3fc(l.head);
        r3_list_h(&l)
    };
    214 R3_8026C42C |ip, g, ov, x| {
        let l = r3_list(ip as *mut Item, g);
        it_8026c42c(l.head);
        r3_list_h(&l)
    };
    215 R3_8027A13C |ip, g, ov, x| {
        r3_pk_fill(&mut ip.xDD4_itemVar.pokemon_spawn, g);
        it_8027a13c(ip);
        r3_pk_h(&ip.xDD4_itemVar.pokemon_spawn)
    };
    216 R3_8027A344 |ip, g, ov, x| {
        r3_pk_fill(&mut ip.xDD4_itemVar.pokemon_spawn, g);
        it_8027a344(ip);
        r3_pk_h(&ip.xDD4_itemVar.pokemon_spawn)
    };
    217 R3_8027A364 |ip, g, ov, x| {
        let mut pk = r3_pk_attr(g);
        x.art.x4_specialAttributes = &mut *pk as *mut itPokemonSpawn_DatAttrs as *mut _;
        let mut sel = PokemonSelectionState::zeroed();
        sel.last_kind = r3_pkind(g);
        sel.previous_kind = r3_pkind(g);
        it_8027a364(ip, &sel) as u32
    };
    218 R3_8027AAA0 |ip, g, ov, x| {
        let mut pk = r3_pk_attr(g);
        let mut art1 = Box::new(Article::zeroed());
        let mut it1 = Box::new(Item::zeroed());
        art1.x4_specialAttributes = &mut *pk as *mut itPokemonSpawn_DatAttrs as *mut _;
        it1.xC4_article_data = &mut *art1;
        x.attr.x60_scale = g.rf();
        r3_pk_fill(&mut ip.xDD4_itemVar.pokemon_spawn, g);
        let a2 = if g.r() & 1 != 0 { It_PKind_Marumine } else { g.r() as i32 };
        it_8027aaa0(&it1, ip, a2);
        r3_pk_h(&ip.xDD4_itemVar.pokemon_spawn)
    };
    219 R3_802795EC |ip, g, ov, x| {
        let mut cb = Box::new([CmdUnion { set_hitbox_scale: set_hitbox_scale::zeroed() }; 2]);
        let mut ci = CommandInfo::zeroed();
        cb[0].set_hitbox_scale.set_opcode(g.r());
        cb[0].set_hitbox_scale.set_idx(g.r() % 4);
        cb[0].set_hitbox_scale.set_value(g.r());
        let base = cb.as_mut_ptr();
        ci.anon_8.u = base;
        it_802795ec(ip, &mut ci);
        hmix(ci.anon_8.u.offset_from(base) as u32, ip.x3C.to_bits())
    };
    220 R3_802796FC |ip, g, ov, x| {
        let mut cb = Box::new([CmdUnion { set_throw_flags: set_throw_flags::zeroed() }; 2]);
        let mut ci = CommandInfo::zeroed();
        cb[0].set_throw_flags.set_opcode(g.r());
        cb[0].set_throw_flags.set_hit_idx(g.r());
        let base = cb.as_mut_ptr();
        ci.anon_8.u = base;
        it_802796fc(ip, &mut ci);
        hmix(ci.anon_8.u.offset_from(base) as u32, ip.xDAC_itcmd_var0)
    };
    221 R3_80270CD8 |ip, g, ov, x| {
        let mut h = HitCapsule::zeroed();
        h.x24 = g.r() % 200;
        h.x28 = if g.r() & 1 != 0 { 0 } else { g.r() % 200 };
        h.x2C = g.r() % 100;
        h.damage = g.rm();
        for k in 7..13 {
            x.common.x80_float[k] = g.rm();
        }
        if g.r() & 1 != 0 {
            x.common.x80_float[7] = 1e30;
        }
        ip.xC9C = (g.r() % 300) as i32;
        ip.xCA0 = (g.r() % 50) as i32;
        x.attr.x1C_damage_mul = g.rm();
        it_80270cd8(ip, &h, &x.common).to_bits()
    };
    222 R3_80268E40 |ip, g, ov, x| {
        let mut d = ItemStateDesc::zeroed();
        d.xC_script = g.ptr() as *mut _;
        ip.x524_cmd.loop_count = g.r();
        ip.x524_cmd.timer = g.rf();
        item_80268e40(ip, &d);
        hmix(hmix(g.pidx(ip.x524_cmd.anon_8.u), ip.x524_cmd.loop_count), ip.x524_cmd.timer.to_bits())
    };
    223 R3_80269CC4 |ip, g, ov, x| {
        r3_fill_extra(ip, g);
        item_80269cc4(ip);
        r3_hdr_extra(ip, g)
    };
    224 R3_8026AE60 |ip, g, ov, x| {
        let mut ctr = if g.r() % 3 == 0 { 0xFFFF_FFFF } else { g.r() };
        let r = item_8026ae60(&mut ctr);
        hmix(r, ctr)
    };
    225 R3_ISGRABBABLE |ip, g, ov, x| {
        let mut lt = Box::new(ItemLogicTable::zeroed());
        lt.picked_up = if g.r() & 1 != 0 { Some(r3_pick) } else { None };
        ip.xB8_itemLogicTable = &mut *lt;
        item_is_grabbable(ip) as u32
    };
    226 R3_802756E0 |ip, g, ov, x| { it_802756e0(ip); 0 };
    227 R3_80276CEC |ip, g, ov, x| {
        ip.x378_itemColl.env_flags = if g.r() % 4 == 0 { 0 } else { (g.r() & g.r()) as i32 };
        ip.xC2C = g.r() as i32;
        it_80276cec(ip);
        ip.xC2C as u32
    };
    228 R3_8026CA4C |ip, g, ov, x| {
        let mut w = [0i32; 64];
        for k in 0..64 {
            w[k] = if g.r() & 7 != 0 { (g.r() % 100) as i32 } else { (g.r() % 100000) as i32 };
        }
        let mut m = (g.r() as u64) << 32;
        m |= g.r() as u64;
        let st = (g.r() % 40) as i32;
        let a4 = if g.r() & 1 != 0 { g.rm() } else { (g.r() % 1000) as f32 / 997.0 };
        let mut t = ItemPickTable::zeroed();
        it_8026ca4c(&mut t, &w, m, st, a4);
        t.x8 as u32
    };
    231 R3_80270CD8_NOX28 |ip, g, ov, x| {
        let mut h = HitCapsule::zeroed();
        h.x24 = g.r() % 200;
        h.x2C = g.r() % 100;
        h.damage = g.rm();
        for k in 7..13 {
            x.common.x80_float[k] = g.rm();
        }
        x.common.x80_float[9] = g.rm() / 4096.0;
        x.common.x80_float[7] = 1e30;
        ip.xC9C = (g.r() % 300) as i32;
        ip.xCA0 = (g.r() % 50) as i32;
        x.attr.x1C_damage_mul = g.rm();
        it_80270cd8(ip, &h, &x.common).to_bits()
    };
    232 R3_8026CA4C_EDGE |ip, g, ov, x| {
        let wv = (g.r() % 200) as i32 + 1;
        let n = (g.r() % 50) as i32 + 1;
        let w = [wv; 64];
        let st = (g.r() % 35) as i32;
        let mut a4 = (n as f32 - 0.99f32) / wv as f32;
        if g.r() & 1 != 0 {
            a4 = f32::from_bits(a4.to_bits() + 1);
        } else if g.r() & 1 != 0 {
            a4 = f32::from_bits(a4.to_bits() - 1);
        }
        let mut t = ItemPickTable::zeroed();
        it_8026ca4c(&mut t, &w, !0u64, st, a4);
        t.x8 as u32
    };
    229 R3_8027B730 |ip, g, ov, x| {
        it_8027b730(ip);
        let z = &ip.xDD4_itemVar.zako;
        let r = hmix(hmix(hmix(hmix(g.pidx(z.jobj), z.idx as u32), z.x8.x.to_bits()), z.x8.y.to_bits()),
            z.x8.z.to_bits());
        hmix(hmix(hmix(r, z.x14.x as u32), z.x14.y as u32), z.x14.z as u32)
    };
    230 R3_8027CBFC |ip, g, ov, x| {
        let mut d = it_804D6D40_t::zeroed();
        d.xC = g.rf();
        d.x10 = g.rf();
        ip.xCC4 = (g.r() % 18) as i32;
        it_8027cbfc(ip, &d).to_bits()
    };
}
