//! Port of src/sysdolphin/baselib/hsd_3933.c: the MCC (USB host-IO) server side. A 256-slot ring
//! of `{type, flags, value}` system events is filled by `fn_803932D0` and drained by
//! `hsd_80393844`, which reads request packets from the host, answers them (channel open/close,
//! handlers 0 and 1 of the 32-entry table) and `hsd_80393A5C` writes a file to the host over FIO.
//!
//! The TU's globals live in [`Hsd3933`]. Calls into MCC / FIO / OS and the globals of other TUs
//! (`hsd_804D78A0` of hsd_392C.c, `HSD_Rand`, `hsd_80392E80`) go through [`Hsd3933Host`]; its
//! default [`ext::Ext`] is the unported-callee stub set (TODO panics named like the C).
#![allow(non_snake_case)]

use melee_types::{MccPacketBuffer, ParticleLogEntry, ParticleUsbMessages, _MCCPacket};

/// `MCCPacket` of hsd_3933.h.
pub type MCCPacket = _MCCPacket;

/// Callees of hsd_3933.c. `OS_TIMER_CLOCK` is `__OSBusClock / 4`, a runtime value.
pub trait Hsd3933Host {
    fn MCCNotify(&mut self, channel: i32, value: i32) -> i32;
    fn MCCGetLastError(&mut self) -> u8;
    /// `OSReport` with the C format already applied.
    fn OSReport(&mut self, msg: &str);
    fn MCCWrite(&mut self, channel: i32, offset: i32, packet: &MCCPacket, size: i32, cb: i32) -> i32;
    fn MCCRead(&mut self, channel: i32, offset: i32, packet: &mut MCCPacket, size: i32, cb: i32) -> i32;
    fn MCCGetFreeBlocks(&mut self, channel: i32) -> u8;
    fn MCCStreamOpen(&mut self, channel: i32, num_blocks: i32);
    /// `MCCOpen(channel, num_blocks, 0)`.
    fn MCCOpen(&mut self, channel: i32, num_blocks: i32);
    fn MCCClose(&mut self, channel: i32);
    fn OSGetTick(&mut self) -> u32;
    /// `OS_TIMER_CLOCK` (`__OSBusClock / 4`).
    fn OS_TIMER_CLOCK(&mut self) -> u32;
    fn OSDisableInterrupts(&mut self) -> i32;
    fn OSRestoreInterrupts(&mut self, level: i32);
    fn HSD_Rand(&mut self) -> u32;
    fn hsd_80392E80(&mut self);
    /// `hsd_804D78A0` (hsd_392C.c): USB server set up.
    fn hsd_804D78A0(&mut self) -> &mut i32;
    fn FIOQuery(&mut self) -> i32;
    fn FIOFopen(&mut self, filename: &str, mode: i32) -> i32;
    /// `FIOFwrite(fd, data, size)`; `data` is the guest address of the buffer.
    fn FIOFwrite(&mut self, fd: i32, data: i32, size: i32) -> i32;
    fn FIOFclose(&mut self, fd: i32);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl Hsd3933Host for Ext {
        fn MCCNotify(&mut self, _: i32, _: i32) -> i32 { todo!("MCCNotify (dolphin/mcc)") }
        fn MCCGetLastError(&mut self) -> u8 { todo!("MCCGetLastError (dolphin/mcc)") }
        fn OSReport(&mut self, _: &str) { todo!("OSReport (dolphin/os)") }
        fn MCCWrite(&mut self, _: i32, _: i32, _: &MCCPacket, _: i32, _: i32) -> i32 { todo!("MCCWrite (dolphin/mcc)") }
        fn MCCRead(&mut self, _: i32, _: i32, _: &mut MCCPacket, _: i32, _: i32) -> i32 { todo!("MCCRead (dolphin/mcc)") }
        fn MCCGetFreeBlocks(&mut self, _: i32) -> u8 { todo!("MCCGetFreeBlocks (dolphin/mcc)") }
        fn MCCStreamOpen(&mut self, _: i32, _: i32) { todo!("MCCStreamOpen (dolphin/mcc)") }
        fn MCCOpen(&mut self, _: i32, _: i32) { todo!("MCCOpen (dolphin/mcc)") }
        fn MCCClose(&mut self, _: i32) { todo!("MCCClose (dolphin/mcc)") }
        fn OSGetTick(&mut self) -> u32 { todo!("OSGetTick (dolphin/os)") }
        fn OS_TIMER_CLOCK(&mut self) -> u32 { todo!("OS_TIMER_CLOCK (dolphin/os)") }
        fn OSDisableInterrupts(&mut self) -> i32 { todo!("OSDisableInterrupts (dolphin/os)") }
        fn OSRestoreInterrupts(&mut self, _: i32) { todo!("OSRestoreInterrupts (dolphin/os)") }
        fn HSD_Rand(&mut self) -> u32 { todo!("HSD_Rand (baselib/random.c)") }
        fn hsd_80392E80(&mut self) { todo!("hsd_80392E80 (baselib/hsd_392C.c)") }
        fn hsd_804D78A0(&mut self) -> &mut i32 { todo!("hsd_804D78A0 (baselib/hsd_392C.c)") }
        fn FIOQuery(&mut self) -> i32 { todo!("FIOQuery (dolphin/fio)") }
        fn FIOFopen(&mut self, _: &str, _: i32) -> i32 { todo!("FIOFopen (dolphin/fio)") }
        fn FIOFwrite(&mut self, _: i32, _: i32, _: i32) -> i32 { todo!("FIOFwrite (dolphin/fio)") }
        fn FIOFclose(&mut self, _: i32) { todo!("FIOFclose (dolphin/fio)") }
    }
}

