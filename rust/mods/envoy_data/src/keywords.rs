//! The item keyword language (owner, Nov 13: "reserved words, bolded or coloured, like a card game"). A fixed
//! vocabulary of keywords in categories, each category its own colour; every item's card text is written with them
//! in braces ("{ON} {NAIR} {HIT}: a spark {BURST} rings out."). Each trigger maps 1:1 to its opening words
//! (`trigger_words`), and a compile-time lint checks every item's card starts with its trigger's words and every
//! braced word is a keyword, so the mechanics and the words cannot drift.
use crate::items::*;

/// Keyword categories (each drawn in its own colour).
pub const KC_TIMING: u8 = 0;
pub const KC_COND: u8 = 1;
pub const KC_MOVE: u8 = 2;
pub const KC_TECH: u8 = 3;
pub const KC_OUTCOME: u8 = 4;
pub const KC_EFFECT: u8 = 5;
pub const KC_STATUS: u8 = 6;
pub const KC_STACK: u8 = 7;
pub const KC_N: usize = 8;
pub const KC_NAME: [&str; KC_N] = ["Timing", "Condition", "Move", "Technique", "Outcome", "Effect", "Status", "Stacking"];
/// Category colours (RGBA) on a light sticker and on a dark one.
pub const KC_LIGHT: [u32; KC_N] = [0x5A4C_99FF, 0x7A3F_B0FF, 0xC218_3FFF, 0x0087_7DFF, 0xB05A_00FF, 0x2A6A_D0FF, 0xC030_10FF, 0x2A1A_6EFF];
pub const KC_DARK: [u32; KC_N] = [0xFFD8_4AFF, 0xD0A0_FFFF, 0xFF7A_90FF, 0x40E0_D0FF, 0xFFB0_50FF, 0x80B8_FFFF, 0xFF80_60FF, 0xFFFF_FFFF];

pub struct Keyword {
    pub word: &'static str,
    pub cat: u8,
    /// The glossary line.
    pub def: &'static str,
}

const fn k(word: &'static str, cat: u8, def: &'static str) -> Keyword {
    Keyword { word, cat, def }
}

pub const KEYWORDS: [Keyword; 49] = [
    k("ON", KC_TIMING, "Each time the action happens."),
    k("WHEN", KC_TIMING, "Each time this happens to you."),
    k("WHILE", KC_TIMING, "For as long as it stays true."),
    k("AFTER", KC_TIMING, "Once the action has finished."),
    k("ALWAYS", KC_TIMING, "Works all the time; nothing to trigger."),
    k("MODIFIER", KC_TIMING, "Changes what your other items make."),
    k("PICKUP", KC_TIMING, "The moment you take the item."),
    k("IF", KC_COND, "Only when the condition holds."),
    k("NAIR", KC_MOVE, "Neutral air attack."),
    k("BAIR", KC_MOVE, "Back air attack."),
    k("FAIR", KC_MOVE, "Forward air attack."),
    k("UAIR", KC_MOVE, "Up air attack."),
    k("DAIR", KC_MOVE, "Down air attack."),
    k("JAB", KC_MOVE, "The A-button jab."),
    k("SMASH", KC_MOVE, "Any smash attack."),
    k("TILT", KC_MOVE, "Any tilt attack."),
    k("SPECIAL", KC_MOVE, "Any B move."),
    k("GRAB", KC_MOVE, "A grab."),
    k("THROW", KC_MOVE, "A throw from a grab."),
    k("SHINE", KC_MOVE, "Fox / Falco down-B reflector."),
    k("LASER", KC_MOVE, "Fox / Falco blaster shot."),
    k("WAVEDASH", KC_TECH, "Air-dodge into the ground and slide."),
    k("SHORT HOP", KC_TECH, "A short jump (tap jump)."),
    k("L-CANCEL", KC_TECH, "Shield as an aerial lands: half the landing lag."),
    k("TECH", KC_TECH, "Shield as you hit the ground: no knockdown."),
    k("LEDGE GRAB", KC_TECH, "Catching the stage's edge."),
    k("ROLL", KC_TECH, "A shield roll."),
    k("AIRDODGE", KC_TECH, "An air dodge (a wavedash starts with one)."),
    k("DASH", KC_TECH, "A dash or a run."),
    k("CROUCH CANCEL", KC_TECH, "Crouch as a hit lands: less knockback."),
    k("POWERSHIELD", KC_TECH, "Shield on the exact frame of a hit."),
    k("HIT", KC_OUTCOME, "Your attack connects with a foe."),
    k("WHIFF", KC_OUTCOME, "Your attack hits nothing."),
    k("SHIELD", KC_OUTCOME, "The attack lands on a shield."),
    k("KO", KC_OUTCOME, "A foe loses a stock."),
    k("TAKE HIT", KC_OUTCOME, "A foe's attack connects with you."),
    k("PROJECTILE", KC_EFFECT, "A shot that flies; modifiers change it."),
    k("BURST", KC_EFFECT, "A one-shot blast around a point."),
    k("TRAIL", KC_EFFECT, "Left on the ground behind you."),
    k("AURA", KC_EFFECT, "Around you for a while."),
    k("ORBIT", KC_EFFECT, "Circles you and hits what it touches."),
    k("ORB", KC_EFFECT, "A hitting ball that guards you: blocks shots; a foe hit shatters it, back in 4 s or on your next hit."),
    k("SHIELDING", KC_TECH, "Holding your shield up."),
    k("BEAM", KC_EFFECT, "A straight line that hits all along it."),
    k("BURN", KC_STATUS, "Damage over time; stacks."),
    k("CHILL", KC_STATUS, "Slowed for a while."),
    k("MARK", KC_STATUS, "Marked foes take more from your items."),
    k("STACK", KC_STACK, "Copies of the same item."),
    k("PER STACK", KC_STACK, "For each copy past the first."),
];

