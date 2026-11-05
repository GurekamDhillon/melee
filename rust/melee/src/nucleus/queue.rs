//! The download queue and its saved form (port of gw_nucleus_queue.h). A press of A on a mod queues it (`toggle`);
//! Start begins processing; the worker takes the first QUEUED entry, installs it and marks it DONE or FAILED with a
//! plain-words reason. `queue.json` is written atomically after every change (a sibling .tmp flushed, then renamed
//! over), so a crash or a power cut leaves the last complete queue. On load a RUNNING entry is QUEUED again (its
//! install left only a `.staging-*` folder, which the engine removes), a DONE one is dropped, a FAILED one stays.
//! The `processing` flag is saved too: a restart picks the downloads up where they were. Same JSON as the C client.
use super::catalog::{ascii, q};
use crate::cosmetics_import::Json;

pub const MAX: usize = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum St {
    Queued,
    Running,
    Done,
    Failed,
}
const NAMES: [&str; 4] = ["queued", "running", "done", "failed"];
impl St {
    pub fn name(self) -> &'static str {
        NAMES[self as usize]
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: i64,
    pub state: St,
    pub pct: u32,
    pub attempts: u32,
    pub added: i64,
    pub title: String,
    pub author: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default)]
pub struct Queue {
    pub it: Vec<Item>,
    pub processing: bool,
    pub ver: u64,
}

impl Queue {
    pub fn find(&self, id: i64) -> Option<usize> {
        self.it.iter().position(|t| t.id == id)
    }
    pub fn pending(&self) -> usize {
        self.it.iter().filter(|t| matches!(t.state, St::Queued | St::Running)).count()
    }
    fn item(id: i64, title: &str, author: &str, now: i64) -> Item {
        Item { id, state: St::Queued, pct: 0, attempts: 0, added: now, title: ascii(title, 80), author: ascii(author, 40), reason: String::new() }
    }
    /// A on a mod: queue it, or take it out when it is waiting. +1 queued, -1 removed, 0 refused (being installed / full).
    pub fn toggle(&mut self, id: i64, title: &str, author: &str, now: i64) -> i32 {
        match self.find(id) {
            Some(at) if self.it[at].state == St::Queued => {
                self.it.remove(at);
                self.ver += 1;
                -1
            }
            Some(at) if self.it[at].state == St::Running => 0,
            at => {
                if let Some(at) = at {
                    self.it.remove(at); // a DONE or FAILED entry is replaced
                }
                if self.it.len() >= MAX {
                    return 0;
                }
                self.it.push(Queue::item(id, title, author, now));
                self.ver += 1;
                1
            }
        }
    }
    pub fn next(&self) -> Option<usize> {
        self.it.iter().position(|t| t.state == St::Queued)
    }
    pub fn mark_running(&mut self, id: i64) {
        if let Some(at) = self.find(id) {
            let t = &mut self.it[at];
            t.state = St::Running;
            t.pct = 0;
            t.reason.clear();
            self.ver += 1;
        }
    }
    /// Changes the version only when the shown percent does.
    pub fn progress(&mut self, id: i64, pct: u32) {
        if let Some(at) = self.find(id) {
            let p = pct.min(100);
            if self.it[at].state == St::Running && self.it[at].pct != p {
                self.it[at].pct = p;
                self.ver += 1;
            }
        }
    }
    pub fn finish(&mut self, id: i64, ok: bool, reason: &str) {
        if let Some(at) = self.find(id) {
            let t = &mut self.it[at];
            t.state = if ok { St::Done } else { St::Failed };
            t.pct = if ok { 100 } else { 0 };
            if !ok {
                t.attempts += 1;
            }
            t.reason = ascii(reason, 120);
            self.ver += 1;
        }
    }
    pub fn requeue(&mut self, id: i64) {
        if let Some(at) = self.find(id) {
            self.it[at].state = St::Queued;
            self.it[at].pct = 0;
            self.ver += 1;
        }
    }

    pub fn to_json(&self) -> String {
        let mut b = format!("{{\"schema\":1,\"processing\":{},\"items\":[", self.processing as i32);
        for (i, t) in self.it.iter().enumerate() {
            b += &format!(
                "{}\n{{\"id\":{},\"state\":{},\"attempts\":{},\"added\":{},\"title\":{},\"author\":{},\"reason\":{}}}",
                if i > 0 { "," } else { "" },
                t.id,
                q(t.state.name()),
                t.attempts,
                t.added,
                q(&t.title),
                q(&t.author),
                q(&t.reason)
            );
        }
        b += "\n]}\n";
        b
    }

    /// Parse a saved queue with the recovery rules (RUNNING -> QUEUED, DONE dropped). None when it is not a queue.
    pub fn from_json(text: &str) -> Option<Queue> {
        let j = Json::parse(text.trim()).ok()?;
        let Some(Json::Arr(items)) = j.get("items") else { return None };
        let num = |e: &Json, k: &str| if let Some(Json::Num(n)) = e.get(k) { *n as i64 } else { 0 };
        let mut q = Queue { processing: num(&j, "processing") != 0, ..Default::default() };
        for e in items {
            let id = num(e, "id");
            if id <= 0 || q.find(id).is_some() || q.it.len() >= MAX {
                continue;
            }
            let st = e.get("state").and_then(|s| s.str()).unwrap_or("queued");
            let Some(s) = NAMES.iter().position(|n| *n == st) else { continue };
            let state = [St::Queued, St::Queued, St::Done, St::Failed][s];
            if state == St::Done {
                continue;
            }
            let mut t = Queue::item(id, e.get("title").and_then(|s| s.str()).unwrap_or(""), e.get("author").and_then(|s| s.str()).unwrap_or(""), num(e, "added"));
            t.state = state;
            t.attempts = num(e, "attempts") as u32;
            t.reason = ascii(e.get("reason").and_then(|s| s.str()).unwrap_or(""), 120);
            q.it.push(t);
        }
        q.ver = 1;
        Some(q)
    }
}

/// Write `path` so that a crash leaves the old file or the new one, never half: a flushed sibling .tmp renamed over.
pub fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = path.with_extension(format!("{}.tmp", path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()));
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_round_trip_recovers() {
        let mut q = Queue::default();
        assert_eq!(q.toggle(5, "A", "x", 1), 1);
        assert_eq!(q.toggle(6, "B", "y", 2), 1);
        assert_eq!(q.toggle(7, "C", "z", 3), 1);
        q.mark_running(5);
        q.finish(6, true, "");
        q.processing = true;
        let r = Queue::from_json(&q.to_json()).unwrap();
        assert!(r.processing);
        assert_eq!(r.it.iter().map(|t| (t.id, t.state)).collect::<Vec<_>>(), vec![(5, St::Queued), (7, St::Queued)]);
        let mut r = r;
        assert_eq!(r.toggle(7, "C", "z", 4), -1);
        assert!(Queue::from_json("not json").is_none());
    }
}
