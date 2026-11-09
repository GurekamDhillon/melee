//! The fighter half of gw_mexid.c's local identity table: every retail fighter, m-ex slot and native Geno define
//! whose files are here, keyed by its CharacterKind, with a 64-bit hash of what it is built from. The fighter
//! controls preferences are saved under it (gw_fighter_controls_runtime.inc's `fc_preference_key`), so a fighter
//! whose files change starts from fresh preferences. The stage entries, the peer list exchange and the scene tokens
//! are not here: this port's netplay handshake has its own identity (play_net.rs).
use std::collections::HashMap;
use std::sync::Mutex;

/// `MX_CK_MEX0`: m-ex slot s is CharacterKind 0x22 + s
const CK_MEX0: i32 = 0x22;
/// `MX_MEX_SLOTS` (GW_MEX_SLOT_COUNT)
const MEX_SLOTS: i32 = 94;
/// `MX_MAX`
const MAX: usize = 512;
/// `MX_FILE_CACHE`
const FILE_CACHE: usize = 1024;

/// mx_entry (fighters only)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub local: i32,
    pub hash: u64,
    pub name: String,
}

/// mx_vanilla_fighters: CharacterKind order 0..25, the Pl code, the partner loaded with it and a display name
const VANILLA: [(&str, Option<&str>, &str); 26] = [
    ("Ca", None, "Captain Falcon"), ("Dk", None, "Donkey Kong"), ("Fx", None, "Fox"),
    ("Gw", None, "Mr. Game & Watch"), ("Kb", None, "Kirby"), ("Kp", None, "Bowser"),
    ("Lk", None, "Link"), ("Lg", None, "Luigi"), ("Mr", None, "Mario"),
    ("Ms", None, "Marth"), ("Mt", None, "Mewtwo"), ("Ns", None, "Ness"),
    ("Pe", None, "Peach"), ("Pk", None, "Pikachu"), ("Pp", Some("Nn"), "Ice Climbers"),
    ("Pr", None, "Jigglypuff"), ("Ss", None, "Samus"), ("Ys", None, "Yoshi"),
    ("Zd", Some("Sk"), "Zelda"), ("Sk", Some("Zd"), "Sheik"), ("Fc", None, "Falco"),
    ("Cl", None, "Young Link"), ("Dr", None, "Dr. Mario"), ("Fe", None, "Roy"),
    ("Pc", None, "Pichu"), ("Gn", None, "Ganondorf"),
];

/// mx_mix
pub fn mix(mut h: u64, v: u64) -> u64 {
    h ^= v;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    h
}

/// mx_hash_bytes: little-endian 8-byte words (the C's memcpy on x86), the tail zero-padded and marked
pub fn hash_bytes(p: &[u8]) -> u64 {
    let n = p.len() as u32;
    let mut h = mix(0x6A09_E667_F3BC_C909, n as u64);
    let mut c = p.chunks_exact(8);
    for w in c.by_ref() {
        h = mix(h, u64::from_le_bytes(w.try_into().unwrap()));
    }
    let mut v = [0u8; 8];
    v[..c.remainder().len()].copy_from_slice(c.remainder());
    mix(h, u64::from_le_bytes(v) ^ 0xA5)
}

/// mx_hash_str
pub fn hash_str(h: u64, s: &str) -> u64 {
    mix(h, hash_bytes(s.as_bytes()))
}

/// Where the table reads a file's bytes (gw_DVDFileExists + gw_DVDReadFileAlloc: the disc with the mods over it);
/// None when it is absent.
pub type Reader = fn(&str) -> Option<Vec<u8>>;

fn disc_read(path: &str) -> Option<Vec<u8>> {
    if !crate::host::Host::is_booted() {
        return None;
    }
    let d = &mut crate::host::Host::get().disc;
    if d.entry_for_path(path) < 0 {
        return None;
    }
    d.read(&format!("/{path}")).ok()
}

struct State {
    table: Option<Vec<Entry>>,
    files: Vec<(String, u64)>,
    read: Reader,
}

static STATE: Mutex<State> = Mutex::new(State { table: None, files: Vec::new(), read: disc_read });

