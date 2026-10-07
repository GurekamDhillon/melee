# Atlas HUD layer (gd.ui.hud)

Shows the in-match HUD layer and the retail takeover's mask. One HUD (`gd.ui.hud`) holds a **build strip** (slot pips, a keystone stone, "1 waiting") at the top left, a **port card** for every human port (name, percent and stocks are read when it draws), a **toast** (`gd.ui.toast`) at the top right, a **banner with the A glyph** at the top centre and a **one-line note** at the bottom left. The engine places the parts inside the title-safe box and keeps them clear of the retail percent plates and the match timer (the keep-out rectangles; `atlas keepout on` in the console draws them as thin rose outlines). A HUD never takes focus.

Launch with `MELEE_SCENE="mode=lab;stage=fd;p1=fox/hu;p2=falco/cpu0;cpus=idle"`, then in the console `load <absolute path of this folder>`.

- **F7** shows the HUD and sends a toast; F7 again clears it.
- **F8** steps the retail mask: nothing, then `hud.damage`, `hud.stock`, both, and round again. Each step is logged. What you should see: with `hud.damage` hidden the retail percent plates are gone and the stocks stay; with `hud.stock` hidden the stock icons are gone and the percents stay; with both, the bottom of the screen is empty of retail HUD, and the keep-out frees that space. Percent still changes in the match (`= gd.player(1).percent`).
- The match clock (`hud.timer`) can never be hidden by a mod. **Online nothing retail is hidden** (the mask reads empty and `gd.ui.retail_hide` is refused); the HUD still draws online (presentation only).
- Everything the demo sets is released when it unloads or the scene changes.

The mask is **off by default for everyone**: with no script, environment variable or console command, retail damage, stocks, timer and the pause look exactly as they always did. Without a gameplay script and a match, `retail_hide` refuses, which is why this demo is a gameplay script.

No disc-derived art. Needs the Atlas font roles in `ui/` (`menu/pipeline/font_atlas.py`). Offline stand-in checked; not yet seen in the game. `unload demo_atlas_hud` releases it.

API reading: workspace `docs/scripting.md`, **Atlas screens (`gd.ui`)**, the HUD layer and the retail takeover.
