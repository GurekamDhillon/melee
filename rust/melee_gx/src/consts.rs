//! GX enum values, from extern/aurora/include/dolphin/gx/GXEnum.h (+ GXCommandList.h). Plain `u32`
//! constants, as melee_types does for C enums; values were extracted mechanically from the header.
#![allow(missing_docs, dead_code)]

// GXProjectionType
pub const GX_PERSPECTIVE: u32 = 0x0;
pub const GX_ORTHOGRAPHIC: u32 = 0x1;

// GXCompare
pub const GX_NEVER: u32 = 0x0;
pub const GX_LESS: u32 = 0x1;
pub const GX_EQUAL: u32 = 0x2;
pub const GX_LEQUAL: u32 = 0x3;
pub const GX_GREATER: u32 = 0x4;
pub const GX_NEQUAL: u32 = 0x5;
pub const GX_GEQUAL: u32 = 0x6;
pub const GX_ALWAYS: u32 = 0x7;

// GXAlphaOp
pub const GX_AOP_AND: u32 = 0x0;
pub const GX_AOP_OR: u32 = 0x1;
pub const GX_AOP_XOR: u32 = 0x2;
pub const GX_AOP_XNOR: u32 = 0x3;
pub const GX_MAX_ALPHAOP: u32 = 0x4;

// GXZFmt16
pub const GX_ZC_LINEAR: u32 = 0x0;
pub const GX_ZC_NEAR: u32 = 0x1;
pub const GX_ZC_MID: u32 = 0x2;
pub const GX_ZC_FAR: u32 = 0x3;

// GXGamma
pub const GX_GM_1_0: u32 = 0x0;
pub const GX_GM_1_7: u32 = 0x1;
pub const GX_GM_2_2: u32 = 0x2;

// GXPixelFmt
pub const GX_PF_RGB8_Z24: u32 = 0x0;
pub const GX_PF_RGBA6_Z24: u32 = 0x1;
pub const GX_PF_RGB565_Z16: u32 = 0x2;
pub const GX_PF_Z24: u32 = 0x3;
pub const GX_PF_Y8: u32 = 0x4;
pub const GX_PF_U8: u32 = 0x5;
pub const GX_PF_V8: u32 = 0x6;
pub const GX_PF_YUV420: u32 = 0x7;

// GXPrimitive
pub const GX_QUADS: u32 = 0x80;
pub const GX_TRIANGLES: u32 = 0x90;
pub const GX_TRIANGLESTRIP: u32 = 0x98;
pub const GX_TRIANGLEFAN: u32 = 0xa0;
pub const GX_LINES: u32 = 0xa8;
pub const GX_LINESTRIP: u32 = 0xb0;
pub const GX_POINTS: u32 = 0xb8;

// GXVtxFmt
pub const GX_VTXFMT0: u32 = 0x0;
pub const GX_VTXFMT1: u32 = 0x1;
pub const GX_VTXFMT2: u32 = 0x2;
pub const GX_VTXFMT3: u32 = 0x3;
pub const GX_VTXFMT4: u32 = 0x4;
pub const GX_VTXFMT5: u32 = 0x5;
pub const GX_VTXFMT6: u32 = 0x6;
pub const GX_VTXFMT7: u32 = 0x7;
pub const GX_MAX_VTXFMT: u32 = 0x8;

// GXAttr
pub const GX_VA_PNMTXIDX: u32 = 0x0;
pub const GX_VA_TEX0MTXIDX: u32 = 0x1;
pub const GX_VA_TEX1MTXIDX: u32 = 0x2;
pub const GX_VA_TEX2MTXIDX: u32 = 0x3;
pub const GX_VA_TEX3MTXIDX: u32 = 0x4;
pub const GX_VA_TEX4MTXIDX: u32 = 0x5;
pub const GX_VA_TEX5MTXIDX: u32 = 0x6;
pub const GX_VA_TEX6MTXIDX: u32 = 0x7;
pub const GX_VA_TEX7MTXIDX: u32 = 0x8;
pub const GX_VA_POS: u32 = 0x9;
pub const GX_VA_NRM: u32 = 0xa;
pub const GX_VA_CLR0: u32 = 0xb;
pub const GX_VA_CLR1: u32 = 0xc;
pub const GX_VA_TEX0: u32 = 0xd;
pub const GX_VA_TEX1: u32 = 0xe;
pub const GX_VA_TEX2: u32 = 0xf;
pub const GX_VA_TEX3: u32 = 0x10;
pub const GX_VA_TEX4: u32 = 0x11;
pub const GX_VA_TEX5: u32 = 0x12;
pub const GX_VA_TEX6: u32 = 0x13;
pub const GX_VA_TEX7: u32 = 0x14;
pub const GX_POS_MTX_ARRAY: u32 = 0x15;
pub const GX_NRM_MTX_ARRAY: u32 = 0x16;
pub const GX_TEX_MTX_ARRAY: u32 = 0x17;
pub const GX_LIGHT_ARRAY: u32 = 0x18;
pub const GX_VA_NBT: u32 = 0x19;
pub const GX_VA_MAX_ATTR: u32 = 0x1a;
pub const GX_VA_NULL: u32 = 0xff;

// GXAttrType
pub const GX_NONE: u32 = 0x0;
pub const GX_DIRECT: u32 = 0x1;
pub const GX_INDEX8: u32 = 0x2;
pub const GX_INDEX16: u32 = 0x3;

