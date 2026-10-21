//! Wire-level equality with the shipping C: the transcript of oracle/oracle.c (gw_net.c compiled
//! as the shipping build compiles native shims) must equal the Rust port's, line for line: every
//! datagram (time, endpoints, length, FNV-1a 64 of its bytes with the session id masked), every
//! event / desync / lobby delivery, and the end state of each scenario. Regenerate the data with
//! `bash rust/melee_net/oracle/build.sh`.

mod common;

#[test]
fn transcript_matches_c() {
    let want = include_str!("data/net_oracle.txt");
    let want: Vec<&str> = want.lines().collect();
    let got = common::run_oracle_scenarios();
    let n = want.len().min(got.len());
    for i in 0..n {
        if want[i] != got[i] {
            let lo = i.saturating_sub(5);
            panic!(
                "transcript differs at line {}:\n  C:    {}\n  Rust: {}\ncontext (C):\n{}\ncontext (Rust):\n{}",
                i + 1,
                want[i],
                got[i],
                want[lo..=i].join("\n"),
                got[lo..=i].join("\n")
            );
        }
    }
    assert_eq!(want.len(), got.len(), "transcript lengths differ");
    let packets = want.iter().filter(|l| l.contains('>')).count();
    assert!(packets > 20000, "only {} datagrams compared", packets);
}
