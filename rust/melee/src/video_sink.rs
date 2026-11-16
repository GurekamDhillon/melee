//! `--video-out -|PATH [--video-size WxH] [--video-realtime]` (golden and play): the byte sink and the options.
//! With `-` the process's stdout is claimed for the video before anything prints: the original stdout handle is
//! kept for the frames and stdout itself is pointed at stderr, so every `println!` (ours or a crate's) lands on
//! stderr and the stream stays pure RGBA8.

use std::io::Write;

static mut CLAIMED: Option<std::fs::File> = None;

/// When the command line asks for `--video-out -`: take stdout for the video (call first thing in main).
pub fn claim_stdout_if_asked(args: &[String]) {
    let wants = args.windows(2).any(|w| w[0] == "--video-out" && w[1] == "-");
    if !wants {
        return;
    }
    let _ = std::io::stdout().flush();
    // SAFETY: single-threaded start-up; the handles are the process's own standard handles.
    unsafe {
        *std::ptr::addr_of_mut!(CLAIMED) = take_stdout();
    }
}

#[cfg(windows)]
unsafe fn take_stdout() -> Option<std::fs::File> {
    use std::os::windows::io::FromRawHandle;
    unsafe extern "system" {
        fn GetStdHandle(n: u32) -> *mut core::ffi::c_void;
        fn SetStdHandle(n: u32, h: *mut core::ffi::c_void) -> i32;
    }
    const STD_OUTPUT: u32 = -11i32 as u32;
    const STD_ERROR: u32 = -12i32 as u32;
    unsafe {
        let out = GetStdHandle(STD_OUTPUT);
        let err = GetStdHandle(STD_ERROR);
        if out.is_null() || out as isize == -1 {
            return None;
        }
        // Rust's stdout looks the handle up on every write: from here on it is stderr's
        SetStdHandle(STD_OUTPUT, err);
        Some(std::fs::File::from_raw_handle(out))
    }
}

#[cfg(unix)]
unsafe fn take_stdout() -> Option<std::fs::File> {
    use std::os::fd::FromRawFd;
    unsafe extern "C" {
        fn dup(fd: i32) -> i32;
        fn dup2(a: i32, b: i32) -> i32;
    }
    unsafe {
        let v = dup(1);
        if v < 0 {
            return None;
        }
        dup2(2, 1);
        Some(std::fs::File::from_raw_fd(v))
    }
}

pub struct Opts {
    pub path: String,
    pub w: u32,
    pub h: u32,
    pub realtime: bool,
}

/// The video options, when `--video-out` is given. Default size 1280x720 (16:9: the engine's renderer widescreen).
pub fn opts(args: &[String]) -> Result<Option<Opts>, String> {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(path) = get("--video-out") else { return Ok(None) };
    let (w, h) = match get("--video-size") {
        Some(s) => parse_size(&s).ok_or_else(|| format!("--video-size {s}: expected WxH (even, 64..=7680)"))?,
        None => (1280, 720),
    };
    Ok(Some(Opts { path, w, h, realtime: args.iter().any(|a| a == "--video-realtime") }))
}

pub fn parse_size(s: &str) -> Option<(u32, u32)> {
    let (a, b) = s.split_once(['x', 'X'])?;
    let (w, h) = (a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?);
    // even sizes: yuv420 encoders (ffmpeg's h264) refuse odd ones
    ((64..=7680).contains(&w) && (64..=4320).contains(&h) && w % 2 == 0 && h % 2 == 0).then_some((w, h))
}

/// The content aspect for a video target: 16:9 (renderer widescreen) when the target is wider than 4:3.
pub fn content_aspect(w: u32, h: u32) -> f32 {
    if w as u64 * 3 > h as u64 * 4 { 16.0 / 9.0 } else { 4.0 / 3.0 }
}

/// Open the sink: `-` is the claimed stdout, anything else a file or a named pipe (`\\.\pipe\NAME` / a FIFO).
pub fn open(path: &str) -> Result<Box<dyn Write + Send>, String> {
    if path == "-" {
        // SAFETY: written once at start-up, before any thread exists.
        return match unsafe { (*std::ptr::addr_of_mut!(CLAIMED)).take() } {
            Some(f) => Ok(Box::new(f)),
            None => Err("--video-out -: stdout is not available".into()),
        };
    }
    let f = std::fs::OpenOptions::new().write(true).create(true).truncate(!path.starts_with(r"\\.\pipe\")).open(path).map_err(|e| format!("{path}: {e}"))?;
    Ok(Box::new(f))
}

/// The one stderr line a consumer reads before the first frame.
pub fn announce(w: u32, h: u32) {
    eprintln!("video-out: {w}x{h} rgba 60fps");
}
