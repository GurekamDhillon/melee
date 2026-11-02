/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 GurekamDhillon
 *
 * geno_behaviours.inc - Geno's v5.1 behaviour procedures: glide, tornado, drill, cape, multi-dash,
 * anim_motion. Included by geno_game_v2.inc (one translation unit with geno_game.c, so the helpers
 * there are visible). Implemented from docs/specs/geno-v5.1-behaviours.md alone.
 *
 * Layout: (1) the pure procedures, each a step function over plain structs (geno_behaviours.h);
 * (2) the engine adapters that map a fighter onto them and supply the callbacks geno_game_v2.inc's
 * behaviour tables name.
 *
 * Numeric rules (spec section 1): f32 throughout, no fused multiply-add. Every product is written
 * into a named temporary on its own statement and added on the next, in the order the spec writes the
 * operations. Do not "tidy" these into a*b+c expressions: the compiler may contract them.
 */

#include "geno_behaviours.h"

/* ---- 1. numeric helpers ------------------------------------------------------------------------- */

static f32 gb_floor(f32 x)
{
    s32 i = (s32) x;
    f32 fi = (f32) i;
    return fi > x ? fi - 1.0f : fi;
}

static f32 gb_clamp(f32 x, f32 lo, f32 hi)
{
    if (x < lo) {
        return lo;
    } else if (x > hi) {
        return hi;
    }
    return x;
}

static f32 gb_approach(f32 x, f32 target, f32 step)
{
    f32 x2;
    if (x < target) {
        x2 = x + step;
        if (x2 > target) {
            return target;
        }
        return x2;
    }
    x2 = x - step;
    if (x2 < target) {
        return target;
    }
    return x2;
}

static f32 gb_abs(f32 x)
{
    return x < 0.0f ? -x : x;
}

/* The quarter-wave table (spec 1.1): sin(i * 90 / 1024 degrees) for i = 0..1024, each the f32 nearest
 * the exact value (computed offline at 300 bits and rounded to nearest-even), as bit patterns.
 * GenoBhv_SinqCrc32 is checked against the committed CRC in the test suite. */