// GXTexFmt
pub const GX_TF_I4: u32 = 0x0;
pub const GX_TF_I8: u32 = 0x1;
pub const GX_TF_IA4: u32 = 0x2;
pub const GX_TF_IA8: u32 = 0x3;
pub const GX_TF_RGB565: u32 = 0x4;
pub const GX_TF_RGB5A3: u32 = 0x5;
pub const GX_TF_RGBA8: u32 = 0x6;
pub const GX_TF_CMPR: u32 = 0xe;
pub const GX_CTF_R4: u32 = 0x20;
pub const GX_CTF_RA4: u32 = 0x22;
pub const GX_CTF_RA8: u32 = 0x23;
pub const GX_CTF_YUVA8: u32 = 0x26;
pub const GX_CTF_A8: u32 = 0x27;
pub const GX_CTF_R8: u32 = 0x28;
pub const GX_CTF_G8: u32 = 0x29;
pub const GX_CTF_B8: u32 = 0x2a;
pub const GX_CTF_RG8: u32 = 0x2b;
pub const GX_CTF_GB8: u32 = 0x2c;
pub const GX_TF_Z8: u32 = 0x11;
pub const GX_TF_Z16: u32 = 0x13;
pub const GX_TF_Z24X8: u32 = 0x16;
pub const GX_CTF_Z4: u32 = 0x30;
pub const GX_CTF_Z8M: u32 = 0x39;
pub const GX_CTF_Z8L: u32 = 0x3a;
pub const GX_CTF_Z16L: u32 = 0x3c;
pub const GX_TF_A8: u32 = 0x27;
pub const GX_TF_R8_PC: u32 = 0x41;
pub const GX_TF_RG8_PC: u32 = 0x43;
pub const GX_TF_RGBA8_PC: u32 = 0x46;
pub const GX_TF_BC1_PC: u32 = 0x4e;

// GXCITexFmt
pub const GX_TF_C4: u32 = 0x8;
pub const GX_TF_C8: u32 = 0x9;
pub const GX_TF_C14X2: u32 = 0xa;

// GXTexWrapMode
pub const GX_CLAMP: u32 = 0x0;
pub const GX_REPEAT: u32 = 0x1;
pub const GX_MIRROR: u32 = 0x2;
pub const GX_MAX_TEXWRAPMODE: u32 = 0x3;

// GXTexFilter
pub const GX_NEAR: u32 = 0x0;
pub const GX_LINEAR: u32 = 0x1;
pub const GX_NEAR_MIP_NEAR: u32 = 0x2;
pub const GX_LIN_MIP_NEAR: u32 = 0x3;
pub const GX_NEAR_MIP_LIN: u32 = 0x4;
pub const GX_LIN_MIP_LIN: u32 = 0x5;

// GXAnisotropy
pub const GX_ANISO_1: u32 = 0x0;
pub const GX_ANISO_2: u32 = 0x1;
pub const GX_ANISO_4: u32 = 0x2;
pub const GX_MAX_ANISOTROPY: u32 = 0x3;

// GXTexMapID
pub const GX_TEXMAP0: u32 = 0x0;
pub const GX_TEXMAP1: u32 = 0x1;
pub const GX_TEXMAP2: u32 = 0x2;
pub const GX_TEXMAP3: u32 = 0x3;
pub const GX_TEXMAP4: u32 = 0x4;
pub const GX_TEXMAP5: u32 = 0x5;
pub const GX_TEXMAP6: u32 = 0x6;
pub const GX_TEXMAP7: u32 = 0x7;
pub const GX_MAX_TEXMAP: u32 = 0x8;
pub const GX_TEXMAP_NULL: u32 = 0xff;
pub const GX_TEX_DISABLE: u32 = 0x100;

// GXTexCoordID
pub const GX_TEXCOORD0: u32 = 0x0;
pub const GX_TEXCOORD1: u32 = 0x1;
pub const GX_TEXCOORD2: u32 = 0x2;
pub const GX_TEXCOORD3: u32 = 0x3;
pub const GX_TEXCOORD4: u32 = 0x4;
pub const GX_TEXCOORD5: u32 = 0x5;
pub const GX_TEXCOORD6: u32 = 0x6;
pub const GX_TEXCOORD7: u32 = 0x7;
pub const GX_MAX_TEXCOORD: u32 = 0x8;
pub const GX_TEXCOORD_NULL: u32 = 0xff;

// GXTevStageID
pub const GX_TEVSTAGE0: u32 = 0x0;
pub const GX_TEVSTAGE1: u32 = 0x1;
pub const GX_TEVSTAGE2: u32 = 0x2;
pub const GX_TEVSTAGE3: u32 = 0x3;
pub const GX_TEVSTAGE4: u32 = 0x4;
pub const GX_TEVSTAGE5: u32 = 0x5;
pub const GX_TEVSTAGE6: u32 = 0x6;
pub const GX_TEVSTAGE7: u32 = 0x7;
pub const GX_TEVSTAGE8: u32 = 0x8;
pub const GX_TEVSTAGE9: u32 = 0x9;
pub const GX_TEVSTAGE10: u32 = 0xa;
pub const GX_TEVSTAGE11: u32 = 0xb;
pub const GX_TEVSTAGE12: u32 = 0xc;
pub const GX_TEVSTAGE13: u32 = 0xd;
pub const GX_TEVSTAGE14: u32 = 0xe;
pub const GX_TEVSTAGE15: u32 = 0xf;
pub const GX_MAX_TEVSTAGE: u32 = 0x10;

// GXTevMode
pub const GX_MODULATE: u32 = 0x0;
pub const GX_DECAL: u32 = 0x1;
pub const GX_BLEND: u32 = 0x2;
pub const GX_REPLACE: u32 = 0x3;
pub const GX_PASSCLR: u32 = 0x4;

// GXTexMtxType
pub const GX_MTX3x4: u32 = 0x0;
pub const GX_MTX2x4: u32 = 0x1;

// GXTexGenType
pub const GX_TG_MTX3x4: u32 = 0x0;
pub const GX_TG_MTX2x4: u32 = 0x1;
pub const GX_TG_BUMP0: u32 = 0x2;
pub const GX_TG_BUMP1: u32 = 0x3;
pub const GX_TG_BUMP2: u32 = 0x4;
pub const GX_TG_BUMP3: u32 = 0x5;
pub const GX_TG_BUMP4: u32 = 0x6;
pub const GX_TG_BUMP5: u32 = 0x7;
pub const GX_TG_BUMP6: u32 = 0x8;
pub const GX_TG_BUMP7: u32 = 0x9;
pub const GX_TG_SRTG: u32 = 0xa;