/// The retail message table `psUsbMessages` (0x8040A9D0), each field NUL-padded to its slot.
pub fn ps_usb_messages() -> ParticleUsbMessages {
    melee_core::ct!("ps_usb_messages");
    fn put<const N: usize>(dst: &mut [i8; N], s: &str) {
        for (d, b) in dst.iter_mut().zip(s.bytes()) {
            *d = b as i8;
        }
    }
    let mut m = ParticleUsbMessages::zeroed();
    put(&mut m.loading, "loading file (%s) from USB ...");
    put(&mut m.cannot_open, "cannot open file\n");
    put(&mut m.cannot_get_files_stat, "cannot get files stat\n");
    put(&mut m.file_too_large, "too large file size\n");
    put(&mut m.cannot_allocate, "cannot allocate memory\n");
    put(&mut m.cannot_load, "cannot load file\n");
    put(&mut m.done, "Done %s size:%d time:%f spped:%fkbps\n");
    put(&mut m.cannot_save, "cannot save file\n");
    put(&mut m.searching, "Seaching files (%s) via USB ...");
    put(&mut m.cannot_get_dir_stat, "cannot get dir stat\n");
    put(&mut m.dir_too_large, "too large dir size\n");
    put(&mut m.cannot_use_usb, "cannot use USB now.\n");
    m
}

fn cstr(field: &[i8]) -> String {
    let bytes: Vec<u8> = field.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// C `%f` of a double (six decimals; non-finite as C prints them).
fn c_f(v: f64) -> String {
    if v.is_nan() {
        "nan".into()
    } else if v.is_infinite() {
        if v < 0.0 { "-inf".into() } else { "inf".into() }
    } else {
        format!("{:.6}", v)
    }
}

/// Handler table `lbl_8040A93C[32]`: entries 0 and 1 are set, the rest NULL.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum McHandler {
    H80393440,
    H80393840,
}

fn handler(cmd: u8) -> Option<McHandler> {
    match cmd {
        0 => Some(McHandler::H80393440),
        1 => Some(McHandler::H80393840),
        _ => None,
    }
}

/// Globals of hsd_3933.c.
#[derive(Clone)]
pub struct Hsd3933 {
    pub hsd_804D78C0: i32,
    pub hsd_804D78BC: i32,
    pub hsd_804D78B8: i32,
    pub hsd_804D78B4: i32,
    pub hsd_804D78B0: i32,
    pub hsd_804D78AC: i32,
    pub hsd_804D78A8: i32,
    /// `hsd_804CEB40[0x100]`
    pub hsd_804CEB40: [ParticleLogEntry; 0x100],
    /// `hsd_804CF740[16]`: which MCC channels are open.
    pub hsd_804CF740: [i32; 16],
    /// Function-local statics of `hsd_80393844`.
    pub response: MccPacketBuffer,
    pub request: MccPacketBuffer,
}

impl Default for Hsd3933 {
    fn default() -> Self {
        Hsd3933 {
            hsd_804D78C0: 0,
            hsd_804D78BC: 0,
            hsd_804D78B8: 0,
            hsd_804D78B4: 0,
            hsd_804D78B0: 0,
            hsd_804D78AC: 0,
            hsd_804D78A8: 0,
            hsd_804CEB40: [ParticleLogEntry::zeroed(); 0x100],
            hsd_804CF740: [0; 16],
            response: MccPacketBuffer::zeroed(),
            request: MccPacketBuffer::zeroed(),
        }
    }
}