static const u32 gb_sinq[GENO_BHV_SINQ_N] = {
    0x00000000, 0x3AC90FD5, 0x3B490FC6, 0x3B96CBC1, 0x3BC90F88, 0x3BFB5330, 0x3C16CB58, 0x3C2FED02,
    0x3C490E90, 0x3C622FFF, 0x3C7B514B, 0x3C8A3938, 0x3C96C9B6, 0x3CA35A1C, 0x3CAFEA69, 0x3CBC7A9B,
    0x3CC90AB0, 0x3CD59AA6, 0x3CE22A7A, 0x3CEEBA2C, 0x3CFB49BA, 0x3D03EC90, 0x3D0A342F, 0x3D107BB8,
    0x3D16C32C, 0x3D1D0A88, 0x3D2351CB, 0x3D2998F6, 0x3D2FE007, 0x3D3626FC, 0x3D3C6DD5, 0x3D42B491,
    0x3D48FB30, 0x3D4F41AF, 0x3D55880E, 0x3D5BCE4C, 0x3D621469, 0x3D685A62, 0x3D6EA038, 0x3D74E5E9,
    0x3D7B2B74, 0x3D80B86C, 0x3D83DB0A, 0x3D86FD94, 0x3D8A200A, 0x3D8D426A, 0x3D9064B4, 0x3D9386E7,
    0x3D96A905, 0x3D99CB0A, 0x3D9CECF9, 0x3DA00ECF, 0x3DA3308C, 0x3DA65230, 0x3DA973BA, 0x3DAC952B,
    0x3DAFB680, 0x3DB2D7BB, 0x3DB5F8DA, 0x3DB919DD, 0x3DBC3AC3, 0x3DBF5B8D, 0x3DC27C39, 0x3DC59CC6,
    0x3DC8BD36, 0x3DCBDD86, 0x3DCEFDB7, 0x3DD21DC8, 0x3DD53DB9, 0x3DD85D89, 0x3DDB7D37, 0x3DDE9CC4,
    0x3DE1BC2E, 0x3DE4DB76, 0x3DE7FA9A, 0x3DEB199A, 0x3DEE3876, 0x3DF1572E, 0x3DF475C0, 0x3DF7942C,
    0x3DFAB273, 0x3DFDD092, 0x3E007745, 0x3E02062E, 0x3E039502, 0x3E0523C2, 0x3E06B26E, 0x3E084105,
    0x3E09CF86, 0x3E0B5DF3, 0x3E0CEC4A, 0x3E0E7A8B, 0x3E1008B7, 0x3E1196CC, 0x3E1324CA, 0x3E14B2B2,
    0x3E164083, 0x3E17CE3D, 0x3E195BE0, 0x3E1AE96B, 0x3E1C76DE, 0x3E1E0438, 0x3E1F917B, 0x3E211EA5,
    0x3E22ABB6, 0x3E2438AD, 0x3E25C58C, 0x3E275251, 0x3E28DEFC, 0x3E2A6B8D, 0x3E2BF804, 0x3E2D8461,
    0x3E2F10A2, 0x3E309CC9, 0x3E3228D4, 0x3E33B4C4, 0x3E354098, 0x3E36CC50, 0x3E3857EC, 0x3E39E36C,
    0x3E3B6ECF, 0x3E3CFA15, 0x3E3E853E, 0x3E401049, 0x3E419B37, 0x3E432607, 0x3E44B0B9, 0x3E463B4D,
    0x3E47C5C2, 0x3E495018, 0x3E4ADA4F, 0x3E4C6467, 0x3E4DEE60, 0x3E4F7838, 0x3E5101F1, 0x3E528B89,
    0x3E541501, 0x3E559E58, 0x3E57278F, 0x3E58B0A4, 0x3E5A3997, 0x3E5BC26A, 0x3E5D4B1A, 0x3E5ED3A8,
    0x3E605C13, 0x3E61E45C, 0x3E636C83, 0x3E64F486, 0x3E667C66, 0x3E680422, 0x3E698BBA, 0x3E6B132F,
    0x3E6C9A7F, 0x3E6E21AB, 0x3E6FA8B2, 0x3E712F94, 0x3E72B651, 0x3E743CE8, 0x3E75C35A, 0x3E7749A6,
    0x3E78CFCC, 0x3E7A55CB, 0x3E7BDBA4, 0x3E7D6156, 0x3E7EE6E1, 0x3E803622, 0x3E80F8C0, 0x3E81BB4A,
    0x3E827DC0, 0x3E834022, 0x3E840270, 0x3E84C4AA, 0x3E8586CE, 0x3E8648DF, 0x3E870ADA, 0x3E87CCC1,
    0x3E888E93, 0x3E895050, 0x3E8A11F7, 0x3E8AD38A, 0x3E8B9507, 0x3E8C566E, 0x3E8D17C0, 0x3E8DD8FC,
    0x3E8E9A22, 0x3E8F5B32, 0x3E901C2C, 0x3E90DD10, 0x3E919DDD, 0x3E925E94, 0x3E931F35, 0x3E93DFBF,
    0x3E94A031, 0x3E95608D, 0x3E9620D2, 0x3E96E100, 0x3E97A117, 0x3E986116, 0x3E9920FE, 0x3E99E0CE,
    0x3E9AA086, 0x3E9B6027, 0x3E9C1FAF, 0x3E9CDF20, 0x3E9D9E78, 0x3E9E5DB8, 0x3E9F1CDF, 0x3E9FDBEE,
    0x3EA09AE5, 0x3EA159C2, 0x3EA21887, 0x3EA2D733, 0x3EA395C5, 0x3EA4543F, 0x3EA5129F, 0x3EA5D0E5,
    0x3EA68F12, 0x3EA74D25, 0x3EA80B1F, 0x3EA8C8FE, 0x3EA986C4, 0x3EAA446F, 0x3EAB0201, 0x3EABBF77,
    0x3EAC7CD4, 0x3EAD3A15, 0x3EADF73C, 0x3EAEB449, 0x3EAF713A, 0x3EB02E10, 0x3EB0EACB, 0x3EB1A76B,
    0x3EB263EF, 0x3EB32058, 0x3EB3DCA5, 0x3EB498D6, 0x3EB554EC, 0x3EB610E6, 0x3EB6CCC3, 0x3EB78884,
    0x3EB8442A, 0x3EB8FFB2, 0x3EB9BB1E, 0x3EBA766E, 0x3EBB31A0, 0x3EBBECB6, 0x3EBCA7AF, 0x3EBD628B,
    0x3EBE1D4A, 0x3EBED7EB, 0x3EBF926F, 0x3EC04CD5, 0x3EC1071E, 0x3EC1C148, 0x3EC27B55, 0x3EC33544,
    0x3EC3EF15, 0x3EC4A8C8, 0x3EC5625C, 0x3EC61BD2, 0x3EC6D529, 0x3EC78E62, 0x3EC8477C, 0x3EC90077,
    0x3EC9B953, 0x3ECA7210, 0x3ECB2AAE, 0x3ECBE32C, 0x3ECC9B8B, 0x3ECD53CA, 0x3ECE0BEA, 0x3ECEC3EA,
    0x3ECF7BCA, 0x3ED0338A, 0x3ED0EB2A, 0x3ED1A2AA, 0x3ED25A09, 0x3ED31148, 0x3ED3C867, 0x3ED47F64,
    0x3ED53641, 0x3ED5ECFD, 0x3ED6A399, 0x3ED75A13, 0x3ED8106B, 0x3ED8C6A3, 0x3ED97CB9, 0x3EDA32AD,
    0x3EDAE880, 0x3EDB9E31, 0x3EDC53C1, 0x3EDD092E, 0x3EDDBE79, 0x3EDE73A2, 0x3EDF28A9, 0x3EDFDD8D,
    0x3EE0924F, 0x3EE146EE, 0x3EE1FB6A, 0x3EE2AFC4, 0x3EE363FA, 0x3EE4180E, 0x3EE4CBFE, 0x3EE57FCB,
    0x3EE63375, 0x3EE6E6FB, 0x3EE79A5D, 0x3EE84D9C, 0x3EE900B7, 0x3EE9B3AE, 0x3EEA6681, 0x3EEB1930,
    0x3EEBCBBB, 0x3EEC7E21, 0x3EED3063, 0x3EEDE280, 0x3EEE9479, 0x3EEF464C, 0x3EEFF7FB, 0x3EF0A985,
    0x3EF15AEA, 0x3EF20C29, 0x3EF2BD43, 0x3EF36E38, 0x3EF41F07, 0x3EF4CFB1, 0x3EF58035, 0x3EF63093,
    0x3EF6E0CB, 0x3EF790DC, 0x3EF840C8, 0x3EF8F08E, 0x3EF9A02D, 0x3EFA4FA5, 0x3EFAFEF7, 0x3EFBAE22,
    0x3EFC5D27, 0x3EFD0C04, 0x3EFDBABB, 0x3EFE694A, 0x3EFF17B2, 0x3EFFC5F3, 0x3F003A06, 0x3F0090FF,
    0x3F00E7E4, 0x3F013EB5, 0x3F019573, 0x3F01EC1C, 0x3F0242B1, 0x3F029932, 0x3F02EF9F, 0x3F0345F8,
    0x3F039C3D, 0x3F03F26D, 0x3F044889, 0x3F049E91, 0x3F04F484, 0x3F054A62, 0x3F05A02C, 0x3F05F5E2,
    0x3F064B82, 0x3F06A10E, 0x3F06F686, 0x3F074BE8, 0x3F07A136, 0x3F07F66F, 0x3F084B92, 0x3F08A0A1,
    0x3F08F59B, 0x3F094A7F, 0x3F099F4E, 0x3F09F409, 0x3F0A48AD, 0x3F0A9D3D, 0x3F0AF1B7, 0x3F0B461C,
    0x3F0B9A6B, 0x3F0BEEA5, 0x3F0C42C9, 0x3F0C96D7, 0x3F0CEAD0, 0x3F0D3EB3, 0x3F0D9281, 0x3F0DE638,
    0x3F0E39DA, 0x3F0E8D65, 0x3F0EE0DB, 0x3F0F343B, 0x3F0F8784, 0x3F0FDAB8, 0x3F102DD5, 0x3F1080DC,
    0x3F10D3CD, 0x3F1126A7, 0x3F11796B, 0x3F11CC19, 0x3F121EB0, 0x3F127130, 0x3F12C39A, 0x3F1315EE,
    0x3F13682A, 0x3F13BA50, 0x3F140C5F, 0x3F145E58, 0x3F14B039, 0x3F150204, 0x3F1553B7, 0x3F15A554,
    0x3F15F6D9, 0x3F164847, 0x3F16999F, 0x3F16EADE, 0x3F173C07, 0x3F178D18, 0x3F17DE12, 0x3F182EF5,
    0x3F187FC0, 0x3F18D073, 0x3F19210F, 0x3F197194, 0x3F19C200, 0x3F1A1255, 0x3F1A6293, 0x3F1AB2B8,
    0x3F1B02C6, 0x3F1B52BB, 0x3F1BA299, 0x3F1BF25F, 0x3F1C420C, 0x3F1C91A2, 0x3F1CE11F, 0x3F1D3084,
    0x3F1D7FD1, 0x3F1DCF06, 0x3F1E1E22, 0x3F1E6D26, 0x3F1EBC12, 0x3F1F0AE5, 0x3F1F599F, 0x3F1FA841,
    0x3F1FF6CB, 0x3F20453B, 0x3F209393, 0x3F20E1D2, 0x3F212FF9, 0x3F217E06, 0x3F21CBFB, 0x3F2219D7,
    0x3F226799, 0x3F22B543, 0x3F2302D3, 0x3F23504B, 0x3F239DA9, 0x3F23EAEE, 0x3F24381A, 0x3F24852C,
    0x3F24D225, 0x3F251F04, 0x3F256BCB, 0x3F25B877, 0x3F26050A, 0x3F265184, 0x3F269DE3, 0x3F26EA2A,
    0x3F273656, 0x3F278268, 0x3F27CE61, 0x3F281A40, 0x3F286605, 0x3F28B1B0, 0x3F28FD41, 0x3F2948B8,
    0x3F299415, 0x3F29DF57, 0x3F2A2A80, 0x3F2A758E, 0x3F2AC082, 0x3F2B0B5B, 0x3F2B561B, 0x3F2BA0BF,
    0x3F2BEB4A, 0x3F2C35B9, 0x3F2C800F, 0x3F2CCA49, 0x3F2D1469, 0x3F2D5E6F, 0x3F2DA859, 0x3F2DF229,
    0x3F2E3BDE, 0x3F2E8578, 0x3F2ECEF7, 0x3F2F185B, 0x3F2F61A5, 0x3F2FAAD3, 0x3F2FF3E6, 0x3F303CDE,
    0x3F3085BB, 0x3F30CE7C, 0x3F311722, 0x3F315FAD, 0x3F31A81D, 0x3F31F071, 0x3F3238AA, 0x3F3280C7,
    0x3F32C8C9, 0x3F3310AF, 0x3F33587A, 0x3F33A029, 0x3F33E7BC, 0x3F342F34, 0x3F34768F, 0x3F34BDCF,
    0x3F3504F3, 0x3F354BFB, 0x3F3592E7, 0x3F35D9B8, 0x3F36206C, 0x3F366704, 0x3F36AD7F, 0x3F36F3DF,
    0x3F373A23, 0x3F37804A, 0x3F37C655, 0x3F380C43, 0x3F385216, 0x3F3897CB, 0x3F38DD65, 0x3F3922E1,
    0x3F396842, 0x3F39AD85, 0x3F39F2AC, 0x3F3A37B7, 0x3F3A7CA4, 0x3F3AC175, 0x3F3B0629, 0x3F3B4AC1,
    0x3F3B8F3B, 0x3F3BD398, 0x3F3C17D9, 0x3F3C5BFC, 0x3F3CA003, 0x3F3CE3EC, 0x3F3D27B8, 0x3F3D6B67,
    0x3F3DAEF9, 0x3F3DF26E, 0x3F3E35C5, 0x3F3E78FF, 0x3F3EBC1B, 0x3F3EFF1B, 0x3F3F41FC, 0x3F3F84C0,
    0x3F3FC767, 0x3F4009F0, 0x3F404C5C, 0x3F408EA9, 0x3F40D0DA, 0x3F4112EC, 0x3F4154E1, 0x3F4196B7,
    0x3F41D870, 0x3F421A0B, 0x3F425B89, 0x3F429CE8, 0x3F42DE29, 0x3F431F4C, 0x3F436051, 0x3F43A138,
    0x3F43E200, 0x3F4422AB, 0x3F446337, 0x3F44A3A5, 0x3F44E3F5, 0x3F452426, 0x3F456439, 0x3F45A42D,
    0x3F45E403, 0x3F4623BB, 0x3F466354, 0x3F46A2CE, 0x3F46E22A, 0x3F472167, 0x3F476085, 0x3F479F84,
    0x3F47DE65, 0x3F481D27, 0x3F485BCA, 0x3F489A4E, 0x3F48D8B3, 0x3F4916FA, 0x3F495521, 0x3F499329,
    0x3F49D112, 0x3F4A0EDC, 0x3F4A4C87, 0x3F4A8A13, 0x3F4AC77F, 0x3F4B04CC, 0x3F4B41FA, 0x3F4B7F09,
    0x3F4BBBF8, 0x3F4BF8C7, 0x3F4C3578, 0x3F4C7208, 0x3F4CAE79, 0x3F4CEACB, 0x3F4D26FD, 0x3F4D6310,
    0x3F4D9F02, 0x3F4DDAD5, 0x3F4E1689, 0x3F4E521C, 0x3F4E8D90, 0x3F4EC8E4, 0x3F4F0417, 0x3F4F3F2B,
    0x3F4F7A1F, 0x3F4FB4F4, 0x3F4FEFA8, 0x3F502A3B, 0x3F5064AF, 0x3F509F03, 0x3F50D937, 0x3F51134A,
    0x3F514D3D, 0x3F518710, 0x3F51C0C2, 0x3F51FA54, 0x3F5233C6, 0x3F526D18, 0x3F52A649, 0x3F52DF59,
    0x3F531849, 0x3F535118, 0x3F5389C7, 0x3F53C255, 0x3F53FAC3, 0x3F54330F, 0x3F546B3B, 0x3F54A347,
    0x3F54DB31, 0x3F5512FB, 0x3F554AA4, 0x3F55822C, 0x3F55B993, 0x3F55F0D9, 0x3F5627FE, 0x3F565F02,
    0x3F5695E5, 0x3F56CCA7, 0x3F570348, 0x3F5739C7, 0x3F577026, 0x3F57A663, 0x3F57DC7F, 0x3F581279,
    0x3F584853, 0x3F587E0B, 0x3F58B3A1, 0x3F58E916, 0x3F591E6A, 0x3F59539C, 0x3F5988AD, 0x3F59BD9C,
    0x3F59F26A, 0x3F5A2716, 0x3F5A5BA0, 0x3F5A9009, 0x3F5AC450, 0x3F5AF875, 0x3F5B2C79, 0x3F5B605A,
    0x3F5B941A, 0x3F5BC7B8, 0x3F5BFB34, 0x3F5C2E8E, 0x3F5C61C7, 0x3F5C94DD, 0x3F5CC7D1, 0x3F5CFAA3,
    0x3F5D2D53, 0x3F5D5FE1, 0x3F5D924D, 0x3F5DC497, 0x3F5DF6BE, 0x3F5E28C3, 0x3F5E5AA6, 0x3F5E8C67,
    0x3F5EBE05, 0x3F5EEF81, 0x3F5F20DB, 0x3F5F5212, 0x3F5F8327, 0x3F5FB419, 0x3F5FE4E9, 0x3F601596,
    0x3F604621, 0x3F607689, 0x3F60A6CF, 0x3F60D6F2, 0x3F6106F2, 0x3F6136D0, 0x3F61668A, 0x3F619622,
    0x3F61C598, 0x3F61F4EA, 0x3F62241A, 0x3F625326, 0x3F628210, 0x3F62B0D7, 0x3F62DF7B, 0x3F630DFC,
    0x3F633C5A, 0x3F636A95, 0x3F6398AC, 0x3F63C6A1, 0x3F63F473, 0x3F642221, 0x3F644FAC, 0x3F647D14,
    0x3F64AA59, 0x3F64D77B, 0x3F650479, 0x3F653154, 0x3F655E0B, 0x3F658AA0, 0x3F65B710, 0x3F65E35E,
    0x3F660F88, 0x3F663B8E, 0x3F666771, 0x3F669330, 0x3F66BECC, 0x3F66EA45, 0x3F671599, 0x3F6740CA,
    0x3F676BD8, 0x3F6796C1, 0x3F67C187, 0x3F67EC29, 0x3F6816A8, 0x3F684103, 0x3F686B39, 0x3F68954C,
    0x3F68BF3C, 0x3F68E907, 0x3F6912AE, 0x3F693C32, 0x3F696591, 0x3F698ECC, 0x3F69B7E4, 0x3F69E0D7,
    0x3F6A09A7, 0x3F6A3252, 0x3F6A5AD9, 0x3F6A833C, 0x3F6AAB7B, 0x3F6AD395, 0x3F6AFB8C, 0x3F6B235E,
    0x3F6B4B0C, 0x3F6B7295, 0x3F6B99FB, 0x3F6BC13B, 0x3F6BE858, 0x3F6C0F50, 0x3F6C3624, 0x3F6C5CD4,
    0x3F6C835E, 0x3F6CA9C5, 0x3F6CD007, 0x3F6CF624, 0x3F6D1C1D, 0x3F6D41F2, 0x3F6D67A1, 0x3F6D8D2D,
    0x3F6DB293, 0x3F6DD7D5, 0x3F6DFCF2, 0x3F6E21EB, 0x3F6E46BE, 0x3F6E6B6D, 0x3F6E8FF8, 0x3F6EB45D,
    0x3F6ED89E, 0x3F6EFCBA, 0x3F6F20B0, 0x3F6F4483, 0x3F6F6830, 0x3F6F8BB8, 0x3F6FAF1B, 0x3F6FD25A,
    0x3F6FF573, 0x3F701867, 0x3F703B37, 0x3F705DE1, 0x3F708066, 0x3F70A2C6, 0x3F70C501, 0x3F70E717,
    0x3F710908, 0x3F712AD4, 0x3F714C7A, 0x3F716DFB, 0x3F718F57, 0x3F71B08E, 0x3F71D19F, 0x3F71F28C,
    0x3F721352, 0x3F7233F4, 0x3F725470, 0x3F7274C7, 0x3F7294F8, 0x3F72B504, 0x3F72D4EB, 0x3F72F4AC,
    0x3F731447, 0x3F7333BE, 0x3F73530E, 0x3F737239, 0x3F73913F, 0x3F73B01F, 0x3F73CED9, 0x3F73ED6E,
    0x3F740BDD, 0x3F742A27, 0x3F74484B, 0x3F746649, 0x3F748422, 0x3F74A1D5, 0x3F74BF62, 0x3F74DCC9,
    0x3F74FA0B, 0x3F751727, 0x3F75341D, 0x3F7550ED, 0x3F756D97, 0x3F758A1C, 0x3F75A67B, 0x3F75C2B3,
    0x3F75DEC6, 0x3F75FAB3, 0x3F76167A, 0x3F76321B, 0x3F764D97, 0x3F7668EC, 0x3F76841B, 0x3F769F24,
    0x3F76BA07, 0x3F76D4C4, 0x3F76EF5B, 0x3F7709CC, 0x3F772417, 0x3F773E3C, 0x3F77583A, 0x3F777213,
    0x3F778BC5, 0x3F77A551, 0x3F77BEB7, 0x3F77D7F7, 0x3F77F110, 0x3F780A04, 0x3F7822D1, 0x3F783B77,
    0x3F7853F8, 0x3F786C52, 0x3F788486, 0x3F789C93, 0x3F78B47B, 0x3F78CC3B, 0x3F78E3D6, 0x3F78FB4A,
    0x3F791298, 0x3F7929BF, 0x3F7940C0, 0x3F79579A, 0x3F796E4E, 0x3F7984DC, 0x3F799B43, 0x3F79B183,
    0x3F79C79D, 0x3F79DD91, 0x3F79F35E, 0x3F7A0904, 0x3F7A1E84, 0x3F7A33DD, 0x3F7A4910, 0x3F7A5E1C,
    0x3F7A7302, 0x3F7A87C1, 0x3F7A9C59, 0x3F7AB0CB, 0x3F7AC516, 0x3F7AD93A, 0x3F7AED37, 0x3F7B010E,
    0x3F7B14BE, 0x3F7B2848, 0x3F7B3BAB, 0x3F7B4EE7, 0x3F7B61FC, 0x3F7B74EA, 0x3F7B87B2, 0x3F7B9A53,
    0x3F7BACCD, 0x3F7BBF20, 0x3F7BD14D, 0x3F7BE353, 0x3F7BF531, 0x3F7C06E9, 0x3F7C187A, 0x3F7C29E5,
    0x3F7C3B28, 0x3F7C4C44, 0x3F7C5D3A, 0x3F7C6E08, 0x3F7C7EB0, 0x3F7C8F31, 0x3F7C9F8A, 0x3F7CAFBD,
    0x3F7CBFC9, 0x3F7CCFAE, 0x3F7CDF6C, 0x3F7CEF03, 0x3F7CFE73, 0x3F7D0DBC, 0x3F7D1CDD, 0x3F7D2BD8,
    0x3F7D3AAC, 0x3F7D4959, 0x3F7D57DE, 0x3F7D663D, 0x3F7D7474, 0x3F7D8285, 0x3F7D906E, 0x3F7D9E30,
    0x3F7DABCC, 0x3F7DB940, 0x3F7DC68C, 0x3F7DD3B2, 0x3F7DE0B1, 0x3F7DED88, 0x3F7DFA38, 0x3F7E06C2,
    0x3F7E1324, 0x3F7E1F5E, 0x3F7E2B72, 0x3F7E375E, 0x3F7E4323, 0x3F7E4EC1, 0x3F7E5A38, 0x3F7E6588,
    0x3F7E70B0, 0x3F7E7BB1, 0x3F7E868B, 0x3F7E913D, 0x3F7E9BC9, 0x3F7EA62D, 0x3F7EB069, 0x3F7EBA7F,
    0x3F7EC46D, 0x3F7ECE34, 0x3F7ED7D4, 0x3F7EE14C, 0x3F7EEA9D, 0x3F7EF3C7, 0x3F7EFCC9, 0x3F7F05A4,
    0x3F7F0E58, 0x3F7F16E4, 0x3F7F1F49, 0x3F7F2787, 0x3F7F2F9D, 0x3F7F378C, 0x3F7F3F54, 0x3F7F46F4,
    0x3F7F4E6D, 0x3F7F55BF, 0x3F7F5CE9, 0x3F7F63EC, 0x3F7F6AC7, 0x3F7F717B, 0x3F7F7808, 0x3F7F7E6D,
    0x3F7F84AB, 0x3F7F8AC2, 0x3F7F90B1, 0x3F7F9678, 0x3F7F9C18, 0x3F7FA191, 0x3F7FA6E3, 0x3F7FAC0D,
    0x3F7FB10F, 0x3F7FB5EA, 0x3F7FBA9E, 0x3F7FBF2A, 0x3F7FC38F, 0x3F7FC7CC, 0x3F7FCBE2, 0x3F7FCFD1,
    0x3F7FD397, 0x3F7FD737, 0x3F7FDAAF, 0x3F7FDE00, 0x3F7FE129, 0x3F7FE42B, 0x3F7FE705, 0x3F7FE9B8,
    0x3F7FEC43, 0x3F7FEEA7, 0x3F7FF0E3, 0x3F7FF2F8, 0x3F7FF4E6, 0x3F7FF6AC, 0x3F7FF84A, 0x3F7FF9C1,
    0x3F7FFB11, 0x3F7FFC39, 0x3F7FFD39, 0x3F7FFE13, 0x3F7FFEC4, 0x3F7FFF4E, 0x3F7FFFB1, 0x3F7FFFEC,
    0x3F800000,
};

