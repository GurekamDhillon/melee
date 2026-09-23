# Geno Lab UI art (private)

The Lab's icons, timeline pieces, key chips and compact panel, built in the kit's style on the
kit's own pipeline. This folder is on `private/geno` only. Nothing is added to or changed in the
root `menu/` tree. `menu/pipeline/*.py` (colour, kit, icons, build, quantise_check) is imported
by path and only read.

```
python lab_art.py                             # render, check, convert to ../ui, preview
python lab_art.py --install-menu-icon <dir>   # also copy ico_lab.gxtex into <dir>
python lab_palette.py                         # palette checks only
```

Needs the same tools as `menu/README.md`: playwright + chromium, pillow, numpy, scipy. It finds
`menu/` by walking up from here, or from `GD_MENU_ROOT`. It finds the converter at
`pc/tools/png2gx.py` in this checkout.

**Provenance:** every pixel is generated from `lab_art.py` (SVG on the kit's 64 grid) and
`lab_style.css`, rendered by headless Chromium. Nothing is traced, sampled or referenced from
Melee or any other game.

## Output

| where | what |
|---|---|
| `../ui/*.gxtex` | the 2x textures as GX data (`png2gx.py --layout`), loaded by name |
| `../ui/lab_ui.json` | manifest: every texture's size, format, 1x draw size, tint, use; the palette; panel, timeline and key-chip geometry |
| `out/2x`, `out/1x` | the PNGs (git-ignored, rebuilt every run) |
| `preview/lab_sheet.png` | the review sheet: Lab art beside the kit's own, composed from the decoded `.gxtex` |

## Naming

- `ico_lab_<name>`: control icons, 64x64 @2x I4. `gd.kit.icon("lab_<name>")` resolves to
  `ico_lab_<name>.gxtex`, the same `ico_` prefix the kit's icons use.
- `ico_lab`: the SOLO > LAB menu icon, 256x256 @2x I4, the same size as the kit's hub icons.
- `lab_mk_<event>`: timeline markers. `lab_tl_*`: timeline track pieces. `lab_key_*`: keycap
  3-slice.
- `lab_frame_*`: the compact panel 9-slice. It has the kit's `frame_*` slots
  (`corner_tl/tr/bl/br`, `edge_h`, `edge_v`) plus `fill`, so a 9-slice drawer takes a prefix.

## The LAB menu icon (SOLO > LAB)

The menus load `ico_*` through `gw_GxTex_OpenUI`, not through the mod. That function looks in
`MELEE_MENUTEX_DIR` if it is set. Otherwise it looks in `<exe dir>/ui`, then `<exe>/../../ui`
(`_build/ui`), one texture name at a time. So a private build picks the icon up without it going
into `_build/ui`, which is committed and public:

1. `python lab_art.py --install-menu-icon <run dir>/ui` puts `ico_lab.gxtex` beside the Geno
   exe. Every other `ico_*` still comes from `_build/ui`.
2. In `src/melee/gm/gmfrontend_menus.inc`, change the LAB entry's texture from `"ico_training"`
   to `"ico_lab"`. That is not done yet. A missing texture draws no icon, only the label (and
   logs `art missing`), so either keep the install step next to the change or fall back:
   `fp_tex("ico_lab") >= 0 ? "ico_lab" : "ico_training"`.

Releases copy only the committed `_build/ui`, so a private release has to copy
`ico_lab.gxtex` into its `ui/` in the same way.
