//! Linux transport: port of `pc/platform/gc_adapter_linux.inc` (libusb) onto usbfs directly, so
//! no libusb is needed at build or run time. Same steps as the C's `gc_open`/`gc_close`: find
//! 057E:0337, pick the interface with an interrupt IN and OUT endpoint, detach the kernel driver
//! (usbhid; libusb's auto-detach), claim it, send the start command 0x13; on close send
//! `{0x11,0,0,0,0}`, release and hand the interface back to the kernel driver. Reads are bounded
//! by the same 20 ms as the Windows transports, so the reader thread notices a quit.
//!
//! Access: the node /dev/bus/usb/BBB/DDD is root-only by default. See
//! docs/rust-port/notes/linux.md for the udev rule (EACCES is logged with that hint, as in the C).
use super::decode::GC_PAYLOAD_SIZE;
use super::{Device, ReadResult, GC_ADAPTER_PID, GC_ADAPTER_VID};
use crate::plog;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

// linux/usbdevice_fs.h (64-bit layouts)
const USBDEVFS_BULK: libc::c_ulong = 0xC018_5502; // _IOWR('U', 2, struct usbdevfs_bulktransfer)
const USBDEVFS_SETINTERFACE: libc::c_ulong = 0x8008_5504; // _IOR('U', 4, struct usbdevfs_setinterface)
const USBDEVFS_CLAIMINTERFACE: libc::c_ulong = 0x8004_550F; // _IOR('U', 15, unsigned int)
const USBDEVFS_RELEASEINTERFACE: libc::c_ulong = 0x8004_5510; // _IOR('U', 16, unsigned int)
const USBDEVFS_IOCTL: libc::c_ulong = 0xC010_5512; // _IOWR('U', 18, struct usbdevfs_ioctl)
const USBDEVFS_DISCONNECT: libc::c_int = 0x5516; // _IO('U', 22)
const USBDEVFS_CONNECT: libc::c_int = 0x5517; // _IO('U', 23)

#[repr(C)]
struct BulkTransfer {
    ep: libc::c_uint,
    len: libc::c_uint,
    timeout: libc::c_uint, // ms
    data: *mut libc::c_void,
}
#[repr(C)]
struct SetInterface {
    interface: libc::c_uint,
    altsetting: libc::c_uint,
}
#[repr(C)]
struct UsbIoctl {
    ifno: libc::c_int,
    ioctl_code: libc::c_int,
    data: *mut libc::c_void,
}

/// The read bound (the Windows transports' PIPE_TRANSFER_TIMEOUT; the C's libusb loop uses 50).
const READ_TIMEOUT_MS: u32 = 20;

/// An interface setting with an interrupt IN and OUT endpoint (the C's endpoint walk).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Endpoints {
    pub iface: u8,
    pub alt: u8,
    pub ep_in: u8,
    pub ep_out: u8,
}

/// Walk the descriptors usbfs returns on read (device, then the configurations) and return the
/// first interface setting of the first configuration that has both interrupt endpoints.
pub fn find_endpoints(desc: &[u8]) -> Option<Endpoints> {
    let (mut i, mut configs) = (0usize, 0);
    let mut cur: Option<Endpoints> = None;
    let finish = |c: Option<Endpoints>| c.filter(|e| e.ep_in != 0 && e.ep_out != 0);
    while i + 2 <= desc.len() {
        let (len, kind) = (desc[i] as usize, desc[i + 1]);
        if len < 2 || i + len > desc.len() {
            break;
        }
        match kind {
            2 => {
                // configuration: only the first one (the C reads the active one; the adapter has one)
                configs += 1;
                if configs > 1 {
                    break;
                }
            }
            4 if len >= 9 => {
                if let Some(e) = finish(cur) {
                    return Some(e);
                }
                cur = Some(Endpoints { iface: desc[i + 2], alt: desc[i + 3], ep_in: 0, ep_out: 0 });
            }
            5 if len >= 7 => {
                if let Some(c) = cur.as_mut() {
                    let (addr, attr) = (desc[i + 2], desc[i + 3]);
                    if attr & 3 == 3 {
                        // LIBUSB_TRANSFER_TYPE_INTERRUPT
                        if addr & 0x80 != 0 {
                            c.ep_in = addr;
                        } else {
                            c.ep_out = addr;
                        }
                    }
                }
            }
            _ => {}
        }
        i += len;
    }
    finish(cur)
}

