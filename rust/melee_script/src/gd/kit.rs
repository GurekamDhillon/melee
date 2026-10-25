//! `gd.kit` (`gs_push_kit`, the `l_kit_*` entry points of gw_script.c): the menu kit's text, panels, icons and models.
//! The kit itself (gw_kit.c: manifest, fonts, quad recording) is a platform service behind [`Host::kit_data`] and
//! [`Host::kit_call`]; with none attached `gd.kit.available()` is false and every drawing call raises the C build's
//! "the kit is not available" error, which is what the LAB's `kit_ok()` fallback expects.
use super::TableExt;
use crate::args::Args;
use crate::engine::Engine;
use crate::host::KitVal;
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

pub const KIT_FUNCS: [&str; 13] =
    ["available", "text", "measure", "paragraph", "metrics", "texture", "image", "model", "icon", "panel", "button", "list", "color"];

fn to_val(v: &Value) -> KitVal {
    match v {
        Value::Boolean(b) => KitVal::Bool(*b),
        Value::Integer(n) => KitVal::Int(*n),
        Value::Number(n) => KitVal::Num(*n),
        Value::String(s) => KitVal::Str(s.to_string_lossy().to_string()),
        _ => KitVal::Nil,
    }
}
fn from_val(lua: &mlua::Lua, v: &KitVal) -> LuaResult<Value> {
    Ok(match v {
        KitVal::Nil => Value::Nil,
        KitVal::Bool(b) => Value::Boolean(*b),
        KitVal::Int(n) => Value::Integer(*n),
        KitVal::Num(n) => Value::Number(*n),
        KitVal::Str(s) => Value::String(lua.create_string(s)?),
    })
}

/// `gs_push_kit`
pub fn push_kit(e: &Engine) -> LuaResult<Table> {
    let lua = &e.lua;
    let kit = lua.create_table()?;
    for name in KIT_FUNCS {
        if name == "available" {
            super::reg(e, &kit, "available", |lua, st, _a| {
                let (ok, why) = st.borrow().host.borrow_mut().kit_available();
                Ok(MultiValue::from_iter([Value::Boolean(ok), Value::String(lua.create_string(&why)?)]))
            })?;
            continue;
        }
        super::reg(e, &kit, name, move |lua, st, a: Args| {
            let (ok, why) = st.borrow().host.borrow_mut().kit_available();
            if !ok {
                return Err(super::lua_err(format!("gd.kit: the kit is not available ({why})")));
            }
            let args: Vec<KitVal> = a.get_all().iter().map(to_val).collect();
            let host = st.borrow().host.clone();
            let r = host.borrow_mut().kit_call(name, &args).map_err(|m| super::lua_err(format!("gd.kit.{name}: {m}")))?;
            let vals: Vec<Value> = r.iter().map(|v| from_val(lua, v)).collect::<LuaResult<_>>()?;
            Ok(MultiValue::from_iter(vals))
        })?;
    }
    let d = e.st.borrow().host.borrow_mut().kit_data();
    let colors = lua.create_table()?;
    for (name, rgba) in &d.palette {
        colors.int(name, *rgba as i64)?;
    }
    for (name, which) in &d.sections {
        let t = lua.create_table()?;
        for (k, key) in ["face", "bg", "band", "face_hi"].iter().enumerate() {
            t.int(key, which[k] as i64)?;
        }
        colors.set(name.as_str(), t)?;
    }
    for (name, rgba) in &d.ports {
        colors.int(name, *rgba as i64)?;
    }
    kit.set("colors", colors)?;
    let roles = lua.create_table()?;
    for (i, r) in d.roles.iter().enumerate() {
        roles.raw_set(i as i64 + 1, r.as_str())?;
    }
    kit.set("roles", roles)?;
    let row = lua.create_table()?;
    for (k, v) in [("h", d.row[0]), ("pitch", d.row[1]), ("label_x", d.row[2]), ("label_base", d.row[3]), ("lift", d.row[4])] {
        row.num(k, v as f64)?;
    }
    kit.set("row", row)?;
    kit.num("shear", d.shear as f64)?;
    Ok(kit)
}
