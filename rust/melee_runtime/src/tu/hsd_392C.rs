//! Port of src/sysdolphin/baselib/hsd_392C.c: the USB/MCC debug console bring-up and its system
//! event queue (a 256-slot ring filled by the MCC system-event callback `fn_80392E2C`).
//!
//! The TU's globals live in [`Hsd392C`]. Calls into MCC / FIO / OS and into the unported
//! hsd_3933.c (including its globals `hsd_804CF740`, `hsd_804D78A8/AC/B8/BC`) go through
//! [`Hsd392CHost`]; its default [`ext::Ext`] is the stub set (TODO panics named like the C).
//!
//! Callbacks the C hands to MCC are documented on the host methods: the host must route
//! `MCCInit`'s system-event callback to [`Hsd392C::fn_80392E2C`]; `MCCOpen`'s event callback is
//! `fn_803932D0` (hsd_3933.c, unported); `MCCEnumDevices` takes a closure standing for
//! [`Hsd392C::fn_80392CCC`].
#![allow(non_snake_case)]

/// Globals of hsd_3933.c that this TU touches.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hsd3933Vars {
    /// `hsd_804CF740[16]`
    pub hsd_804CF740: [i32; 16],
    pub hsd_804D78A8: i32,
    pub hsd_804D78AC: i32,
    pub hsd_804D78B8: i32,
    pub hsd_804D78BC: i32,
}

/// Callees of hsd_392C.c.
pub trait Hsd392CHost {
    fn MCCGetLastError(&mut self) -> u8;
    /// `OSReport` with the C format already applied.
    fn OSReport(&mut self, msg: &str);
    fn FIOExit(&mut self);
    fn MCCExit(&mut self);
    /// `MCCInit(exi_channel, 0, fn_80392E2C)`: nonzero on success. The host routes system
    /// events to [`Hsd392C::fn_80392E2C`].
    fn MCCInit(&mut self, exi_channel: i32, flags: i32) -> i32;
    fn FIOInit(&mut self, exi_channel: i32, a: i32, b: i32) -> i32;
    /// `MCCOpen(ch, mode, fn_803932D0)`: nonzero on success.
    fn MCCOpen(&mut self, ch: i32, mode: i32) -> i32;
    /// Returns nonzero with `*status` set (`MCC_CONNECT`; 3 = connected).
    fn MCCGetConnectionStatus(&mut self, ch: i32, status: &mut i32) -> i32;
    fn OSGetTick(&mut self) -> u32;
    /// `OSTicksToSeconds` macro (ticks / (bus clock / 4 / 1000) / 1000).
    fn OSTicksToSeconds(&mut self, ticks: u32) -> u32;
    fn OSDisableInterrupts(&mut self) -> i32;
    fn OSRestoreInterrupts(&mut self, level: i32);
    /// `MCCEnumDevices(fn_80392CCC)`: calls `cb(channel)` per device.
    fn MCCEnumDevices(&mut self, cb: &mut dyn FnMut(i32) -> i32);
    fn hsd_80393328(&mut self) -> i32;
    fn hsd_80393844(&mut self);
    fn hsd_80393A54(&mut self, a: i32);
    /// Globals of hsd_3933.c.
    fn hsd_3933(&mut self) -> &mut Hsd3933Vars;
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl Hsd392CHost for Ext {
        fn MCCGetLastError(&mut self) -> u8 { todo!("MCCGetLastError (dolphin/mcc)") }
        fn OSReport(&mut self, _: &str) { todo!("OSReport (dolphin/os)") }
        fn FIOExit(&mut self) { todo!("FIOExit (dolphin/fio)") }
        fn MCCExit(&mut self) { todo!("MCCExit (dolphin/mcc)") }
        fn MCCInit(&mut self, _: i32, _: i32) -> i32 { todo!("MCCInit (dolphin/mcc)") }
        fn FIOInit(&mut self, _: i32, _: i32, _: i32) -> i32 { todo!("FIOInit (dolphin/fio)") }
        fn MCCOpen(&mut self, _: i32, _: i32) -> i32 { todo!("MCCOpen (dolphin/mcc)") }
        fn MCCGetConnectionStatus(&mut self, _: i32, _: &mut i32) -> i32 { todo!("MCCGetConnectionStatus (dolphin/mcc)") }
        fn OSGetTick(&mut self) -> u32 { todo!("OSGetTick (dolphin/os)") }
        fn OSTicksToSeconds(&mut self, _: u32) -> u32 { todo!("OSTicksToSeconds (dolphin/os)") }
        fn OSDisableInterrupts(&mut self) -> i32 { todo!("OSDisableInterrupts (dolphin/os)") }
        fn OSRestoreInterrupts(&mut self, _: i32) { todo!("OSRestoreInterrupts (dolphin/os)") }
        fn MCCEnumDevices(&mut self, _: &mut dyn FnMut(i32) -> i32) { todo!("MCCEnumDevices (dolphin/mcc)") }
        fn hsd_80393328(&mut self) -> i32 { todo!("hsd_80393328 (baselib/hsd_3933.c)") }
        fn hsd_80393844(&mut self) { todo!("hsd_80393844 (baselib/hsd_3933.c)") }
        fn hsd_80393A54(&mut self, _: i32) { todo!("hsd_80393A54 (baselib/hsd_3933.c)") }
        fn hsd_3933(&mut self) -> &mut Hsd3933Vars { todo!("hsd_804CF740/hsd_804D78A8.. (baselib/hsd_3933.c)") }
    }
}

