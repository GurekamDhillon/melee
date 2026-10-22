"""Helpers for spec_<stem>.py files (shard 569-576).

snap(items): the TU's initialised statics (callback tables, StageData, joint tables) are zeroed by the
oracle before every case, so the initial image is snapshotted once at process start (C: a constructor;
Rust: install_hooks) and copied back before every case. The state hash then covers the tables, with
function pointers named by their position in `fnvals`, so a wrong initialiser in the Rust port shows.

  items = [dict(c="grKr_StageCallbacks", rs="grKr_StageCallbacks", ty="StageCallbacks", n=5), ...]
  S = snap(items); SPEC.update(S)   # keys: data_c, use_rs, hooks_rs, links_c, links_rs (concatenate with yours)
"""


def snap(items, ctor="snap_ctor"):
    data_c = []
    use_rs = []
    hooks_rs = []
    links_c = []
    links_rs = []
    ctor_body = []
    for it in items:
        c, rs, ty, n = it["c"], it["rs"], it["ty"], it.get("n", 1)
        key = c.replace(".", "_")
        data_c.append(f"static unsigned char SNAP_{key}[sizeof({ty}) * {n}];")
        ctor_body.append(f"    {{ unsigned i; for (i = 0; i < sizeof SNAP_{key}; i++) SNAP_{key}[i] = ((const unsigned char*) &{c})[i]; }}")
        links_c.append(f"    {{ unsigned i; for (i = 0; i < sizeof SNAP_{key}; i++) ((unsigned char*) &{c})[i] = SNAP_{key}[i]; }}")
        use_rs.append(f"static mut SNAP_{key}: [u8; core::mem::size_of::<{ty}>() * {n}] = [0; core::mem::size_of::<{ty}>() * {n}];")
        hooks_rs.append(f"    core::ptr::copy_nonoverlapping(addr_of!({rs}) as *const u8, addr_of_mut!(SNAP_{key}) as *mut u8, core::mem::size_of::<{ty}>() * {n});")
        links_rs.append(f"    core::ptr::copy_nonoverlapping(addr_of!(SNAP_{key}) as *const u8, addr_of_mut!({rs}) as *mut u8, core::mem::size_of::<{ty}>() * {n});")
    data_c.append(f"__attribute__((constructor)) static void {ctor}(void)\n{{\n" + "\n".join(ctor_body) + "\n}")
    return dict(data_c="\n".join(data_c), use_rs="\n".join(use_rs), hooks_rs="\n".join(hooks_rs),
                links_c="\n".join(links_c), links_rs="\n".join(links_rs))


def merge(spec, extra):
    """Append the snapshot snippets to the spec's own (same keys)."""
    for k, v in extra.items():
        spec[k] = (spec.get(k, "") + "\n" + v) if spec.get(k) else v
    return spec


def cb_flags(c, rs, n):
    """statics entries that hash StageCallbacks[i].flags by value (the bitfield overlay of that union
    is skipped by the generic hash, so the flags word would otherwise go unchecked)."""
    return [dict(c=f"{c}[{i}].flags", rs=f"{rs}[{i}].anon_10.flags", kind="u", fill=False) for i in range(n)]