fn sysfs_hex(dir: &std::path::Path, name: &str) -> Option<u16> {
    u16::from_str_radix(std::fs::read_to_string(dir.join(name)).ok()?.trim(), 16).ok()
}
fn sysfs_dec(dir: &std::path::Path, name: &str) -> Option<u32> {
    std::fs::read_to_string(dir.join(name)).ok()?.trim().parse().ok()
}

/// usbfs nodes of every plugged 057E:0337.
fn adapter_nodes() -> Vec<String> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir("/sys/bus/usb/devices") else { return out };
    for e in rd.flatten() {
        let d = e.path();
        if sysfs_hex(&d, "idVendor") != Some(GC_ADAPTER_VID) || sysfs_hex(&d, "idProduct") != Some(GC_ADAPTER_PID) {
            continue;
        }
        if let (Some(bus), Some(dev)) = (sysfs_dec(&d, "busnum"), sysfs_dec(&d, "devnum")) {
            out.push(format!("/dev/bus/usb/{bus:03}/{dev:03}"));
        }
    }
    out
}

pub struct UsbfsAdapter {
    file: File,
    ep: Endpoints,
}

impl UsbfsAdapter {
    fn transfer(&self, ep: u8, buf: &mut [u8], timeout_ms: u32) -> Result<usize, i32> {
        let mut t = BulkTransfer { ep: ep as _, len: buf.len() as _, timeout: timeout_ms, data: buf.as_mut_ptr() as *mut _ };
        // USBDEVFS_BULK also drives interrupt endpoints (usb_bulk_msg picks an interrupt URB)
        let rc = unsafe { libc::ioctl(self.file.as_raw_fd(), USBDEVFS_BULK, &mut t) };
        if rc < 0 {
            Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
        } else {
            Ok(rc as usize)
        }
    }
    fn driver_ioctl(fd: i32, ifno: u8, code: libc::c_int) -> i32 {
        let mut io = UsbIoctl { ifno: ifno as _, ioctl_code: code, data: std::ptr::null_mut() };
        unsafe { libc::ioctl(fd, USBDEVFS_IOCTL, &mut io) }
    }
}

impl Drop for UsbfsAdapter {
    /// `gc_close`: rumble off, release, reattach the kernel driver (libusb's auto-detach undo).
    fn drop(&mut self) {
        let mut stop = [0x11u8, 0, 0, 0, 0];
        let _ = self.transfer(self.ep.ep_out, &mut stop, 100);
        let fd = self.file.as_raw_fd();
        let mut ifno = self.ep.iface as libc::c_uint;
        unsafe { libc::ioctl(fd, USBDEVFS_RELEASEINTERFACE, &mut ifno) };
        Self::driver_ioctl(fd, self.ep.iface, USBDEVFS_CONNECT);
    }
}

/// The C's `previous_error`: an open failure is logged once per change.
static LAST_ERROR: AtomicI32 = AtomicI32::new(0);

fn report_error(node: &str, errno: i32, what: &str) {
    if LAST_ERROR.swap(errno, Ordering::SeqCst) == errno {
        return;
    }
    let hint = match errno {
        libc::EACCES | libc::EPERM => " (install the 057e:0337 udev rule, docs/rust-port/notes/linux.md)",
        libc::EBUSY => " (another process owns the interface)",
        _ => "",
    };
    plog!("gw: gc adapter: {node}: {what} failed: {}{hint}", std::io::Error::from_raw_os_error(errno));
}