// GXPosNrmMtx
pub const GX_PNMTX0: u32 = 0x0;
pub const GX_PNMTX1: u32 = 0x3;
pub const GX_PNMTX2: u32 = 0x6;
pub const GX_PNMTX3: u32 = 0x9;
pub const GX_PNMTX4: u32 = 0xc;
pub const GX_PNMTX5: u32 = 0xf;
pub const GX_PNMTX6: u32 = 0x12;
pub const GX_PNMTX7: u32 = 0x15;
pub const GX_PNMTX8: u32 = 0x18;
pub const GX_PNMTX9: u32 = 0x1b;

// GXTexMtx
pub const GX_TEXMTX0: u32 = 0x1e;
pub const GX_TEXMTX1: u32 = 0x21;
pub const GX_TEXMTX2: u32 = 0x24;
pub const GX_TEXMTX3: u32 = 0x27;
pub const GX_TEXMTX4: u32 = 0x2a;
pub const GX_TEXMTX5: u32 = 0x2d;
pub const GX_TEXMTX6: u32 = 0x30;
pub const GX_TEXMTX7: u32 = 0x33;
pub const GX_TEXMTX8: u32 = 0x36;
pub const GX_TEXMTX9: u32 = 0x39;
pub const GX_IDENTITY: u32 = 0x3c;

// GXChannelID
pub const GX_COLOR0: u32 = 0x0;
pub const GX_COLOR1: u32 = 0x1;
pub const GX_ALPHA0: u32 = 0x2;
pub const GX_ALPHA1: u32 = 0x3;
pub const GX_COLOR0A0: u32 = 0x4;
pub const GX_COLOR1A1: u32 = 0x5;
pub const GX_COLOR_ZERO: u32 = 0x6;
pub const GX_ALPHA_BUMP: u32 = 0x7;
pub const GX_ALPHA_BUMPN: u32 = 0x8;
pub const GX_COLOR_NULL: u32 = 0xff;

// GXTexGenSrc
pub const GX_TG_POS: u32 = 0x0;
pub const GX_TG_NRM: u32 = 0x1;
pub const GX_TG_BINRM: u32 = 0x2;
pub const GX_TG_TANGENT: u32 = 0x3;
pub const GX_TG_TEX0: u32 = 0x4;
pub const GX_TG_TEX1: u32 = 0x5;
pub const GX_TG_TEX2: u32 = 0x6;
pub const GX_TG_TEX3: u32 = 0x7;
pub const GX_TG_TEX4: u32 = 0x8;
pub const GX_TG_TEX5: u32 = 0x9;
pub const GX_TG_TEX6: u32 = 0xa;
pub const GX_TG_TEX7: u32 = 0xb;
pub const GX_TG_TEXCOORD0: u32 = 0xc;
pub const GX_TG_TEXCOORD1: u32 = 0xd;
pub const GX_TG_TEXCOORD2: u32 = 0xe;
pub const GX_TG_TEXCOORD3: u32 = 0xf;
pub const GX_TG_TEXCOORD4: u32 = 0x10;
pub const GX_TG_TEXCOORD5: u32 = 0x11;
pub const GX_TG_TEXCOORD6: u32 = 0x12;
pub const GX_TG_COLOR0: u32 = 0x13;
pub const GX_TG_COLOR1: u32 = 0x14;
pub const GX_MAX_TEXGENSRC: u32 = 0x15;

// GXBlendMode
pub const GX_BM_NONE: u32 = 0x0;
pub const GX_BM_BLEND: u32 = 0x1;
pub const GX_BM_LOGIC: u32 = 0x2;
pub const GX_BM_SUBTRACT: u32 = 0x3;
pub const GX_MAX_BLENDMODE: u32 = 0x4;

// GXBlendFactor
pub const GX_BL_ZERO: u32 = 0x0;
pub const GX_BL_ONE: u32 = 0x1;
pub const GX_BL_SRCCLR: u32 = 0x2;
pub const GX_BL_INVSRCCLR: u32 = 0x3;
pub const GX_BL_SRCALPHA: u32 = 0x4;
pub const GX_BL_INVSRCALPHA: u32 = 0x5;
pub const GX_BL_DSTALPHA: u32 = 0x6;
pub const GX_BL_INVDSTALPHA: u32 = 0x7;
pub const GX_BL_DSTCLR: u32 = 0x2;
pub const GX_BL_INVDSTCLR: u32 = 0x3;

// GXLogicOp
pub const GX_LO_CLEAR: u32 = 0x0;
pub const GX_LO_AND: u32 = 0x1;
pub const GX_LO_REVAND: u32 = 0x2;
pub const GX_LO_COPY: u32 = 0x3;
pub const GX_LO_INVAND: u32 = 0x4;
pub const GX_LO_NOOP: u32 = 0x5;
pub const GX_LO_XOR: u32 = 0x6;
pub const GX_LO_OR: u32 = 0x7;
pub const GX_LO_NOR: u32 = 0x8;
pub const GX_LO_EQUIV: u32 = 0x9;
pub const GX_LO_INV: u32 = 0xa;
pub const GX_LO_REVOR: u32 = 0xb;
pub const GX_LO_INVCOPY: u32 = 0xc;
pub const GX_LO_INVOR: u32 = 0xd;
pub const GX_LO_NAND: u32 = 0xe;
pub const GX_LO_SET: u32 = 0xf;

