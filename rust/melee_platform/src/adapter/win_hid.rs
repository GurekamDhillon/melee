//! Windows HID transport for the adapter: port of `gw_gc_open_hid`, the HID start command in
//! `gw_gc_open_locked`, the HID branch of `gw_gc_read_blocking`, and the HID rumble write.
#![allow(non_snake_case)]
use super::decode::{accept_report, GC_PAYLOAD_SIZE};
use super::{Device, ReadResult, GC_ADAPTER_PID, GC_ADAPTER_VID};
use crate::plog;
use std::sync::Arc;
use windows_sys::core::GUID;
use windows_sys::Win32::Devices::DeviceAndDriverInstallation::*;
use windows_sys::Win32::Devices::HumanInterfaceDevice::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::Threading::*;
use windows_sys::Win32::System::IO::*;

pub struct HidAdapter {
    h: HANDLE,
    read_event: HANDLE,
    in_len: u32,
    out_len: u32,
}

// The HANDLEs are kernel handles, usable from any thread; reads use their own event.
unsafe impl Send for HidAdapter {}
unsafe impl Sync for HidAdapter {}

impl Drop for HidAdapter {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.h);
            CloseHandle(self.read_event);
        }
    }
}

/// Find and open 057E:0337 through the HID class driver, then send the start command (0x13).
pub fn open() -> Option<Arc<HidAdapter>> {
    unsafe {
        let mut hid_guid: GUID = std::mem::zeroed();
        HidD_GetHidGuid(&mut hid_guid);
        let info = SetupDiGetClassDevsA(&hid_guid, std::ptr::null(), std::ptr::null_mut(), DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
        if info == INVALID_HANDLE_VALUE as isize {
            return None;
        }
        let mut found: Option<(HANDLE, u32, u32)> = None;
        let mut ifdata: SP_DEVICE_INTERFACE_DATA = std::mem::zeroed();
        ifdata.cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;
        let mut index = 0u32;
        while SetupDiEnumDeviceInterfaces(info, std::ptr::null(), &hid_guid, index, &mut ifdata) != 0 {
            index += 1;
            let mut needed = 0u32;
            SetupDiGetDeviceInterfaceDetailA(info, &ifdata, std::ptr::null_mut(), 0, &mut needed, std::ptr::null_mut());
            if needed == 0 {
                continue;
            }
            // u64 storage keeps the detail struct aligned.
            let mut storage = vec![0u64; (needed as usize).div_ceil(8)];
            let detail = storage.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_A;
            (*detail).cbSize = std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as u32;
            if SetupDiGetDeviceInterfaceDetailA(info, &ifdata, detail, needed, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
                continue;
            }
            let path = (*detail).DevicePath.as_ptr() as *const u8;
            let h = CreateFileA(
                path,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                std::ptr::null_mut(),
            );
            if h == INVALID_HANDLE_VALUE {
                continue;
            }
            let mut attr: HIDD_ATTRIBUTES = std::mem::zeroed();
            attr.Size = std::mem::size_of::<HIDD_ATTRIBUTES>() as u32;
            let mut ok = false;
            if HidD_GetAttributes(h, &mut attr) != 0 && attr.VendorID == GC_ADAPTER_VID && attr.ProductID == GC_ADAPTER_PID {
                let mut pp: PHIDP_PREPARSED_DATA = 0;
                if HidD_GetPreparsedData(h, &mut pp) != 0 {
                    let mut caps: HIDP_CAPS = std::mem::zeroed();
                    if HidP_GetCaps(pp, &mut caps) == HIDP_STATUS_SUCCESS {
                        let (i, o) = (caps.InputReportByteLength as u32, caps.OutputReportByteLength as u32);
                        if i > 0 && i <= 64 {
                            found = Some((h, i, o));
                            ok = true;
                        }
                    }
                    HidD_FreePreparsedData(pp);
                }
            }
            if ok {
                break;
            }
            CloseHandle(h);
        }
        SetupDiDestroyDeviceInfoList(info);
        let (h, in_len, out_len) = found?;
        let read_event = CreateEventA(std::ptr::null(), 1, 0, std::ptr::null());
        if read_event.is_null() {
            CloseHandle(h);
            return None;
        }
        let dev = HidAdapter { h, read_event, in_len, out_len };
        // Start polling: padded to the output report length, byte 0 = report ID (0).
        let len = if out_len == 0 || out_len > 64 { 2 } else { out_len };
        let mut out = [0u8; 64];
        out[1] = 0x13;
        if HidD_SetOutputReport(h, out.as_ptr() as *const _, len) == 0 {
            let ev = CreateEventA(std::ptr::null(), 1, 0, std::ptr::null());
            let mut ov: OVERLAPPED = std::mem::zeroed();
            ov.hEvent = ev;
            let mut written = 0u32;
            if WriteFile(h, out.as_ptr(), len, &mut written, &mut ov) == 0 && GetLastError() != ERROR_IO_PENDING {
                plog!("gw: gc adapter: HID start command failed (error {})", GetLastError());
            } else {
                WaitForSingleObject(ev, 50);
            }
            CancelIoEx(h, &ov);
            CloseHandle(ev);
        }
        plog!(
            "gw: gc adapter: opened over HID (vid {:04X} pid {:04X}, report {} bytes), reading raw reports",
            GC_ADAPTER_VID, GC_ADAPTER_PID, in_len
        );
        Some(Arc::new(dev))
    }
}

impl Device for HidAdapter {
    fn read(&self) -> ReadResult {
        unsafe {
            let mut buf = [0u8; 64];
            let mut got = 0u32;
            let mut ov: OVERLAPPED = std::mem::zeroed();
            ov.hEvent = self.read_event;
            ResetEvent(self.read_event);
            if ReadFile(self.h, buf.as_mut_ptr(), self.in_len, &mut got, &mut ov) == 0 {
                if GetLastError() != ERROR_IO_PENDING {
                    return ReadResult::Lost;
                }
                if WaitForSingleObject(self.read_event, 20) != WAIT_OBJECT_0 {
                    CancelIoEx(self.h, &ov);
                    WaitForSingleObject(self.read_event, 50);
                    return ReadResult::Timeout;
                }
                if GetOverlappedResult(self.h, &ov, &mut got, 0) == 0 {
                    return ReadResult::Lost;
                }
            }
            match accept_report(&buf[..(got as usize).min(64)]) {
                Some(r) => ReadResult::Report(r),
                None => ReadResult::Timeout,
            }
        }
    }

    fn rumble(&self, cmd: &[u8; 5]) {
        let mut out = [0u8; 64];
        let len = if self.out_len == 0 || self.out_len > 64 { cmd.len() as u32 + 1 } else { self.out_len };
        out[1..6].copy_from_slice(cmd);
        unsafe {
            HidD_SetOutputReport(self.h, out.as_ptr() as *const _, len);
        }
    }

    fn cancel(&self) {
        unsafe {
            CancelIoEx(self.h, std::ptr::null());
        }
    }

    fn name(&self) -> &str {
        "HID"
    }
}

const _: () = assert!(GC_PAYLOAD_SIZE == 37);