static f32 gb_sinq_at(int i)
{
    union {
        u32 u;
        f32 f;
    } b;
    b.u = gb_sinq[i];
    return b.f;
}

u32 GenoBhv_SinqWord(int i)
{
    return i >= 0 && i < GENO_BHV_SINQ_N ? gb_sinq[i] : 0u;
}

u32 GenoBhv_SinqCrc32(void)
{
    u32 crc = 0xFFFFFFFFu;
    int i, b, k;
    for (i = 0; i < GENO_BHV_SINQ_N; i++) {
        u32 w = gb_sinq[i];
        for (b = 0; b < 4; b++) {
            crc ^= (w >> (8 * b)) & 0xFFu;
            for (k = 0; k < 8; k++) {
                u32 mask = (crc & 1u) ? 0xEDB88320u : 0u;
                crc = (crc >> 1) ^ mask;
            }
        }
    }
    return ~crc;
}

f32 GenoBhv_SinDeg(f32 a)
{
    f32 q, sgn, w, k, kt, p, fr, lo, hi, d, t, v, fi;
    s32 i;
    k = gb_floor(a / 360.0f);
    kt = k * 360.0f;
    w = a - kt;
    if (w < 90.0f) {
        q = w;
        sgn = 1.0f;
    } else if (w < 180.0f) {
        q = 180.0f - w;
        sgn = 1.0f;
    } else if (w < 270.0f) {
        q = w - 180.0f;
        sgn = -1.0f;
    } else {
        q = 360.0f - w;
        sgn = -1.0f;
    }
    p = q * 11.377778f; /* KQ: the f32 nearest 1024 / 90 */
    i = (s32) p;
    if (i < 0) {
        i = 0;
    }
    if (i > 1023) {
        i = 1023;
    }
    fi = (f32) i;
    fr = p - fi;
    lo = gb_sinq_at(i);
    hi = gb_sinq_at(i + 1);
    d = hi - lo;
    t = d * fr;
    v = lo + t;
    return sgn * v;
}

f32 GenoBhv_CosDeg(f32 a)
{
    f32 b = a + 90.0f;
    return GenoBhv_SinDeg(b);
}

void GenoBhv_Stick(f32 rx, f32 ry, f32* sx, f32* sy)
{
    f32 a = rx * rx;
    f32 b = ry * ry;
    f32 m = a + b;
    if (m < GENO_BHV_DZ2) {
        *sx = 0.0f;
        *sy = 0.0f;
    } else {
        *sx = rx;
        *sy = ry;
    }
}

/* ---- 2. glide (spec 2) -------------------------------------------------------------------------- */

void GenoBhv_GlideParams(GbGlideP* p)
{
    p->max_glides = 2;
    p->entry_hold = 6;
    p->flick_window = 12;
    p->start_frames = 12;
    p->trip_frames = 30;
    p->land_lag = 14;
    p->jumpout_frames = 8;
    p->attack_frames = 28;
    p->attack_hit0 = 8;
    p->attack_hit1 = 14;
    p->attack_endlag = 14;
    p->attack_buf_frames = 6;
    p->entry_vy_max = 0.6f;
    p->apex_vy = 0.25f;
    p->flick_thr = 0.55f;
    p->s_start = 1.50f; /* spec 2.5 rev 1 (was 1.20): G-2 within 3% at 600 frames */
    p->start_accel = 0.12f;
    p->s_start_cap = 2.0f;
    p->theta_def = -5.0f;
    p->r_down = 60.0f;
    p->r_up = 85.0f;
    p->turn_rate = 3.0f;
    p->gk = 0.0366f;
    p->drag = 0.002f;
    p->s_max = 2.40f;
    p->s_stall = 0.35f;
    p->trip_angle = 30.0f;
    p->attack_speed_keep = 0.6f;
    p->chain_speed_mul = 0.75f;
    p->chain_turn_mul = 1.4f;
    p->attack_damage = 12;
}

void GenoBhv_GlideEntryInit(GbGlideEntry* e)
{
    e->hold_frames = -1;
    e->last_dir = 0;
    e->age = 1000;
}

int GenoBhv_GlideEntryStep(GbGlideEntry* e, const GbGlideP* p, s32 midair_jumped, s32 jump_held, f32 vy,
                           f32 sx, f32 f, s32 glides_used, s32 airborne, s32 helpless, s32 chain)
{
    f32 s = sx * f;
    f32 avy = gb_abs(vy);
    int cur = 0, flick = 0;
    if (s <= -p->flick_thr) {
        cur = -1;
    } else if (s >= p->flick_thr) {
        cur = 1;
    }
    if (cur != 0) {
        if (e->last_dir == -cur && e->age <= p->flick_window) {
            flick = 1;
        }
        e->last_dir = cur;
        e->age = 0;
    } else if (e->age < 1000) {
        e->age++;
    }
    if (!midair_jumped || !airborne) {
        e->hold_frames = -1;
    } else if (jump_held && e->hold_frames >= 0) {
        e->hold_frames++;
    } else {
        e->hold_frames = -1;
    }
    if (!airborne || helpless || glides_used >= p->max_glides) {
        return GB_GLENTRY_NONE;
    }
    if (chain) {
        return GB_GLENTRY_CHAIN;
    }
    if (e->hold_frames >= p->entry_hold && vy <= p->entry_vy_max) {
        return GB_GLENTRY_HOLD;
    }
    if (flick && avy <= p->apex_vy) {
        return GB_GLENTRY_FLICK;
    }
    return GB_GLENTRY_NONE;
}

