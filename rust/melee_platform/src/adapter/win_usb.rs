//! WinUSB transport for adapters bound with Zadig: port of `gw_gc_open` (device-interface walk
//! for "vid_057e&pid_0337"), the WinUSB half of `gw_gc_open_locked` (endpoint discovery, pipe
//! reset, 20 ms transfer timeout, start command 0x13), `gw_gc_read_blocking`'s WinUSB branch and
//! the WinUSB rumble write. Tried before HID, as in the C.
#![allow(non_snake_case)]
use super::decode::GC_PAYLOAD_SIZE;
use super::{Device, ReadResult, GC_ADAPTER_PID, GC_ADAPTER_VID};
use crate::plog;
use std::sync::Arc;
use windows_sys::core::GUID;
use windows_sys::Win32::Devices::DeviceAndDriverInstallation::*;
use windows_sys::Win32::Devices::Usb::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;

/// {A5DCBF10-6530-11D2-901F-00C04FB951ED}: GUID_DEVINTERFACE_USB_DEVICE.
const GUID_DEVINTERFACE_USB_DEVICE: GUID = GUID {
    data1: 0xA5DCBF10,
    data2: 0x6530,
    data3: 0x11D2,
    data4: [0x90, 0x1F, 0x00, 0xC0, 0x4F, 0xB9, 0x51, 0xED],
};

pub struct UsbAdapter {
    dev: HANDLE,
    usb: WINUSB_INTERFACE_HANDLE,
    ep_in: u8,
    ep_out: u8,
}
unsafe impl Send for UsbAdapter {}
unsafe impl Sync for UsbAdapter {}

impl Drop for UsbAdapter {
    fn drop(&mut self) {
        unsafe {
            WinUsb_Free(self.usb);
            CloseHandle(self.dev);
        }
    }
}

/// Every present device-interface path of `guid` (SetupDi walk shared with the HID transport).
pub(crate) fn interface_paths(guid: &GUID) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    unsafe {
        let info = SetupDiGetClassDevsA(guid, std::ptr::null(), std::ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
        if info == INVALID_HANDLE_VALUE as isize {
            return out;
        }
        let mut ifdata: SP_DEVICE_INTERFACE_DATA = std::mem::zeroed();
        ifdata.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;
        let mut index = 0;
        while SetupDiEnumDeviceInterfaces(info, std::ptr::null(), guid, index, &mut ifdata) != 0 {
            index += 1;
            let mut needed = 0u32;
            SetupDiGetDeviceInterfaceDetailA(info, &ifdata, std::ptr::null_mut(), 0, &mut needed, std::ptr::null_mut());
            if needed == 0 {
                continue;
            }
            let mut storage = vec![0u64; (needed as usize).div_ceil(8)];
            let detail = storage.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_A;
            (*detail).cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;
            if SetupDiGetDeviceInterfaceDetailA(info, &ifdata, detail, needed, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
                continue;
            }
            let p = (*detail).DevicePath.as_ptr() as *const u8;
            let mut v = std::ffi::CStr::from_ptr(p as *const _).to_bytes_with_nul().to_vec();
            v.shrink_to_fit();
            out.push(v);
        }
        SetupDiDestroyDeviceInfoList(info);
    }
    out
}