// GXCompCnt
pub const GX_POS_XY: u32 = 0x0;
pub const GX_POS_XYZ: u32 = 0x1;
pub const GX_NRM_XYZ: u32 = 0x0;
pub const GX_NRM_NBT: u32 = 0x1;
pub const GX_NRM_NBT3: u32 = 0x2;
pub const GX_CLR_RGB: u32 = 0x0;
pub const GX_CLR_RGBA: u32 = 0x1;
pub const GX_TEX_S: u32 = 0x0;
pub const GX_TEX_ST: u32 = 0x1;

// GXCompType
pub const GX_U8: u32 = 0x0;
pub const GX_S8: u32 = 0x1;
pub const GX_U16: u32 = 0x2;
pub const GX_S16: u32 = 0x3;
pub const GX_F32: u32 = 0x4;
pub const GX_RGB565: u32 = 0x0;
pub const GX_RGB8: u32 = 0x1;
pub const GX_RGBX8: u32 = 0x2;
pub const GX_RGBA4: u32 = 0x3;
pub const GX_RGBA6: u32 = 0x4;
pub const GX_RGBA8: u32 = 0x5;

// GXPTTexMtx
pub const GX_PTTEXMTX0: u32 = 0x40;
pub const GX_PTTEXMTX1: u32 = 0x43;
pub const GX_PTTEXMTX2: u32 = 0x46;
pub const GX_PTTEXMTX3: u32 = 0x49;
pub const GX_PTTEXMTX4: u32 = 0x4c;
pub const GX_PTTEXMTX5: u32 = 0x4f;
pub const GX_PTTEXMTX6: u32 = 0x52;
pub const GX_PTTEXMTX7: u32 = 0x55;
pub const GX_PTTEXMTX8: u32 = 0x58;
pub const GX_PTTEXMTX9: u32 = 0x5b;
pub const GX_PTTEXMTX10: u32 = 0x5e;
pub const GX_PTTEXMTX11: u32 = 0x61;
pub const GX_PTTEXMTX12: u32 = 0x64;
pub const GX_PTTEXMTX13: u32 = 0x67;
pub const GX_PTTEXMTX14: u32 = 0x6a;
pub const GX_PTTEXMTX15: u32 = 0x6d;
pub const GX_PTTEXMTX16: u32 = 0x70;
pub const GX_PTTEXMTX17: u32 = 0x73;
pub const GX_PTTEXMTX18: u32 = 0x76;
pub const GX_PTTEXMTX19: u32 = 0x79;
pub const GX_PTIDENTITY: u32 = 0x7d;

// GXTevRegID
pub const GX_TEVPREV: u32 = 0x0;
pub const GX_TEVREG0: u32 = 0x1;
pub const GX_TEVREG1: u32 = 0x2;
pub const GX_TEVREG2: u32 = 0x3;
pub const GX_MAX_TEVREG: u32 = 0x4;

// GXDiffuseFn
pub const GX_DF_NONE: u32 = 0x0;
pub const GX_DF_SIGN: u32 = 0x1;
pub const GX_DF_CLAMP: u32 = 0x2;

// GXColorSrc
pub const GX_SRC_REG: u32 = 0x0;
pub const GX_SRC_VTX: u32 = 0x1;

// GXAttnFn
pub const GX_AF_SPEC: u32 = 0x0;
pub const GX_AF_SPOT: u32 = 0x1;
pub const GX_AF_NONE: u32 = 0x2;

// GXLightID
pub const GX_LIGHT0: u32 = 0x1;
pub const GX_LIGHT1: u32 = 0x2;
pub const GX_LIGHT2: u32 = 0x4;
pub const GX_LIGHT3: u32 = 0x8;
pub const GX_LIGHT4: u32 = 0x10;
pub const GX_LIGHT5: u32 = 0x20;
pub const GX_LIGHT6: u32 = 0x40;
pub const GX_LIGHT7: u32 = 0x80;
pub const GX_MAX_LIGHT: u32 = 0x100;
pub const GX_LIGHT_NULL: u32 = 0x0;

// GXTexOffset
pub const GX_TO_ZERO: u32 = 0x0;
pub const GX_TO_SIXTEENTH: u32 = 0x1;
pub const GX_TO_EIGHTH: u32 = 0x2;
pub const GX_TO_FOURTH: u32 = 0x3;
pub const GX_TO_HALF: u32 = 0x4;
pub const GX_TO_ONE: u32 = 0x5;
pub const GX_MAX_TEXOFFSET: u32 = 0x6;

// GXSpotFn
pub const GX_SP_OFF: u32 = 0x0;
pub const GX_SP_FLAT: u32 = 0x1;
pub const GX_SP_COS: u32 = 0x2;
pub const GX_SP_COS2: u32 = 0x3;
pub const GX_SP_SHARP: u32 = 0x4;
pub const GX_SP_RING1: u32 = 0x5;
pub const GX_SP_RING2: u32 = 0x6;

// GXDistAttnFn
pub const GX_DA_OFF: u32 = 0x0;
pub const GX_DA_GENTLE: u32 = 0x1;
pub const GX_DA_MEDIUM: u32 = 0x2;
pub const GX_DA_STEEP: u32 = 0x3;

// GXCullMode
pub const GX_CULL_NONE: u32 = 0x0;
pub const GX_CULL_FRONT: u32 = 0x1;
pub const GX_CULL_BACK: u32 = 0x2;
pub const GX_CULL_ALL: u32 = 0x3;

// GXTevSwapSel
pub const GX_TEV_SWAP0: u32 = 0x0;
pub const GX_TEV_SWAP1: u32 = 0x1;
pub const GX_TEV_SWAP2: u32 = 0x2;
pub const GX_TEV_SWAP3: u32 = 0x3;
pub const GX_MAX_TEVSWAP: u32 = 0x4;

// GXTevColorChan
pub const GX_CH_RED: u32 = 0x0;
pub const GX_CH_GREEN: u32 = 0x1;
pub const GX_CH_BLUE: u32 = 0x2;
pub const GX_CH_ALPHA: u32 = 0x3;