/// mx_file: a file's content hash (never 0 for a present file), 0 when absent; cached by lower-cased path
fn file(s: &mut State, path: Option<&str>) -> u64 {
    let Some(path) = path else { return 0 };
    let path = path.trim_start_matches(['/', '\\']);
    if path.is_empty() || path.len() >= 64 {
        return 0;
    }
    let key = path.to_ascii_lowercase();
    if let Some(&(_, h)) = s.files.iter().find(|f| f.0 == key) {
        return h;
    }
    let h = (s.read)(path).map_or(0, |b| hash_bytes(&b).max(1));
    if s.files.len() < FILE_CACHE {
        s.files.push((key, h));
    }
    h
}

/// mx_fighter_identity: 0 when the Pl file is absent; a Geno overlay's salt joins it
fn fighter_identity(s: &mut State, pl: &str, aj: Option<&str>, pl2: Option<&str>, aj2: Option<&str>, cap: Option<&str>) -> u64 {
    let mut h = hash_str(0, "fighter");
    let f = file(s, Some(pl));
    if f == 0 {
        return 0;
    }
    h = mix(h, f);
    h = mix(h, file(s, aj));
    h = mix(h, file(s, pl2));
    h = mix(h, file(s, aj2));
    h = mix(h, file(s, cap));
    let salt = crate::geno_host::SaltForPlFile(pl);
    if salt != 0 {
        h = mix(h, salt);
    }
    h
}

/// mx_add: a 0 hash is stored as 1
fn add(t: &mut Vec<Entry>, local: i32, hash: u64, name: &str) {
    if t.len() < MAX {
        t.push(Entry { local, hash: if hash != 0 { hash } else { 1 }, name: name.to_string() });
    }
}

