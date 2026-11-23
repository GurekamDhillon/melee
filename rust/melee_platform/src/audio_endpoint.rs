//! Audio endpoint ids (what Melee Unlocked's `audio_device` key holds: a WASAPI endpoint id such as
//! `{0.0.0.00000000}.{guid}`) to the device names our output opens by (cpal's WASAPI names are the endpoint's
//! friendly name, `PKEY_Device_FriendlyName`).

/// The friendly name of the render endpoint `id`, None when Windows does not know it (unplugged, a stale id) or off
/// Windows. COM runs on a thread of its own: the main thread's apartment is winit's (OLE) to set up.
#[cfg(windows)]
pub fn friendly_name(id: &str) -> Option<String> {
    let id = id.to_string();
    std::thread::spawn(move || unsafe { resolve(&id) }).join().ok().flatten()
}

#[cfg(not(windows))]
pub fn friendly_name(_id: &str) -> Option<String> {
    None
}

/// The default render endpoint's id (tests).
#[cfg(windows)]
pub fn default_endpoint_id() -> Option<String> {
    std::thread::spawn(|| unsafe { default_id() }).join().ok().flatten()
}

#[cfg(windows)]
unsafe fn com<R>(f: impl FnOnce() -> Option<R>) -> Option<R> {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};
    if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() {
        return None;
    }
    let r = f();
    CoUninitialize();
    r
}

#[cfg(windows)]
unsafe fn resolve(id: &str) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL, STGM_READ};
    use windows::Win32::System::Com::StructuredStorage::PropVariantToString;
    com(|| {
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let wide: Vec<u16> = id.encode_utf16().chain(Some(0)).collect();
        let dev = en.GetDevice(PCWSTR(wide.as_ptr())).ok()?;
        let store = dev.OpenPropertyStore(STGM_READ).ok()?;
        let pv = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
        let mut buf = [0u16; 512];
        PropVariantToString(&pv, &mut buf).ok()?;
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]))
    })
}

#[cfg(windows)]
unsafe fn default_id() -> Option<String> {
    use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};
    com(|| {
        let en: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let dev = en.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let p = dev.GetId().ok()?;
        let out = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        out
    })
}

/// What `audio_device` from port-settings.ini means for our output: the device name to open (None = the system
/// default) and the log line. `names` are the output devices we can open; `resolve` turns an endpoint id into a name.
pub fn choose_device(value: &str, names: &[String], resolve: &dyn Fn(&str) -> Option<String>) -> (Option<String>, String) {
    let v = value.trim();
    if v.is_empty() {
        return (None, "audio_device: empty, the system default".into());
    }
    // already one of our names (a settings.cfg style value)
    if names.iter().any(|n| n == v) {
        return (Some(v.to_string()), format!("audio_device: {v:?}"));
    }
    match resolve(v) {
        Some(n) if names.iter().any(|x| *x == n) => (Some(n.clone()), format!("audio_device: endpoint id -> {n:?}")),
        Some(n) => (None, format!("audio_device: {n:?} is not an output we can open; using the system default")),
        None => (None, "audio_device: endpoint id not found (unplugged or stale); using the system default".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["Speakers (Realtek Audio)".into(), "Headset (USB)".into()]
    }

    #[test]
    fn choosing_a_device() {
        let known = |id: &str| (id == "{0.0.0.00000000}.{aaa}").then(|| "Headset (USB)".to_string());
        assert_eq!(choose_device("{0.0.0.00000000}.{aaa}", &names(), &known).0.as_deref(), Some("Headset (USB)"));
        assert_eq!(choose_device("Speakers (Realtek Audio)", &names(), &known).0.as_deref(), Some("Speakers (Realtek Audio)"));
        let (d, why) = choose_device("{0.0.0.00000000}.{gone}", &names(), &known);
        assert!(d.is_none() && why.contains("system default"), "{why}");
        let gone = |_: &str| Some("HDMI (Monitor)".to_string());
        assert!(choose_device("{x}", &names(), &gone).0.is_none());
        assert!(choose_device("", &names(), &known).0.is_none());
    }

    #[cfg(windows)]
    #[test]
    fn the_default_endpoint_resolves_to_a_name() {
        // skipped on a machine with no audio output
        let Some(id) = default_endpoint_id() else { return };
        let n = friendly_name(&id).expect("friendly name of the default endpoint");
        assert!(!n.is_empty());
        assert_eq!(friendly_name("{0.0.0.00000000}.{00000000-0000-0000-0000-000000000000}"), None);
    }
}