// GXFogType
pub const GX_FOG_NONE: u32 = 0x0;
pub const GX_FOG_PERSP_LIN: u32 = 0x2;
pub const GX_FOG_PERSP_EXP: u32 = 0x4;
pub const GX_FOG_PERSP_EXP2: u32 = 0x5;
pub const GX_FOG_PERSP_REVEXP: u32 = 0x6;
pub const GX_FOG_PERSP_REVEXP2: u32 = 0x7;
pub const GX_FOG_ORTHO_LIN: u32 = 0xa;
pub const GX_FOG_ORTHO_EXP: u32 = 0xc;
pub const GX_FOG_ORTHO_EXP2: u32 = 0xd;
pub const GX_FOG_ORTHO_REVEXP: u32 = 0xe;
pub const GX_FOG_ORTHO_REVEXP2: u32 = 0xf;
pub const GX_FOG_LIN: u32 = 0x2;
pub const GX_FOG_EXP: u32 = 0x4;
pub const GX_FOG_EXP2: u32 = 0x5;
pub const GX_FOG_REVEXP: u32 = 0x6;
pub const GX_FOG_REVEXP2: u32 = 0x7;

// GXTevColorArg
pub const GX_CC_CPREV: u32 = 0x0;
pub const GX_CC_APREV: u32 = 0x1;
pub const GX_CC_C0: u32 = 0x2;
pub const GX_CC_A0: u32 = 0x3;
pub const GX_CC_C1: u32 = 0x4;
pub const GX_CC_A1: u32 = 0x5;
pub const GX_CC_C2: u32 = 0x6;
pub const GX_CC_A2: u32 = 0x7;
pub const GX_CC_TEXC: u32 = 0x8;
pub const GX_CC_TEXA: u32 = 0x9;
pub const GX_CC_RASC: u32 = 0xa;
pub const GX_CC_RASA: u32 = 0xb;
pub const GX_CC_ONE: u32 = 0xc;
pub const GX_CC_HALF: u32 = 0xd;
pub const GX_CC_KONST: u32 = 0xe;
pub const GX_CC_ZERO: u32 = 0xf;

// GXTevAlphaArg
pub const GX_CA_APREV: u32 = 0x0;
pub const GX_CA_A0: u32 = 0x1;
pub const GX_CA_A1: u32 = 0x2;
pub const GX_CA_A2: u32 = 0x3;
pub const GX_CA_TEXA: u32 = 0x4;
pub const GX_CA_RASA: u32 = 0x5;
pub const GX_CA_KONST: u32 = 0x6;
pub const GX_CA_ZERO: u32 = 0x7;

// GXTevOp
pub const GX_TEV_ADD: u32 = 0x0;
pub const GX_TEV_SUB: u32 = 0x1;
pub const GX_TEV_COMP_R8_GT: u32 = 0x8;
pub const GX_TEV_COMP_R8_EQ: u32 = 0x9;
pub const GX_TEV_COMP_GR16_GT: u32 = 0xa;
pub const GX_TEV_COMP_GR16_EQ: u32 = 0xb;
pub const GX_TEV_COMP_BGR24_GT: u32 = 0xc;
pub const GX_TEV_COMP_BGR24_EQ: u32 = 0xd;
pub const GX_TEV_COMP_RGB8_GT: u32 = 0xe;
pub const GX_TEV_COMP_RGB8_EQ: u32 = 0xf;
pub const GX_TEV_COMP_A8_GT: u32 = 0xe;
pub const GX_TEV_COMP_A8_EQ: u32 = 0xf;

// GXTevBias
pub const GX_TB_ZERO: u32 = 0x0;
pub const GX_TB_ADDHALF: u32 = 0x1;
pub const GX_TB_SUBHALF: u32 = 0x2;
pub const GX_MAX_TEVBIAS: u32 = 0x3;

// GXTevScale
pub const GX_CS_SCALE_1: u32 = 0x0;
pub const GX_CS_SCALE_2: u32 = 0x1;
pub const GX_CS_SCALE_4: u32 = 0x2;
pub const GX_CS_DIVIDE_2: u32 = 0x3;
pub const GX_MAX_TEVSCALE: u32 = 0x4;

// GXTevKColorSel
pub const GX_TEV_KCSEL_8_8: u32 = 0x0;
pub const GX_TEV_KCSEL_7_8: u32 = 0x1;
pub const GX_TEV_KCSEL_6_8: u32 = 0x2;
pub const GX_TEV_KCSEL_5_8: u32 = 0x3;
pub const GX_TEV_KCSEL_4_8: u32 = 0x4;
pub const GX_TEV_KCSEL_3_8: u32 = 0x5;
pub const GX_TEV_KCSEL_2_8: u32 = 0x6;
pub const GX_TEV_KCSEL_1_8: u32 = 0x7;
pub const GX_TEV_KCSEL_1: u32 = 0x0;
pub const GX_TEV_KCSEL_3_4: u32 = 0x2;
pub const GX_TEV_KCSEL_1_2: u32 = 0x4;
pub const GX_TEV_KCSEL_1_4: u32 = 0x6;
pub const GX_TEV_KCSEL_K0: u32 = 0xc;
pub const GX_TEV_KCSEL_K1: u32 = 0xd;
pub const GX_TEV_KCSEL_K2: u32 = 0xe;
pub const GX_TEV_KCSEL_K3: u32 = 0xf;
pub const GX_TEV_KCSEL_K0_R: u32 = 0x10;
pub const GX_TEV_KCSEL_K1_R: u32 = 0x11;
pub const GX_TEV_KCSEL_K2_R: u32 = 0x12;
pub const GX_TEV_KCSEL_K3_R: u32 = 0x13;
pub const GX_TEV_KCSEL_K0_G: u32 = 0x14;
pub const GX_TEV_KCSEL_K1_G: u32 = 0x15;
pub const GX_TEV_KCSEL_K2_G: u32 = 0x16;
pub const GX_TEV_KCSEL_K3_G: u32 = 0x17;
pub const GX_TEV_KCSEL_K0_B: u32 = 0x18;
pub const GX_TEV_KCSEL_K1_B: u32 = 0x19;
pub const GX_TEV_KCSEL_K2_B: u32 = 0x1a;
pub const GX_TEV_KCSEL_K3_B: u32 = 0x1b;
pub const GX_TEV_KCSEL_K0_A: u32 = 0x1c;
pub const GX_TEV_KCSEL_K1_A: u32 = 0x1d;
pub const GX_TEV_KCSEL_K2_A: u32 = 0x1e;
pub const GX_TEV_KCSEL_K3_A: u32 = 0x1f;