fn open_node(node: &str) -> Result<UsbfsAdapter, i32> {
    let errno = |e: std::io::Error| e.raw_os_error().unwrap_or(0);
    let mut file = OpenOptions::new().read(true).write(true).open(node).map_err(|e| {
        report_error(node, errno(e), "open");
        LAST_ERROR.load(Ordering::SeqCst)
    })?;
    let mut desc = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut desc).map_err(|e| errno(e))?;
    let Some(ep) = find_endpoints(&desc) else {
        report_error(node, libc::ENOENT, "interrupt endpoint lookup");
        return Err(libc::ENOENT);
    };
    let fd = file.as_raw_fd();
    // ENODATA: no kernel driver was bound
    UsbfsAdapter::driver_ioctl(fd, ep.iface, USBDEVFS_DISCONNECT);
    let mut ifno = ep.iface as libc::c_uint;
    if unsafe { libc::ioctl(fd, USBDEVFS_CLAIMINTERFACE, &mut ifno) } < 0 {
        let e = errno(std::io::Error::last_os_error());
        report_error(node, e, "claim");
        UsbfsAdapter::driver_ioctl(fd, ep.iface, USBDEVFS_CONNECT);
        return Err(e);
    }
    // From here Drop releases the claim and reattaches the driver.
    let a = UsbfsAdapter { file, ep };
    if ep.alt != 0 {
        let mut s = SetInterface { interface: ep.iface as _, altsetting: ep.alt as _ };
        if unsafe { libc::ioctl(fd, USBDEVFS_SETINTERFACE, &mut s) } < 0 {
            let e = errno(std::io::Error::last_os_error());
            report_error(node, e, "set alt setting");
            return Err(e);
        }
    }
    let mut start = [0x13u8];
    match a.transfer(ep.ep_out, &mut start, 100) {
        Ok(1) => {}
        Ok(_) => return Err(libc::EIO),
        Err(e) => {
            report_error(node, e, "start command");
            return Err(e);
        }
    }
    LAST_ERROR.store(0, Ordering::SeqCst);
    plog!(
        "gw: gc adapter: opened over usbfs ({node}, interface {}, IN 0x{:02X}, OUT 0x{:02X}), reading raw reports",
        ep.iface,
        ep.ep_in,
        ep.ep_out
    );
    Ok(a)
}

pub fn open() -> Option<Arc<UsbfsAdapter>> {
    adapter_nodes().iter().find_map(|n| open_node(n).ok()).map(Arc::new)
}

impl Device for UsbfsAdapter {
    fn read(&self) -> ReadResult {
        let mut buf = [0u8; GC_PAYLOAD_SIZE];
        match self.transfer(self.ep.ep_in, &mut buf, READ_TIMEOUT_MS) {
            // the C's `size==37 && report[0]==0x21`
            Ok(GC_PAYLOAD_SIZE) if buf[0] == 0x21 => ReadResult::Report(buf),
            Ok(_) => ReadResult::Timeout,
            Err(libc::ETIMEDOUT) | Err(libc::EINTR) => ReadResult::Timeout,
            Err(_) => ReadResult::Lost,
        }
    }
    fn rumble(&self, cmd: &[u8; 5]) {
        let mut c = *cmd;
        let _ = self.transfer(self.ep.ep_out, &mut c, 100);
    }
    fn name(&self) -> &str {
        "usbfs"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The WUP-028's descriptors as usbfs returns them (device, configuration, HID interface 0
    /// with its HID class descriptor, interrupt IN 0x81 and OUT 0x02).
    const WUP028: &[u8] = &[
        18, 1, 0x00, 0x02, 0, 0, 0, 64, 0x7E, 0x05, 0x37, 0x03, 0x00, 0x01, 1, 2, 0, 1, // device
        9, 2, 41, 0, 1, 1, 0, 0xE0, 250, // configuration
        9, 4, 0, 0, 2, 3, 0, 0, 0, // interface 0, HID
        9, 0x21, 0x10, 0x01, 0, 1, 0x22, 214, 0, // HID class descriptor
        7, 5, 0x81, 3, 37, 0, 1, // interrupt IN
        7, 5, 0x02, 3, 5, 0, 1, // interrupt OUT
    ];

    #[test]
    fn finds_the_adapter_endpoints() {
        assert_eq!(find_endpoints(WUP028), Some(Endpoints { iface: 0, alt: 0, ep_in: 0x81, ep_out: 0x02 }));
    }

    #[test]
    fn needs_both_interrupt_endpoints() {
        let mut only_in = WUP028[..WUP028.len() - 7].to_vec();
        only_in[18 + 2] = 34; // wTotalLength, cosmetic
        assert_eq!(find_endpoints(&only_in), None);
        let mut bulk = WUP028.to_vec();
        let n = bulk.len();
        bulk[n - 4] = 2; // OUT endpoint as bulk
        assert_eq!(find_endpoints(&bulk), None);
    }

    #[test]
    fn truncated_descriptors_do_not_panic() {
        for n in 0..WUP028.len() {
            let _ = find_endpoints(&WUP028[..n]);
        }
        assert_eq!(find_endpoints(&[0, 0, 0]), None);
    }
}