static void gb_glide_velocity(GbGlide* g)
{
    f32 s_th = GenoBhv_SinDeg(g->theta);
    f32 c_th = GenoBhv_CosDeg(g->theta);
    f32 vx = g->speed * c_th;
    vx = vx * g->f;
    g->vx = vx;
    g->vy = g->speed * s_th;
}

void GenoBhv_GlideBegin(GbGlide* g, const GbGlideP* p, f32 vx, f32 vy, f32 f, f32 sx, s32 slow, s32 helpless,
                        s32 glides_used)
{
    f32 a = vx * vx;
    f32 b = vy * vy;
    f32 ss = a + b;
    f32 len = sqrtf(ss);
    f32 t = sx * f;
    g->phase = GB_GL_START;
    g->t = 0;
    g->attack_buf = 0;
    g->slow = slow;
    g->helpless = helpless;
    g->end_reason = GB_GLE_NONE;
    g->lag = 0;
    g->hit_active = 0;
    g->jump_now = 0;
    g->endlag = 0;
    g->speed = gb_clamp(len, 0.0f, p->s_start_cap);
    g->theta = p->theta_def;
    g->f = f;
    if (t < -p->flick_thr) {
        g->f = -f;
    }
    g->glides_used = glides_used + 1;
    g->vx = 0.0f;
    g->vy = 0.0f;
}

void GenoBhv_GlideGround(GbGlide* g, const GbGlideP* p)
{
    if (g->attack_buf > 0 || g->phase == GB_GL_ATTACK) {
        g->phase = GB_GL_LAND_CANCEL;
        g->lag = 0;
    } else if (g->theta < -p->trip_angle) {
        g->phase = GB_GL_TRIP;
        g->lag = p->trip_frames;
    } else {
        g->phase = GB_GL_LAND;
        g->lag = p->land_lag;
    }
}

void GenoBhv_GlideStep(GbGlide* g, const GbGlideIn* in, const GbGlideP* p)
{
    f32 t0, t1, d, r, theta_t, rate, s_th, grav, gk, dr, cap, d4;
    int pressed;
    g->hit_active = 0;
    g->jump_now = 0;
    if (g->phase >= GB_GL_LAND) {
        return; /* terminal */
    }
    if (g->attack_buf > 0) {
        g->attack_buf--;
    }
    pressed = in->attack_pressed || in->special_pressed;
    if (pressed && in->height >= 0.0f) {
        f32 vym = gb_abs(g->vy);
        f32 reach = vym * (f32) p->attack_buf_frames;
        if (in->height <= reach) {
            g->attack_buf = p->attack_buf_frames;
        }
    }
    if (g->phase == GB_GL_START) {
        g->theta = p->theta_def;
        g->speed = gb_approach(g->speed, p->s_start, p->start_accel);
        gb_glide_velocity(g);
        g->t++;
        if (g->t == p->start_frames) {
            g->phase = GB_GL_FLY;
            g->t = 0;
        }
    } else if (g->phase == GB_GL_FLY) {
        t0 = in->sx * g->f;
        t1 = 0.0f - in->sy;
        d = t0 + t1;
        d = gb_clamp(d, -1.0f, 1.0f);
        if (d > 0.0f) {
            r = d * p->r_down;
        } else {
            r = d * p->r_up;
        }
        theta_t = p->theta_def - r;
        rate = p->turn_rate;
        if (g->slow) {
            rate = rate * p->chain_turn_mul;
        }
        g->theta = gb_approach(g->theta, theta_t, rate);
        s_th = GenoBhv_SinDeg(g->theta);
        gk = p->gk;
        if (g->slow) {
            gk = gk * p->chain_speed_mul; /* spec 2.3 rev 1: the slow glide's gravity gain, so its steady speed is x0.75 */
        }
        grav = gk * s_th;
        g->speed = g->speed - grav;
        dr = g->speed * p->drag;
        g->speed = g->speed - dr;
        cap = p->s_max;
        if (g->slow) {
            cap = cap * p->chain_speed_mul;
        }
        if (g->speed > cap) {
            g->speed = cap;
        }
        gb_glide_velocity(g);
        g->t++;
    } else if (g->phase == GB_GL_ATTACK) {
        d4 = p->drag * 4.0f;
        dr = g->speed * d4;
        g->speed = g->speed - dr;
        gb_glide_velocity(g);
        g->hit_active = g->t >= p->attack_hit0 && g->t <= p->attack_hit1;
        g->t++;
        if (in->ground) {
            GenoBhv_GlideGround(g, p);
        } else if (g->t >= p->attack_frames) {
            g->phase = GB_GL_END;
            g->end_reason = GB_GLE_ATTACK_END;
            g->endlag = p->attack_endlag;
        }
        return;
    } else if (g->phase == GB_GL_JUMPOUT) {
        dr = g->speed * p->drag;
        g->speed = g->speed - dr;
        gb_glide_velocity(g);
        g->t++;
        if (in->ground) {
            GenoBhv_GlideGround(g, p);
        } else if (g->t >= p->jumpout_frames) {
            g->jump_now = 1;
            g->phase = GB_GL_END;
            g->end_reason = GB_GLE_JUMPOUT;
        }
        return;
    }
    /* exits, in priority order (spec 2.4) */
    if (in->ground) {
        GenoBhv_GlideGround(g, p);
        return;
    }
    if (in->jump_pressed && in->jump_left) {
        g->phase = GB_GL_JUMPOUT;
        g->t = 0;
        return;
    }
    if (pressed) {
        g->phase = GB_GL_ATTACK;
        g->t = 0;
        g->speed = g->speed * p->attack_speed_keep;
        gb_glide_velocity(g);
        return;
    }
    if (g->speed < p->s_stall) {
        g->phase = GB_GL_END;
        g->end_reason = GB_GLE_STALL;
        g->endlag = 0;
    }
}

/* ---- 3. tornado (spec 3) ------------------------------------------------------------------------ */

void GenoBhv_TornadoParams(GbTornadoP* p)
{
    p->start_frames = 12;
    p->read_period = 10;
    p->read_limit = 70;
    p->end_frames = 29;
    p->ledge_lock = 6;
    p->final_hit_ground = 3;
    p->final_hit_air = 2;
    p->loop_damage = 1;
    p->final_damage = 3;
    p->spin0 = 80.0f;
    p->decay_a = 1.5f;
    p->decay_b = 2.0f;
    p->mash_add = 16.0f;
    p->spin_max = 80.0f;
    p->spin_end = 10.0f;
    p->grav = 0.0239f;
    p->rise_impulse = 0.9f;
    p->rise_max = 1.6f;
    p->fall_max = 0.50f;
    p->vx_max = 1.1f;
    p->vx_max_ground = 1.3f;
    p->vx_accel = 0.06f;
    p->jump_bonus = 1.0f;
    p->start_vy_keep = 0.2f;
    p->start_vx_keep = 0.5f;
}

void GenoBhv_TornadoBegin(GbTornado* t, f32 vx, f32 vy)
{
    t->phase = GB_TN_START;
    t->n = 0;
    t->k = 0;
    t->end_t = 0;
    t->press_latch = 0;
    t->rise_impulse = 0;
    t->jump_start = 0;
    t->hits = 0;
    t->hit_spawn = 0;
    t->final_spawn = 0;
    t->final_active = 0;
    t->ledge_locked = 0;
    t->helpless = 0;
    t->spin = 0.0f;
    t->rot = 0.0f;
    t->vx = vx;
    t->vy = vy;
}

void GenoBhv_TornadoStep(GbTornado* t, const GbTornadoIn* in, const GbTornadoP* p)
{
    f32 tx;
    t->hit_spawn = 0;
    t->final_spawn = 0;
    t->final_active = 0;
    t->ledge_locked = 0;
    if (t->phase == GB_TN_DONE) {
        return;
    }
    if (t->n == 0) {
        t->jump_start = in->jump_held ? 1 : 0;
        t->vy = t->vy * p->start_vy_keep;
        t->vx = t->vx * p->start_vx_keep;
    }
    if (in->special_pressed) {
        t->press_latch = 1;
    }
    if (t->phase == GB_TN_START && t->n >= p->start_frames) {
        t->phase = GB_TN_LOOP;
        t->k = 0;
        t->spin = p->spin0;
        t->rot = 0.0f;
        if (t->jump_start) {
            t->vy = t->vy + p->jump_bonus;
        }
    }
    if (t->phase == GB_TN_LOOP) {
        if (t->spin <= p->spin_end) {
            t->phase = GB_TN_END;
            t->end_t = 0;
        } else {
            t->ledge_locked = 1;
            if (t->k == 0) {
                t->hit_spawn = 1; /* spec 3.2 rev 1: the first loop hit lands on loop frame 0 (global 12, T-1) */
                t->hits++;
            }
            t->rot = t->rot + t->spin;
            if (t->rot >= 360.0f) {
                t->rot = t->rot - 360.0f;
                t->hit_spawn = 1;
                t->hits++;
            }
            if (t->k < p->read_limit && (t->k % p->read_period) == 0 && t->press_latch) {
                t->spin = t->spin + p->mash_add;
                if (t->spin > p->spin_max) {
                    t->spin = p->spin_max;
                }
                t->rise_impulse = 1;
                t->press_latch = 0;
            }
            if (t->k < p->read_limit) {
                t->spin = t->spin - p->decay_a;
            } else {
                t->spin = t->spin - p->decay_b;
            }
            t->k++;
        }
    }
    if (t->phase == GB_TN_END) {
        t->end_t++;
        if (t->end_t == 1) {
            t->final_spawn = 1;
        }
        t->final_active = t->end_t <= (in->airborne ? p->final_hit_air : p->final_hit_ground);
        t->ledge_locked = t->end_t <= p->ledge_lock;
    }
    /* movement (spec 3.3) */
    if (in->airborne) {
        t->vy = t->vy - p->grav;
        if (t->rise_impulse) {
            t->vy = t->vy + p->rise_impulse;
            if (t->vy > p->rise_max) {
                t->vy = p->rise_max;
            }
            t->rise_impulse = 0;
        }
        if (t->vy < -p->fall_max) {
            t->vy = -p->fall_max;
        }
        tx = in->sx * p->vx_max;
        t->vx = gb_approach(t->vx, tx, p->vx_accel);
    } else {
        tx = in->sx * p->vx_max_ground;
        t->vx = gb_approach(t->vx, tx, p->vx_accel);
        t->vy = 0.0f;
        t->rise_impulse = 0;
    }
    t->n++;
    if (t->phase == GB_TN_END && t->end_t >= p->end_frames) {
        t->phase = GB_TN_DONE;
        t->helpless = in->airborne ? 1 : 0;
    }
}