// GXTevKAlphaSel
pub const GX_TEV_KASEL_8_8: u32 = 0x0;
pub const GX_TEV_KASEL_7_8: u32 = 0x1;
pub const GX_TEV_KASEL_6_8: u32 = 0x2;
pub const GX_TEV_KASEL_5_8: u32 = 0x3;
pub const GX_TEV_KASEL_4_8: u32 = 0x4;
pub const GX_TEV_KASEL_3_8: u32 = 0x5;
pub const GX_TEV_KASEL_2_8: u32 = 0x6;
pub const GX_TEV_KASEL_1_8: u32 = 0x7;
pub const GX_TEV_KASEL_1: u32 = 0x0;
pub const GX_TEV_KASEL_3_4: u32 = 0x2;
pub const GX_TEV_KASEL_1_2: u32 = 0x4;
pub const GX_TEV_KASEL_1_4: u32 = 0x6;
pub const GX_TEV_KASEL_K0_R: u32 = 0x10;
pub const GX_TEV_KASEL_K1_R: u32 = 0x11;
pub const GX_TEV_KASEL_K2_R: u32 = 0x12;
pub const GX_TEV_KASEL_K3_R: u32 = 0x13;
pub const GX_TEV_KASEL_K0_G: u32 = 0x14;
pub const GX_TEV_KASEL_K1_G: u32 = 0x15;
pub const GX_TEV_KASEL_K2_G: u32 = 0x16;
pub const GX_TEV_KASEL_K3_G: u32 = 0x17;
pub const GX_TEV_KASEL_K0_B: u32 = 0x18;
pub const GX_TEV_KASEL_K1_B: u32 = 0x19;
pub const GX_TEV_KASEL_K2_B: u32 = 0x1a;
pub const GX_TEV_KASEL_K3_B: u32 = 0x1b;
pub const GX_TEV_KASEL_K0_A: u32 = 0x1c;
pub const GX_TEV_KASEL_K1_A: u32 = 0x1d;
pub const GX_TEV_KASEL_K2_A: u32 = 0x1e;
pub const GX_TEV_KASEL_K3_A: u32 = 0x1f;

// GXTevKColorID
pub const GX_KCOLOR0: u32 = 0x0;
pub const GX_KCOLOR1: u32 = 0x1;
pub const GX_KCOLOR2: u32 = 0x2;
pub const GX_KCOLOR3: u32 = 0x3;
pub const GX_MAX_KCOLOR: u32 = 0x4;

// GXZTexOp
pub const GX_ZT_DISABLE: u32 = 0x0;
pub const GX_ZT_ADD: u32 = 0x1;
pub const GX_ZT_REPLACE: u32 = 0x2;
pub const GX_MAX_ZTEXOP: u32 = 0x3;

// GXIndTexFormat
pub const GX_ITF_8: u32 = 0x0;
pub const GX_ITF_5: u32 = 0x1;
pub const GX_ITF_4: u32 = 0x2;
pub const GX_ITF_3: u32 = 0x3;
pub const GX_MAX_ITFORMAT: u32 = 0x4;

// GXIndTexBiasSel
pub const GX_ITB_NONE: u32 = 0x0;
pub const GX_ITB_S: u32 = 0x1;
pub const GX_ITB_T: u32 = 0x2;
pub const GX_ITB_ST: u32 = 0x3;
pub const GX_ITB_U: u32 = 0x4;
pub const GX_ITB_SU: u32 = 0x5;
pub const GX_ITB_TU: u32 = 0x6;
pub const GX_ITB_STU: u32 = 0x7;
pub const GX_MAX_ITBIAS: u32 = 0x8;

// GXIndTexAlphaSel
pub const GX_ITBA_OFF: u32 = 0x0;
pub const GX_ITBA_S: u32 = 0x1;
pub const GX_ITBA_T: u32 = 0x2;
pub const GX_ITBA_U: u32 = 0x3;
pub const GX_MAX_ITBALPHA: u32 = 0x4;

// GXIndTexMtxID
pub const GX_ITM_OFF: u32 = 0x0;
pub const GX_ITM_0: u32 = 0x1;
pub const GX_ITM_1: u32 = 0x2;
pub const GX_ITM_2: u32 = 0x3;
pub const GX_ITM_S0: u32 = 0x5;
pub const GX_ITM_S1: u32 = 0x6;
pub const GX_ITM_S2: u32 = 0x7;
pub const GX_ITM_T0: u32 = 0x9;
pub const GX_ITM_T1: u32 = 0xa;
pub const GX_ITM_T2: u32 = 0xb;

// GXIndTexWrap
pub const GX_ITW_OFF: u32 = 0x0;
pub const GX_ITW_256: u32 = 0x1;
pub const GX_ITW_128: u32 = 0x2;
pub const GX_ITW_64: u32 = 0x3;
pub const GX_ITW_32: u32 = 0x4;
pub const GX_ITW_16: u32 = 0x5;
pub const GX_ITW_0: u32 = 0x6;
pub const GX_MAX_ITWRAP: u32 = 0x7;

