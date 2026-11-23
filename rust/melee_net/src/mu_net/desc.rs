//! The session descriptor (`MUSD`): what both peers agreed before they connect, in one canonical byte form whose
//! BLAKE2b digest the two compare in VERIFY (plan section 2.6), and the chained descriptor of a session's next game
//! (section 2.12).

use super::blake2b::hash32;
use super::crypto::Bytes32;
use super::wire::{R, W};

pub const PROTOCOL_VERSION: u16 = 1;
/// Protocol revision: 1 = several games (INFO/NEXT), 2 = early-input SAMPLEs.
pub const REVISION: u8 = 2;
pub const DESCRIPTOR_FORMAT: u16 = 1;
pub const ROUTE_DIRECT: u8 = 1;
pub const ROUTE_UNRANKED: u8 = 2;
pub const ROLLBACK_HORIZON: u8 = 7;
pub const MIN_DELAY: u8 = 1;
pub const MAX_DELAY: u8 = 15;
pub const MAX_NAME: usize = 31;
pub const MAX_BUILD_ID: usize = 64;
pub const FEATURE_SUBFRAME: u32 = 1 << 0;
pub const FEATURE_PAUSE: u32 = 1 << 1;
pub const KNOWN_FEATURES: u32 = FEATURE_SUBFRAME | FEATURE_PAUSE;
/// Frame numbers of game k travel raised by (k - 1) x this.
pub const GAME_FRAME_SPAN: i32 = 1 << 24;
pub const MAX_GAMES: u32 = 127;
/// The stage pool the games after the first are drawn from (the default singles rules' legal stages, in the
/// protocol's fixed order): Battlefield, Final Destination, Dream Land, Yoshi's Story, Fountain, Pokemon Stadium.
pub const STAGE_POOL: [u16; 6] = [0x1F, 0x20, 0x1C, 0x08, 0x02, 0x03];

const MAGIC: &[u8; 4] = b"MUSD";
const DIGEST_DOMAIN: &[u8] = b"MeleeUnlockedNet1 descriptor";
const NEXT_DOMAIN: &[u8] = b"MeleeUnlockedNet1 next";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Slot {
    pub present: bool,
    /// external character id (the character select screen's)
    pub character: u8,
    pub color: u8,
    pub name: String,
    pub identity: Bytes32,
    /// the slot's X25519 ephemeral public key of this session
    pub session: Bytes32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub protocol_version: u16,
    pub route: u8,
    pub build_id: String,
    pub content_hash: Bytes32,
    pub rules_profile: u32,
    pub match_id: [u8; 16],
    pub players: [Slot; 4],
    /// external stage id
    pub stage: u16,
    pub alt_stage: u8,
    pub rng_seed: u32,
    pub input_delay: u8,
    pub rollback_horizon: u8,
    pub stocks: u8,
    pub timer_seconds: u32,
    pub features: u32,
}

impl Default for Descriptor {
    fn default() -> Self {
        Descriptor {
            protocol_version: PROTOCOL_VERSION,
            route: ROUTE_DIRECT,
            build_id: String::new(),
            content_hash: [0; 32],
            rules_profile: 0,
            match_id: [0; 16],
            players: Default::default(),
            stage: 0x1F,
            alt_stage: 0,
            rng_seed: 0,
            input_delay: 2,
            rollback_horizon: ROLLBACK_HORIZON,
            stocks: 4,
            timer_seconds: 8 * 60,
            features: 0,
        }
    }
}

