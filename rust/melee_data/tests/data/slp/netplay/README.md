# Netplay .slp fixture pair

The two peers' `.slp` exports of one loopback netplay match (Falcon host vs Fox guest, Final Destination,
60 s timed, 3098 frames -123..2974, played through a local `melee_relay`). The two files are byte-identical
(`cmp` clean). `melee slpcheck --full` reports `match` on both (6196 fighter-frames compared, vanilla disc,
`SLPCHECK_VANILLA_SPAWN=1`). A .slp holds inputs and game state only; no disc data is in these files.

Build: `cargo build --profile fast -p melee -p melee_relay` (from `rust/`, commit 0cd085d0b, port/candidate12).

Commands (ISO = the vanilla disc image; RPORT / NPORT = free UDP ports; run each peer in its own cwd and
MELEE_SLP_DIR; pad scripts are tools/rust-port/netplay/np_fight.txt (host) and np_fight2.txt (guest)):

    melee_relay --port RPORT --bind 127.0.0.1 --bind6 off --quiet &
    export MELEE_VOLUME=3 MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_WINDOW_HIDDEN=1 MELEE_NETPLAY_BIND=127.0.0.1
    # host (stderr prints "MATCH CODE XXXX")
    MELEE_SLP_DIR=host-slp MELEE_PAD_SCRIPT=np_fight.txt melee play --iso ISO --time-secs 60 --max-frames 9000 \
      --netplay host --np-server 127.0.0.1:RPORT --np-lan 127.0.0.1 --np-bind 127.0.0.1 --np-port NPORT \
      --match falcon,32 --np-seed 4242 --record host.mrpl --slp auto
    # guest
    MELEE_SLP_DIR=guest-slp MELEE_PAD_SCRIPT=np_fight2.txt melee play --iso ISO --time-secs 60 --max-frames 9000 \
      --netplay join CODE --np-server 127.0.0.1:RPORT --np-lan 127.0.0.1 --np-bind 127.0.0.1 \
      --match fox --record guest.mrpl --slp auto
    melee slpcheck --slp host-slp/Game_*.slp --full --iso ISO

Each peer's `.slp` appears when its process exits (a headless child replays the .mrpl). Both were named
`Game_20261115T233556.slp` and are stored here as `netplay_host.slp` / `netplay_guest.slp`.