// GXIndTexStageID
pub const GX_INDTEXSTAGE0: u32 = 0x0;
pub const GX_INDTEXSTAGE1: u32 = 0x1;
pub const GX_INDTEXSTAGE2: u32 = 0x2;
pub const GX_INDTEXSTAGE3: u32 = 0x3;
pub const GX_MAX_INDTEXSTAGE: u32 = 0x4;

// GXIndTexScale
pub const GX_ITS_1: u32 = 0x0;
pub const GX_ITS_2: u32 = 0x1;
pub const GX_ITS_4: u32 = 0x2;
pub const GX_ITS_8: u32 = 0x3;
pub const GX_ITS_16: u32 = 0x4;
pub const GX_ITS_32: u32 = 0x5;
pub const GX_ITS_64: u32 = 0x6;
pub const GX_ITS_128: u32 = 0x7;
pub const GX_ITS_256: u32 = 0x8;
pub const GX_MAX_ITSCALE: u32 = 0x9;

// GXPerf0
pub const GX_PERF0_VERTICES: u32 = 0x0;
pub const GX_PERF0_CLIP_VTX: u32 = 0x1;
pub const GX_PERF0_CLIP_CLKS: u32 = 0x2;
pub const GX_PERF0_XF_WAIT_IN: u32 = 0x3;
pub const GX_PERF0_XF_WAIT_OUT: u32 = 0x4;
pub const GX_PERF0_XF_XFRM_CLKS: u32 = 0x5;
pub const GX_PERF0_XF_LIT_CLKS: u32 = 0x6;
pub const GX_PERF0_XF_BOT_CLKS: u32 = 0x7;
pub const GX_PERF0_XF_REGLD_CLKS: u32 = 0x8;
pub const GX_PERF0_XF_REGRD_CLKS: u32 = 0x9;
pub const GX_PERF0_CLIP_RATIO: u32 = 0xa;
pub const GX_PERF0_TRIANGLES: u32 = 0xb;
pub const GX_PERF0_TRIANGLES_CULLED: u32 = 0xc;
pub const GX_PERF0_TRIANGLES_PASSED: u32 = 0xd;
pub const GX_PERF0_TRIANGLES_SCISSORED: u32 = 0xe;
pub const GX_PERF0_TRIANGLES_0TEX: u32 = 0xf;
pub const GX_PERF0_TRIANGLES_1TEX: u32 = 0x10;
pub const GX_PERF0_TRIANGLES_2TEX: u32 = 0x11;
pub const GX_PERF0_TRIANGLES_3TEX: u32 = 0x12;
pub const GX_PERF0_TRIANGLES_4TEX: u32 = 0x13;
pub const GX_PERF0_TRIANGLES_5TEX: u32 = 0x14;
pub const GX_PERF0_TRIANGLES_6TEX: u32 = 0x15;
pub const GX_PERF0_TRIANGLES_7TEX: u32 = 0x16;
pub const GX_PERF0_TRIANGLES_8TEX: u32 = 0x17;
pub const GX_PERF0_TRIANGLES_0CLR: u32 = 0x18;
pub const GX_PERF0_TRIANGLES_1CLR: u32 = 0x19;
pub const GX_PERF0_TRIANGLES_2CLR: u32 = 0x1a;
pub const GX_PERF0_QUAD_0CVG: u32 = 0x1b;
pub const GX_PERF0_QUAD_NON0CVG: u32 = 0x1c;
pub const GX_PERF0_QUAD_1CVG: u32 = 0x1d;
pub const GX_PERF0_QUAD_2CVG: u32 = 0x1e;
pub const GX_PERF0_QUAD_3CVG: u32 = 0x1f;
pub const GX_PERF0_QUAD_4CVG: u32 = 0x20;
pub const GX_PERF0_AVG_QUAD_CNT: u32 = 0x21;
pub const GX_PERF0_CLOCKS: u32 = 0x22;
pub const GX_PERF0_NONE: u32 = 0x23;

// GXPerf1
pub const GX_PERF1_TEXELS: u32 = 0x0;
pub const GX_PERF1_TX_IDLE: u32 = 0x1;
pub const GX_PERF1_TX_REGS: u32 = 0x2;
pub const GX_PERF1_TX_MEMSTALL: u32 = 0x3;
pub const GX_PERF1_TC_CHECK1_2: u32 = 0x4;
pub const GX_PERF1_TC_CHECK3_4: u32 = 0x5;
pub const GX_PERF1_TC_CHECK5_6: u32 = 0x6;
pub const GX_PERF1_TC_CHECK7_8: u32 = 0x7;
pub const GX_PERF1_TC_MISS: u32 = 0x8;
pub const GX_PERF1_VC_ELEMQ_FULL: u32 = 0x9;
pub const GX_PERF1_VC_MISSQ_FULL: u32 = 0xa;
pub const GX_PERF1_VC_MEMREQ_FULL: u32 = 0xb;
pub const GX_PERF1_VC_STATUS7: u32 = 0xc;
pub const GX_PERF1_VC_MISSREP_FULL: u32 = 0xd;
pub const GX_PERF1_VC_STREAMBUF_LOW: u32 = 0xe;
pub const GX_PERF1_VC_ALL_STALLS: u32 = 0xf;
pub const GX_PERF1_VERTICES: u32 = 0x10;
pub const GX_PERF1_FIFO_REQ: u32 = 0x11;
pub const GX_PERF1_CALL_REQ: u32 = 0x12;
pub const GX_PERF1_VC_MISS_REQ: u32 = 0x13;
pub const GX_PERF1_CP_ALL_REQ: u32 = 0x14;
pub const GX_PERF1_CLOCKS: u32 = 0x15;
pub const GX_PERF1_NONE: u32 = 0x16;