/* ---- 4. drill (spec 4) -------------------------------------------------------------------------- */

void GenoBhv_DrillParams(GbDrillP* p)
{
    p->move0 = 16;
    p->ramp = 4;
    p->hit0 = 26;
    p->hit_period = 4;
    p->nhits = 10;
    p->hit_end = 65;
    p->final_frame = 66;
    p->final_end_ground = 67;
    p->final_end_air = 70;
    p->total = 96;
    p->rush_damage = 1;
    p->final_damage = 3;
    p->dive_default = -20.0f;
    p->psi_stick = 45.0f;
    p->psi_min = -60.0f;
    p->psi_max = 30.0f;
    p->steer_rate = 2.0f;
    p->speed = 2.0f;
    p->recoil_vy = 1.0f;
    p->end_vx_keep = 0.2f;
}

void GenoBhv_DrillBegin(GbDrill* d, const GbDrillP* p, f32 f, f32 sy)
{
    f32 a = sy * p->psi_stick;
    f32 psi = p->dive_default + a;
    d->psi = gb_clamp(psi, p->psi_min, p->psi_max);
    d->phase = GB_DR_WIND;
    d->n = 0;
    d->hit_spawn = 0;
    d->hits = 0;
    d->final_active = 0;
    d->helpless = 0;
    d->done = 0;
    d->speed = 0.0f;
    d->f = f;
    d->vx = 0.0f;
    d->vy = 0.0f;
}

void GenoBhv_DrillStep(GbDrill* d, const GbDrillIn* in, const GbDrillP* p)
{
    int n = d->n;
    f32 a, tgt, step, cs, sn;
    d->hit_spawn = 0;
    d->final_active = 0;
    if (d->done) {
        return;
    }
    if (n >= p->move0 && n <= p->hit_end) {
        a = in->sy * p->psi_stick;
        tgt = p->dive_default + a;
        tgt = gb_clamp(tgt, p->psi_min, p->psi_max);
        d->psi = gb_approach(d->psi, tgt, p->steer_rate);
    }
    if (!in->airborne && d->psi < in->slope) {
        d->psi = in->slope;
    }
    if (n < p->final_frame) {
        if (n < p->move0) {
            d->speed = 0.0f;
        } else {
            step = p->speed / (f32) p->ramp;
            d->speed = gb_approach(d->speed, p->speed, step);
        }
        cs = GenoBhv_CosDeg(d->psi);
        sn = GenoBhv_SinDeg(d->psi);
        d->vx = d->speed * cs;
        d->vx = d->vx * d->f;
        d->vy = d->speed * sn;
        if (!in->airborne) {
            d->vy = 0.0f;
        }
    } else if (n == p->final_frame) {
        d->vx = d->vx * p->end_vx_keep;
        if (in->airborne) {
            d->vy = p->recoil_vy;
        }
    }
    if (n >= p->hit0 && n <= p->hit_end && ((n - p->hit0) % p->hit_period) == 0 && d->hits < p->nhits) {
        d->hit_spawn = 1;
        d->hits++;
    }
    if (n >= p->final_frame) {
        d->phase = GB_DR_FINAL;
        d->final_active = n <= (in->airborne ? p->final_end_air : p->final_end_ground);
    } else if (n >= p->hit0) {
        d->phase = GB_DR_RUSH;
    } else {
        d->phase = GB_DR_WIND;
    }
    d->n++;
    if (d->n >= p->total) {
        d->done = 1;
        d->phase = GB_DR_DONE;
        d->helpless = in->airborne ? 1 : 0;
    }
}

/* ---- 5. cape (spec 5) --------------------------------------------------------------------------- */

void GenoBhv_CapeParams(GbCapeP* p)
{
    p->start = 6;
    p->end = 25;
    p->total = 40;
    p->turn_thr = 0.5f;
    p->stall_x = 0.80f;
    p->stall_y_up = 0.85f;
    p->stall_vy_hi = 0.0f;
    p->stall_grav = 0.015f;
    p->stall_fall = 0.45f;
    p->ground_x = 0.5f;
    p->reflect_dmg_mul = 1.5f;
    p->reflect_speed_mul = 1.2f;
}

void GenoBhv_CapeBegin(GbCape* c, f32 f, f32 vx, f32 vy)
{
    c->n = 0;
    c->flipped = 0;
    c->flip_now = 0;
    c->reflect = 0;
    c->done = 0;
    c->f = f;
    c->vx = vx;
    c->vy = vy;
}

void GenoBhv_CapeStep(GbCape* c, const GbCapeIn* in, const GbCapeP* p)
{
    int active;
    f32 t;
    c->flip_now = 0;
    c->reflect = 0;
    if (c->done) {
        return;
    }
    if (c->n == p->start && !c->flipped) {
        c->flipped = 1;
        t = in->sx * c->f;
        if (t < -p->turn_thr) {
            c->flip_now = 1;
            c->f = -c->f;
        }
    }
    active = c->n >= p->start && c->n <= p->end;
    if (active) {
        c->reflect = 1;
        if (in->airborne) {
            c->vx = c->vx * p->stall_x;
            if (c->vy > p->stall_vy_hi) {
                c->vy = c->vy * p->stall_y_up;
            } else {
                c->vy = c->vy - p->stall_grav;
            }
            if (c->vy < -p->stall_fall) {
                c->vy = -p->stall_fall;
            }
        } else {
            c->vx = c->vx * p->ground_x;
        }
    }
    c->n++;
    if (c->n >= p->total) {
        c->done = 1;
    }
}

/* ---- 6. multi-dash (spec 6) --------------------------------------------------------------------- */

void GenoBhv_MultiDashParams(GbMultiDashP* p)
{
    p->max_seg = 4;
    p->aim_first = 6;
    p->aim_later = 4;
    p->dash_frames = 10;
    p->pause_frames = 3;
    p->repress_frames = 8;
    p->end_lag = 20;
    p->dash_speed = 3.2f;
    p->end_keep = 0.15f;
    p->aim_slow = 0.1f;
    p->ay_max = 0.9397f;
    p->pause_keep = 0.3f;
}

void GenoBhv_MultiDashBegin(GbMultiDash* m, f32 f)
{
    m->seg = 0;
    m->phase = GB_MD_AIM;
    m->t = 0;
    m->repress = 0;
    m->hit_spawn = 0;
    m->helpless = 0;
    m->done = 0;
    m->f = f;
    m->ax = f;
    m->ay = 0.0f;
    m->speed = 0.0f;
    m->vx = 0.0f;
    m->vy = 0.0f;
    m->dist = 0.0f;
}

static void gb_md_fix_direction(GbMultiDash* m, const GbMultiDashIn* in, const GbMultiDashP* p)
{
    f32 m2 = in->sx * in->sx;
    f32 n2 = in->sy * in->sy;
    f32 mm = m2 + n2;
    f32 ax, ay, len, l2, l3, ll, l, chk;
    if (mm < GENO_BHV_DZ2) {
        ax = m->f;
        ay = 0.0f;
    } else {
        len = sqrtf(mm);
        ax = in->sx / len;
        ay = in->sy / len;
    }
    chk = ax * m->f;
    if (chk < 0.0f) {
        m->f = -m->f;
    }
    /* spec 6.2 rev 1: clamp the unit vector's vertical part, then rebuild the horizontal part from it
       (renormalising (ax, clamp(ay)) would undo the clamp when ax = 0) */
    if (ay > p->ay_max || ay < -p->ay_max) {
        ay = gb_clamp(ay, -p->ay_max, p->ay_max);
        l3 = ay * ay;
        ll = 1.0f - l3;
        l = sqrtf(ll);
        l2 = m->f;
        if (ax < 0.0f) {
            l2 = -1.0f;
        } else if (ax > 0.0f) {
            l2 = 1.0f;
        }
        ax = l * l2;
    }
    m->ax = ax;
    m->ay = ay;
}

void GenoBhv_MultiDashStep(GbMultiDash* m, const GbMultiDashIn* in, const GbMultiDashP* p)
{
    int aim_len;
    f32 step;
    m->hit_spawn = 0;
    if (m->done) {
        return;
    }
    if (in->special_pressed) {
        if ((m->phase == GB_MD_DASH && m->t >= p->dash_frames - p->repress_frames) || m->phase == GB_MD_PAUSE) {
            m->repress = 1;
        }
    }
    if (m->phase == GB_MD_AIM) {
        aim_len = m->seg == 0 ? p->aim_first : p->aim_later;
        m->vx = m->vx * p->aim_slow;
        m->vy = m->vy * p->aim_slow;
        m->t++;
        if (m->t >= aim_len) {
            gb_md_fix_direction(m, in, p);
            m->phase = GB_MD_DASH;
            m->t = 0;
            m->speed = 0.0f;
            m->dist = 0.0f;
        }
    } else if (m->phase == GB_MD_DASH) {
        step = p->dash_speed / 3.0f; /* spec 6.2 rev 1: two ramp frames, full speed from dash frame 3 */
        m->speed = gb_approach(m->speed, p->dash_speed, step);
        m->vx = m->speed * m->ax;
        m->vy = m->speed * m->ay;
        m->dist = m->dist + m->speed;
        m->hit_spawn = 1;
        m->t++;
        if (m->t >= p->dash_frames) {
            m->phase = GB_MD_PAUSE;
            m->t = 0;
        }
    } else if (m->phase == GB_MD_PAUSE) {
        m->vx = m->vx * p->pause_keep;
        m->vy = m->vy * p->pause_keep;
        m->t++;
        if (m->t >= p->pause_frames) {
            if (m->repress && m->seg + 1 < p->max_seg) {
                m->seg++;
                m->phase = GB_MD_AIM;
                m->t = 0;
                m->repress = 0;
            } else {
                m->phase = GB_MD_END;
                m->t = 0;
                m->vx = m->vx * p->end_keep;
                m->vy = m->vy * p->end_keep;
            }
        }
    } else if (m->phase == GB_MD_END) {
        m->t++;
        if (m->t >= p->end_lag) {
            m->phase = GB_MD_DONE;
            m->done = 1;
            m->helpless = m->seg + 1 == p->max_seg ? 1 : 0;
        }
    }
}

/* ---- 7. anim_motion (spec 7) -------------------------------------------------------------------- */

void GenoBhv_AnimMotionParams(GbAnimMotionP* p)
{
    p->rate = 1.0f;
    p->scale = 1.0f;
    p->grav = 0.095f;
    p->mode = GB_AM_DELTA;
    p->gravity_on = 0;
    p->ignore_y = 0;
    p->loop = 0;
}

void GenoBhv_AnimMotionSample(const f32* track, int n, f32 t, f32* x, f32* y)
{
    f32 last, fr, ax, bx, dx, tx, ay, by, dy, ty, fi;
    int i;
    if (t <= 0.0f) {
        *x = track[0];
        *y = track[1];
        return;
    }
    last = (f32) (n - 1);
    if (t >= last) {
        *x = track[2 * (n - 1)];
        *y = track[2 * (n - 1) + 1];
        return;
    }
    i = (int) gb_floor(t);
    fi = (f32) i;
    fr = t - fi;
    ax = track[2 * i];
    bx = track[2 * (i + 1)];
    dx = bx - ax;
    tx = dx * fr;
    *x = ax + tx;
    ay = track[2 * i + 1];
    by = track[2 * (i + 1) + 1];
    dy = by - ay;
    ty = dy * fr;
    *y = ay + ty;
}