pub fn open() -> Option<Arc<UsbAdapter>> {
    let want = format!("vid_{:04x}&pid_{:04x}", GC_ADAPTER_VID, GC_ADAPTER_PID);
    for path in interface_paths(&GUID_DEVINTERFACE_USB_DEVICE) {
        let lower = String::from_utf8_lossy(&path).to_ascii_lowercase();
        if !lower.contains(&want) {
            continue;
        }
        unsafe {
            let dev = CreateFileA(
                path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                std::ptr::null_mut(),
            );
            if dev == INVALID_HANDLE_VALUE {
                let e = GetLastError();
                plog!(
                    "gw: gc adapter: found {} but CreateFile failed (error {e}{})",
                    lower.trim_end_matches('\0'),
                    if e == ERROR_ACCESS_DENIED { ": another program holds it" } else { "" }
                );
                continue;
            }
            let mut usb: WINUSB_INTERFACE_HANDLE = std::ptr::null_mut();
            if WinUsb_Initialize(dev, &mut usb) == 0 {
                plog!("gw: gc adapter: WinUsb_Initialize failed (error {})", GetLastError());
                CloseHandle(dev);
                continue;
            }
            let mut a = UsbAdapter { dev, usb, ep_in: 0x81, ep_out: 0x02 };
            let mut ifd: USB_INTERFACE_DESCRIPTOR = std::mem::zeroed();
            if WinUsb_QueryInterfaceSettings(usb, 0, &mut ifd) != 0 {
                for i in 0..ifd.bNumEndpoints {
                    let mut pipe: WINUSB_PIPE_INFORMATION = std::mem::zeroed();
                    if WinUsb_QueryPipe(usb, 0, i, &mut pipe) != 0 && pipe.PipeType == UsbdPipeTypeInterrupt {
                        if pipe.PipeId & 0x80 != 0 {
                            a.ep_in = pipe.PipeId;
                        } else {
                            a.ep_out = pipe.PipeId;
                        }
                    }
                }
            }
            if WinUsb_ResetPipe(usb, a.ep_in) == 0 {
                plog!("gw: gc adapter: reset of IN pipe 0x{:02X} failed (error {})", a.ep_in, GetLastError());
            }
            if WinUsb_ResetPipe(usb, a.ep_out) == 0 {
                plog!("gw: gc adapter: reset of OUT pipe 0x{:02X} failed (error {})", a.ep_out, GetLastError());
            }
            let timeout: u32 = 20;
            let tp = &timeout as *const u32 as *const _;
            WinUsb_SetPipePolicy(usb, a.ep_in, PIPE_TRANSFER_TIMEOUT, 4, tp);
            WinUsb_SetPipePolicy(usb, a.ep_out, PIPE_TRANSFER_TIMEOUT, 4, tp);
            let start = [0x13u8];
            let mut written = 0u32;
            if WinUsb_WritePipe(usb, a.ep_out, start.as_ptr(), 1, &mut written, std::ptr::null()) == 0 {
                plog!("gw: gc adapter: found, but the start command failed (error {})", GetLastError());
                continue; // drop closes it
            }
            plog!("gw: gc adapter: opened over WinUSB (vid {:04X} pid {:04X}), reading raw reports", GC_ADAPTER_VID, GC_ADAPTER_PID);
            return Some(Arc::new(a));
        }
    }
    None
}

impl Device for UsbAdapter {
    fn read(&self) -> ReadResult {
        let mut buf = [0u8; 64];
        let mut got = 0u32;
        unsafe {
            if WinUsb_ReadPipe(self.usb, self.ep_in, buf.as_mut_ptr(), GC_PAYLOAD_SIZE as u32, &mut got, std::ptr::null()) != 0 {
                if got as usize == GC_PAYLOAD_SIZE && buf[0] == 0x21 {
                    let mut r = [0u8; GC_PAYLOAD_SIZE];
                    r.copy_from_slice(&buf[..GC_PAYLOAD_SIZE]);
                    return ReadResult::Report(r);
                }
                return ReadResult::Timeout;
            }
            if GetLastError() == ERROR_SEM_TIMEOUT {
                return ReadResult::Timeout;
            }
        }
        ReadResult::Lost
    }
    fn rumble(&self, cmd: &[u8; 5]) {
        let mut written = 0u32;
        unsafe {
            WinUsb_WritePipe(self.usb, self.ep_out, cmd.as_ptr(), 5, &mut written, std::ptr::null());
        }
    }
    fn cancel(&self) {
        unsafe {
            WinUsb_AbortPipe(self.usb, self.ep_in);
        }
    }
    fn name(&self) -> &str {
        "WinUSB"
    }
}