// GXVCachePerf
pub const GX_VC_POS: u32 = 0x0;
pub const GX_VC_NRM: u32 = 0x1;
pub const GX_VC_CLR0: u32 = 0x2;
pub const GX_VC_CLR1: u32 = 0x3;
pub const GX_VC_TEX0: u32 = 0x4;
pub const GX_VC_TEX1: u32 = 0x5;
pub const GX_VC_TEX2: u32 = 0x6;
pub const GX_VC_TEX3: u32 = 0x7;
pub const GX_VC_TEX4: u32 = 0x8;
pub const GX_VC_TEX5: u32 = 0x9;
pub const GX_VC_TEX6: u32 = 0xa;
pub const GX_VC_TEX7: u32 = 0xb;
pub const GX_VC_ALL: u32 = 0xf;

// GXClipMode
pub const GX_CLIP_ENABLE: u32 = 0x0;
pub const GX_CLIP_DISABLE: u32 = 0x1;

// GXFBClamp
pub const GX_CLAMP_NONE: u32 = 0x0;
pub const GX_CLAMP_TOP: u32 = 0x1;
pub const GX_CLAMP_BOTTOM: u32 = 0x2;

// GXTlut
pub const GX_TLUT0: u32 = 0x0;
pub const GX_TLUT1: u32 = 0x1;
pub const GX_TLUT2: u32 = 0x2;
pub const GX_TLUT3: u32 = 0x3;
pub const GX_TLUT4: u32 = 0x4;
pub const GX_TLUT5: u32 = 0x5;
pub const GX_TLUT6: u32 = 0x6;
pub const GX_TLUT7: u32 = 0x7;
pub const GX_TLUT8: u32 = 0x8;
pub const GX_TLUT9: u32 = 0x9;
pub const GX_TLUT10: u32 = 0xa;
pub const GX_TLUT11: u32 = 0xb;
pub const GX_TLUT12: u32 = 0xc;
pub const GX_TLUT13: u32 = 0xd;
pub const GX_TLUT14: u32 = 0xe;
pub const GX_TLUT15: u32 = 0xf;
pub const GX_BIGTLUT0: u32 = 0x10;
pub const GX_BIGTLUT1: u32 = 0x11;
pub const GX_BIGTLUT2: u32 = 0x12;
pub const GX_BIGTLUT3: u32 = 0x13;

// GXTlutFmt
pub const GX_TL_IA8: u32 = 0x0;
pub const GX_TL_RGB565: u32 = 0x1;
pub const GX_TL_RGB5A3: u32 = 0x2;
pub const GX_MAX_TLUTFMT: u32 = 0x3;

// GXTlutSize
pub const GX_TLUT_16: u32 = 0x1;
pub const GX_TLUT_32: u32 = 0x2;
pub const GX_TLUT_64: u32 = 0x4;
pub const GX_TLUT_128: u32 = 0x8;
pub const GX_TLUT_256: u32 = 0x10;
pub const GX_TLUT_512: u32 = 0x20;
pub const GX_TLUT_1K: u32 = 0x40;
pub const GX_TLUT_2K: u32 = 0x80;
pub const GX_TLUT_4K: u32 = 0x100;
pub const GX_TLUT_8K: u32 = 0x200;
pub const GX_TLUT_16K: u32 = 0x400;

// GXAlphaReadMode
pub const GX_READ_00: u32 = 0x0;
pub const GX_READ_FF: u32 = 0x1;
pub const GX_READ_NONE: u32 = 0x2;

// GXTexCacheSize
pub const GX_TEXCACHE_32K: u32 = 0x0;
pub const GX_TEXCACHE_128K: u32 = 0x1;
pub const GX_TEXCACHE_512K: u32 = 0x2;
pub const GX_TEXCACHE_NONE: u32 = 0x3;

// GXMiscToken
pub const GX_MT_XF_FLUSH: u32 = 0x1;
pub const GX_MT_DL_SAVE_CONTEXT: u32 = 0x2;
pub const GX_MT_NULL: u32 = 0x0;

// GXCommand
pub const GX_CMD_LOAD_INDX_A: u32 = 0x20;
pub const GX_CMD_LOAD_INDX_B: u32 = 0x28;
pub const GX_CMD_LOAD_INDX_C: u32 = 0x30;
pub const GX_CMD_LOAD_INDX_D: u32 = 0x38;
pub const GX_CMD_LOAD_CP_REG: u32 = 0x8;
pub const GX_CMD_LOAD_XF_REG: u32 = 0x10;
pub const GX_CMD_LOAD_BP_REG: u32 = 0x61;

// GXCPRegs
pub const GX_CP_REG_MTXIDXA: u32 = 0x30;
pub const GX_CP_REG_MTXIDXB: u32 = 0x40;
pub const GX_CP_REG_VCD_LO: u32 = 0x50;
pub const GX_CP_REG_VCD_HI: u32 = 0x60;
pub const GX_CP_REG_VAT_GRP0: u32 = 0x70;
pub const GX_CP_REG_VAT_GRP1: u32 = 0x80;
pub const GX_CP_REG_VAT_GRP2: u32 = 0x90;
pub const GX_CP_REG_ARRAYBASE: u32 = 0xa0;
pub const GX_CP_REG_ARRAYSTRIDE: u32 = 0xb0;

// GXCommandList.h
pub const GX_NOP: u8 = 0x00;
pub const GX_LOAD_BP_REG: u8 = 0x61;
pub const GX_LOAD_CP_REG: u8 = 0x08;
pub const GX_LOAD_XF_REG: u8 = 0x10;
pub const GX_LOAD_INDX_A: u8 = 0x20;
pub const GX_LOAD_INDX_B: u8 = 0x28;
pub const GX_LOAD_INDX_C: u8 = 0x30;
pub const GX_LOAD_INDX_D: u8 = 0x38;
pub const GX_CMD_CALL_DL: u8 = 0x40;
pub const GX_CMD_INVL_VC: u8 = 0x48;
pub const GX_AURORA: u8 = 0x50;
pub const GX_OPCODE_MASK: u8 = 0xF8;
pub const GX_VAT_MASK: u8 = 0x07;