/// The opening words of every card with this trigger.
pub const fn trigger_words(tr: u8) -> &'static str {
    match tr {
        TR_PASSIVE => "{ALWAYS}",
        TR_NAIR_HIT => "{ON} {NAIR} {HIT}",
        TR_WAVEDASH => "{ON} {WAVEDASH}",
        TR_SHINE => "{ON} {SHINE}",
        TR_SHORT_HOP => "{ON} {SHORT HOP}",
        TR_LEDGE_GRAB => "{ON} {LEDGE GRAB}",
        TR_WHIFF => "{ON} {WHIFF}",
        TR_ROLL => "{ON} {ROLL}",
        TR_SMASH_HIT => "{ON} {SMASH} {HIT}",
        TR_KO => "{ON} {KO}",
        TR_MODIFIER => "{MODIFIER}",
        TR_JAB_HIT => "{ON} {JAB} {HIT}",
        TR_TAKE_HIT => "{WHEN} {TAKE HIT}",
        TR_LASER => "{ON} {LASER} {HIT}",
        TR_CC => "{ON} {CROUCH CANCEL}",
        TR_POWERSHIELD => "{ON} {POWERSHIELD}",
        TR_PICKUP => "{ON} {PICKUP}",
        TR_WHILE_AIRDODGE => "{WHILE} {AIRDODGE}",
        TR_WHILE_NAIR => "{WHILE} {NAIR}",
        TR_WHILE_DASH => "{WHILE} {DASH}",
        TR_WHILE_SHIELD => "{WHILE} {SHIELDING}",
        // the synergy lane: a proc off another item's hit, and the co-op items (2P only)
        TR_ITEM_HIT => "{ON}",
        TR_COOP => "{IF}",
        _ => "",
    }
}

