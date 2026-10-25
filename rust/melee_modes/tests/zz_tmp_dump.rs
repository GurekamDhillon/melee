use melee_modes::tu::geno_game as m;
#[test] fn dump() { unsafe {
 let mut s=String::new();
 for p in (*core::ptr::addr_of!(m::geno_params)).iter() { s+=&format!("{} {} {} {:08x}\n", std::ffi::CStr::from_ptr(p.family).to_str().unwrap(), std::ffi::CStr::from_ptr(p.name).to_str().unwrap(), p.id, p.def.to_bits()); }
 for p in (*core::ptr::addr_of!(m::geno_cbs)).iter() { s+=&format!("cb {} {} {}\n", p.id, p.slot, std::ffi::CStr::from_ptr(p.name).to_str().unwrap()); }
 std::fs::write("C:/Users/Gurek/AppData/Local/Temp/rparams.txt", s).unwrap(); } }