void GenoBhv_AnimMotionBegin(GbAnimMotion* a, f32 pos_x, f32 pos_y)
{
    a->anim_t = 0.0f;
    a->vy_carry = 0.0f;
    a->origin_x = pos_x;
    a->origin_y = pos_y;
    a->vx = 0.0f;
    a->vy = 0.0f;
    a->pos_x = pos_x;
    a->pos_y = pos_y;
    a->ended = 0;
    a->started = 1;
}

void GenoBhv_AnimMotionStep(GbAnimMotion* a, const f32* track, int n, const GbAnimMotionP* p, f32 facing,
                            s32 blocked)
{
    f32 t0, t1, x0, y0, x1, y1, dx, dy, last, tx, ty, tgx, tgy, delta;
    if (a->ended) {
        return;
    }
    last = (f32) (n - 1);
    t0 = a->anim_t;
    t1 = a->anim_t + p->rate;
    GenoBhv_AnimMotionSample(track, n, t0, &x0, &y0);
    GenoBhv_AnimMotionSample(track, n, t1, &x1, &y1);
    dx = x1 - x0;
    dy = y1 - y0;
    dx = dx * p->scale;
    dy = dy * p->scale;
    dx = dx * facing;
    if (p->ignore_y) {
        dy = 0.0f;
    }
    if (p->mode == GB_AM_DELTA) {
        if (blocked) {
            dx = 0.0f;
        }
        a->pos_x = a->pos_x + dx;
        a->pos_y = a->pos_y + dy;
    } else {
        tx = x1 * p->scale;
        tx = tx * facing;
        ty = y1 * p->scale;
        tgx = a->origin_x + tx;
        tgy = a->origin_y + ty;
        if (p->ignore_y) {
            tgy = a->origin_y;
        }
        if (blocked) {
            delta = tgx - a->pos_x;
            a->origin_x = a->origin_x - delta;
            tgx = a->pos_x;
            dx = 0.0f;
        }
        a->pos_x = tgx;
        a->pos_y = tgy;
    }
    a->vx = dx;
    a->vy = dy;
    if (p->gravity_on) {
        a->vy = a->vy + a->vy_carry;
        a->vy_carry = a->vy_carry - p->grav;
    }
    a->anim_t = t1;
    if (a->anim_t >= last) {
        if (p->loop) {
            a->anim_t = a->anim_t - last;
        } else {
            a->ended = 1;
        }
    }
}

/* ==================================================================================================
 * 8. Engine adapters
 *
 * Each behaviour keeps its pure struct in the fighter's GenoState move variables (they survive the
 * action changes between the states of one move, and they are in the rollback snapshot):
 *   glide    move_i[0] phase | end_reason << 4 | slow << 8 | helpless << 9;  move_i[1] t;
 *            move_i[2] attack_buf | endlag << 8;  move_i[3] landing lag;  move_i[6] entry history
 *            (age << 2 | last_dir + 1);  move_i[7] glides used this airtime (cleared on the ground);
 *            move_f[0..3] theta, speed, vx, vy.   The jump-hold count of entry path A is GenoState's
 *            hold_motion / hold_frames.
 *   tornado  move_i[0] phase | latch << 4 | rise << 5 | jump_start << 6;  move_i[1] n;  move_i[2] k;
 *            move_i[3] end_t;  move_i[5] hits;  move_f[0..1] spin, rot.
 *   drill    move_i[0] phase;  move_i[1] n;  move_i[2] hits;  move_i[3] done;
 *            move_f[0..3] psi, speed, vx, vy.
 *   cape     move_i[0] n | flipped << 8.
 *   multi-dash move_i[0] phase | seg << 4 | repress << 8 | helpless << 9 | done << 10;  move_i[1] t;
 *            move_f[0..6] ax, ay, f, speed, vx, vy, dist.
 * ================================================================================================== */

#include <melee/ft/ftcoll.h>
#include <melee/ft/kinds/ftCommon/ftCo_AirCatch.h>
#include <melee/ft/kinds/ftCommon/ftCo_Attack100.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackS4.h>
#include <melee/ft/kinds/ftCommon/ftCo_Dash.h>
#include <melee/ft/kinds/ftCommon/ftCo_ItemThrow.h>
#include <melee/ft/kinds/ftCommon/ftCo_Jump.h>
#include <melee/ft/kinds/ftCommon/ftCo_JumpAerial.h>
#include <melee/ft/kinds/ftCommon/ftCo_Turn.h>
#include <melee/ft/kinds/ftCommon/ftCo_Walk.h>

#define GB_PRESSED(fp, m) (((fp)->input.pressed_buttons & (m)) != 0)

static void gb_stick_of(Fighter* fp, f32* sx, f32* sy)
{
    GenoBhv_Stick(fp->input.lstick[0].x, fp->input.lstick[0].y, sx, sy);
}

static int gb_airborne(Fighter* fp)
{
    return fp->ground_or_air != GA_Ground;
}

static int gb_jumps_left(Fighter* fp)
{
    return (int) fp->co_attrs.max_jumps - (int) fp->x1968_jumpsUsed > 0;
}

static int gb_state_bhv(GenoState* st, int s)
{
    return st->profile >= 0 && s >= 0 && s < GENO_MAX_STATES && s < Geno_StateCount(st->profile)
               ? Geno_StateBehavior(st->profile, s)
               : 0;
}

static int gb_cur_bhv(Fighter* fp, GenoState* st)
{
    return gb_state_bhv(st, geno_cur_state(fp, st));
}

/* The behaviour of the Geno state this one was entered from, 0 when it came from elsewhere. */
static int gb_from_bhv(GenoState* st)
{
    return gb_state_bhv(st, st->enter_from - GENO_MOTION_BASE);
}

static void gb_to_bhv(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int bhv)
{
    geno_go(gobj, fp, st, GENO_TARGET(GENO_TGT_BHV, bhv));
}

/* The end of a move that is not helpless-by-default: the state's own "next", else `fallback`. */
static void gb_finish(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int helpless)
{
    int s = geno_cur_state(fp, st);
    u32 nx = s >= 0 ? (u32) Geno_StateNext(st->profile, s) : GENO_NONE_T; /* the state's own "next" only */
    u32 dflt = helpless ? GENO_TGT_HELPLESS : GENO_AUTO;
    geno_go(gobj, fp, st, nx != GENO_NONE_T ? nx : dflt);
}

static void gb_clear_hit_lists(Fighter* fp)
{
    int i;
    for (i = 0; i < GENO_MAX_REHIT; i++) {
        lbColl_80008440(&fp->x914[i]);
    }
}

/* ---- glide ---------------------------------------------------------------------------------------- */

static void gb_glide_load(GenoState* st, GbGlide* g)
{
    s32 i0 = st->move_i[0];
    g->phase = i0 & 15;
    g->end_reason = (i0 >> 4) & 15;
    g->slow = (i0 >> 8) & 1;
    g->helpless = (i0 >> 9) & 1;
    g->t = st->move_i[1];
    g->attack_buf = st->move_i[2] & 255;
    g->endlag = (st->move_i[2] >> 8) & 255;
    g->lag = st->move_i[3];
    g->glides_used = st->move_i[7];
    g->theta = st->move_f[0];
    g->speed = st->move_f[1];
    g->vx = st->move_f[2];
    g->vy = st->move_f[3];
    g->hit_active = 0;
    g->jump_now = 0;
    g->f = 1.0f;
}

static void gb_glide_store(GenoState* st, const GbGlide* g)
{
    st->move_i[0] = g->phase | (g->end_reason << 4) | (g->slow << 8) | (g->helpless << 9);
    st->move_i[1] = g->t;
    st->move_i[2] = g->attack_buf | (g->endlag << 8);
    st->move_i[3] = g->lag;
    st->move_i[7] = g->glides_used;
    st->move_f[0] = g->theta;
    st->move_f[1] = g->speed;
    st->move_f[2] = g->vx;
    st->move_f[3] = g->vy;
}

/* Preanim hook (geno_v2_preanim): entry paths A (midair-jump hold) and B (apex flick) of the glide. */
static int geno_glide_entry(Fighter_GObj* gobj, Fighter* fp, GenoState* st)
{
    GbGlideP p;
    GbGlideEntry e;
    f32 sx, sy;
    int msid, allowed, jumps, midair, path, s_start;
    s_start = geno_state_with(st, GENO_BHV_GLIDE_START);
    if (s_start < 0) {
        return 0;
    }
    if (!gb_airborne(fp)) {
        st->move_i[7] = 0;
        st->hold_motion = -1;
        st->hold_frames = -1;
        st->move_i[6] = (1000 << 2) | 1;
        return 0;
    }
    if (geno_cur_state(fp, st) >= 0 || geno_param(st, GENO_P_GLIDE_ENTRY) == 0.0f) {
        return 0;
    }
    msid = (int) fp->motion_id;
    /* A midair jump is the jump counter passing 1 (the ground jump counts too, and the counter is 1 in
     * the air after leaving the ground). hold_motion packs (counter << 16 | the motion that jump began
     * in): a character's own extra-jump motions (not Melee's JumpAerial pair) are accepted for as long
     * as that jump's motion lasts. */
    jumps = (int) fp->x1968_jumpsUsed;
    if (jumps >= 2 && (st->hold_motion < 0 || (st->hold_motion >> 16) != jumps)) {
        st->hold_motion = (jumps << 16) | (msid & 0xFFFF);
        st->hold_frames = 0;
    }
    midair = st->hold_motion >= 0 && (st->hold_motion >> 16) >= 2;
    allowed = msid == ftCo_MS_JumpF || msid == ftCo_MS_JumpB || msid == ftCo_MS_JumpAerialF ||
              msid == ftCo_MS_JumpAerialB || msid == ftCo_MS_Fall || msid == ftCo_MS_FallAerial ||
              msid == ftCo_MS_FallAerialF || msid == ftCo_MS_FallAerialB ||
              (midair && msid == (st->hold_motion & 0xFFFF));
    if (!allowed) {
        return 0;
    }
    GenoBhv_GlideParams(&p);
    gb_stick_of(fp, &sx, &sy);
    e.hold_frames = st->hold_frames;
    e.last_dir = (st->move_i[6] & 3) - 1;
    e.age = st->move_i[6] >> 2;
    path = GenoBhv_GlideEntryStep(&e, &p, midair, geno_jump_held(fp), fp->self_vel.y, sx, fp->facing_dir,
                                  st->move_i[7], 1, 0, 0);
    st->hold_frames = e.hold_frames;
    st->move_i[6] = (e.age << 2) | (e.last_dir + 1);
    if (path == GB_GLENTRY_NONE) {
        return 0;
    }
    st->hold_motion = -1; /* the jump's hold is spent: a new glide needs a new midair jump */
    st->hold_frames = -1;
    st->move_i[6] = (1000 << 2) | 1;
    return geno_go(gobj, fp, st, GENO_TARGET(GENO_TGT_BHV, GENO_BHV_GLIDE_START));
}