/// Every item's card text (by id).
pub const CARDS: [(i32, &str); 38] = [
    (IT_SPARK_RING, "{ON} {NAIR} {HIT}: a spark {BURST} rings out. {PER STACK}: more sparks."),
    (IT_GLIDE_WAKE, "{ON} {WAVEDASH}: leave a burning {TRAIL} that can {BURN}. {PER STACK}: longer."),
    (IT_SHINE_BELL, "{ON} {SHINE}: a shock-ring {BURST} that launches. {PER STACK}: bigger."),
    (IT_HOP_SHOT, "{ON} {SHORT HOP}: spit a star {PROJECTILE} at a foe. {PER STACK}: more stars."),
    (IT_LEDGE_SPIKES, "{ON} {LEDGE GRAB}: ice spikes {BURST} upward. {PER STACK}: stronger."),
    (IT_WHIFF_ECHO, "{ON} {WHIFF}: a ghost {BURST} swings the move again. {PER STACK}: more ghosts."),
    (IT_ROLL_MINES, "{ON} {ROLL}: drop a blinking mine; it {BURST} on a foe. {PER STACK}: more mines."),
    (IT_SMASH_QUAKE, "{ON} {SMASH} {HIT}: a quake {BURST} runs along the floor. {PER STACK}: bigger."),
    (IT_MOON_DROPS, "{ALWAYS}: two moon {ORB} circle you and nick foes. {PER STACK}: more moons."),
    (IT_COIN_MAGNET, "{ON} {KO}: the foe bursts into coins for you. {PER STACK}: more coins."),
    (IT_HOMING_INK, "{MODIFIER}: your {PROJECTILE} curve toward foes."),
    (IT_FIRE_LACE, "{MODIFIER}: all you make leaves embers that {BURN}."),
    (IT_LASER_PRISM, "{ON} {LASER} {HIT}: it splits into three {PROJECTILE}. {PER STACK}: more."),
    (IT_CROUCH_THORNS, "{ON} {CROUCH CANCEL}: thorns {BURST} back at the foe."),
    (IT_PERFECT_GUARD, "{ON} {POWERSHIELD}: fire a gold {BEAM} back."),
    (IT_BRIMSTONE_JAB, "{ON} {JAB} {HIT}: fire a blood-red {BEAM}."),
    (IT_DEMON_WINGS, "{ALWAYS}: one more air jump; it rains feather {PROJECTILE}."),
    (IT_CRIMSON_COUNT, "{ON} {KO}: gain a sigil that will {ORBIT} you and {BURN} foes."),
    (IT_BLOODLUST, "{WHEN} {TAKE HIT}: red tendrils lash back as a {BURST}."),
    (IT_HALO, "{MODIFIER}: your {PROJECTILE} turn holy and pierce."),
    (IT_GUARDIAN_FEATHER, "{ALWAYS}: once a stock, it eats a hit that would {KO} you."),
    (IT_ICE_GLASS, "{MODIFIER}: your {PROJECTILE} turn to ice, {CHILL}, and shatter in three."),
    (IT_CHAIN_SPARK, "{MODIFIER}: {IF} an item makes a {HIT}, it arcs on to the next foe."),
    (IT_TWIN_SHOT, "{MODIFIER}: double your {PROJECTILE} and {ORBIT}, each at 60%."),
    (IT_GRAVITY_WELL, "{MODIFIER}: each {BURST} pulls foes in, then blows out."),
    (IT_BIG_BANG, "{MODIFIER}: each {BURST} is twice as big, with a quake; -20% damage."),
    (IT_BOUNCY, "{MODIFIER}: your {PROJECTILE} bounce off floors and walls."),
    (IT_DODGE_ORB, "{WHILE} {AIRDODGE}: an {ORB} leads your dodge and the {WAVEDASH} slide. {PER STACK}: more orbs."),
    (IT_NAIR_RING, "{WHILE} {NAIR}: a ring of {ORB} circles you. {PER STACK}: more orbs."),
    (IT_DASH_ORB, "{WHILE} {DASH}: an {ORB} leads your run. {PER STACK}: more orbs."),
    (IT_SHIELD_ORBS, "{WHILE} {SHIELDING}: {ORB} circle your shield and break on blocks. {PER STACK}: more orbs."),
    (IT_SPARE_HEART, "{ON} {PICKUP}: +1 stock limit and +1 stock (to 8)."),
    // the synergy lane (envoy-synergy.md): procs and co-op items
    (IT_EMBER_SEED, "{ON} {BURST} {HIT}: a fire seed may bloom, a second {BURST} that can {BURN}. {PER STACK}: more often."),
    (IT_STATIC_COIL, "{ON} {PROJECTILE} {HIT}: the shot may arc on to another foe. {PER STACK}: more often."),
    (IT_FROST_BUD, "{ON} spark {HIT}: may {BURST} into frost that can {CHILL}. {PER STACK}: more often."),
    (IT_KIN_TETHER, "{IF} 2P: a {TRAIL} of light to your partner nicks foes that cross it. {PER STACK}: stronger."),
    (IT_RALLY_BANNER, "{IF} 2P, {ON} {KO}: your partner is rallied for 5 s. {PER STACK}: stronger."),
    (IT_KINDRED_SPARK, "{IF} 2P: your {HIT} near your partner can fire their items. {PER STACK}: more often."),
];

