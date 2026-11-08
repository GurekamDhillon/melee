//! The ENet matchmaking client (`slippi_mm::join_enet`) against a tiny in-test server on loopback that speaks the
//! Slippi 3.x protocol: create-ticket -> create-ticket-resp, then get-ticket-resp with the match.

use melee_net::enet::{Event, Host};
use melee_net::slippi_mm::{join_enet, JoinRequest, Json, Target};
use melee_net::transport::UdpTransport;
use std::time::{Duration, Instant};

#[test]
fn enet_ticket_flow_on_loopback() {
    let udp = UdpTransport::open(0x7F00_0001, 0).unwrap();
    let sport = { use melee_net::transport::Transport; udp.local_port() };
    let server = std::thread::spawn(move || {
        let mut h = Host::new(Box::new(udp), 4, 3);
        let t0 = Instant::now();
        let mut got = None;
        while t0.elapsed() < Duration::from_secs(10) {
            h.flush();
            match h.service() {
                Some(Event::Receive { peer, data, .. }) => {
                    let j = Json::parse(std::str::from_utf8(&data).unwrap()).unwrap();
                    assert_eq!(j.get("type").unwrap().str(), Some("create-ticket"));
                    got = Some(j);
                    h.send(peer, 0, br#"{"type":"create-ticket-resp"}"#, true);
                    let m = r#"{"type":"get-ticket-resp","matchId":"t","isHost":false,"stages":[32],"players":[{"port":1,"isLocalPlayer":false,"ipAddress":"127.0.0.1:9","ipAddressLan":"127.0.0.1:9"},{"port":2,"isLocalPlayer":true,"ipAddress":"127.0.0.1:8"}]}"#;
                    h.send(peer, 0, m.as_bytes(), true);
                }
                Some(Event::Disconnect { .. }) => break,
                Some(_) => {}
                None => std::thread::sleep(Duration::from_millis(2)),
            }
        }
        got
    });
    let req = JoinRequest { uid: "u".into(), play_key: String::new(), lan_ip: String::new(), display_name: "D".into(), connect_code: "AB#1".into(), mode: 2, search_code: "CD#2".into(), app_version: "3.6.4".into(), port: 0 };
    let m = join_enet(&format!("127.0.0.1:{sport}"), 0x7F00_0001, &req, Duration::from_secs(10), Target::Loopback, |_| {}, |_| false).unwrap();
    assert_eq!(m.local_index(), Some(1));
    assert_eq!(m.stages, vec![32]);
    let j = server.join().unwrap().expect("server got create-ticket");
    assert_eq!(j.get("search").unwrap().get("connectCode").unwrap().arr().len(), 4);
}