fn cstr(p: *const core::ffi::c_char) -> Option<String> {
    (!p.is_null()).then(|| unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

/// mx_build's fighter part, in the C's order: the retail fighters, the m-ex slots up to the first empty one, then
/// mx_add_defines
fn build(s: &mut State) -> Vec<Entry> {
    use crate::mex_host as mx;
    let mut t = Vec::new();
    for (i, (code, partner, name)) in VANILLA.iter().enumerate() {
        let (pl, aj) = (format!("Pl{code}.dat"), format!("Pl{code}AJ.dat"));
        let (pl2, aj2) = (partner.map(|p| format!("Pl{p}.dat")), partner.map(|p| format!("Pl{p}AJ.dat")));
        let h = fighter_identity(s, &pl, Some(&aj), pl2.as_deref(), aj2.as_deref(), None);
        if h != 0 {
            add(&mut t, i as i32, h, name);
        }
    }
    for slot in 0..MEX_SLOTS {
        let k = mx::SlotInternal(slot);
        if k < 0 {
            break;
        }
        let pl = cstr(mx::FtPlFile(k)).unwrap_or_default();
        let h = fighter_identity(s, &pl, cstr(mx::FtAnimFile(k)).as_deref(), None, None, cstr(mx::KirbyCapFile(k)).as_deref());
        let name = cstr(mx::FighterName(mx::ExtForInternal(k)));
        if h != 0 {
            add(&mut t, CK_MEX0 + slot, h, name.as_deref().unwrap_or(&pl));
        }
    }
    // mx_add_defines: the content id says what the define does, its resource files what it is built on
    for d in 0..crate::geno_host::DefineCount() {
        let Some(o) = crate::geno_host::DefineOnlineInfo(d) else { continue };
        let mut h = mix(hash_str(0, "geno-define"), o.id);
        h = mix(h, o.none as u64);
        let mut ok = true;
        for f in &o.files {
            let fh = file(s, Some(f));
            if fh == 0 {
                eprintln!("mexid: define {} has no online identity: resource file {f} is missing", o.key);
                ok = false;
                break;
            }
            h = mix(h, fh);
        }
        if ok {
            add(&mut t, o.ck, h, &o.name);
        }
    }
    eprintln!("mexid: {} fighter identities", t.len());
    t
}

fn with_table<R>(f: impl FnOnce(&[Entry]) -> R) -> R {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if s.table.is_none() {
        let t = build(&mut s);
        s.table = Some(t);
    }
    f(s.table.as_deref().unwrap())
}

/// gw_MexId_FindFighter + gw_MexId_Hash: the first entry of CharacterKind `ck`
pub fn FighterHash(ck: i32) -> Option<u64> {
    with_table(|t| t.iter().find(|e| e.local == ck).map(|e| e.hash))
}

/// The whole fighter table (for `melee mexid`, which prints it as gw_mexid.c's disc-table test logs it).
pub fn Fighters() -> Vec<Entry> {
    with_table(|t| t.to_vec())
}

/// Forget the table and the file hashes (the C clears `mx_local.built` in its tests); `read` replaces the disc
/// reader, None restores it.
pub fn reset(read: Option<Reader>) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.table = None;
    s.files.clear();
    s.read = read.unwrap_or(disc_read);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_primitives_match_the_c() {
        // values from gw_mexid.c's mx_mix / mx_hash_bytes / mx_hash_str compiled as-is (x86-64, gcc)
        assert_eq!(mix(0, 0), 0);
        assert_eq!(mix(1, 2), MIX_1_2);
        assert_eq!(hash_bytes(b""), HB_EMPTY);
        assert_eq!(hash_bytes(b"PlMr.dat"), HB_8);
        assert_eq!(hash_bytes(b"0123456789abc"), HB_13);
        assert_eq!(hash_str(0, "fighter"), HS_FIGHTER);
        assert_eq!(hash_str(0, "geno-define"), HS_DEFINE);
    }

    include!("mexid_c_values.inc");

    fn fake(path: &str) -> Option<Vec<u8>> {
        match path.to_ascii_lowercase().as_str() {
            "plmr.dat" => Some(b"mario model".to_vec()),
            "plmraj.dat" => Some(b"mario anims".to_vec()),
            "plpp.dat" => Some(b"popo".to_vec()),
            "plnn.dat" => Some(b"nana".to_vec()),
            "plppaj.dat" | "plnnaj.dat" => Some(Vec::new()),
            _ => None,
        }
    }

    #[test]
    fn retail_table_from_the_files_here() {
        let _l = crate::geno_host::TEST_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        reset(Some(fake));
        let t = Fighters();
        // only the fighters whose Pl file is here; a present empty file still hashes (an absent one is 0)
        assert_eq!(t.iter().map(|e| (e.local, e.name.as_str())).collect::<Vec<_>>(), vec![(8, "Mario"), (14, "Ice Climbers")]);
        let f = |b: &[u8]| hash_bytes(b).max(1);
        let mario = [f(b"mario model"), f(b"mario anims"), 0, 0, 0].iter().fold(hash_str(0, "fighter"), |h, &v| mix(h, v));
        let ics = [f(b"popo"), f(b""), f(b"nana"), f(b""), 0].iter().fold(hash_str(0, "fighter"), |h, &v| mix(h, v));
        assert_eq!((FighterHash(8), FighterHash(14), FighterHash(2), FighterHash(0x22)), (Some(mario), Some(ics), None, None));

        // mx_add_defines: a donor define after the retail fighters, its content id, "none" flag and the donor's files;
        // the attach-free registry salts no retail fighter
        let gh = crate::geno_host::TestInstall;
        assert_eq!(gh(b"{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"mexid-fixture\",\"name\":\"Mexid Fixture\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}]}\0".as_ptr() as *const i8), 1);
        reset(Some(fake));
        let ck = crate::geno_host::DefineResolve("mexid-fixture");
        let id = crate::geno_host::DefineContentId(ck).unwrap();
        let want = [f(b"mario model"), f(b"mario anims")].iter().fold(mix(mix(hash_str(0, "geno-define"), id), 0), |h, &v| mix(h, v));
        let t = Fighters();
        assert_eq!((t.len(), t[2].local, t[2].name.as_str(), FighterHash(ck), FighterHash(8)), (3, ck, "Mexid Fixture", Some(want), Some(mario)));
        // a define whose resource file is missing has no identity
        reset(Some(|p| if p.eq_ignore_ascii_case("PlMrAJ.dat") { None } else { fake(p) }));
        // (Mario keeps one: only a fighter's Pl file is required, a missing AJ file hashes as 0)
        let mario_no_aj = [f(b"mario model"), 0, 0, 0, 0].iter().fold(hash_str(0, "fighter"), |h, &v| mix(h, v));
        assert_eq!((FighterHash(ck), FighterHash(8)), (None, Some(mario_no_aj)));
        crate::geno_host::TestRestore();
        reset(None);
    }
}