impl Hsd3933 {
    /// `fn_803932D0` (0x803932D0): MCC system-event callback; queues type-0x100 events.
    pub fn fn_803932D0(&mut self, type_: i32, flags: u32, value: i32) {
        let count = self.hsd_804D78BC;
        if count <= 0x100 && flags == 0x100 {
            let idx = (self.hsd_804D78B8.wrapping_add(count) % 256) as usize;
            let e = &mut self.hsd_804CEB40[idx];
            e.x0 = type_;
            e.x4 = flags;
            e.x8 = value;
            self.hsd_804D78BC += 1;
        }
    }

    /// `hsd_80393328` (0x80393328): ping the host with a random token; 1 if it answered in time.
    pub fn hsd_80393328<H: Hsd3933Host + ?Sized>(&mut self, h: &mut H) -> i32 {
        if self.hsd_804D78B4 != 0 {
            return 0;
        }
        self.hsd_804D78B4 = 1;
        self.hsd_804D78B0 = (h.HSD_Rand() % 255) as i32;
        if h.MCCNotify(0xF, self.hsd_804D78B0 + 256) == 0 {
            let err = h.MCCGetLastError();
            h.OSReport(&format!("Error(0x{:x}) in MCCNotify.\n", err));
        }
        let mut result = 1;
        let start_tick = h.OSGetTick();
        let ticks_per_unit = h.OS_TIMER_CLOCK();
        loop {
            h.hsd_80392E80();
            if self.hsd_804D78B0 == 0 {
                result = 0;
                break;
            }
            if <u32 as melee_core::ppc::Ppc>::pdiv(h.OSGetTick().wrapping_sub(start_tick), ticks_per_unit) >= 3 {
                break;
            }
        }
        if result != 0 {
            self.hsd_804D78B4 = 0;
            return 0;
        }
        self.hsd_804D78B4 = 0;
        1
    }

    /// The repeated tail of every reply: write the response block, notify, advance the slot.
    fn send_response<H: Hsd3933Host + ?Sized>(&mut self, h: &mut H, response: &MCCPacket) {
        h.MCCWrite(0xF, (self.hsd_804D78AC << 5) + 0x1000, response, 0x20, 0);
        if h.MCCNotify(0xF, self.hsd_804D78AC + 0x80) == 0 {
            let err = h.MCCGetLastError();
            h.OSReport(&format!("Error(0x{:x}) in MCCNotify.\n", err));
        }
        self.hsd_804D78AC = (self.hsd_804D78AC + 1) % 128;
    }

    /// `hsd_80393440` (0x80393440): command 0, channel open (0x100) / close (0x200).
    pub fn hsd_80393440<H: Hsd3933Host + ?Sized>(&mut self, h: &mut H, request: &MCCPacket, response: &mut MCCPacket) {
        let cmd = request.x6;
        match (cmd & 0xFF00) as i32 {
            0x100 => {
                let streaming = cmd & 0x80 != 0;
                let num_blocks = (cmd & 0xF) as i32;
                let mut channel = 2usize;
                while channel < 16 {
                    if self.hsd_804CF740[channel] == 0 {
                        break;
                    }
                    channel += 1;
                }
                if channel == 16 {
                    response.x6 = 0x8001;
                    self.send_response(h, response);
                    return;
                }
                let free_blocks = h.MCCGetFreeBlocks(1);
                if free_blocks == 0 {
                    let err = h.MCCGetLastError();
                    if err != 0 {
                        h.OSReport(&format!("MCCError: {}\n", err as i32));
                    }
                }
                if num_blocks > free_blocks as i32 {
                    response.x6 = (free_blocks as u16).wrapping_add(0x8010);
                    self.send_response(h, response);
                    return;
                }
                self.hsd_804CF740[channel] = 1;
                response.x6 = channel as u16;
                self.send_response(h, response);
                if streaming {
                    h.MCCStreamOpen(channel as i32, num_blocks);
                } else {
                    h.MCCOpen(channel as i32, num_blocks);
                }
            }
            0x200 => {
                let i = (cmd & 0xF) as usize;
                if self.hsd_804CF740[i] != 1 {
                    response.x6 = 0x8002;
                    self.send_response(h, response);
                    return;
                }
                response.x6 = 0;
                self.send_response(h, response);
                self.hsd_804CF740[i] = 0;
                h.MCCClose(i as i32);
            }
            _ => {}
        }
    }

    /// `hsd_80393840` (0x80393840): command 1, empty.
    pub fn hsd_80393840(&mut self, _request: &MCCPacket, _response: &mut MCCPacket) {}