/// The card text of item `id` ("" if none).
pub fn card(id: i32) -> &'static str {
    let mut i = 0;
    while i < CARDS.len() {
        if CARDS[i].0 == id {
            return CARDS[i].1;
        }
        i += 1;
    }
    ""
}

/// The keyword index of `w` (exact), or None.
pub fn keyword(w: &str) -> Option<usize> {
    KEYWORDS.iter().position(|k| k.word == w)
}

/// One span of a card: plain text, or a keyword (index into KEYWORDS).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Span<'a> {
    Text(&'a str),
    Key(usize),
}

/// The card's spans in order (plain text between keywords; an unknown braced word comes out as plain text).
pub fn spans(s: &str) -> impl Iterator<Item = Span<'_>> {
    let mut rest = s;
    core::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        if let Some(r) = rest.strip_prefix('{') {
            if let Some(end) = r.find('}') {
                let w = &r[..end];
                rest = &r[end + 1..];
                return Some(keyword(w).map_or(Span::Text(w), Span::Key));
            }
        }
        let end = rest.find('{').unwrap_or(rest.len()).max(1);
        let t = &rest[..end];
        rest = &rest[end..];
        Some(Span::Text(t))
    })
}

// ---- the compile-time lint: every item has a card, opening with its trigger's words, with only known keywords

const fn starts_with(s: &[u8], p: &[u8]) -> bool {
    if p.len() > s.len() {
        return false;
    }
    let mut i = 0;
    while i < p.len() {
        if s[i] != p[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn eq_at(s: &[u8], from: usize, to: usize, w: &[u8]) -> bool {
    if to - from != w.len() {
        return false;
    }
    let mut i = 0;
    while i < w.len() {
        if s[from + i] != w[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn known(s: &[u8], from: usize, to: usize) -> bool {
    let mut k = 0;
    while k < KEYWORDS.len() {
        if eq_at(s, from, to, KEYWORDS[k].word.as_bytes()) {
            return true;
        }
        k += 1;
    }
    false
}

const fn lint() -> bool {
    let mut i = 0;
    while i < ITEMS.len() {
        let it = &ITEMS[i];
        let mut j = 0;
        let mut found = false;
        while j < CARDS.len() {
            if CARDS[j].0 == it.id {
                assert!(!found, "an item has two cards");
                found = true;
                let s = CARDS[j].1.as_bytes();
                let tw = trigger_words(it.trigger).as_bytes();
                assert!(!tw.is_empty(), "a trigger has no words");
                assert!(starts_with(s, tw), "an item's card must open with its trigger's words");
                // every {word} is a keyword
                let mut c = 0;
                while c < s.len() {
                    if s[c] == b'{' {
                        let mut e = c + 1;
                        while e < s.len() && s[e] != b'}' {
                            e += 1;
                        }
                        assert!(e < s.len(), "an unclosed brace in a card");
                        assert!(known(s, c + 1, e), "a braced word in a card is not a keyword");
                        c = e;
                    }
                    c += 1;
                }
            }
            j += 1;
        }
        assert!(found, "an item has no card");
        i += 1;
    }
    true
}
const _: bool = lint();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_split() {
        let v: std::vec::Vec<_> = spans("{ON} {NAIR} {HIT}: a spark {BURST}.").collect();
        assert_eq!(v[0], Span::Key(keyword("ON").unwrap()));
        assert_eq!(v[1], Span::Text(" "));
        assert_eq!(v.last(), Some(&Span::Text(".")));
        for it in ITEMS.iter() {
            assert!(card(it.id).starts_with(trigger_words(it.trigger)));
        }
    }
}