static void geno_glide_start_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    GbGlideP p;
    GbGlide g;
    f32 sx, sy;
    GenoBhv_GlideParams(&p);
    gb_stick_of(fp, &sx, &sy);
    GenoBhv_GlideBegin(&g, &p, fp->self_vel.x, fp->self_vel.y, fp->facing_dir, sx, 0,
                       fp->x1968_jumpsUsed >= fp->co_attrs.max_jumps, st->move_i[7]);
    fp->facing_dir = g.f;
    gb_glide_store(st, &g);
}

/* The Glide state: kept when it follows GlideStart; entered from anywhere else (a script, a chained
 * up-special) it starts a slow glide (path C). */
static void geno_glide_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    GbGlideP p;
    GbGlide g;
    f32 sx, sy;
    if (gb_from_bhv(st) == GENO_BHV_GLIDE_START) {
        gb_glide_load(st, &g);
        g.phase = GB_GL_FLY;
        g.t = 0;
        gb_glide_store(st, &g);
        return;
    }
    GenoBhv_GlideParams(&p);
    gb_stick_of(fp, &sx, &sy);
    GenoBhv_GlideBegin(&g, &p, fp->self_vel.x, fp->self_vel.y, fp->facing_dir, sx, 1,
                       fp->x1968_jumpsUsed >= fp->co_attrs.max_jumps, st->move_i[7]);
    fp->facing_dir = g.f;
    g.phase = GB_GL_FLY;
    g.t = 0;
    gb_glide_store(st, &g);
}

static void geno_glide_attack_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s) {}
static void geno_glide_end_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s) {}

/* The landing state: its animation lasts the landing lag the ground exit chose. */
static void geno_glide_landing_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    s32 lag = st->move_i[3];
    if (lag > 0 && !geno_test_capture) {
        ftAnim_SetAnimRate(gobj, (0.1f + fp->x2EC) / (f32) lag);
    }
}

/* One frame of the glide, from the anim callback of whichever glide state is current. */
static void gb_glide_tick(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GbGlideP p;
    GbGlide g;
    GbGlideIn in;
    int bhv;
    gb_glide_load(st, &g);
    if (g.phase >= GB_GL_END) {
        return;
    }
    GenoBhv_GlideParams(&p);
    gb_stick_of(fp, &in.sx, &in.sy);
    in.jump_pressed = GB_PRESSED(fp, HSD_PAD_XY);
    in.attack_pressed = GB_PRESSED(fp, HSD_PAD_A);
    in.special_pressed = GB_PRESSED(fp, HSD_PAD_B);
    in.jump_left = gb_jumps_left(fp);
    in.ground = 0;
    in.height = -1.0f;
    g.f = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    GenoBhv_GlideStep(&g, &in, &p);
    gb_glide_store(st, &g);
    bhv = gb_cur_bhv(fp, st);
    if (g.phase == GB_GL_FLY && bhv != GENO_BHV_GLIDE) {
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE);
    } else if (g.phase == GB_GL_ATTACK && bhv != GENO_BHV_GLIDE_ATTACK) {
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_ATTACK);
    } else if (g.phase == GB_GL_JUMPOUT && bhv != GENO_BHV_GLIDE_END) {
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_END);
    } else if (g.phase == GB_GL_END) {
        if (g.end_reason == GB_GLE_JUMPOUT) {
            if (gb_jumps_left(fp) && !geno_test_capture) {
                ftCo_JumpAerial_Enter_Basic(gobj);
            } else {
                gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_END);
            }
        } else if (g.helpless) {
            geno_go(gobj, fp, st, GENO_TGT_HELPLESS);
        } else if (bhv != GENO_BHV_GLIDE_END) {
            gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_END);
        }
    }
}

static void geno_glide_start_anim(Fighter_GObj* gobj)
{
    gb_glide_tick(gobj);
}

static void geno_glide_anim(Fighter_GObj* gobj)
{
    gb_glide_tick(gobj);
}

/* anim "glide.after": the attack and the jump-out are timed by the procedure, not the clip; once the
 * glide is over (END) the clip's end falls back to the state's next. */
static void geno_glide_after_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GbGlide g;
    gb_glide_load(st, &g);
    if (g.phase == GB_GL_ATTACK || g.phase == GB_GL_JUMPOUT) {
        gb_glide_tick(gobj);
    } else {
        geno_cb_anim_next(gobj);
    }
}

static void geno_glide_iasa(Fighter_GObj* gobj) {}

static void gb_glide_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GbGlide g;
    gb_glide_load(geno_state(fp), &g);
    if (g.phase <= GB_GL_JUMPOUT) {
        fp->self_vel.x = g.vx;
        fp->self_vel.y = g.vy;
    } else {
        geno_cb_phys_air(gobj);
    }
}

static void geno_glide_start_phys(Fighter_GObj* gobj)
{
    gb_glide_phys(gobj);
}

static void geno_glide_phys(Fighter_GObj* gobj)
{
    gb_glide_phys(gobj);
}

static void geno_glide_attack_phys(Fighter_GObj* gobj)
{
    gb_glide_phys(gobj);
}

static void geno_glide_end_phys(Fighter_GObj* gobj)
{
    gb_glide_phys(gobj);
}

/* Landing out of any glide state (called from geno_land): the ground exit of spec 2.4. Returns 1 when
 * it handled the landing. */
static int gb_glide_land(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    int bhv = gb_state_bhv(st, s);
    GbGlideP p;
    GbGlide g;
    if (bhv != GENO_BHV_GLIDE_START && bhv != GENO_BHV_GLIDE && bhv != GENO_BHV_GLIDE_ATTACK &&
        bhv != GENO_BHV_GLIDE_END)
    {
        return 0;
    }
    GenoBhv_GlideParams(&p);
    gb_glide_load(st, &g);
    GenoBhv_GlideGround(&g, &p);
    st->move_i[3] = g.lag;
    gb_glide_store(st, &g);
    if (g.phase == GB_GL_LAND_CANCEL) {
        ftCommon_8007D7FC(fp);
        geno_go(gobj, fp, st, GENO_AUTO);
    } else if (g.helpless) {
        ftCo_LandingFallSpecial_Enter_Basic(gobj);
    } else if (g.phase == GB_GL_TRIP) {
        ftCo_LandingFallSpecial_Enter(gobj, true, (f32) g.lag);
    } else if (geno_state_with(st, GENO_BHV_GLIDE_LANDING) >= 0) {
        ftCommon_8007D7FC(fp);
        gb_to_bhv(gobj, fp, st, GENO_BHV_GLIDE_LANDING);
    } else {
        ftCo_LandingFallSpecial_Enter(gobj, true, (f32) g.lag);
    }
    return 1;
}

static void geno_glide_coll(Fighter_GObj* gobj)
{
    ft_80082C74(gobj, geno_land);
}

/* ---- tornado -------------------------------------------------------------------------------------- */

static void gb_tornado_load(GenoState* st, GbTornado* t)
{
    s32 i0 = st->move_i[0];
    t->phase = i0 & 15;
    t->press_latch = (i0 >> 4) & 1;
    t->rise_impulse = (i0 >> 5) & 1;
    t->jump_start = (i0 >> 6) & 1;
    t->n = st->move_i[1];
    t->k = st->move_i[2];
    t->end_t = st->move_i[3];
    t->hits = st->move_i[5];
    t->spin = st->move_f[0];
    t->rot = st->move_f[1];
}

static void gb_tornado_store(GenoState* st, const GbTornado* t)
{
    st->move_i[0] = t->phase | (t->press_latch << 4) | (t->rise_impulse << 5) | (t->jump_start << 6);
    st->move_i[1] = t->n;
    st->move_i[2] = t->k;
    st->move_i[3] = t->end_t;
    st->move_i[5] = t->hits;
    st->move_f[0] = t->spin;
    st->move_f[1] = t->rot;
}

static void geno_tornado_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    GbTornado t;
    int air = gb_airborne(fp);
    GenoBhv_TornadoBegin(&t, air ? fp->self_vel.x : fp->gr_vel, air ? fp->self_vel.y : 0.0f);
    gb_tornado_store(st, &t);
}

/* One frame, from the anim callback: the spin, the hit cadence, the mash reads and the movement. The
 * loop's hits are the clip's hitboxes re-armed (the victim lists cleared) on each spawn. */
static void geno_tornado_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GbTornadoP p;
    GbTornado t;
    GbTornadoIn in;
    f32 sy;
    int air = gb_airborne(fp);
    GenoBhv_TornadoParams(&p);
    gb_tornado_load(st, &t);
    gb_stick_of(fp, &in.sx, &sy);
    in.special_pressed = GB_PRESSED(fp, HSD_PAD_B);
    in.jump_held = geno_jump_held(fp);
    in.airborne = air;
    t.vx = air ? fp->self_vel.x : fp->gr_vel;
    t.vy = air ? fp->self_vel.y : 0.0f;
    GenoBhv_TornadoStep(&t, &in, &p);
    if (air) {
        fp->self_vel.x = t.vx;
        fp->self_vel.y = t.vy;
    } else {
        fp->gr_vel = t.vx;
    }
    if (t.hit_spawn || t.final_spawn) {
        gb_clear_hit_lists(fp);
    }
    gb_tornado_store(st, &t);
    if (t.phase == GB_TN_DONE) {
        gb_finish(gobj, fp, st, t.helpless);
    }
}

static void geno_tornado_phys(Fighter_GObj* gobj)
{
    if (!gb_airborne(GET_FIGHTER(gobj))) {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    }
}

/* ---- drill ---------------------------------------------------------------------------------------- */

static void gb_drill_load(GenoState* st, GbDrill* d)
{
    d->phase = st->move_i[0];
    d->n = st->move_i[1];
    d->hits = st->move_i[2];
    d->done = st->move_i[3];
    d->psi = st->move_f[0];
    d->speed = st->move_f[1];
    d->vx = st->move_f[2];
    d->vy = st->move_f[3];
    d->f = 1.0f;
}

static void gb_drill_store(GenoState* st, const GbDrill* d)
{
    st->move_i[0] = d->phase;
    st->move_i[1] = d->n;
    st->move_i[2] = d->hits;
    st->move_i[3] = d->done;
    st->move_f[0] = d->psi;
    st->move_f[1] = d->speed;
    st->move_f[2] = d->vx;
    st->move_f[3] = d->vy;
}

static void gb_drill_begin(Fighter* fp, GenoState* st)
{
    GbDrillP p;
    GbDrill d;
    f32 sx, sy;
    GenoBhv_DrillParams(&p);
    gb_stick_of(fp, &sx, &sy);
    GenoBhv_DrillBegin(&d, &p, fp->facing_dir < 0.0f ? -1.0f : 1.0f, sy);
    gb_drill_store(st, &d);
}

static int gb_drill_continues(GenoState* st)
{
    int b = gb_from_bhv(st);
    return b == GENO_BHV_DRILL_START || b == GENO_BHV_DRILL || b == GENO_BHV_DRILL_END;
}

