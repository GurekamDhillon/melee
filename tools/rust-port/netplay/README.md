# Netplay proofs (port/plat-net)

Helper scripts used for the loopback proofs in RUST_PORT.md ("netplay on the real game loop"). They
assume `NP_SCRATCH` (default `$TEMP/np`) as a scratch dir, the workspace `.env` for `GW_ISO_VANILLA`,
and the release build `build/cargo/release/melee.exe`.

- `run.sh PORT [melee netplay args]`: Rust host + Rust guest on loopback, compares both CSVs.
- `srv.sh PORT`: the matchmaking server (`tools/netplay/server/gdmelee_server.py`) on 127.0.0.1.
- `cnet.sh PORT TAG [VAR=val]`: two C clients (scripted pads), neutral hash logs. Needs
  `pad_np_host.lua` / `pad_np_guest.lua` in the scratch dir: `sed 's/^local role = .*/local role = "host"/'
  tools/rust-port/golden/pad_np.lua` (and `"guest"`). Env `CREF_ROOT` = the C build root to run.
- `mix.sh host|guest PORT [args]`: one Rust peer against one C client. Needs `MELEE_NP_IDS`
  (`<id hex>=ck:N;...;<id hex>=ext:N`, the content identities the C client prints in its
  "match agreed" log line; hashes of disc files, so never committed) and a C log with the
  handshake words (`cnet-c-host/melee-pc.log`).
- The C build needs `MELEE_RB_NEUTRAL_HASH=1` on its side so both checksums are the neutral hash.