impl Descriptor {
    /// Whether this build can play it (the reason when not).
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err("the match was set up for another network version");
        }
        if self.route != ROUTE_DIRECT && self.route != ROUTE_UNRANKED {
            return Err("the match uses an unknown route");
        }
        if self.build_id.is_empty() || self.build_id.len() > MAX_BUILD_ID {
            return Err("the match has no usable build id");
        }
        if !(MIN_DELAY..=MAX_DELAY).contains(&self.input_delay) {
            return Err("the input delay is outside 1 to 15 frames");
        }
        if self.rollback_horizon != ROLLBACK_HORIZON {
            return Err("the rollback window is not 7 frames");
        }
        if self.features & !KNOWN_FEATURES != 0 || self.features & FEATURE_SUBFRAME != 0 {
            return Err("the match asks for a feature this build does not have");
        }
        if self.stocks == 0 || self.stocks > 99 {
            return Err("the stock count is not playable");
        }
        let p = &self.players;
        if !p[0].present || !p[1].present || p[2].present || p[3].present {
            return Err("a P2P match is two players, in the first two slots");
        }
        if p.iter().any(|s| s.name.len() > MAX_NAME) {
            return Err("a player name is too long");
        }
        Ok(())
    }

    /// The canonical bytes (the caller has validated the lengths).
    pub fn canonical(&self) -> Vec<u8> {
        let mut w = W::default();
        w.bytes(MAGIC).u16(DESCRIPTOR_FORMAT).u16(self.protocol_version).u8(self.route);
        w.str8(&self.build_id, MAX_BUILD_ID).bytes(&self.content_hash).u32(self.rules_profile).bytes(&self.match_id);
        for p in &self.players {
            w.u8(p.present as u8).u8(p.character).u8(p.color).str8(&p.name, MAX_NAME).bytes(&p.identity).bytes(&p.session);
        }
        w.u16(self.stage).u8(self.alt_stage).u32(self.rng_seed).u8(self.input_delay).u8(self.rollback_horizon);
        w.u8(self.stocks).u32(self.timer_seconds).u32(self.features);
        w.0
    }

    pub fn parse(d: &[u8]) -> Option<Descriptor> {
        if d.len() > 1024 {
            return None;
        }
        let mut r = R::new(d);
        if &r.arr::<4>() != MAGIC || r.u16() != DESCRIPTOR_FORMAT {
            return None;
        }
        let mut x = Descriptor { protocol_version: r.u16(), route: r.u8(), build_id: r.str8(MAX_BUILD_ID), ..Default::default() };
        x.content_hash = r.arr();
        x.rules_profile = r.u32();
        x.match_id = r.arr();
        for p in x.players.iter_mut() {
            let present = r.u8();
            if present > 1 {
                return None; // one spelling per value
            }
            *p = Slot { present: present == 1, character: r.u8(), color: r.u8(), name: r.str8(MAX_NAME), identity: r.arr(), session: r.arr() };
        }
        x.stage = r.u16();
        x.alt_stage = r.u8();
        x.rng_seed = r.u32();
        x.input_delay = r.u8();
        x.rollback_horizon = r.u8();
        x.stocks = r.u8();
        x.timer_seconds = r.u32();
        x.features = r.u32();
        r.done().then_some(x)
    }

    pub fn digest(&self) -> Bytes32 {
        hash32(&[DIGEST_DOMAIN, &self.canonical()])
    }

    /// Writes the keys the handshake saw into both slots. A key already set to something else fails.
    pub fn bind_keys(&mut self, keys: [(Bytes32, Bytes32); 2]) -> Result<(), &'static str> {
        for (p, (identity, session)) in self.players.iter_mut().zip(keys) {
            if (p.identity != [0; 32] && p.identity != identity) || (p.session != [0; 32] && p.session != session) {
                return Err("a player key differs from the handshake's");
            }
            p.identity = identity;
            p.session = session;
        }
        Ok(())
    }

    /// The descriptor of game `next_game` (n + 1) from this one (game n, keys bound) and its digest.
    pub fn next(&self, digest: &Bytes32, next_game: u32, pool: &[u16]) -> Descriptor {
        let h = hash32(&[NEXT_DOMAIN, digest, &next_game.to_le_bytes()]);
        let le = |at: usize| u32::from_le_bytes(h[at..at + 4].try_into().unwrap());
        let mut n = self.clone();
        n.match_id.copy_from_slice(&h[..16]);
        n.rng_seed = le(16);
        if !pool.is_empty() {
            n.stage = pool[(le(20) % pool.len() as u32) as usize];
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Descriptor {
        let mut d = Descriptor { build_id: "melee-rust test".into(), rules_profile: 2, rng_seed: 0x1234_5678, stage: 0x20, ..Default::default() };
        d.content_hash[..11].copy_from_slice(b"retail-1.02");
        d.players[0] = Slot { present: true, character: 2, color: 1, name: "Fox".into(), ..Default::default() };
        d.players[1] = Slot { present: true, character: 9, color: 0, name: "Marth".into(), ..Default::default() };
        d
    }

    #[test]
    fn canonical_round_trip_and_layout() {
        let d = sample();
        assert_eq!(d.validate(), Ok(()));
        let b = d.canonical();
        // magic, format, protocol, route, str8 build id
        assert_eq!(&b[..10], b"MUSD\x01\x00\x01\x00\x01\x0f");
        // 4 + 2 + 2 + 1 + (1 + 15) + 32 + 4 + 16 + slots (3 + 1 + name + 64 each) + 2 + 1 + 4 + 1 + 1 + 1 + 4 + 4
        assert_eq!(b.len(), 4 + 2 + 2 + 1 + 16 + 32 + 4 + 16 + (68 + 3) + (68 + 5) + 68 * 2 + 18);
        assert_eq!(&b[b.len() - 18..], &[0x20, 0, 0, 0x78, 0x56, 0x34, 0x12, 2, 7, 4, 0xe0, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(Descriptor::parse(&b), Some(d.clone()));
        let mut bad = b.clone();
        bad.push(0);
        assert_eq!(Descriptor::parse(&bad), None);
    }

    #[test]
    fn digest_covers_the_keys_and_next_is_chained() {
        let mut a = sample();
        let d0 = a.digest();
        a.bind_keys([([1; 32], [2; 32]), ([3; 32], [4; 32])]).unwrap();
        assert_ne!(a.digest(), d0);
        assert!(a.clone().bind_keys([([9; 32], [2; 32]), ([3; 32], [4; 32])]).is_err());
        let d1 = a.digest();
        let n = a.next(&d1, 2, &STAGE_POOL);
        assert!(STAGE_POOL.contains(&n.stage));
        assert_ne!(n.match_id, a.match_id);
        assert_eq!(n.players, a.players);
        assert_eq!(n, a.next(&d1, 2, &STAGE_POOL), "deterministic");
        assert_ne!(n.digest(), a.next(&d1, 3, &STAGE_POOL).digest());
        assert_eq!(a.next(&d1, 2, &[]).stage, a.stage);
    }

    #[test]
    fn validate_refuses() {
        let mut d = sample();
        d.features = FEATURE_SUBFRAME;
        assert!(d.validate().is_err());
        let mut d = sample();
        d.players[2].present = true;
        assert!(d.validate().is_err());
        let mut d = sample();
        d.input_delay = 16;
        assert!(d.validate().is_err());
    }
}