/// Globals of hsd_392C.c.
#[derive(Clone, Debug)]
pub struct Hsd392C {
    /// `hsd_804D7890`: EXI channel in use.
    pub hsd_804D7890: i32,
    /// `hsd_804D7894`: channel found by the device enumeration.
    pub hsd_804D7894: i32,
    /// `hsd_804D7898`: queued event count.
    pub hsd_804D7898: i32,
    /// `hsd_804D789C`: queue head index.
    pub hsd_804D789C: i32,
    /// `hsd_804D78A0`
    pub hsd_804D78A0: i32,
    /// `hsd_804CE728[256]`
    pub hsd_804CE728: [i32; 256],
}

impl Default for Hsd392C {
    fn default() -> Self {
        Hsd392C {
            hsd_804D7890: 0,
            hsd_804D7894: 0,
            hsd_804D7898: 0,
            hsd_804D789C: 0,
            hsd_804D78A0: 0,
            hsd_804CE728: [0; 256],
        }
    }
}

fn reset_channel_flags(v: &mut Hsd3933Vars) {
    v.hsd_804CF740 = [0; 16];
    v.hsd_804CF740[0] = 1;
    v.hsd_804CF740[8] = 1;
    v.hsd_804CF740[15] = 1;
}

impl Hsd392C {
    /// `fn_80392CCC`: device-enumeration callback; remembers the channel.
    pub fn fn_80392CCC(&mut self, channel: i32) -> i32 {
        self.hsd_804D7894 = channel;
        0
    }

    /// `fn_80392CD8`: report `MCCGetLastError` with a description; returns the code.
    pub fn fn_80392CD8<H: Hsd392CHost>(&mut self, h: &mut H, caller: Option<&str>) -> u8 {
        let err = h.MCCGetLastError();
        let msg = match err {
            0 => return err,
            1 => "MCC is no initialize",
            2 => "No responce",
            3 => "PING error",
            4 => "Could not initialize HIO",
            5 => "Could not read HIO mailbox",
            6 => "Could not write HIO mailbox",
            7 => "Could not read HIO memory",
            8 => "Could not write HIO memory",
            9 => "Could not read HIO status",
            10 => "Could not flush channelInfo",
            11 => "Could not load channelInfo",
            12 => "Not enough memory block",
            13 => "Invalid function parameter",
            14 => "Invalid channel parameter",
            15 => "Invalid data size",
            16 => "Invalid offset parameter",
            17 => "Channel was (already) opened",
            18 => "Channel was (already) closed",
            19 => "Channel was (already) locked",
            20 => "Channel was (already) unlocked",
            21 => "Channel (read/write) busy",
            _ => "Unknown error",
        };
        match caller {
            Some(c) => h.OSReport(&format!("{c}: MCC Error, {msg} ({err})\n")),
            None => h.OSReport(&format!("MCC Error, {msg} ({err})\n")),
        }
        err
    }

    /// `fn_80392E2C`: MCC system-event callback; queues event types 1 and 2.
    pub fn fn_80392E2C(&mut self, event_type: i32) {
        if self.hsd_804D7898 <= 0x100 && (event_type.wrapping_sub(1) as u32) <= 1 {
            let idx = self.hsd_804D789C.wrapping_add(self.hsd_804D7898);
            self.hsd_804CE728[idx.wrapping_rem(256) as usize & 0xFF] = event_type;
            self.hsd_804D7898 += 1;
        }
    }

