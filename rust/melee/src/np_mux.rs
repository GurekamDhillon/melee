//! One UDP socket for the whole netplay run (`melee play --netplay`, net3): the first room, the menus' lockstep and
//! every rematch room share it, so a NAT mapping punched (or a relay opened) once serves the whole connection.
//!
//! Channels, told apart by their first bytes:
//! - 0: the first room, unprefixed (the gw_net wire, `GN...`; stays compatible with the C build's direct room);
//! - 1: the lockstep menus (`MLS1`, np_lockstep.rs; already self-identifying, so unprefixed), or the online Lab's
//!   lockstep with its commands (`MLS2`);
//! - 2 + k: the k-th rematch room (k >= 1), each packet prefixed `MRM` + k (a new `Net` must not see the old one's
//!   late datagrams).
//! - EARLY_CH: early inputs (`MEI1`, melee_net::early, np_early.rs), sent from the sampler thread straight to the
//!   socket and read by the match each tick.
//! Anything else (the hole punch `GDPH`, stray datagrams) is dropped. Each channel's queue is bounded; a channel nobody
//! reads (the lockstep during a match) loses its oldest datagrams, which its protocols resend anyway.

use melee_net::transport::{Addr, Transport};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

const QUEUE_MAX: usize = 1024;
/// The early-inputs channel.
pub const EARLY_CH: u32 = 1000;

pub struct MuxCore {
    base: Box<dyn Transport>,
    q: HashMap<u32, VecDeque<(Addr, Vec<u8>)>>,
}

pub type MuxRef = Rc<RefCell<MuxCore>>;

pub fn new(base: Box<dyn Transport>) -> MuxRef {
    Rc::new(RefCell::new(MuxCore { base, q: HashMap::new() }))
}

impl MuxCore {
    fn pump(&mut self) {
        let mut b = [0u8; 2048];
        loop {
            let mut from = Addr::default();
            let n = self.base.recv(&mut from, &mut b);
            if n <= 0 {
                break;
            }
            let d = &b[..n as usize];
            let (ch, body) = if d.len() >= 4 && &d[..3] == b"MRM" {
                (2 + d[3] as u32, &d[4..])
            } else if d.len() >= 4 && (&d[..4] == b"MLS1" || &d[..4] == b"MLS2") {
                (1, d)
            } else if d.len() >= 4 && &d[..4] == melee_net::early::MAGIC {
                (EARLY_CH, d)
            } else if d.len() >= 2 && &d[..2] == b"GN" {
                (0, d)
            } else {
                continue;
            };
            let q = self.q.entry(ch).or_default();
            if q.len() >= QUEUE_MAX {
                q.pop_front();
            }
            q.push_back((from, body.to_vec()));
        }
    }
}

impl MuxCore {
    /// A send path for another thread (the socket below, with the relay's wrapping when relayed).
    pub fn thread_sender(&self) -> Option<melee_net::transport::ThreadSender> {
        self.base.thread_sender()
    }
    /// The next datagram of a channel (pumping the socket first).
    pub fn take(&mut self, ch: u32) -> Option<Vec<u8>> {
        self.pump();
        self.q.get_mut(&ch).and_then(|q| q.pop_front()).map(|x| x.1)
    }
}

/// One channel as a transport.
pub struct Chan {
    pub core: MuxRef,
    pub ch: u32,
}

impl Transport for Chan {
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        let mut c = self.core.borrow_mut();
        if self.ch >= 2 {
            let mut b = Vec::with_capacity(data.len() + 4);
            b.extend_from_slice(b"MRM");
            b.push((self.ch - 2) as u8);
            b.extend_from_slice(data);
            c.base.send(to, &b)
        } else {
            c.base.send(to, data)
        }
    }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        let mut c = self.core.borrow_mut();
        c.pump();
        let Some((a, d)) = c.q.get_mut(&self.ch).and_then(|q| q.pop_front()) else { return 0 };
        *from = a;
        let n = d.len().min(buf.len());
        buf[..n].copy_from_slice(&d[..n]);
        n as i32
    }
    fn local_port(&self) -> u16 {
        self.core.borrow().base.local_port()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Loop(Rc<RefCell<VecDeque<Vec<u8>>>>);
    impl Transport for Loop {
        fn send(&mut self, _: &Addr, d: &[u8]) -> i32 {
            self.0.borrow_mut().push_back(d.to_vec());
            d.len() as i32
        }
        fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
            let Some(d) = self.0.borrow_mut().pop_front() else { return 0 };
            *from = Addr::new(1, 2);
            buf[..d.len()].copy_from_slice(&d);
            d.len() as i32
        }
    }

    #[test]
    fn channels() {
        let m = new(Box::new(Loop::default()));
        let mut c0 = Chan { core: m.clone(), ch: 0 };
        let mut c1 = Chan { core: m.clone(), ch: 1 };
        let mut c3 = Chan { core: m.clone(), ch: 3 };
        let a = Addr::new(1, 2);
        c3.send(&a, b"GNxyz");
        c0.send(&a, b"GNabc");
        c1.send(&a, b"MLS1....");
        c0.send(&a, b"GDPH");
        let mut b = [0u8; 64];
        let mut f = Addr::default();
        assert_eq!(c1.recv(&mut f, &mut b), 8);
        assert_eq!(c0.recv(&mut f, &mut b), 5);
        assert_eq!(&b[..5], b"GNabc");
        assert_eq!(c0.recv(&mut f, &mut b), 0, "the punch is dropped");
        assert_eq!(c3.recv(&mut f, &mut b), 5);
        assert_eq!(&b[..5], b"GNxyz", "the rematch prefix is stripped");
    }
}
