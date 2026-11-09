/* Records the Rust port still relies on after upstream dropped them (doldecomp #3583 ff.). Declared here, in the
 * decomp's own words at the commit before the drop, so gen_types.py keeps generating their native type, GameCube
 * layout and decoder like any other record (the layouts are still cross-checked against the ppc32 dump).
 *
 * sislib: upstream now reads the SIS font archive and the text state through `u8*` (HSD_Text.sis_buffer, x60,
 * state_stack) and types alloc_data as SisBuffer; the port's sislib.rs / melee::retail::sis and the engine's DAT
 * decode (arrays.tsv: SIS.kerning / SIS.textures run to the next object) still use the typed views below. */
#ifndef MELEE_RUST_PORT_LEGACY_H
#define MELEE_RUST_PORT_LEGACY_H
#include <sysdolphin/baselib/sislib.h>

typedef struct TextKerning {
    /*0x00*/ u8 left;
    /*0x01*/ u8 right;
} TextKerning;

typedef struct SIS {
    /*0x00*/ TextKerning* kerning;
    /*0x04*/ TextGlyphTexture* textures;
} SIS;

typedef struct sisLib_803A7664_t {
    void* x0;
    void* x4;
    u32 x8; ///< alloc size
    u32 xC;
} sisLib_803A7664_t;

#endif
