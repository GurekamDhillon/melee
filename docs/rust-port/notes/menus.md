# Menus

Decision: the port's own "kit" menus (`src/melee/gm/gmfrontend*.c/.inc`: menu tree, settings pages, online lobby,
select screens) are NOT ported. `melee play` uses stand-in rectangles and a bitmap font (`rust/melee/src/ui.rs`)
for every screen that is not a retail scene; the retail `mn*` scenes are not driven yet. Rust replaces the C build,
so there is no C-menu compatibility to keep.

Where the history lives: `play.md` (the `melee play` flow, scripts in `tools/rust-port/play/`), `audio.md`
(menu / title / results audio replayed against the C mix), `platform.md` (status table, "What only the user can check").