    /// `hsd_80393844` (0x80393844): drain the event ring and answer any request packets.
    pub fn hsd_80393844<H: Hsd3933Host + ?Sized>(&mut self, h: &mut H) {
        loop {
            let irq = h.OSDisableInterrupts();
            if self.hsd_804D78BC == 0 {
                h.OSRestoreInterrupts(irq);
                return;
            }
            let e = self.hsd_804CEB40[self.hsd_804D78B8 as usize];
            let (type_, flags, value) = (e.x0, e.x4, e.x8);
            self.hsd_804D78BC -= 1;
            self.hsd_804D78B8 = (self.hsd_804D78B8 + 1) % 256;
            h.OSRestoreInterrupts(irq);

            if flags != 0x100 || type_ != 0xF {
                continue;
            }
            let v = value as u32;
            if v & 0xFFFF_FF00 == 0x100 {
                if h.MCCNotify(0xF, (value as u8 as i32) | 0x200) == 0 {
                    let err = h.MCCGetLastError();
                    h.OSReport(&format!("Error(0x{:x}) in MCCNotify.\n", err));
                }
            } else if v & 0xFFFF_FF00 == 0x200 {
                if self.hsd_804D78B0 as u32 == (value as u8) as u32 {
                    self.hsd_804D78B0 = 0;
                }
            } else if v & 0xFFFF_FF80 == 0x80 {
                self.request.packet = MCCPacket::zeroed();
                self.hsd_804D78A8 = value & 0x7F;
                let mut request = self.request.packet;
                let read = h.MCCRead(0xF, self.hsd_804D78A8 << 5, &mut request, 0x20, 0);
                self.request.packet = request;
                if read != 0 && request.x4_b7() == 0 {
                    let mut response = MCCPacket::zeroed();
                    response.set_x4_b7(1);
                    response.x0 = request.x0;
                    let cmd = request.command;
                    response.command = cmd;
                    self.response.packet = response;
                    if cmd < 0x20 {
                        match handler(cmd) {
                            Some(McHandler::H80393440) => self.hsd_80393440(h, &request, &mut response),
                            Some(McHandler::H80393840) => self.hsd_80393840(&request, &mut response),
                            None => {}
                        }
                    }
                    self.response.packet = response;
                }
            }
        }
    }

    /// `hsd_80393A04` (0x80393A04): is the USB server set up (drops the flag if FIO lost it).
    pub fn hsd_80393A04<H: Hsd3933Host + ?Sized>(&mut self, h: &mut H) -> bool {
        if *h.hsd_804D78A0() == 0 {
            return false;
        }
        if h.FIOQuery() == 0 {
            *h.hsd_804D78A0() = 0;
            return false;
        }
        true
    }

    /// `hsd_80393A54` (0x80393A54)
    pub fn hsd_80393A54(&mut self, level: i32) {
        self.hsd_804D78C0 = level;
    }
}

/// `hsd_80393A5C` (0x80393A5C): write `size` bytes at guest address `data` to `filename` on the
/// host. Returns the byte count, -5 if USB is not ready, -1 on open failure, else the short write.
pub fn hsd_80393A5C<H: Hsd3933Host + ?Sized>(h: &mut H, filename: &str, data: i32, size: i32) -> i32 {
    melee_core::ct!("hsd_80393A5C");
    let messages = ps_usb_messages();
    let ready = if *h.hsd_804D78A0() == 0 {
        false
    } else if h.FIOQuery() == 0 {
        *h.hsd_804D78A0() = 0;
        false
    } else {
        true
    };
    if !ready {
        return -5;
    }
    let filename = filename.strip_prefix('/').unwrap_or(filename);

    let start = h.OSGetTick();
    let fd = h.FIOFopen(filename, 0xA02);
    if (fd as u32).wrapping_add(0x10000) == 0xFFFF {
        h.OSReport(&cstr(&messages.cannot_open));
        return -1;
    }
    let written_f = h.FIOFwrite(fd, data, size) as f32;
    if size as f32 != written_f {
        h.OSReport(&cstr(&messages.cannot_save));
        h.FIOFclose(fd);
        return melee_core::ppc::fctiwz_f32(written_f);
    }
    h.FIOFclose(fd);
    // OSSecondsToTicks(1.0F) = 1.0F * OS_TIMER_CLOCK in f32; the tick delta divides as f32.
    let ticks = 1.0f32 * h.OS_TIMER_CLOCK() as f32;
    let elapsed = h.OSGetTick().wrapping_sub(start) as f32 / ticks;
    let kbps = 8.0f32 * size as f32 / elapsed * (1.0f32 / 1024.0f32);
    let msg = cstr(&messages.done)
        .replacen("%s", filename, 1)
        .replacen("%d", &size.to_string(), 1)
        .replacen("%f", &c_f(elapsed as f64), 1)
        .replacen("%f", &c_f(kbps as f64), 1);
    h.OSReport(&msg);
    size
}