    /// `usb_exit_init` (static): tear down FIO/MCC and re-init MCC.
    fn usb_exit_init<H: Hsd392CHost>(&mut self, h: &mut H) {
        h.OSReport("usb exit and initialize.\n");
        h.FIOExit();
        self.hsd_804D78A0 = 0;
        h.MCCExit();
        if h.MCCInit(self.hsd_804D7890, 0) == 0 {
            h.OSReport("MCCInit NG.\n");
        }
    }

    /// `hsd_80392E80`: drain the event queue (ready / reboot handling). Returns 0.
    pub fn hsd_80392E80<H: Hsd392CHost>(&mut self, h: &mut H) -> i32 {
        loop {
            let intr = h.OSDisableInterrupts();
            if self.hsd_804D7898 == 0 {
                h.OSRestoreInterrupts(intr);
                break;
            }
            let head = self.hsd_804D789C + 1;
            let event = self.hsd_804CE728[self.hsd_804D789C as usize & 0xFF];
            self.hsd_804D7898 -= 1;
            self.hsd_804D789C = head.wrapping_rem(256);
            h.OSRestoreInterrupts(intr);

            match event {
                2 => {
                    h.OSReport("mcc ready.\n");
                    h.hsd_3933().hsd_804D78AC = 0;
                    h.hsd_3933().hsd_804D78A8 = 0;
                    h.OSReport("Initialize FIO-Connection ... ");
                    if h.FIOInit(self.hsd_804D7890, 1, 10) == 0 {
                        h.OSReport("NG\n");
                    } else {
                        h.OSReport("OK\n");
                        self.hsd_804D78A0 = 1;
                    }
                    h.OSReport("Initialize MCC-Connection ... ");
                    if h.MCCOpen(0xF, 1) == 0 {
                        h.OSReport("NG\n");
                        self.usb_exit_init(h);
                    } else {
                        h.OSReport("Open ... ");
                        let mut waiting = 1;
                        let start_tick = h.OSGetTick();
                        loop {
                            let mut status = 0;
                            if h.MCCGetConnectionStatus(0xF, &mut status) != 0 && status == 3 {
                                waiting = 0;
                            } else {
                                let now = h.OSGetTick();
                                if h.OSTicksToSeconds(now.wrapping_sub(start_tick)) < 3 {
                                    continue;
                                }
                            }
                            break;
                        }
                        if waiting != 0 {
                            h.OSReport("NG\n");
                            self.usb_exit_init(h);
                        } else {
                            h.OSReport("OK\n");
                            reset_channel_flags(h.hsd_3933());
                        }
                    }
                }
                1 => {
                    h.OSReport("receive event 'REBOOT'. Query server ...");
                    if h.hsd_80393328() == 0 {
                        h.OSReport("Query NG.\n");
                        self.usb_exit_init(h);
                    } else {
                        h.OSReport("Query OK. This event is dumped.\n");
                    }
                }
                _ => {}
            }
        }
        h.hsd_80393844();
        0
    }

    /// `hsd_803931A4`: initialise the USB console on `exi_channel` (negative: enumerate devices
    /// and use the last one found, else 0). Returns whether `MCCInit` succeeded.
    pub fn hsd_803931A4<H: Hsd392CHost>(&mut self, h: &mut H, exi_channel: i32) -> bool {
        reset_channel_flags(h.hsd_3933());

        let mut channel = exi_channel;
        if channel < 0 {
            self.hsd_804D7894 = -1;
            let mut found = self.hsd_804D7894;
            h.MCCEnumDevices(&mut |c| {
                found = c;
                0
            });
            self.hsd_804D7894 = found;
            channel = self.hsd_804D7894;
            if self.hsd_804D7894 < 0 {
                channel = 0;
            }
        }

        self.hsd_804D7890 = channel;
        h.hsd_80393A54(1);
        self.hsd_804D78A0 = 0;
        self.hsd_804D7898 = 0;
        self.hsd_804D789C = 0;
        h.hsd_3933().hsd_804D78BC = 0;
        h.hsd_3933().hsd_804D78B8 = 0;

        if h.MCCInit(self.hsd_804D7890, 0) == 0 {
            let e = h.MCCGetLastError();
            h.OSReport(&format!("EXI_{channel}:not support USB (Error={e})\n"));
            return false;
        }

        h.OSReport(&format!("EXI initialized (EXI_{channel})\n"));
        true
    }
}