static void geno_drill_start_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    gb_drill_begin(fp, st);
}

static void geno_drill_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (!gb_drill_continues(st)) {
        gb_drill_begin(fp, st);
    }
}

static void geno_drill_end_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    if (!gb_drill_continues(st)) {
        gb_drill_begin(fp, st);
    }
}

/* One frame of the rush, then the state the timeline asks for (wind-up, rush, flip). */
static void gb_drill_tick(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GbDrillP p;
    GbDrill d;
    GbDrillIn in;
    f32 sx;
    int prev_n, bhv, want, air = gb_airborne(fp);
    GenoBhv_DrillParams(&p);
    gb_drill_load(st, &d);
    if (d.done) {
        return;
    }
    d.f = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    gb_stick_of(fp, &sx, &in.sy);
    in.airborne = air;
    in.slope = 0.0f;
    prev_n = d.n;
    GenoBhv_DrillStep(&d, &in, &p);
    if (prev_n <= p.final_frame) {
        if (air) {
            fp->self_vel.x = d.vx;
            fp->self_vel.y = d.vy;
        } else {
            fp->gr_vel = d.vx;
        }
    }
    if (d.hit_spawn) {
        gb_clear_hit_lists(fp);
    }
    gb_drill_store(st, &d);
    if (d.done) {
        gb_finish(gobj, fp, st, d.helpless);
        return;
    }
    bhv = gb_cur_bhv(fp, st);
    want = prev_n < p.hit0 ? GENO_BHV_DRILL_START : prev_n < p.final_frame ? GENO_BHV_DRILL : GENO_BHV_DRILL_END;
    if (want == GENO_BHV_DRILL_START && geno_state_with(st, want) < 0) {
        want = GENO_BHV_DRILL;
    }
    if (want != bhv && geno_state_with(st, want) >= 0) {
        gb_to_bhv(gobj, fp, st, want);
    }
}

static void gb_drill_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GbDrillP p;
    GbDrill d;
    GenoBhv_DrillParams(&p);
    gb_drill_load(geno_state(fp), &d);
    if (!gb_airborne(fp)) {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    } else if (d.n > p.final_frame + 1) {
        ftCommon_Fall(fp, fp->co_attrs.gravity, fp->co_attrs.terminal_velocity); /* gravity returns after the recoil */
    }
}

static void geno_drill_anim(Fighter_GObj* gobj)
{
    gb_drill_tick(gobj);
}

static void geno_drill_end_anim(Fighter_GObj* gobj)
{
    gb_drill_tick(gobj);
}

static void geno_drill_phys(Fighter_GObj* gobj)
{
    gb_drill_phys(gobj);
}

static void geno_drill_end_phys(Fighter_GObj* gobj)
{
    gb_drill_phys(gobj);
}

/* The wind-up state's clip ends on its own schedule ("next"), so its frame is stepped here. */
static void geno_drill_start_phys(Fighter_GObj* gobj)
{
    gb_drill_tick(gobj);
    gb_drill_phys(gobj);
}

static void geno_drill_coll(Fighter_GObj* gobj)
{
    geno_coll_both(gobj);
}

static void geno_drill_start_coll(Fighter_GObj* gobj)
{
    geno_coll_both(gobj);
}

/* ---- cape ----------------------------------------------------------------------------------------- */

static void gb_cape_reflect(Fighter_GObj* gobj, Fighter* fp, const GbCapeP* p, int on)
{
    ReflectDesc d;
    if (!on) {
        fp->reflecting = false;
        return;
    }
    if (fp->reflecting) {
        return;
    }
    if (geno_test_capture) {
        fp->reflecting = true;
        return;
    }
    geno_zero(&d, sizeof(d));
    d.x0_bone_id = 1;
    d.x4_max_damage = 999;
    d.x14_size = 12.0f;
    d.x18_damage_mul = p->reflect_dmg_mul;
    d.x1C_speed_mul = p->reflect_speed_mul;
    d.x20_behavior = 0;
    ftColl_CreateReflectHit(gobj, &d, NULL);
}

static void geno_cape_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    st->move_i[0] = 0;
}

static void geno_cape_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GbCapeP p;
    GbCape c;
    GbCapeIn in;
    f32 sy;
    int air = gb_airborne(fp);
    GenoBhv_CapeParams(&p);
    c.n = st->move_i[0] & 255;
    c.flipped = (st->move_i[0] >> 8) & 1;
    c.done = 0;
    c.f = fp->facing_dir < 0.0f ? -1.0f : 1.0f;
    c.vx = air ? fp->self_vel.x : fp->gr_vel;
    c.vy = air ? fp->self_vel.y : 0.0f;
    gb_stick_of(fp, &in.sx, &sy);
    in.airborne = air;
    GenoBhv_CapeStep(&c, &in, &p);
    if (c.flip_now) {
        fp->facing_dir = c.f;
    }
    if (air) {
        fp->self_vel.x = c.vx;
        fp->self_vel.y = c.vy;
    } else {
        fp->gr_vel = c.vx;
    }
    gb_cape_reflect(gobj, fp, &p, c.reflect);
    st->move_i[0] = c.n | (c.flipped << 8);
    if (c.done) {
        gb_finish(gobj, fp, st, 0);
    }
}

/* Gravity and drift apply before the reflect window and after it; inside it the stall owns the velocity. */
static void geno_cape_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GbCapeP p;
    int n = geno_state(fp)->move_i[0] & 255;
    int stalled;
    GenoBhv_CapeParams(&p);
    stalled = n > p.start && n <= p.end + 1;
    if (gb_airborne(fp)) {
        if (!stalled) {
            geno_cb_phys_air(gobj);
        }
    } else if (stalled) {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    } else {
        ft_80084F3C(gobj);
    }
}

static void geno_cape_coll(Fighter_GObj* gobj)
{
    geno_coll_both(gobj);
}

static void geno_cape_after_coll(Fighter_GObj* gobj)
{
    geno_cb_coll_anim_motion(gobj);
}

/* ---- multi-dash ----------------------------------------------------------------------------------- */

static void gb_md_load(GenoState* st, GbMultiDash* m)
{
    s32 w = st->move_i[0];
    m->phase = w & 15;
    m->seg = (w >> 4) & 15;
    m->repress = (w >> 8) & 1;
    m->helpless = (w >> 9) & 1;
    m->done = (w >> 10) & 1;
    m->t = st->move_i[1];
    m->hit_spawn = 0;
    m->ax = st->move_f[0];
    m->ay = st->move_f[1];
    m->f = st->move_f[2];
    m->speed = st->move_f[3];
    m->vx = st->move_f[4];
    m->vy = st->move_f[5];
    m->dist = st->move_f[6];
}

static void gb_md_store(GenoState* st, const GbMultiDash* m)
{
    st->move_i[0] = m->phase | (m->seg << 4) | (m->repress << 8) | (m->helpless << 9) | (m->done << 10);
    st->move_i[1] = m->t;
    st->move_f[0] = m->ax;
    st->move_f[1] = m->ay;
    st->move_f[2] = m->f;
    st->move_f[3] = m->speed;
    st->move_f[4] = m->vx;
    st->move_f[5] = m->vy;
    st->move_f[6] = m->dist;
}

static void geno_multi_dash_enter(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s)
{
    GbMultiDash m;
    int air = gb_airborne(fp);
    GenoBhv_MultiDashBegin(&m, fp->facing_dir < 0.0f ? -1.0f : 1.0f);
    m.vx = air ? fp->self_vel.x : fp->gr_vel;
    m.vy = air ? fp->self_vel.y : 0.0f;
    gb_md_store(st, &m);
}

/* One frame, from the anim callback: aim, dash, pause, end lag. The procedure owns the velocity for the
 * whole move (no gravity, spec 6.2); each segment's hits are the clip's hitboxes re-armed (the victim
 * lists cleared) on the segment's first dash frame (DASH_HIT(seg)). */
static void geno_multi_dash_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    GbMultiDashP p;
    GbMultiDash m;
    GbMultiDashIn in;
    int air = gb_airborne(fp);
    GenoBhv_MultiDashParams(&p);
    gb_md_load(st, &m);
    gb_stick_of(fp, &in.sx, &in.sy);
    in.special_pressed = GB_PRESSED(fp, HSD_PAD_B);
    GenoBhv_MultiDashStep(&m, &in, &p);
    fp->facing_dir = m.f;
    if (air) {
        fp->self_vel.x = m.vx;
        fp->self_vel.y = m.vy;
    } else {
        fp->gr_vel = m.vx;
    }
    if (m.hit_spawn && m.t == 1) {
        gb_clear_hit_lists(fp);
    }
    gb_md_store(st, &m);
    if (m.done) {
        gb_finish(gobj, fp, st, air && m.helpless);
    }
}

static void geno_multi_dash_phys(Fighter_GObj* gobj)
{
    if (!gb_airborne(GET_FIGHTER(gobj))) {
        ftCommon_SetSelfMovementFromGroundedMovement(gobj);
    }
}

static void geno_multi_dash_coll(Fighter_GObj* gobj)
{
    geno_coll_both(gobj);
}

/* ---- anim_motion ---------------------------------------------------------------------------------- */

/* geno.anim_motion's per-frame displacement through the spec 7 procedure (DELTA mode, rate 1, scale 1).
 * The host (Melee's animation system) samples the clip's root track: this frame's two samples are the
 * previous root offset (0, 0) and this frame's TransN delta (fwd, up), so P = [(0,0), (fwd,up)] and one
 * step plays it. Gravity and collision stay with the host (geno_cb_phys_anim_motion). */
void GenoBhv_AnimMotionFrameDelta(f32 fwd, f32 up, f32 lr, f32* dx, f32* dy)
{
    f32 track[4];
    GbAnimMotionP p;
    GbAnimMotion a;
    track[0] = 0.0f;
    track[1] = 0.0f;
    track[2] = fwd;
    track[3] = up;
    GenoBhv_AnimMotionParams(&p);
    GenoBhv_AnimMotionBegin(&a, 0.0f, 0.0f);
    GenoBhv_AnimMotionStep(&a, track, 2, &p, lr, 0);
    *dx = a.vx;
    *dy = a.vy;
}

/* ---- iasa "interrupt" ----------------------------------------------------------------------------- */

/* The state may be cancelled into Melee's ordinary actions once Allow Interrupt is set (an IASA
 * script command). Ground: attacks, jump, dash, turn, walk. Air: the aerial interrupt set. */
static void geno_cb_iasa_interrupt(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (!fp->allow_interrupt) {
        return;
    }
    if (fp->ground_or_air == GA_Ground) {
        if (ftCo_AttackS4_CheckInput(gobj) || ftCo_Jump_CheckInput(gobj) || ftCo_Dash_CheckInput(gobj) ||
            ftCo_Turn_CheckInput(gobj) || ftCo_Walk_CheckInput(gobj))
        {
            return;
        }
    } else {
        if (ftCo_80095328(gobj, NULL) || ftCo_800D7100(gobj) || ftCo_800C3B10(gobj) || ftCo_800D705C(gobj) ||
            ftCo_800CB870(gobj))
        {
            return;
        }
    }
}
