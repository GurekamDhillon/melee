/*
 * script_game.c - the game-side half of the Lua scripting API (pc/platform/gw_script.c).
 *
 * Game-world code: compiled for ppc32 and run through gwtool like the rest of melee, so the
 * fighter and player structs are read with the right byte order and nothing here swaps by hand.
 * The native side calls these as gw_ScriptGame_* (gwtool prefixes every game symbol) and only
 * ever passes and receives scalars, so no struct crosses the boundary.
 *
 * Fields are numbered (SCRIPT_F_* / SCRIPT_I_*); gw_script.c owns the names scripts see.
 */

#include <Runtime/platform.h>

#include <melee/ft/fighter.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>
#include <melee/gm/gm_1A3F.h>
#include <melee/gm/gmmain_lib.h>
#include <melee/gm/gmscene.h>
#include <melee/gr/ground.h>
#include <melee/gr/types.h>
#include <melee/pl/player.h>
#include <melee/it/types.h>
#include <melee/it/forward.h>
#include <melee/it/inlines.h>
#include <melee/it/item.h>
#include <melee/it/it_3F14.h>
#include <melee/it/it_26B1.h>
#include <melee/it/itzako.h>
#include <melee/it/kinds/itleadead.h>
#include <melee/it/kinds/itlikelike.h>
#include <melee/it/kinds/itnokonoko.h>
#include <melee/lb/lbarchive.h>
#include <melee/mp/mpcoll.h>
#include "../geno/geno.h"
#include "script_items.h"
#include "script_model.h"
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjobject.h>
#include <sysdolphin/baselib/memory.h>
#include <melee/mp/mplib.h>
#include <melee/mp/types.h>
#include <melee/lb/lbarchive.h>
#include <melee/lb/lbfile.h>
#include <melee/lb/lbheap.h>
#include <math.h>
#include <string.h>
#include <dolphin/dvd.h>
#include <dolphin/gx.h>
#include <dolphin/mtx.h>
#include <melee/gr/forward.h>
#include <sysdolphin/baselib/cobj.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <sysdolphin/baselib/gobjplink.h>
#include <sysdolphin/baselib/state.h>
#include <sysdolphin/baselib/tev.h>
#include <sysdolphin/baselib/archive.h>
#include <sysdolphin/baselib/jobj.h>

/* Each scripted line owns two vertices and one joint. mpCheckFloor and its wall/ceiling
 * siblings walk joint ranges (mplib.c), so a single appended global range cannot mix kinds.
 * These arrays live in MEM1; the pool below is this object's BSS, which gw_snap.c saves
 * (pc_gameworld_script_game.c.obj is in its game set since this change). */
/* largemap: retain mpIsland's 1536-line scratch limit and signed vertex indices.
 * Only the joint allocation grows; joint storage is allocated once per scene. */
#define SCRIPT_STAGE_LINES 768
#define SCRIPT_STAGE_JOINTS 1024
#define SCRIPT_STAGE_TARGETS 128
#define SCRIPT_STAGE_AREAS 64
#define SCRIPT_STAGE_ENEMIES 32
#define SCRIPT_ENEMY_KINDS 7 /* enemies2: append kinds to preserve the Lua event indices */
#define SCRIPT_STAGE_MODELS 64
#define SCRIPT_STAGE_ARCHIVES 8
#define SCRIPT_STAGE_ARCHIVE_MAX (8 * 1024 * 1024)
typedef struct {
    int handle, active, kind, flags, model_handle, area;
    float x0, y0, x1, y1;
} ScriptStageLine;
typedef struct {
    int handle, active, area;
    Item_GObj* gobj;
    float x, y;
} ScriptStageTarget;
typedef struct {
    int handle, kind, active, defeated;
    Item_GObj* gobj;
} ScriptStageEnemy;
typedef struct {
    int handle, active, line_handle;
    HSD_GObj* gobj;
    float x, y, z, scale, rot;
} ScriptStageModel;
typedef struct {
    char file[32];
    void* data;
    HSD_Archive* archive;
    size_t bytes;
} ScriptStageArchive;
extern int Script_ModelInput(int field, int line);
extern void Script_ModelDraw(int slot, Mtx view);
extern void Script_StageModelsReset(void);
static int script_mesh_bits(float f) { union { float f; int i; } u; u.f = f; return u.i; }
static float script_mesh_float(int i) { union { float f; int i; } u; u.i = i; return u.f; }
static struct {
    MapCollData* map;
    int base_v, base_l, base_j, target_remaining;
    int target_model_ready;
    int cap;           /* lines reserved on this stage (<= SCRIPT_STAGE_LINES) */
    HSD_GObj* draw;    /* the world-pass drawing GObj (gxlink 3, with the stage) */
    u8* cube;          /* the unit box's 8 corners, MEM1 (a GX_INDEX8 position array) */
    ScriptStageLine line[SCRIPT_STAGE_LINES];
    ScriptStageTarget target[SCRIPT_STAGE_TARGETS];
    ScriptStageEnemy enemy[SCRIPT_STAGE_ENEMIES];
    int enemy_ready[SCRIPT_ENEMY_KINDS];
    int enemy_attempted[SCRIPT_ENEMY_KINDS];
    HSD_Archive* enemy_archive[3];
    ScriptStageModel model[SCRIPT_STAGE_MODELS];
    ScriptStageArchive archives[SCRIPT_STAGE_ARCHIVES];
    ScriptMeshInstance instance[SCRIPT_MESH_INSTANCES];
    struct { int token, refs, instances; } asset[SCRIPT_MESH_ASSETS];
    /* largemap: memberships and names are snapshot state, not Lua bookkeeping. */
    struct { int owner; char name[48]; } area[SCRIPT_STAGE_AREAS];
    int loading_area, bounds_saved;
    StageBlastZone saved_camera, saved_blast;
} script_stage;
static Article* script_target_old_article;
#include "script_bounds.inc"

/* ---- arena-hooks: deterministic origin, bounds and retail collision groups ---- */
#include "script_arena.inc"

static int script_stage_same_file(const char* a, const char* b)
{
    for (; *a && *b; ++a, ++b) {
        int ac = *a, bc = *b;
        if (ac >= 'A' && ac <= 'Z') ac += 'a' - 'A';
        if (bc >= 'A' && bc <= 'Z') bc += 'a' - 'A';
        if (ac != bc) return 0;
    }
    return *a == *b;
}

/* Called at the end of gm_801A4D34, while this scene's objects still exist. */
void ScriptGame_StageEnd(void)
{
    int i;
    /* arena-hooks: no scene's ownership or transition can survive teardown. */
    memset(&script_arena, 0, sizeof script_arena);
    if (script_stage.bounds_saved) {
        extern void Camera_LargeMapRestore(void);
        Camera_LargeMapRestore();
        stage_info.cam_info.cam_bounds = script_stage.saved_camera;
        stage_info.blast_zone = script_stage.saved_blast;
    }
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i) {
        if (script_stage.model[i].active && script_stage.model[i].gobj != NULL)
            HSD_GObjFree(script_stage.model[i].gobj);
        script_stage.model[i].active = 0;
        script_stage.model[i].gobj = NULL;
    }
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i) {
        if (script_stage.target[i].active && script_stage.target[i].gobj != NULL) {
            Item_GObj* gobj = script_stage.target[i].gobj;
            script_stage.target[i].active = 0;
            script_stage.target[i].gobj = NULL;
            Item_8026A8EC(gobj);
        }
    }
    if (script_stage.draw != NULL) HSD_GObjFree(script_stage.draw);
    script_stage.draw = NULL;
    if (script_stage.target_model_ready) {
        it_804A0F60[It_Kind_Mato - It_Kind_Old_Kuri] = script_target_old_article;
        script_target_old_article = NULL;
    }
    for (i = 0; i < SCRIPT_STAGE_ARCHIVES; ++i) {
        ScriptStageArchive* a = &script_stage.archives[i];
        if (a->archive != NULL) {
            OSReport("script stage: release /%s (%u bytes, heap 0)\n", a->file,
                     (unsigned) a->bytes);
            lbHeap_80015CA8(0, a->archive);
            lbHeap_80015CA8(0, a->data);
            a->archive = NULL;
            a->data = NULL;
            a->file[0] = 0;
        }
    }
    if (script_stage.map != NULL) {
        /* mpLib still owns references into these scene-heap buffers until the next
         * scene resets the heap. Retire the appended ranges, but do not free the
         * backing map out from under the scene's on_exit callback. */
        script_stage.map->vert_count = script_stage.base_v;
        script_stage.map->line_count = script_stage.base_l;
        script_stage.map->joint_count = script_stage.base_j;
        OSReport("script stage: released %d collision lines and %d targets at scene end\n",
                 script_stage.cap, SCRIPT_STAGE_TARGETS);
    }
    if (script_stage.cube != NULL) HSD_Free(script_stage.cube);
    /* Invalidate every handle now, including enemies and lines. StagePrepare may
     * not run in the next scene (CSS, menus), so it cannot own this reset. */
    memset(&script_stage, 0, sizeof(script_stage));
    Script_StageModelsReset(); /* native mesh/atlas bytes; the memset above cleared the instances */
}

static HSD_Archive* script_stage_archive(const char* file)
{
    int i, entry;
    size_t bytes, read_bytes;
    HSD_Archive* archive;
    void* data;
    ScriptStageArchive* a = NULL;
    char path[34];
    for (i = 0; i < SCRIPT_STAGE_ARCHIVES; ++i) {
        if (script_stage.archives[i].archive != NULL &&
            script_stage_same_file(script_stage.archives[i].file, file))
            return script_stage.archives[i].archive;
        if (a == NULL && script_stage.archives[i].archive == NULL) a = &script_stage.archives[i];
    }
    if (a == NULL) return NULL;
    path[0] = '/';
    strcpy(path + 1, file);
    entry = DVDConvertPathToEntrynum(path); /* shim_dvd also finds a mounted mod's own DAT */
    if (entry < 0) return NULL;
    bytes = lbFile_8001634C(entry);
    if (bytes < sizeof(HSD_ArchiveHeader) || bytes > SCRIPT_STAGE_ARCHIVE_MAX) return NULL;
    data = lbHeap_80015BD0(0, (bytes + 31) & ~(size_t) 31);
    archive = lbHeap_80015BD0(0, sizeof(*archive));
    if (data == NULL || archive == NULL) {
        if (archive != NULL) lbHeap_80015CA8(0, archive);
        if (data != NULL) lbHeap_80015CA8(0, data);
        return NULL;
    }
    lbFile_8001668C(path, data, &read_bytes);
    if (read_bytes != bytes || HSD_ArchiveParse(archive, data, bytes) < 0) {
        lbHeap_80015CA8(0, archive);
        lbHeap_80015CA8(0, data);
        return NULL;
    }
    /* Stage DATs have no unresolved externs; use the same relocation path as lbArchive. */
    for (i = 0; HSD_ArchiveGetExtern(archive, i) != NULL; ++i)
        HSD_ArchiveLocateExtern(archive, HSD_ArchiveGetExtern(archive, i), NULL);
    strcpy(a->file, file);
    a->data = data;
    a->archive = archive;
    a->bytes = bytes;
    OSReport("script stage: loaded /%s (%u bytes, heap 0)\n", file, (unsigned) bytes);
    return archive;
}

static const int script_enemy_kinds[SCRIPT_ENEMY_KINDS] = {
    /* enemies2: active Ottosea uses ItCo.usd's Topi model (TyToppi texture match).
     * ItCo.dat supplies the Japanese seal variant; retain the game's locale choice.
     * It_Kind_Old_Otto is the obsolete stage slot, not this common article. */
    It_Kind_Kuriboh, It_Kind_Nokonoko, It_Kind_Leadead,
    It_Kind_Likelike, It_Kind_Octarock, It_Kind_Whitebea, It_Kind_Ottosea
};

static int script_enemy_index(int kind)
{
    int i;
    for (i = 0; i < SCRIPT_ENEMY_KINDS; ++i)
        if (script_enemy_kinds[i] == kind) return i;
    return -1;
}

/* Ground_801C0800 registers only the current stage's itemdata. The three late monster kinds
 * use itemdata from their Adventure archives, so pin those archives in the game heap before a
 * script can spawn on a VS stage. First-range monsters and Octorok's stone live in ItCo.dat. */
static int script_enemy_preload(int which)
{
    static const char* const files[3] = { "/GrNKr.dat", "/GrNSr.dat", "/GrIm.dat" };
    static const int kinds[3] = { It_Kind_Nokonoko, It_Kind_Likelike, It_Kind_Whitebea };
    struct GroundItemData** rows;
    int i, j;
    if (script_stage.enemy_attempted[which]) return script_stage.enemy_ready[which];
    script_stage.enemy_attempted[which] = 1;
    if (script_enemy_kinds[which] < It_Kind_Old_Kuri) {
        int kind = script_enemy_kinds[which];
        script_stage.enemy_ready[which] = it_804D6D38 != NULL &&
            it_804D6D38[kind - It_Kind_Kuriboh] != NULL;
        OSReport("script enemy: ItCo kind=%d %s\n", kind,
                 script_stage.enemy_ready[which] ? "ready" : "missing article");
        return script_stage.enemy_ready[which];
    }
    i = which == 1 ? 0 : which == 3 ? 1 : 2;
    rows = stage_info.itemdata;
    for (j = 0; rows != NULL && j < 64 && rows[j] != NULL; ++j) {
        if (rows[j]->unk0 == kinds[i] && rows[j]->unk4 != NULL) {
            it_8026B40C(rows[j]->unk4, kinds[i]);
            script_stage.enemy_ready[which] = 1;
            break;
        }
    }
    if (script_stage.enemy_ready[which]) return 1;
    rows = NULL;
    lbArchive_800171CC(&script_stage.enemy_archive[i], files[i], &rows, "itemdata", 0);
    /* every row of the archive, not only the monster's own: a Koopa spawns its shell (and the
     * route's other monsters their projectiles) as separate stage items, and a kind with no
     * article asserts in Item_802675A8 ("not found zako model data"). Rows the running stage
     * already registered are left alone. */
    for (j = 0; rows != NULL && j < 64 && rows[j] != NULL; ++j) {
        int k = rows[j]->unk0;
        if (rows[j]->unk4 == NULL || k < It_Kind_Old_Kuri) continue;
        if (it_804A0F60[k - It_Kind_Old_Kuri] == NULL) {
            it_8026B40C(rows[j]->unk4, k);
            OSReport("script enemy: registered stage item kind=%d from %s\n", k, files[i]);
        }
        if (k == kinds[i]) script_stage.enemy_ready[which] = 1;
    }
    OSReport("script enemy: preload %s kind=%d %s\n", files[i], kinds[i],
             script_stage.enemy_ready[which] ? "ready" : "missing itemdata");
    return script_stage.enemy_ready[which];
}

int ScriptGame_StageGameplayScene(void)
{
    extern struct GameSceneInfo* gm_804D6720;
    /* gm_801A4014 installs this before scene->on_enter loads Ground_801C0800.
     * Script_SceneBegin and the first fighter frame have not happened yet. */
    if (gm_804D6720 == NULL) return 0;
    switch (gm_804D6720->scene_kind) {
    case GS_VS:
    case GS_SUDDEN_DEATH:
    case GS_TRAINING:
    case GS_CAMERA_VS:
        return 1;
    default:
        return 0;
    }
}
extern void Script_StageModelsReset(void);
extern int Script_StageModelFor(int handle);
/* Native Aurora owns the immutable mesh and GXTX bytes. This legacy floor call only
 * issues GX commands; runtime model selection instead lives in ScriptMeshInstance. */
extern void Script_StageModelDraw(int model, Mtx view, float x0, float y0, float x1, float y1);

/* Called from Ground_801C0800 after the Target Test layout merge and before mpLibLoad.
 * mpLibLoad keeps 2048/1536 CollVtx/CollLine arrays; largemap sizes joints on demand. */
MapCollData* ScriptGame_StagePrepare(MapCollData* src)
{
    extern MapCollData mpLib_803BF760;
    MapCollData* dst;
    int i;
    /* arena-hooks: count retail joints before the script pool is appended. */
    memset(&script_arena, 0, sizeof script_arena);
    script_arena.groups = src ? src->joint_count : mpLib_803BF760.joint_count;
    script_stage.map = NULL;
    script_stage.target_remaining = 0;
    script_stage.target_model_ready = 0;
    script_stage.cap = 0;
    script_stage.draw = NULL;
    script_stage.cube = NULL;
    Script_StageModelsReset();
    memset(script_stage.instance, 0, sizeof script_stage.instance);
    memset(script_stage.asset, 0, sizeof script_stage.asset);
    memset(script_stage.area, 0, sizeof script_stage.area);
    script_stage.loading_area = script_stage.bounds_saved = 0;
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i) script_stage.line[i].active = 0;
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i) script_stage.model[i].active = 0;
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i) {
        script_stage.target[i].active = 0;
        script_stage.target[i].gobj = NULL;
    }
    for (i = 0; i < SCRIPT_STAGE_ENEMIES; ++i) {
        script_stage.enemy[i].active = 0;
        script_stage.enemy[i].gobj = NULL;
    }
    for (i = 0; i < 3; ++i) script_stage.enemy_archive[i] = NULL;
    for (i = 0; i < SCRIPT_ENEMY_KINDS; ++i) {
        script_stage.enemy_ready[i] = 0;
        script_stage.enemy_attempted[i] = 0;
    }
    if (src == NULL) src = &mpLib_803BF760; /* mpLibLoad's own fallback map */
    {
        /* Reserve offline even without a loaded script: runtime mod loading needs this
         * capacity later. Online retains the original map and heap layout. */
        extern int Script_StageWanted(void);
        if (!Script_StageWanted()) return src;
    }
    {
        /* mpIsland's visited[0x600] is indexed by line id: do not grow past 1536. */
        int cap = SCRIPT_STAGE_LINES;
        if (cap > (2048 - src->vert_count) / 2) cap = (2048 - src->vert_count) / 2;
        if (cap > 1536 - src->line_count) cap = 1536 - src->line_count;
        if (cap > SCRIPT_STAGE_JOINTS - src->joint_count) cap = SCRIPT_STAGE_JOINTS - src->joint_count;
        if (cap < 1) return src;
        script_stage.cap = cap;
    }
    dst = HSD_MemAlloc(sizeof(*dst));
    if (dst == NULL) return src;
    *dst = *src;
    dst->verts = HSD_MemAlloc((src->vert_count + script_stage.cap * 2) * sizeof(*dst->verts));
    dst->lines = HSD_MemAlloc((src->line_count + script_stage.cap) * sizeof(*dst->lines));
    dst->joints = HSD_MemAlloc((src->joint_count + script_stage.cap) * sizeof(*dst->joints));
    if (dst->verts == NULL || dst->lines == NULL || dst->joints == NULL) {
        if (dst->verts) HSD_Free(dst->verts);
        if (dst->lines) HSD_Free(dst->lines);
        if (dst->joints) HSD_Free(dst->joints);
        HSD_Free(dst);
        script_stage.cap = 0;
        return src;
    }
    for (i = 0; i < src->vert_count; ++i) dst->verts[i] = src->verts[i];
    for (i = 0; i < src->line_count; ++i) dst->lines[i] = src->lines[i];
    for (i = 0; i < src->joint_count; ++i) dst->joints[i] = src->joints[i];
    script_stage.base_v = src->vert_count;
    script_stage.base_l = src->line_count;
    script_stage.base_j = src->joint_count;
    script_stage.map = dst;
    return dst;
}


/* ---- in-game geometry: the scripted lines and targets, drawn with the stage ----------------
 * A GObj on the stage's gxlink (3) draws in the main camera's world pass, after the stage's fog
 * is set, so the shapes are in screenshots, fogged, depth-tested against the stage and fighters.
 * Every shape is the same unit box (8 corners in MEM1, indexed), placed by its own position
 * matrix (view x model): the uncapped renderer pairs a draw's matrix loads frame to frame by
 * (slot, the draw's vertex bytes, the POS array), which are all fixed here, so a moving platform
 * is blended between real frames like the stage's own parts. Colours are per vertex (a lit top,
 * mid front, dark sides), no lighting or texture. Skipped in a re-simulated frame. */
extern int gx_suppress_draws;
static int script_stage_geometry = 1; /* gd.stage_view: 0 off (debug overlay only) */

static const float script_cube_pts[8][3] = {
    { -0.5f, -0.5f, -0.5f }, { 0.5f, -0.5f, -0.5f }, { 0.5f, 0.5f, -0.5f }, { -0.5f, 0.5f, -0.5f },
    { -0.5f, -0.5f, 0.5f },  { 0.5f, -0.5f, 0.5f },  { 0.5f, 0.5f, 0.5f },  { -0.5f, 0.5f, 0.5f },
};
/* six faces, 4 corners each, and the shade each face gets (x/256) */
static const u8 script_cube_faces[6][5] = {
    { 3, 2, 6, 7, 255 }, /* top    (+y) */
    { 4, 5, 6, 7, 210 }, /* front  (+z, toward the camera) */
    { 0, 1, 2, 3, 150 }, /* back   (-z) */
    { 1, 5, 6, 2, 170 }, /* right  (+x) */
    { 0, 3, 7, 4, 170 }, /* left   (-x) */
    { 0, 4, 5, 1, 110 }, /* bottom (-y) */
};

static HSD_Chan script_stage_chan = {
    NULL, GX_COLOR0A0, 0, { 0, 0, 0, 0 }, { 0xFF, 0xFF, 0xFF, 0xFF }, 0,
    GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_CLAMP, GX_AF_NONE, NULL,
};

static void script_box(Mtx view, float ux, float uy, float len, float thick, float depth, float cx,
                       float cy, u32 rgb)
{
    Mtx m, mv;
    int f, k;
    /* columns: x along the line (length), y its left normal (thickness), z depth */
    m[0][0] = ux * len; m[0][1] = -uy * thick; m[0][2] = 0.0f; m[0][3] = cx;
    m[1][0] = uy * len; m[1][1] = ux * thick;  m[1][2] = 0.0f; m[1][3] = cy;
    m[2][0] = 0.0f;     m[2][1] = 0.0f;        m[2][2] = depth; m[2][3] = 0.0f;
    PSMTXConcat(view, m, mv);
    GXLoadPosMtxImm(mv, GX_PNMTX0);
    GXBegin(GX_QUADS, GX_VTXFMT0, 24);
    for (f = 0; f < 6; ++f) {
        u32 s = script_cube_faces[f][4];
        u8 r = (u8) ((((rgb >> 16) & 0xFF) * s) >> 8), g = (u8) ((((rgb >> 8) & 0xFF) * s) >> 8);
        u8 b = (u8) (((rgb & 0xFF) * s) >> 8);
        for (k = 0; k < 4; ++k) {
            GXPosition1x8(script_cube_faces[f][k]);
            GXColor4u8(r, g, b, 0xFF);
        }
    }
    GXEnd();
}

#include "script_model_order.inc"
static void script_mesh_render(Mtx view, int alpha)
{
    int order[SCRIPT_MESH_INSTANCES], i;
    int count = script_mesh_order(view, alpha, order);
    Script_ModelDraw(-1, view);
    for (i = 0; i < count; ++i) Script_ModelDraw(order[i], view);
    Script_ModelDraw(-2, view); /* flush before another GObj changes GX state */
}

static void script_stage_render(HSD_GObj* gobj, int code)
{
    Mtx view;
    HSD_TevDesc tev;
    int i;
    static int seen;
    (void) gobj;
    if (!(seen & (1 << (code & 7)))) {
        seen |= 1 << (code & 7);
        OSReport("script stage: world pass render code %d\n", code); /* once per pass code */
    }
    if ((code != 0 && code != 2) || !script_stage_geometry || gx_suppress_draws || script_stage.cube == NULL) {
        return;
    }
    if (code == 2) {
        HSD_CObjGetViewingMtx(HSD_CObjGetCurrent(), view);
        HSD_StateInvalidate(-1);
        script_mesh_render(view, 1);
        HSD_StateInvalidate(-1);
        return;
    }
    HSD_StateInvalidate(-1);
    HSD_StateInitTev();
    tev.flags = 0;
    tev.stage = HSD_StateAssignTev();
    tev.coord = 0xFF;
    tev.map = 0xFF;
    tev.color = 4;
    tev.u.tevop.tevmode = 4; /* the rasterised (vertex) colour, as mpLib_SetupDraw */
    HSD_SetupTevStage(&tev);
    HSD_SetupPEMode(1, NULL);
    HSD_SetTevRegAll();
    HSD_StateSetNumTevStages();
    HSD_StateSetNumTexGens();
    HSD_StateSetNumChans(1);
    HSD_SetupChannel(&script_stage_chan);
    GXSetCullMode(GX_CULL_NONE);
    GXSetZMode(GX_TRUE, GX_LEQUAL, GX_TRUE);
    GXClearVtxDesc();
    GXSetVtxDesc(GX_VA_POS, GX_INDEX8);
    GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
    GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    GXSetArray(GX_VA_POS, script_stage.cube, 12);
    GXSetCurrentMtx(GX_PNMTX0);
    HSD_CObjGetViewingMtx(HSD_CObjGetCurrent(), view);
    for (i = 0; i < script_stage.cap; ++i) {
        ScriptStageLine* s = &script_stage.line[i];
        float dx, dy, len, ux, uy, mx, my;
        int model;
        if (!s->active || s->model_handle) continue;
        model = s->kind == 1 ? Script_StageModelFor(s->handle) : 0;
        if (model > 0)
            continue; /* textured model is drawn in the one-material pass below */
        dx = s->x1 - s->x0;
        dy = s->y1 - s->y0;
        len = sqrtf(dx * dx + dy * dy);
        if (len < 0.001f) continue;
        ux = dx / len;
        uy = dy / len;
        mx = (s->x0 + s->x1) * 0.5f;
        my = (s->y0 + s->y1) * 0.5f;
        if (s->kind == 1) {
            /* a floor: a slab hanging under the line (its left normal is up), gold when solid,
             * cyan when it can be dropped through */
            const float t = 5.0f;
            script_box(view, ux, uy, len, t, 30.0f, mx + uy * t * 0.5f, my - ux * t * 0.5f,
                       (s->flags & 1) ? 0x38C9D9u : 0xF0B429u);
        } else {
            /* a wall or ceiling: a thin bar on the line itself */
            script_box(view, ux, uy, len, 1.5f, 8.0f, mx, my, s->kind == 2 ? 0xC77DFFu : 0xE5483Bu);
        }
    }
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i) {
        ScriptStageTarget* t = &script_stage.target[i];
        if (!t->active) continue;
        if (script_stage.target_model_ready) continue;
        /* a target: a diamond (the unit box turned 45 degrees), red with a white core */
        script_box(view, 0.70710678f, 0.70710678f, 7.0f, 7.0f, 3.0f, t->x, t->y, 0xE5483Bu);
        script_box(view, 0.70710678f, 0.70710678f, 3.0f, 3.0f, 3.4f, t->x, t->y, 0xF2EFE4u);
    }
    {
        int any = 0;
        for (i = 0; i < script_stage.cap; ++i) {
            ScriptStageLine* s = &script_stage.line[i];
            int model;
            if (!s->active || s->kind != 1) continue;
            model = Script_StageModelFor(s->handle);
            if (model < 1) continue;
            if (!any) { HSD_StateInvalidate(-1); any = 1; }
            Script_StageModelDraw(model, view, s->x0, s->y0, s->x1, s->y1);
        }
    }
    script_mesh_render(view, 0);
    HSD_StateInvalidate(-1);
}

static void script_stage_model_render(HSD_GObj* gobj, int code)
{
    if (!script_stage_geometry || gx_suppress_draws) return;
    HSD_StateInvalidate(-1);
    HSD_GObj_JObjCallback(gobj, code); /* the stage's lit, textured JObj path */
}

/* at stage load (ScriptGame_StageReady): the unit box's corners in MEM1 and the drawing GObj */
static void script_stage_draw_init(void)
{
    int i, k;
    float* p = HSD_MemAlloc(sizeof(script_cube_pts));
    if (p == NULL) return;
    for (i = 0; i < 8; ++i) {
        for (k = 0; k < 3; ++k) p[i * 3 + k] = script_cube_pts[i][k];
    }
    script_stage.cube = (u8*) p;
    /* plink 13: a list nothing walks as Ground / Fighter / Item user data */
    script_stage.draw = GObj_Create(HSD_GOBJ_CLASS_STAGE, 13, 0);
    if (script_stage.draw != NULL) {
        GObj_SetupGXLink(script_stage.draw, script_stage_render, 3, 0);
    }
}

void ScriptGame_StageSetDraw(int on)
{
    script_stage_geometry = on != 0;
}

void ScriptGame_StageReady(void)
{
    MapCollData* map = script_stage.map;
    CollVtx* cv = mpGetGroundCollVtx();
    CollLine* cl = mpGetGroundCollLine();
    CollJoint* cj = mpGetGroundCollJoint();
    int i, k;
    /* A visual-only model also works when the map has no spare collision joints. */
    { extern int Script_StageWanted(void);
      if (Script_StageWanted()) script_stage_draw_init(); }
    if (map == NULL || mpLib_8004D164() != map) return;
    for (i = 0; i < script_stage.cap; ++i) {
        int v = script_stage.base_v + i * 2, l = script_stage.base_l + i;
        MapLine* ml = &map->lines[l];
        MapJoint* mj = &map->joints[script_stage.base_j + i];
        ml->v0_idx = v;
        ml->v1_idx = v + 1;
        ml->prev_id0 = ml->next_id0 = ml->prev_id1 = ml->next_id1 = -1;
        ml->hi_flags = ml->lo_flags = 0;
        cl[l].x0 = ml;
        cl[l].flags = 0;
        for (k = 0; k < 2; ++k) {
            cv[v + k].x0 = cv[v + k].x4 = 0;
            cv[v + k].pos.x = cv[v + k].pos.y = 0;
            cv[v + k].x10 = cv[v + k].x14 = 0;
            map->verts[v + k].x = map->verts[v + k].y = 0;
        }
        for (k = 0; k < MapLineGroup_Count; ++k) {
            mj->ranges[k].start = 0;
            mj->ranges[k].count = 0;
        }
        mj->vtx_start = v;
        mj->vtx_count = 2;
        mj->left_bound = mj->right_bound = mj->bottom_bound = mj->top_bound = 0;
        cj[script_stage.base_j + i].next = NULL;
        cj[script_stage.base_j + i].inner = mj;
        cj[script_stage.base_j + i].flags = 0;
        cj[script_stage.base_j + i].x20 = NULL;
        cj[script_stage.base_j + i].cb_0 = cj[script_stage.base_j + i].cb_1 = NULL;
        cj[script_stage.base_j + i].cb_data_0 = cj[script_stage.base_j + i].cb_data_1 = NULL;
        cj[script_stage.base_j + i].xE = false;
    }
    map->vert_count += script_stage.cap * 2;
    map->line_count += script_stage.cap;
    map->joint_count += script_stage.cap;
    OSReport("script stage: reserved %d collision lines and %d targets (the stage uses %d/2048 "
             "vertices, %d/1536 lines, %d base joints; largemap)\n",
             script_stage.cap, SCRIPT_STAGE_TARGETS, script_stage.base_v, script_stage.base_l,
             script_stage.base_j);
}

int ScriptGame_StageAddLine(int x0b, int y0b, int x1b, int y1b, int kind, int flags,
                            int handle)
{
    union { int i; float f; } u;
    float x0, y0, x1, y1, scale;
    MapCollData* map = script_stage.map;
    int i, v, l, j;
    MapLine* ml;
    MapJoint* mj;
    CollVtx* cv;
    CollLine* cl;
    CollJoint* cj;
    if (map == NULL || mpLib_8004D164() != map || kind < 1 || kind > 4) return -1;
    u.i = x0b; x0 = u.f; u.i = y0b; y0 = u.f;
    u.i = x1b; x1 = u.f; u.i = y1b; y1 = u.f;
    if (x0 == x1 && y0 == y1) return -1;
    for (i = 0; i < script_stage.cap && script_stage.line[i].active; ++i) {}
    if (i >= script_stage.cap) return -1;
    v = script_stage.base_v + 2 * i;
    l = script_stage.base_l + i;
    j = script_stage.base_j + i;
    ml = &map->lines[l]; mj = &map->joints[j];
    cv = mpGetGroundCollVtx(); cl = mpGetGroundCollLine(); cj = mpGetGroundCollJoint();
    scale = Ground_801C0498();
    if (scale <= 0.001f) scale = 1.0f;
    cv[v].x0 = x0 / scale; cv[v].x4 = y0 / scale;
    cv[v + 1].x0 = x1 / scale; cv[v + 1].x4 = y1 / scale;
    cv[v].pos.x = cv[v].x10 = x0; cv[v].pos.y = cv[v].x14 = y0;
    cv[v + 1].pos.x = cv[v + 1].x10 = x1; cv[v + 1].pos.y = cv[v + 1].x14 = y1;
    map->verts[v].x = cv[v].x0; map->verts[v].y = cv[v].x4;
    map->verts[v + 1].x = cv[v + 1].x0; map->verts[v + 1].y = cv[v + 1].x4;
    /* mpIsland_8005B004 builds floor/ceiling islands from the dynamic range when
     * mpJointListAdd runs. The extra 0x10 identifies its dynamic line type. */
    mj->ranges[MapLineGroup_Dynamic].start = l;
    mj->ranges[MapLineGroup_Dynamic].count = 1;
    mj->left_bound = (x0 < x1 ? x0 : x1) / scale;
    mj->right_bound = (x0 > x1 ? x0 : x1) / scale;
    mj->bottom_bound = (y0 < y1 ? y0 : y1) / scale;
    mj->top_bound = (y0 > y1 ? y0 : y1) / scale;
    cj[j].bounding_min.x = (x0 < x1 ? x0 : x1) - 30;
    cj[j].bounding_max.x = (x0 > x1 ? x0 : x1) + 30;
    cj[j].bounding_min.y = (y0 < y1 ? y0 : y1) - 30;
    cj[j].bounding_max.y = (y0 > y1 ? y0 : y1) + 30;
    ml->hi_flags = (1 << (kind - 1)) | 0x10;
    if (flags & 1) ml->hi_flags |= LINE_FLAG_PLATFORM;
    ml->lo_flags = (flags & 2) ? LINE_FLAG_LEDGE : 0;
    if (flags & 1) ml->lo_flags |= LINE_FLAG_PLATFORM;
    cl[l].flags = ml->hi_flags;
    script_stage.line[i].handle = handle;
    script_stage.line[i].active = 1;
    script_stage.line[i].kind = kind;
    script_stage.line[i].flags = flags;
    script_stage.line[i].model_handle = 0;
    script_stage.line[i].area = script_stage.loading_area;
    script_stage.line[i].x0 = x0; script_stage.line[i].y0 = y0;
    script_stage.line[i].x1 = x1; script_stage.line[i].y1 = y1;
    mpJointListAdd(j);
    mpUncheckBounding();
    OSReport("script stage: line handle=%d map line=%d kind=%d\n",
             handle, l, kind);
    return handle;
}

int ScriptGame_StageRemove(int handle)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i) {
        ScriptStageModel* m = &script_stage.model[i];
        if (m->active && m->handle == handle) {
            int attached = m->line_handle;
            HSD_GObj* gobj = m->gobj;
            m->active = 0;
            m->gobj = NULL;
            m->line_handle = 0;
            if (attached) ScriptGame_StageRemove(attached);
            HSD_GObjFree(gobj);
            OSReport("script stage: removed model handle=%d\n", handle);
            return 1;
        }
    }
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i) {
        if (script_stage.line[i].active && script_stage.line[i].handle == handle) {
            int j = script_stage.base_j + i;
            mpLib_80057BC0(j);
            script_stage.map->joints[j].ranges[MapLineGroup_Dynamic].count = 0;
            script_stage.line[i].active = 0;
            script_stage.line[i].model_handle = 0;
            {
                int k;
                for (k = 0; k < SCRIPT_STAGE_MODELS; ++k)
                    if (script_stage.model[k].line_handle == handle)
                        script_stage.model[k].line_handle = 0;
            }
            mpUncheckBounding();
            OSReport("script stage: removed line handle=%d\n", handle);
            return 1;
        }
    }
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i) {
        ScriptStageTarget* t = &script_stage.target[i];
        if (t->active && t->handle == handle) {
            Item_GObj* gobj = t->gobj;
            t->active = 0;
            t->gobj = NULL;
            script_stage.target_remaining--;
            /* Item_8026A8EC is direct cleanup, not Mato's hit callback. */
            Item_8026A8EC(gobj);
            OSReport("script stage: removed target handle=%d\n", handle);
            return 1;
        }
    }
    return 0;
}

static int script_stage_line_set(int handle, float x0, float y0, float x1, float y1)
{
    int i;
    for (i = 0; i < script_stage.cap; ++i) {
        ScriptStageLine* s = &script_stage.line[i];
        CollVtx* v;
        CollJoint* j;
        if (!s->active || s->handle != handle) continue;
        v = &mpGetGroundCollVtx()[script_stage.base_v + 2 * i];
        /* ScriptGame_StageFrame captured the frame-start positions. Preserve those
         * across multiple moves so mpColl sees the whole frame's floor displacement. */
        v[0].pos.x = s->x0 = x0; v[0].pos.y = s->y0 = y0;
        v[1].pos.x = s->x1 = x1; v[1].pos.y = s->y1 = y1;
        j = &mpGetGroundCollJoint()[script_stage.base_j + i];
        j->bounding_min.x = (x0 < x1 ? x0 : x1) - 30;
        j->bounding_max.x = (x0 > x1 ? x0 : x1) + 30;
        j->bounding_min.y = (y0 < y1 ? y0 : y1) - 30;
        j->bounding_max.y = (y0 > y1 ? y0 : y1) + 30;
        j->flags |= CollJoint_B8;
        j->xE = true;
        mpLib_8005667C(script_stage.base_j + i);
        mpUncheckBounding();
        return 1;
    }
    return 0;
}

int ScriptGame_StageMove(int handle, int xb, int yb)
{
    union { int i; float f; } u;
    float x, y, dx, dy;
    int i;
    u.i = xb; x = u.f; u.i = yb; y = u.f;
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i) {
        ScriptStageModel* m = &script_stage.model[i];
        if (m->active && m->handle == handle) {
            HSD_JObj* root = GET_JOBJ(m->gobj);
            Vec3 pos = root->translate;
            if (m->line_handle) {
                int k;
                for (k = 0; k < SCRIPT_STAGE_LINES; ++k) {
                    ScriptStageLine* s = &script_stage.line[k];
                    if (s->active && s->handle == m->line_handle) {
                        int line_x, line_y;
                        u.f = (s->x0 + s->x1) * 0.5f + x - m->x;
                        line_x = u.i;
                        u.f = (s->y0 + s->y1) * 0.5f + y - m->y;
                        line_y = u.i;
                        ScriptGame_StageMove(s->handle, line_x, line_y);
                        break;
                    }
                }
            }
            pos.x += x - m->x;
            pos.y += y - m->y;
            HSD_JObjSetTranslate(root, &pos);
            m->x = x; m->y = y;
            return 1;
        }
    }
    for (i = 0; i < script_stage.cap; ++i) {
        ScriptStageLine* s = &script_stage.line[i];
        if (!s->active || s->handle != handle) continue;
        dx = x - (s->x0 + s->x1) * 0.5f;
        dy = y - (s->y0 + s->y1) * 0.5f;
        return script_stage_line_set(handle, s->x0 + dx, s->y0 + dy, s->x1 + dx, s->y1 + dy);
    }
    return 0;
}

#include "script_model.inc"

/* Ground_801C126C's preorder numbering, scoped to a map_head model group. A copy of
 * the chosen descriptor cuts its next sibling, so HSD_JObjLoadJoint owns this branch only. */
static HSD_Joint* script_stage_joint(HSD_Joint* joint, int* index)
{
    HSD_Joint* found;
    if (joint == NULL) return NULL;
    if ((*index)-- == 0) return joint;
    found = script_stage_joint(joint->child, index);
    if (found != NULL) return found;
    return script_stage_joint(joint->next, index);
}

/* HSD_JObjResolveRefs requires an instance target to have been loaded into the
 * same ID table. A detached branch cannot promise that for an outside sibling. */
static int script_stage_has_instance(HSD_Joint* joint)
{
    for (; joint != NULL; joint = joint->next) {
        if (joint->flags & JOBJ_INSTANCE || script_stage_has_instance(joint->child))
            return 1;
    }
    return 0;
}

static int script_stage_text(int which, char* dst, int cap)
{
    extern int Script_StageTextByte(int which, int at);
    int i, c;
    for (i = 0; i < cap; ++i) {
        c = Script_StageTextByte(which, i);
        if (c == 0) break;
        if (i == cap - 1) return 0;
        if (c < 32 || c > 126) return 0;
        dst[i] = (char) c;
    }
    dst[i] = 0;
    return i < cap && i > 0;
}

int ScriptGame_StageAddModel(int group, int joint_index, int xb, int yb, int zb,
                             int sb, int rb, int handle)
{
    union { int i; float f; } u;
    char file[32], symbol[64];
    HSD_Archive* archive;
    UnkStageDat* head;
    HSD_Joint* source;
    HSD_Joint branch;
    HSD_JObj* root;
    HSD_GObj* gobj;
    Vec3 pos, scale;
    float rot;
    int i, n;
    if (script_stage.map == NULL || mpLib_8004D164() != script_stage.map ||
        !script_stage_text(0, file, sizeof file) ||
        !script_stage_text(1, symbol, sizeof symbol)) return -1;
    for (i = 0; i < SCRIPT_STAGE_MODELS && script_stage.model[i].active; ++i) {}
    if (i == SCRIPT_STAGE_MODELS) return -1;
    archive = script_stage_archive(file);
    if (archive == NULL) return -1;
    if (strcmp(symbol, "map_head") == 0) {
        head = HSD_ArchiveGetPublicAddress(archive, symbol);
        if (head == NULL || group < 0 || group >= head->unkC || head->unk8 == NULL)
            return -1;
        source = head->unk8[group].unk0;
    } else {
        source = HSD_ArchiveGetPublicAddress(archive, symbol);
    }
    if (source == NULL) return -1;
    if (joint_index >= 0) {
        n = joint_index;
        source = script_stage_joint(source, &n);
        if (source == NULL) return -1;
    }
    if (source->flags & JOBJ_INSTANCE || script_stage_has_instance(source->child))
        return -1;
    branch = *source;
    branch.next = NULL;
    root = HSD_JObjLoadJoint(&branch);
    if (root == NULL) return -1;
    gobj = GObj_Create(HSD_GOBJ_CLASS_STAGE, 13, 0);
    if (gobj == NULL) { HSD_JObjUnref(root); return -1; }
    u.i = xb; pos.x = u.f;
    u.i = yb; pos.y = u.f;
    u.i = zb; pos.z = u.f;
    HSD_JObjSetTranslate(root, &pos);
    u.i = sb;
    scale.x = root->scale.x * u.f;
    scale.y = root->scale.y * u.f;
    scale.z = root->scale.z * u.f;
    HSD_JObjSetScale(root, &scale);
    u.i = rb; rot = u.f;
    HSD_JObjSetRotationZ(root, source->rotation.z + rot);
    HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, root);
    GObj_SetupGXLink(gobj, script_stage_model_render, 3, 0);
    script_stage.model[i].handle = handle;
    script_stage.model[i].active = 1;
    script_stage.model[i].line_handle = 0;
    script_stage.model[i].gobj = gobj;
    script_stage.model[i].x = pos.x;
    script_stage.model[i].y = pos.y;
    script_stage.model[i].z = pos.z;
    u.i = sb; script_stage.model[i].scale = u.f;
    script_stage.model[i].rot = rot;
    OSReport("script stage: model handle=%d /%s:%s group=%d joint=%d at (%f,%f,%f)\n",
             handle, file, symbol, group, joint_index, pos.x, pos.y, pos.z);
    return handle;
}

int ScriptGame_StageAttachModel(int model_handle, int line_handle)
{
    int i, k;
    for (i = 0; i < SCRIPT_STAGE_MODELS; ++i) {
        ScriptStageModel* m = &script_stage.model[i];
        if (!m->active || m->handle != model_handle || m->line_handle) continue;
        for (k = 0; k < SCRIPT_STAGE_LINES; ++k) {
            ScriptStageLine* s = &script_stage.line[k];
            if (s->active && s->handle == line_handle && s->kind == 1 &&
                s->model_handle == 0) {
                m->line_handle = line_handle;
                s->model_handle = model_handle;
                OSReport("script stage: model %d carries line %d\n", model_handle,
                         line_handle);
                return 1;
            }
        }
    }
    return 0;
}

/* A line moved by Lua carries its delta for exactly one logic frame. mpGetSpeed
 * (mplib.c) remaps x10/x14 to pos for grounded fighters; leaving the old endpoint
 * there would keep imparting velocity on every later frame. */
void ScriptGame_StageFrame(void)
{
    CollVtx* cv;
    int i;
    /* arena-hooks: also runs for full maps with no spare scripted geometry. */
    script_arena_frame();
    if (script_stage.map == NULL || mpLib_8004D164() != script_stage.map) return;
    cv = mpGetGroundCollVtx();
    for (i = 0; i < SCRIPT_STAGE_LINES; ++i) {
        int v;
        if (!script_stage.line[i].active) continue;
        v = script_stage.base_v + i * 2;
        cv[v].x10 = cv[v].pos.x; cv[v].x14 = cv[v].pos.y;
        cv[v + 1].x10 = cv[v + 1].pos.x; cv[v + 1].x14 = cv[v + 1].pos.y;
        mpGetGroundCollJoint()[script_stage.base_j + i].flags &= ~CollJoint_B8;
    }
}

int ScriptGame_StageLineI(int i, int field)
{
    if (i < 0 || i >= SCRIPT_STAGE_LINES || !script_stage.line[i].active) return 0;
    return field == 0 ? script_stage.line[i].kind : script_stage.line[i].handle;
}

float ScriptGame_StageLineF(int i, int field)
{
    ScriptStageLine* s;
    if (i < 0 || i >= SCRIPT_STAGE_LINES || !script_stage.line[i].active) return 0;
    s = &script_stage.line[i];
    return field == 0 ? s->x0 : field == 1 ? s->y0 : field == 2 ? s->x1 : s->y1;
}

/* Target Test's It_Kind_Mato logic lives in it_3F2F.c / itmato.c. Borrow its actual
 * model descriptor from GrTMr's itemdata; the archive remains in heap 0 for this scene. */
static ItemAttr script_target_attr;
static ItHurtBoneDesc script_target_hurt_desc;
static ItHurtBoneList script_target_hurt = { 1, &script_target_hurt_desc };
static ItemStateArray script_target_states;
static ItemModelDesc script_target_model;
static Article script_target_article = {
    &script_target_attr, NULL, &script_target_hurt, &script_target_states,
    &script_target_model, NULL
};

static int script_target_register(void)
{
    HSD_Archive* archive;
    struct GroundItemData** items;
    int i;
    if (script_stage.target_model_ready) return 1;
    archive = script_stage_archive("GrTMr.dat");
    if (archive == NULL) return 0;
    items = HSD_ArchiveGetPublicAddress(archive, "itemdata");
    if (items == NULL) return 0;
    for (i = 0; items[i] != NULL; ++i) {
        if (items[i]->unk0 == It_Kind_Mato && items[i]->unk4 != NULL &&
            items[i]->unk4->x10_modelDesc != NULL &&
            items[i]->unk4->x10_modelDesc->x0_joint != NULL) break;
    }
    if (items[i] == NULL) return 0;
    script_target_attr.x1_67_cam_kind = 0;
    script_target_attr.x1C_damage_mul = 1.0f;
    script_target_attr.x60_scale = 1.0f;
    script_target_hurt_desc.bone_id = 0;
    script_target_hurt_desc.a_offset.x = 0;
    script_target_hurt_desc.a_offset.y = 0;
    script_target_hurt_desc.a_offset.z = 0;
    script_target_hurt_desc.b_offset = script_target_hurt_desc.a_offset;
    script_target_hurt_desc.scale = 5.0f;
    script_target_model = *items[i]->unk4->x10_modelDesc;
    script_target_old_article = it_804A0F60[It_Kind_Mato - It_Kind_Old_Kuri];
    it_804A0F60[It_Kind_Mato - It_Kind_Old_Kuri] = &script_target_article;
    script_stage.target_model_ready = 1;
    OSReport("script stage: Mato model from /GrTMr.dat itemdata\n");
    return 1;
}

int ScriptGame_StageJoint(int joint_id)
{
    return script_stage.map != NULL &&
           joint_id >= script_stage.base_j &&
           joint_id < script_stage.base_j + script_stage.cap;
}

int ScriptGame_SpawnTarget(int xb, int yb, int handle)
{
    union { int i; float f; } u;
    Vec3 pos;
    Item_GObj* gobj;
    int i;
    if (script_stage.map == NULL || mpLib_8004D164() != script_stage.map) return -1;
    for (i = 0; i < SCRIPT_STAGE_TARGETS && script_stage.target[i].active; ++i) {}
    if (i == SCRIPT_STAGE_TARGETS) return -1;
    u.i = xb; pos.x = u.f; u.i = yb; pos.y = u.f; pos.z = 0;
    if (!script_target_register()) return -1;
    gobj = it_8027B5B0(It_Kind_Mato, &pos, NULL, NULL, 0);
    if (gobj == NULL) return -1;
    /* Targets are allowed outside the current stage's blast rectangle. */
    GET_ITEM(gobj)->xDCC_flag.b4567 = 0;
    script_stage.target[i].handle = handle;
    script_stage.target[i].active = 1;
    script_stage.target[i].area = script_stage.loading_area;
    script_stage.target[i].gobj = gobj;
    script_stage.target[i].x = pos.x;
    script_stage.target[i].y = pos.y;
    script_stage.target_remaining++;
    OSReport("script stage: target handle=%d at (%f,%f), remaining=%d\n",
             handle, pos.x, pos.y, script_stage.target_remaining);
    return handle;
}

/* itmato.c's phys: a scripted target stays where it was spawned (no joint to follow). */
int ScriptGame_TargetPin(Item_GObj* gobj)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i) {
        ScriptStageTarget* t = &script_stage.target[i];
        if (t->active && t->gobj == gobj) {
            Item* ip = GET_ITEM(gobj);
            ip->pos.x = t->x;
            ip->pos.y = t->y;
            ip->pos.z = 0.0f;
            ip->x40_vel.x = ip->x40_vel.y = ip->x40_vel.z = 0.0f;
            return 1;
        }
    }
    return 0;
}

/* itmato.c calls this before Ground_801C4338. Return 1 for a scripted Mato (including
 * gd.stage_remove), so VS stages never decrement Target Test's unrelated stage counter. */
int ScriptGame_TargetDestroyed(Item_GObj* gobj)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_TARGETS; ++i) {
        ScriptStageTarget* t = &script_stage.target[i];
        if (t->gobj != gobj || gobj == NULL) continue;
        t->gobj = NULL;
        if (t->active) {
            extern void Script_TargetBroken(int handle, int remaining);
            t->active = 0;
            script_stage.target_remaining--;
            Script_TargetBroken(t->handle, script_stage.target_remaining);
            OSReport("script stage: target broken handle=%d remaining=%d\n",
                     t->handle, script_stage.target_remaining);
        }
        return 1;
    }
    return 0;
}

int ScriptGame_StageTargetI(int i)
{
    return i >= 0 && i < SCRIPT_STAGE_TARGETS && script_stage.target[i].active;
}

/* Scripted enemies deliberately have no grZakoGenerator entry. The pool is game BSS, so item
 * identity and defeated state follow the item heap through a same-scene savestate/rewind. */
int ScriptGame_SpawnEnemy(int which, int xb, int yb, int facing, int handle)
{
    union { int i; float f; } u;
    Vec3 pos;
    Item_GObj* gobj = NULL;
    Item* ip;
    int i, kind;
    if (which < 0 || which >= SCRIPT_ENEMY_KINDS || (facing != -1 && facing != 1))
        return -1;
    if (!script_enemy_preload(which)) return -1;
    kind = script_enemy_kinds[which];
    for (i = 0; i < SCRIPT_STAGE_ENEMIES && script_stage.enemy[i].active; ++i) {}
    if (i == SCRIPT_STAGE_ENEMIES) return -1;
    u.i = xb; pos.x = u.f; u.i = yb; pos.y = u.f; pos.z = 0.0f;
    switch (kind) {
    case It_Kind_Leadead: gobj = it_802EA9FC(&pos, facing); break;
    case It_Kind_Nokonoko: gobj = it_802DD7F0(0, &pos, NULL, facing); break;
    case It_Kind_Likelike:
        /* enemies2: arg0=1 clings to a ceiling and accelerates upward forever on FD.
         * it_802DC4BC(0) selects the stock falling/floor AI. Its initializer lowers
         * Y by 40 for the buried variant; restore the requested airborne position. */
        gobj = it_802DC4BC(0, &pos, facing);
        if (gobj != NULL) GET_ITEM(gobj)->pos = pos;
        break;
    default: gobj = it_8027B5B0(kind, &pos, NULL, NULL, 1); break;
    }
    if (gobj == NULL) return -1;
    ip = GET_ITEM(gobj);
    ip->facing_dir = ip->init_facing_dir = (float) facing;
    mpCollSetFacingDir(&ip->x378_itemColl, facing);
    if (kind == It_Kind_Kuriboh || kind == It_Kind_Octarock ||
        kind == It_Kind_Whitebea || kind == It_Kind_Ottosea)
        it_8027C56C(gobj, (float) facing);
    script_stage.enemy[i].handle = handle;
    script_stage.enemy[i].kind = kind;
    script_stage.enemy[i].active = 1;
    script_stage.enemy[i].defeated = 0;
    script_stage.enemy[i].gobj = gobj;
    OSReport("script enemy: spawned kind=%d handle=%d at (%f,%f) facing=%d\n",
             kind, handle, pos.x, pos.y, facing);
    return handle;
}

int ScriptGame_EnemyRemove(int handle)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_ENEMIES; ++i) {
        ScriptStageEnemy* e = &script_stage.enemy[i];
        if (!e->active || e->handle != handle) continue;
        e->active = 0;
        /* Item_8026A8EC calls ScriptGame_EnemyDestroyed before releasing this gobj. */
        Item_8026A8EC(e->gobj);
        e->gobj = NULL;
        OSReport("script enemy: removed handle=%d\n", handle);
        return 1;
    }
    return 0;
}

int ScriptGame_EnemyDefeated(Item_GObj* gobj)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_ENEMIES; ++i) {
        ScriptStageEnemy* e = &script_stage.enemy[i];
        if (!e->active || e->gobj != gobj) continue;
        if (e->defeated) return 2;
        {
            extern void Script_EnemyDefeated(int kind, int handle);
            e->defeated = 1;
            Script_EnemyDefeated(script_enemy_index(e->kind), e->handle);
            OSReport("script enemy: defeated kind=%d handle=%d\n", e->kind, e->handle);
        }
        return 1;
    }
    return 0;
}

int ScriptGame_EnemyDestroyed(Item_GObj* gobj)
{
    int i;
    for (i = 0; i < SCRIPT_STAGE_ENEMIES; ++i) {
        ScriptStageEnemy* e = &script_stage.enemy[i];
        if (e->gobj != gobj || gobj == NULL) continue;
        e->active = 0;
        e->gobj = NULL;
        return 1;
    }
    return 0;
}

float ScriptGame_StageTargetF(int i, int field)
{
    if (!ScriptGame_StageTargetI(i)) return 0;
    return field == 0 ? script_stage.target[i].x : script_stage.target[i].y;
}

/* Item_80268B18 gives each item a unique x1C serial. The list is the same one the
 * engine's item collision pass walks; no item pointer crosses into native Lua. */
static Item* script_item(int index)
{
    HSD_GObj* g;
    if (index < 0 || HSD_GObjPLinkHead == NULL) return NULL;
    for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; g != NULL; g = g->next)
        if (index-- == 0) return GET_ITEM(g);
    return NULL;
}

int ScriptGame_ItemCount(void)
{
    HSD_GObj* g;
    int n = 0;
    if (HSD_GObjPLinkHead != NULL)
        for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; g != NULL; g = g->next) ++n;
    return n;
}

int ScriptGame_ItemI(int index, int field)
{
    Item* ip = script_item(index);
    int p, a, slot;
    if (ip == NULL) return -1;
    switch (field) {
    case SCRIPT_ITEM_KIND: return ip->kind;
    case SCRIPT_ITEM_ID: return ip->x1C;
    case SCRIPT_ITEM_STATE: return ip->msid;
    case SCRIPT_ITEM_OWNER:
        for (slot = 0; slot < 6; ++slot)
            if (ip->owner != NULL && Player_GetEntity(slot) == ip->owner) return slot + 1;
        return 0;
    case SCRIPT_ITEM_GENO_PROFILE:
    case SCRIPT_ITEM_GENO_ARTICLE:
    case SCRIPT_ITEM_GENO_FRAME:
        if (ip->kind >= GENO_ART_KIND_BASE && ip->kind < GENO_ART_KIND_END) {
            if (ip->kind < GENO_ART_KIND_BASE2) {
                p = (ip->kind - GENO_ART_KIND_BASE) / GENO_ART_PER_RANGE;
                a = (ip->kind - GENO_ART_KIND_BASE) % GENO_ART_PER_RANGE;
            } else {
                p = (ip->kind - GENO_ART_KIND_BASE2) / GENO_ART_PER_RANGE;
                a = GENO_ART_PER_RANGE + (ip->kind - GENO_ART_KIND_BASE2) % GENO_ART_PER_RANGE;
            }
            if (field == SCRIPT_ITEM_GENO_PROFILE) return p;
            if (field == SCRIPT_ITEM_GENO_ARTICLE) return a;
            /* GenoArtVars begins with profile, article, frame in xDD4_itemVar. */
            return ((s32*) &ip->xDD4_itemVar)[2];
        }
        return -1;
    }
    return -1;
}

float ScriptGame_ItemF(int index, int field)
{
    Item* ip = script_item(index);
    if (ip == NULL) return 0.0f;
    switch (field) {
    case SCRIPT_ITEM_X: return ip->pos.x;
    case SCRIPT_ITEM_Y: return ip->pos.y;
    case SCRIPT_ITEM_Z: return ip->pos.z;
    case SCRIPT_ITEM_VX: return ip->x40_vel.x;
    case SCRIPT_ITEM_VY: return ip->x40_vel.y;
    case SCRIPT_ITEM_FACING: return ip->facing_dir;
    }
    return 0.0f;
}

enum {
    SCRIPT_F_X = 0,
    SCRIPT_F_Y,
    SCRIPT_F_VX,
    SCRIPT_F_VY,
    SCRIPT_F_PERCENT,
    SCRIPT_F_FACING,
    SCRIPT_F_ANIM_FRAME,
    SCRIPT_F_HITLAG,
};

enum {
    SCRIPT_I_PRESENT = 0, /* 1 when the slot has a live fighter */
    SCRIPT_I_KIND,        /* internal FighterKind */
    SCRIPT_I_CHAR,        /* external CharacterKind (what the CSS picked) */
    SCRIPT_I_ACTION,      /* motion/action state id */
    SCRIPT_I_AIRBORNE,
    SCRIPT_I_STOCKS,
    SCRIPT_I_COSTUME,
    SCRIPT_I_SLOT_TYPE, /* 0 human, 1 cpu, 2 demo, 3 none */
};

/* A slot's fighter counts only while its gobj is in the live fighter list. The scene start clears
 * the player table (Player_ForgetEntities, gmscene.c) and a fighter freed mid-scene clears its
 * own slot (Fighter_Unload_8006DABC), so the table should never hold a freed fighter; this is the
 * second line, because every script read and write goes through here and a stale pointer means
 * reading freed memory (the 0x8B8B8B8B fill). At most six fighters, so the walk is cheap. */
static int script_gobj_live(HSD_GObj* gobj)
{
    HSD_GObj* cur;
    if (gobj == NULL || HSD_GObjPLinkHead == NULL) {
        return 0;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        if (cur == gobj) {
            return 1;
        }
    }
    return 0;
}

static Fighter* script_fighter(int slot)
{
    HSD_GObj* gobj;
    if (slot < 0 || slot >= 6) {
        return NULL;
    }
    gobj = Player_GetEntity(slot);
    if (!script_gobj_live(gobj)) {
        return NULL;
    }
    return GET_FIGHTER(gobj);
}

/* Camera_8002A4AC and its script override run in the game TU. Resolve handles here, where the
 * live fighter list and Item_80268B18's unique item serial can be checked without a native
 * pointer crossing the PPC bridge. */
int ScriptGame_CameraFollowTarget(int kind, int id, Vec3* out)
{
    if (kind == 1) {
        Fighter* fp = script_fighter(id);
        if (fp == NULL) return 0;
        *out = fp->cur_pos;
        return 1;
    }
    if (kind == 2 && HSD_GObjPLinkHead != NULL) {
        HSD_GObj* g;
        for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; g != NULL; g = g->next) {
            Item* ip = GET_ITEM(g);
            if (ip != NULL && ip->x1C == id) {
                *out = ip->pos;
                return 1;
            }
        }
    }
    return 0;
}

float ScriptGame_FighterF(int slot, int field)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL) {
        return 0.0f;
    }
    switch (field) {
    case SCRIPT_F_X:
        return fp->cur_pos.x;
    case SCRIPT_F_Y:
        return fp->cur_pos.y;
    case SCRIPT_F_VX:
        return fp->self_vel.x;
    case SCRIPT_F_VY:
        return fp->self_vel.y;
    case SCRIPT_F_PERCENT:
        return fp->dmg.x1830_percent;
    case SCRIPT_F_FACING:
        return fp->facing_dir;
    case SCRIPT_F_ANIM_FRAME:
        return fp->cur_anim_frame;
    case SCRIPT_F_HITLAG:
        return fp->dmg.x195c_hitlag_frames;
    }
    return 0.0f;
}

int ScriptGame_FighterI(int slot, int field)
{
    Fighter* fp;
    if (field == SCRIPT_I_SLOT_TYPE) {
        return (slot >= 0 && slot < 6) ? (int) Player_GetPlayerSlotType(slot) : 3;
    }
    fp = script_fighter(slot);
    if (fp == NULL) {
        return field == SCRIPT_I_PRESENT ? 0 : -1;
    }
    switch (field) {
    case SCRIPT_I_PRESENT:
        return 1;
    case SCRIPT_I_KIND:
        return (int) fp->kind;
    case SCRIPT_I_CHAR:
        return (int) Player_GetPlayerCharacter(slot);
    case SCRIPT_I_ACTION:
        return (int) fp->motion_id;
    case SCRIPT_I_AIRBORNE:
        return fp->ground_or_air == GA_Air ? 1 : 0;
    case SCRIPT_I_STOCKS:
        return (int) Player_GetStocks(slot);
    case SCRIPT_I_COSTUME:
        return (int) Player_GetCostumeId(slot);
    }
    return -1;
}

/* Gameplay writes: only for scripts whose manifest says "gameplay": true (gw_script.c checks). */
void ScriptGame_SetPercent(int slot, int percent)
{
    if (script_fighter(slot) != NULL) {
        Player_SetHUDDamage(slot, percent);
    }
}

void ScriptGame_SetStocks(int slot, int stocks)
{
    if (slot >= 0 && slot < 6) {
        Player_SetStocks(slot, stocks);
    }
}

int ScriptGame_StageKind(void)
{
    return (int) stage_info.grkind;
}

int ScriptGame_GameMode(void)
{
    return (int) gm_GetCurrentGameMode();
}

/* Scene launch at runtime: the native side has already set the scene text
 * (gw_SceneLaunch_SetText). Leaving the current scene for the target mode directly is NOT safe
 * from inside a match (tried: re-entering Training from Training skipped the mode's unload and
 * exhausted the heap in lbMemory_80014FC8), so this takes the console's own soft-reset path -
 * the one the reset switch triggers (gm_801A4014): the mode unwinds cleanly, the game re-enters
 * GM_BOOT, and the boot mode hands over to the configured scene exactly as it does for
 * MELEE_SCENE at start-up (gmboot.c). `game_mode` is only reported. */
void ScriptGame_LaunchScene(int game_mode)
{
    (void) game_mode;
    gmMainLib_8046B0F0.resetting = true;
    gm_801A4B60();
}

/* ============================================================================================
 * Geno Lab: read-only inspection for the Lua Lab (docs/geno.md "Geno Lab"). Field numbers are
 * in script_lab.h, shared with gw_script.c. Nothing below writes game state except the two
 * cosmetic debug-draw switches, which the native side refuses during a netplay session.
 * ============================================================================================ */
#include <melee/cm/camera.h>
#include <melee/ft/ftparts.h>
#include <melee/lb/types.h>
#include <sysdolphin/baselib/cobj.h>
#include <sysdolphin/baselib/jobj.h>

#include "script_lab.h"

float ScriptGame_LabF(int slot, int field)
{
    Fighter* fp = script_fighter(slot);
    CollData* cd;
    if (fp == NULL) {
        return 0.0f;
    }
    cd = &fp->coll_data;
    switch (field) {
    case LAB_F_ANIM_RATE:
        return fp->frame_speed_mul;
    case LAB_F_HITSTUN:
        return fp->x221C_b6 ? fp->mv.co.damage.x0 : 0.0f;
    case LAB_F_KB_VX:
        return fp->x8c_kb_vel.x;
    case LAB_F_KB_VY:
        return fp->x8c_kb_vel.y;
    case LAB_F_SHIELD:
        return fp->shield_health;
    case LAB_F_ECB_TOP_X:
        return cd->cur_pos.x + cd->ecb.top.x;
    case LAB_F_ECB_TOP_Y:
        return cd->cur_pos.y + cd->ecb.top.y;
    case LAB_F_ECB_BOTTOM_X:
        return cd->cur_pos.x + cd->ecb.bottom.x;
    case LAB_F_ECB_BOTTOM_Y:
        return cd->cur_pos.y + cd->ecb.bottom.y;
    case LAB_F_ECB_LEFT_X:
        return cd->cur_pos.x + cd->ecb.left.x;
    case LAB_F_ECB_LEFT_Y:
        return cd->cur_pos.y + cd->ecb.left.y;
    case LAB_F_ECB_RIGHT_X:
        return cd->cur_pos.x + cd->ecb.right.x;
    case LAB_F_ECB_RIGHT_Y:
        return cd->cur_pos.y + cd->ecb.right.y;
    case LAB_F_GR_VEL:
        return fp->gr_vel;
    case LAB_F_KB_APPLIED:
        return fp->dmg.kb_applied;
    case LAB_F_Z:
        return fp->cur_pos.z;
    case LAB_F_SCALE:
        return fp->x34_scale.y;
    case LAB_F_CMD_TIMER:
        return fp->cmd_timer;
    case LAB_F_KB_LAST:
        return fp->dmg.x18d8.kb_applied1;
    case LAB_F_SHIELD_X:
        return fp->shield_hit.pos.x;
    case LAB_F_SHIELD_Y:
        return fp->shield_hit.pos.y;
    case LAB_F_SHIELD_R:
        return fp->shield_hit.size;
    }
    return 0.0f;
}

static int lab_joint_count(Fighter* fp)
{
    int n;
    if (fp->parts == NULL || ftPartsTable == NULL || ftPartsTable[fp->kind] == NULL) {
        return 0;
    }
    n = (int) ftPartsTable[fp->kind]->parts_num;
    return n < 0 ? 0 : n > MAX_FT_PARTS ? MAX_FT_PARTS : n;
}

static int lab_joint_index(Fighter* fp, HSD_JObj* jobj)
{
    int i, n = lab_joint_count(fp);
    if (jobj == NULL) {
        return -1;
    }
    for (i = 0; i < n; i++) {
        if (fp->parts[i].joint == jobj) {
            return i;
        }
    }
    return -1;
}

int ScriptGame_LabI(int slot, int field)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_I_ANIM_ID:
        return (int) fp->anim_id;
    case LAB_I_NAME_KIND:
        return (int) FTKB_CANON_KIND(fp->kind);
    case LAB_I_INTANG_TIMER:
        return (int) fp->x1990;
    case LAB_I_INVINC_TIMER:
        return (int) fp->x1994;
    case LAB_I_BODY_STATE:
        return (int) fp->x1988;
    case LAB_I_TIMED_STATE:
        return (int) fp->x198C;
    case LAB_I_JUMPS_USED:
        return (int) fp->x1968_jumpsUsed;
    case LAB_I_MAX_JUMPS:
        return (int) fp->co_attrs.max_jumps;
    case LAB_I_WALLJUMPS_USED:
        return (int) fp->x1969_walljumpUsed;
    case LAB_I_IN_HITLAG:
        return fp->x2219_b5 ? 1 : 0;
    case LAB_I_IN_HITSTUN:
        return fp->x221C_b6 ? 1 : 0;
    case LAB_I_IASA:
        return fp->allow_interrupt ? 1 : 0;
    case LAB_I_LEDGE_COOLDOWN:
        return (int) fp->x2064_ledgeCooldown;
    case LAB_I_ECB_LOCK:
        return (int) fp->ecb_lock;
    case LAB_I_DRAW_FLAGS:
        return (int) fp->x21FC_flag.byte;
    case LAB_I_SUB:
        return fp->is_sub_fighter ? 1 : 0;
    case LAB_I_HIDDEN:
        return (fp->x221E_b5 || fp->invisible) ? 1 : 0;
    case LAB_I_KIND:
        return (int) fp->kind;
    case LAB_I_LR_AGE:
        return (int) fp->x67F;
    case LAB_I_JUMP_AGE:
        return (int) fp->x67E;
    case LAB_I_SHIELD_ON:
        return fp->x221B_b0 ? 1 : 0;
    case LAB_I_JOINTS:
        return lab_joint_count(fp);
    case LAB_I_HURTBOXES:
        return (int) fp->hurt_capsules_len;
    }
    return -1;
}

/* The fighter's animation symbol for its current subaction (e.g. "PlyKirby5K_Share_ACTION_Wait1_
 * figatree"): read from the fighter's own file, so it names m-ex and custom moves too. */
const char* ScriptGame_LabAnimSymbol(int slot)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || fp->x24 == NULL || (int) fp->anim_id < 0) {
        return NULL;
    }
    return fp->x24[fp->anim_id].x0;
}

static HitCapsule* lab_hit(Fighter* fp, int i)
{
    if (i >= 0 && i < 4) {
        return &fp->x914[i];
    }
    if (i == 4) {
        return &fp->x1064_thrownHitbox;
    }
    return NULL;
}

int ScriptGame_HitI(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    HitCapsule* h;
    if (fp == NULL || (h = lab_hit(fp, i)) == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_HI_STATE:
        /* the thrown hitbox keeps a stale state between throws; it is live only while it has
           an owner (the thrower) */
        if (i == 4 && h->owner == NULL) {
            return 0;
        }
        return (int) h->state;
    case LAB_HI_GROUP:
        return (int) h->x4;
    case LAB_HI_BONE:
        return lab_joint_index(fp, h->jobj);
    case LAB_HI_ANGLE:
        return h->kb_angle;
    case LAB_HI_KBG:
        return (int) h->x24;
    case LAB_HI_WBK:
        return (int) h->x28;
    case LAB_HI_BKB:
        return (int) h->x2C;
    case LAB_HI_ELEMENT:
        return (int) h->element;
    case LAB_HI_SHIELD_DMG:
        return h->x34;
    case LAB_HI_SFX_SEVERITY:
        return h->sfx_severity;
    case LAB_HI_SFX_KIND:
        return (int) h->sfx_kind;
    case LAB_HI_HIT_AIR:
        return h->x40_b2 ? 1 : 0;
    case LAB_HI_HIT_GROUND:
        return h->x40_b3 ? 1 : 0;
    case LAB_HI_CLANK:
        return h->x40_b0 ? 1 : 0;
    case LAB_HI_REBOUND:
        return h->x40_b1 ? 1 : 0;
    }
    return -1;
}

float ScriptGame_HitF(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    HitCapsule* h;
    if (fp == NULL || (h = lab_hit(fp, i)) == NULL) {
        return 0.0f;
    }
    switch (field) {
    case LAB_HF_DAMAGE:
        return h->damage;
    case LAB_HF_SIZE:
        return h->scale;
    case LAB_HF_X:
        return h->x4C.x;
    case LAB_HF_Y:
        return h->x4C.y;
    case LAB_HF_Z:
        return h->x4C.z;
    case LAB_HF_PX:
        return h->x58.x;
    case LAB_HF_PY:
        return h->x58.y;
    case LAB_HF_PZ:
        return h->x58.z;
    case LAB_HF_OX:
        return h->b_offset.x;
    case LAB_HF_OY:
        return h->b_offset.y;
    case LAB_HF_OZ:
        return h->b_offset.z;
    }
    return 0.0f;
}

int ScriptGame_HurtI(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    FighterHurtCapsule* u;
    if (fp == NULL || i < 0 || i >= (int) fp->hurt_capsules_len || i >= 15) {
        return -1;
    }
    u = &fp->hurt_capsules[i];
    switch (field) {
    case LAB_UI_STATE:
        return (int) u->capsule.state;
    case LAB_UI_BONE:
        return lab_joint_index(fp, u->capsule.bone);
    case LAB_UI_HEIGHT:
        return (int) u->height;
    case LAB_UI_GRABBABLE:
        return u->is_grabbable ? 1 : 0;
    }
    return -1;
}

float ScriptGame_HurtF(int slot, int i, int field)
{
    Fighter* fp = script_fighter(slot);
    FighterHurtCapsule* u;
    if (fp == NULL || i < 0 || i >= (int) fp->hurt_capsules_len || i >= 15) {
        return 0.0f;
    }
    u = &fp->hurt_capsules[i];
    switch (field) {
    case LAB_UF_AX:
        return u->capsule.a_pos.x;
    case LAB_UF_AY:
        return u->capsule.a_pos.y;
    case LAB_UF_AZ:
        return u->capsule.a_pos.z;
    case LAB_UF_BX:
        return u->capsule.b_pos.x;
    case LAB_UF_BY:
        return u->capsule.b_pos.y;
    case LAB_UF_BZ:
        return u->capsule.b_pos.z;
    case LAB_UF_SIZE:
        return u->capsule.scale;
    }
    return 0.0f;
}

/* A joint's world position: the translation column of the matrix the last HSD_JObjSetupMatrix
 * left in it (read as-is, never recomputed here, so reading cannot change the game). comp 0-2 =
 * x, y, z. */
float ScriptGame_JointF(int slot, int i, int comp)
{
    Fighter* fp = script_fighter(slot);
    HSD_JObj* j;
    if (fp == NULL || i < 0 || i >= lab_joint_count(fp) || comp < 0 || comp > 2) {
        return 0.0f;
    }
    j = fp->parts[i].joint;
    return j != NULL ? j->mtx[comp][3] : 0.0f;
}

/* gd.joints(port, true): bring every joint's matrix up to date before it is read. HSD computes a
 * matrix only when something needs it, so a joint nothing used this frame (no skin, hitbox or
 * effect on it) keeps an OLD matrix, and the joint probe read tens of units of error from those.
 * HSD_JObjSetupMatrix only recomputes a matrix marked dirty, from the joint's current SRT - the
 * same value the game computes the next time it needs it - so this changes no game state. */
void ScriptGame_JointsSetup(int slot)
{
    Fighter* fp = script_fighter(slot);
    int i, n;
    if (fp == NULL) {
        return;
    }
    n = lab_joint_count(fp);
    for (i = 0; i < n; i++) {
        if (fp->parts[i].joint != NULL) {
            HSD_JObjSetupMatrix(fp->parts[i].joint);
        }
    }
}

/* The parent joint's index, -1 for the root or a joint outside the table, -2 for no joint. */
int ScriptGame_JointParent(int slot, int i)
{
    Fighter* fp = script_fighter(slot);
    HSD_JObj* j;
    if (fp == NULL || i < 0 || i >= lab_joint_count(fp)) {
        return -2;
    }
    j = fp->parts[i].joint;
    if (j == NULL) {
        return -2;
    }
    return lab_joint_index(fp, j->parent);
}

float ScriptGame_CameraF(int field)
{
    HSD_GObj* gobj = Camera_80030A50();
    HSD_CObj* c;
    if (gobj == NULL || (c = GET_COBJ(gobj)) == NULL) {
        return 0.0f;
    }
    if (field >= LAB_CAM_VIEW && field < LAB_CAM_VIEW + 12) {
        int k = field - LAB_CAM_VIEW;
        return c->view_mtx[k / 4][k % 4];
    }
    switch (field) {
    case LAB_CAM_OK:
        return 1.0f;
    case LAB_CAM_PROJ:
        return c->projection_type == PROJ_ORTHO     ? 2.0f
               : c->projection_type == PROJ_FRUSTUM ? 1.0f
                                                    : 0.0f;
    case LAB_CAM_P0:
        return c->projection_param.ortho.top; /* = perspective.fov */
    case LAB_CAM_P1:
        return c->projection_param.ortho.bottom; /* = perspective.aspect */
    case LAB_CAM_P2:
        return c->projection_param.ortho.left;
    case LAB_CAM_P3:
        return c->projection_param.ortho.right;
    case LAB_CAM_NEAR:
        return c->near;
    case LAB_CAM_FAR:
        return c->far;
    case LAB_CAM_VP_XMIN:
        return c->viewport.xmin;
    case LAB_CAM_VP_XMAX:
        return c->viewport.xmax;
    case LAB_CAM_VP_YMIN:
        return c->viewport.ymin;
    case LAB_CAM_VP_YMAX:
        return c->viewport.ymax;
    }
    return 0.0f;
}

/* Fighter.x21FC_flag, the develop-mode visualisation byte (ftdrawcommon.c ftDrawCommon_800805C8,
 * dbanim.c fn_CheckAnimationInfo). set != 0 writes `value` to every fighter object of the slot
 * (Nana too). Returns the byte before the write, -1 when the slot has no fighter. */
int ScriptGame_LabDebugDraw(int slot, int set, int value)
{
    Fighter* fp = script_fighter(slot);
    int old;
    if (fp == NULL) {
        return -1;
    }
    old = (int) fp->x21FC_flag.byte;
    if (set) {
        HSD_GObj* g;
        for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
            Fighter* f = GET_FIGHTER(g);
            if (f->player_id == fp->player_id) {
                f->x21FC_flag.byte = (u8) value;
            }
        }
    }
    return old;
}

/* The match camera's collision display (cm/camera.c: mpLib_8005A2DC and friends). `mask` picks
 * the LAB_STAGE_* bits to write from `value`; returns the bits now set (ZONES has no getter in the
 * decomp and reads back as 0). */
int ScriptGame_LabStageDraw(int mask, int value)
{
    int now = 0;
    if (Camera_80030A50() == NULL) {
        return -1;
    }
    if (mask & LAB_STAGE_COLL) {
        Camera_80030A60((value & LAB_STAGE_COLL) != 0);
    }
    if (mask & LAB_STAGE_TERRAIN) {
        Camera_80030B38((value & LAB_STAGE_TERRAIN) != 0);
    }
    if (mask & LAB_STAGE_LEDGES) {
        Camera_80030B64((value & LAB_STAGE_LEDGES) != 0);
    }
    if (mask & LAB_STAGE_POINTS) {
        Camera_80030B90((value & LAB_STAGE_POINTS) != 0);
    }
    if (mask & LAB_STAGE_ZONES) {
        Camera_80030A8C((value & LAB_STAGE_ZONES) != 0);
    }
    now |= Camera_80030A78() ? LAB_STAGE_COLL : 0;
    now |= Camera_80030B50() ? LAB_STAGE_TERRAIN : 0;
    now |= Camera_80030B7C() ? LAB_STAGE_LEDGES : 0;
    now |= Camera_80030BA8() ? LAB_STAGE_POINTS : 0;
    return now;
}

/* ---- Stage 2: subaction scripts, motions ---------------------------------------------------- */
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>

static MotionState* lab_motion_row(Fighter* fp, int msid)
{
    if (msid < 0) {
        return NULL;
    }
    if (msid >= 0x400) {
        /* a Geno v2 action state (pc/geno/geno_game_v2.inc) */
        extern MotionState* Geno_MotionRowIfAny(Fighter * fp, int msid);
        return Geno_MotionRowIfAny(fp, msid);
    }
    if (msid >= fp->x18) {
        return fp->x20_actionStateList != NULL ? &fp->x20_actionStateList[msid - fp->x18] : NULL;
    }
    return fp->x1C_actionStateList != NULL ? &fp->x1C_actionStateList[msid] : NULL;
}

/* The animation (subaction) index a motion plays, -2 when the motion has no row. The native side
 * bounds `msid` (the decomp's name tables) before asking. */
int ScriptGame_LabMotionAnim(int slot, int msid)
{
    Fighter* fp = script_fighter(slot);
    MotionState* ms;
    if (fp == NULL || (ms = lab_motion_row(fp, msid)) == NULL) {
        return -2;
    }
    return (int) ms->anim_id;
}

/* The number of common motion states (fp->x18): special states start here. */
int ScriptGame_LabCommonCount(int slot)
{
    Fighter* fp = script_fighter(slot);
    return fp != NULL ? (int) fp->x18 : -1;
}

/* The subaction script of animation `anim` (-1 = the one playing), as its guest address. Never
 * the live cursor: the start of the script, which the native side walks read-only. */
const void* ScriptGame_LabScript(int slot, int anim)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || fp->x24 == NULL) {
        return NULL;
    }
    if (anim < 0) {
        anim = (int) fp->anim_id;
    }
    if (anim < 0 || anim > 0x3FF) {
        return NULL;
    }
    return fp->x24[anim].xC;
}

const char* ScriptGame_LabAnimSymbolFor(int slot, int anim)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || fp->x24 == NULL || anim < 0 || anim > 0x3FF) {
        return NULL;
    }
    return fp->x24[anim].x0;
}

/* The playing animation's last frame. */
float ScriptGame_LabAnimEnd(int slot)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || (int) fp->anim_id < 0) {
        return 0.0f;
    }
    return ftAnim_8006F484(fp->gobj);
}

/* Offline only (gw_script.c refuses it in a session and calls it at a frame boundary): put the
 * fighter into motion `msid` from its first frame, the plain Fighter_ChangeMotionState a state's
 * entry function would make. `lift` > 0: airborne and that much higher first (aerials). Floats
 * arrive as bits. Returns 0, or -1 without a fighter / row. */
int ScriptGame_LabSetMotion(int slot, int msid, int rate_bits, int lift_bits)
{
    Fighter* fp = script_fighter(slot);
    MotionState* ms;
    union {
        int i;
        float f;
    } rate, lift;
    if (fp == NULL || (ms = lab_motion_row(fp, msid)) == NULL) {
        return -1;
    }
    rate.i = rate_bits;
    lift.i = lift_bits;
    if (lift.f > 0.0f) {
        /* an aerial: into the air `lift` units up first (ftCommon_8007D5D4, "become airborne"),
           or the next collision check lands it at once */
        if (fp->ground_or_air == GA_Ground) {
            ftCommon_8007D5D4(fp);
        }
        fp->cur_pos.y += lift.f;
        fp->self_vel.x = fp->self_vel.y = 0.0f;
    }
    if (msid >= 0x400) {
        /* a Geno state: its behaviour's own entry (pc/geno/geno_game_v2.inc), not a bare change */
        extern int Geno_LabEnterState(Fighter * fp, int s);
        return Geno_LabEnterState(fp, msid - 0x400);
    }
    Fighter_ChangeMotionState(fp->gobj, msid, 0, 0.0f, rate.f, 0.0f, NULL);
    return 0;
}

/* ---- attributes (ftCo_DatAttrs by decomp name, as the fighter has them now) ---------------- */
#define LAB_ATTR(field, is_int) { #field, (int) __builtin_offsetof(ftCo_DatAttrs, field), is_int }

static const struct {
    const char* name;
    int offset;
    int is_int;
} lab_attrs[] = {
    LAB_ATTR(walk_accel_mul, 0),
    LAB_ATTR(walk_accel_base, 0),
    LAB_ATTR(walk_max_vel, 0),
    LAB_ATTR(slow_walk_max, 0),
    LAB_ATTR(mid_walk_point, 0),
    LAB_ATTR(fast_walk_min, 0),
    LAB_ATTR(ground_friction, 0),
    LAB_ATTR(dash_initial_velocity, 0),
    LAB_ATTR(dash_accel_mul, 0),
    LAB_ATTR(dash_accel_base, 0),
    LAB_ATTR(dash_max_velocity, 0),
    LAB_ATTR(run_animation_scaling, 0),
    LAB_ATTR(max_run_brake_frames, 0),
    LAB_ATTR(ground_max_horizontal_velocity, 0),
    LAB_ATTR(jump_startup_time, 0),
    LAB_ATTR(jump_h_initial_velocity, 0),
    LAB_ATTR(jump_v_initial_velocity, 0),
    LAB_ATTR(ground_to_air_jump_momentum_multiplier, 0),
    LAB_ATTR(jump_h_max_velocity, 0),
    LAB_ATTR(hop_v_initial_velocity, 0),
    LAB_ATTR(air_jump_v_multiplier, 0),
    LAB_ATTR(air_jump_h_multiplier, 0),
    LAB_ATTR(max_jumps, 1),
    LAB_ATTR(gravity, 0),
    LAB_ATTR(terminal_velocity, 0),
    LAB_ATTR(air_drift_stick_mul, 0),
    LAB_ATTR(aerial_drift_base, 0),
    LAB_ATTR(air_drift_max, 0),
    LAB_ATTR(aerial_friction, 0),
    LAB_ATTR(fast_fall_velocity, 0),
    LAB_ATTR(air_max_horizontal_velocity, 0),
    LAB_ATTR(jab_2_input_window, 0),
    LAB_ATTR(jab_3_input_window, 0),
    LAB_ATTR(standing_turn_frames, 0),
    LAB_ATTR(weight, 0),
    LAB_ATTR(model_scaling, 0),
    LAB_ATTR(initial_shield_size, 0),
    LAB_ATTR(shield_break_initial_velocity, 0),
    LAB_ATTR(rapid_jab_window, 1),
    LAB_ATTR(clank_animation_length, 0),
    /* stage E: the landing lags the frame-data export reports */
    LAB_ATTR(normal_landing_lag, 0),
    LAB_ATTR(landingairn_lag, 0),
    LAB_ATTR(landingairf_lag, 0),
    LAB_ATTR(landingairb_lag, 0),
    LAB_ATTR(landingairhi_lag, 0),
    LAB_ATTR(landingairlw_lag, 0),
};
#define LAB_NATTRS ((int) (sizeof(lab_attrs) / sizeof(lab_attrs[0])))

int ScriptGame_LabAttrCount(void)
{
    return LAB_NATTRS;
}

const char* ScriptGame_LabAttrName(int i)
{
    return i >= 0 && i < LAB_NATTRS ? lab_attrs[i].name : NULL;
}

/* The value as a float (ints converted); 0 without a fighter. */
float ScriptGame_LabAttrF(int slot, int i)
{
    Fighter* fp = script_fighter(slot);
    u8* base;
    if (fp == NULL || i < 0 || i >= LAB_NATTRS) {
        return 0.0f;
    }
    base = (u8*) &fp->co_attrs + lab_attrs[i].offset;
    if (lab_attrs[i].is_int) {
        return (float) *(s32*) base;
    }
    return *(f32*) base;
}

/* ---- Lab: the fighter's draw list (DObjs, their materials' TObjs, the model-part states) ---------
 * Read-only, like the joints: what the renderer will draw this frame. `d` indexes fp->dobj_list (the
 * part-visibility tables' DObj numbering), `t` the DObj's MObj TObj chain. docs/geno.md "Geno Lab". */
#include <sysdolphin/baselib/aobj.h>
#include <sysdolphin/baselib/dobj.h>
#include <sysdolphin/baselib/mobj.h>
#include <sysdolphin/baselib/tobj.h>

static HSD_DObj* lab_dobj(Fighter* fp, int d)
{
    if (fp == NULL || d < 0 || d >= (int) fp->dobj_list.count || fp->dobj_list.data == NULL) {
        return NULL;
    }
    return fp->dobj_list.data[d];
}

static HSD_TObj* lab_tobj(HSD_DObj* dobj, int t)
{
    HSD_TObj* tp;
    if (dobj == NULL || dobj->mobj == NULL || t < 0) {
        return NULL;
    }
    for (tp = dobj->mobj->tobj; tp != NULL && t > 0; tp = tp->next) {
        t--;
    }
    return tp;
}

/* field: LAB_DI_* */
int ScriptGame_LabDObjI(int slot, int d, int field)
{
    Fighter* fp = script_fighter(slot);
    HSD_DObj* dobj;
    HSD_TObj* tp;
    int n = 0;
    if (fp == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_DI_COUNT:
        return (int) fp->dobj_list.count;
    case LAB_DI_MODELS:
        return (int) fp->x5AC.model_num;
    case LAB_DI_MODEL_STATE:
        return d >= 0 && d < 12 ? (int) fp->x5F4_arr[d].idx : -1;
    case LAB_DI_COSTUME_TOBJS:
        return (int) fp->tobj_list.n_costume_tobjs;
    }
    dobj = lab_dobj(fp, d);
    if (dobj == NULL) {
        return -1;
    }
    switch (field) {
    case LAB_DI_FLAGS:
        return (int) dobj->flags;
    case LAB_DI_RENDER:
        return dobj->mobj != NULL ? (int) dobj->mobj->rendermode : 0;
    case LAB_DI_TOBJS:
        for (tp = dobj->mobj != NULL ? dobj->mobj->tobj : NULL; tp != NULL; tp = tp->next) {
            n++;
        }
        return n;
    }
    return -1;
}

/* field: LAB_TF_*; d = -1 reads costume TObj t (fp->tobj_list, the eye texture anims) */
float ScriptGame_LabTObjF(int slot, int d, int t, int field)
{
    Fighter* fp = script_fighter(slot);
    HSD_TObj* tp;
    if (fp == NULL) {
        return 0.0f;
    }
    if (d < 0) {
        tp = t >= 0 && t < (int) fp->tobj_list.n_costume_tobjs && t < 5 ? fp->tobj_list.costume_tobjs[t] : NULL;
    } else {
        tp = lab_tobj(lab_dobj(fp, d), t);
    }
    if (tp == NULL) {
        return -1.0f;
    }
    switch (field) {
    case LAB_TF_ID:
        return (float) tp->id;
    case LAB_TF_SRC:
        return (float) tp->src;
    case LAB_TF_FLAGS:
        return (float) (tp->flags & 0x7FFFFFFF);
    case LAB_TF_TU:
        return tp->translate.x;
    case LAB_TF_TV:
        return tp->translate.y;
    case LAB_TF_SU:
        return tp->scale.x;
    case LAB_TF_SV:
        return tp->scale.y;
    case LAB_TF_FRAME:
        return tp->aobj != NULL ? tp->aobj->curr_frame : -1.0f;
    case LAB_TF_FMT:
        return tp->imagedesc != NULL ? (float) tp->imagedesc->format : -1.0f;
    case LAB_TF_W:
        return tp->imagedesc != NULL ? (float) tp->imagedesc->width : -1.0f;
    case LAB_TF_H:
        return tp->imagedesc != NULL ? (float) tp->imagedesc->height : -1.0f;
    }
    return 0.0f;
}

/* ---- Stage E: the knockback preview (the game's own knockback functions) -------------------- */
#include <melee/ft/ftcoll.h>
#include <melee/ft/kinds/ftCommon/ftCo_Damage.h>
#include <melee/lb/forward.h>
#include <melee/gm/gmvs.h>
#include <melee/gr/stage.h>
#include <melee/mp/mplib.h>

/* ---- enemies2: explicit enemy hits; fighter port semantics remain unchanged ---- */
#include <melee/it/itcoll.h>

int ScriptGame_EnemyHit(int handle, int from_slot, int damage, int angle, int kbg, int bkb)
{
    Fighter* from = from_slot >= 0 ? script_fighter(from_slot) : NULL;
    int i;
    if (from_slot >= 0 && from == NULL) return 0;
    for (i = 0; i < SCRIPT_STAGE_ENEMIES; ++i) {
        ScriptStageEnemy* e = &script_stage.enemy[i];
        Item* ip;
        HitCapsule hit = { 0 };
        if (!e->active || e->defeated || e->handle != handle || e->gobj == NULL) continue;
        ip = GET_ITEM(e->gobj);
        if (ip->xB8_itemLogicTable->dmg_received == NULL || damage == 0) return 0;
        hit.damage = (float) damage;
        hit.kb_angle = angle;
        hit.x24 = kbg;
        hit.x2C = bkb;
        /* it_80270CD8 uses the loaded item/common attributes, including the cap.
         * Mirror it_80270E30's result and OnTakeDamageThink's accounting before
         * invoking the real callback. Only that callback can award a defeat. */
        ip->xCA0 = damage;
        ip->xCC8_knockback = it_80270CD8(ip, &hit);
        ip->xCAC_angle = angle;
        ip->xCC4 = HitElement_Normal;
        ip->xCB0_source_ply = from != NULL ? from->player_id : 6;
        ip->xCEC_fighterGObj = from != NULL ? from->gobj : NULL;
        ip->xCF0_itemGObj = NULL;
        ip->xCCC_incDamageDirection = from != NULL && from->cur_pos.x < ip->pos.x
            ? -1.0f : 1.0f;
        Item_80269CA0(ip, damage);
        ip->xCA8 = damage;
        ip->xDC8_word.flags.xB = 1;
        OSReport("script enemy: hit handle=%d kind=%d damage=%d from=%d\n",
                 handle, e->kind, damage, from_slot + 1);
        if (ip->xB8_itemLogicTable->dmg_received(e->gobj)) {
            ip->destroy_type = 2; /* processCallback in item.c */
            Item_8026A8EC(e->gobj);
        } else {
            /* We consumed this result synchronously; the next item collision pass
             * must not process it again. Leave source/angle for the death animation. */
            ip->xCA0 = 0;
            ip->xCC8_knockback = 0.0f;
        }
        return 1;
    }
    return 0;
}
/* ---- end enemies2 hit block ---- */

/* gd.hit: feed the same damage result consumed after ftColl_8007AB48 in Fighter_8006CB94.
 * ftColl_8007A06C chooses the collision's knockback, angle, direction and source;
 * Fighter_ProcessHit_8006D1EC applies percent/HP, damage state and hitlag. The entry and
 * hitbox live on this stack only while the collision result is being consumed. */
int ScriptGame_Hit(int slot, int from_slot, int damage, int angle, int kbg, int bkb)
{
    Fighter* fp = script_fighter(slot);
    Fighter* from = from_slot >= 0 ? script_fighter(from_slot) : NULL;
    HitCapsule hit = { 0 };
    DmgLogEntry entry = { 0 };
    lbColl_80008D30_arg1 env = { 0 };
    float applied = (float) damage;
    if (fp == NULL || (from_slot >= 0 && from == NULL)) {
        return 0;
    }
    if (!ftColl_80076640(fp, &applied)) {
        return 0;
    }
    hit.damage = applied;
    hit.unk_count = (u32) applied;
    hit.kb_angle = angle;
    hit.x24 = kbg;
    hit.x28 = 0;
    hit.x2C = bkb;
    hit.element = HitElement_Normal;
    entry.pos = fp->cur_pos;
    entry.x20 = applied;
    entry.size_of_xC = (size_t) applied;
    if (from != NULL) {
        entry.x0 = 1; /* ftColl_80076ED8: fighter hitbox against fighter hurtbox */
        entry.kind = from->kind;
        entry.gobj = from->gobj;
        entry.hit0 = &hit;
        entry.hurt1 = &fp->hurt_capsules[0];
    } else {
        entry.x0 = 3; /* ftColl_80076764: anonymous environment damage */
        entry.kind = -10;
        env.damage = (u32) applied;
        env.kb_angle = angle;
        env.unkC = kbg;
        env.unk14 = bkb;
        env.element = HitElement_Normal;
        entry.unk_anim0 = (DynamicsDesc*) &env;
        /* ftColl_8007A06C reads best_entry->hit0->kb_angle (the 361 check) for every entry kind */
        entry.hit0 = &hit;
        entry.hurt1 = &fp->hurt_capsules[0];
    }
    {
        extern void ftColl_8007A06C(Fighter_GObj*, void*, void*, size_t, int);
        ftColl_8007A06C(fp->gobj, &fp->dmg.facing_dir_1, &entry, 1, 0);
    }
    Fighter_ProcessHit_8006D1EC(fp->gobj);
    OSReport("script: hit victim=%d from=%d damage=%d angle=%d kbg=%d bkb=%d percent=%d\n",
             slot + 1, from_slot + 1, damage, angle, kbg, bkb,
             (int) fp->dmg.x1830_percent);
    return 1;
}

/* The knockback a hit would give the fighter in `slot` now: ftColl_80079AB0 (the fighter-hit
 * path of ftColl, with the stage factor and both players' attack / defense ratios), then
 * ftCo_Damage_CalcKnockback (crouch, ice, smash charge, Y scale, armour, the minimum). `pct_bits`
 * < 0 (as a float) uses the fighter's percent. The percent and the pending damage the formula reads
 * (x1830 / x1838) and kb_applied are set for the two calls and restored bit for bit before
 * returning, so the game sees no change. Offline reads only (gw_script.c). Floats arrive as bits.
 * `post` = 0 skips the second step (raw formula). Returns -1 without a fighter. */
float ScriptGame_LabKnockback(int slot, int attacker_slot, int dmg_bits, int kbg, int wbk, int bkb,
                              int pct_bits, int post)
{
    Fighter* fp = script_fighter(slot);
    Fighter* at = attacker_slot >= 0 ? script_fighter(attacker_slot) : NULL;
    HitCapsule hit = { 0 };
    union {
        int i;
        float f;
    } dmg, pct;
    float save_pct, save_tmp, save_kb, kb, atk;
    if (fp == NULL) {
        return -1.0f;
    }
    dmg.i = dmg_bits;
    pct.i = pct_bits;
    hit.damage = dmg.f;
    hit.unk_count = (u32) (dmg.f + 0.5f);
    hit.x24 = (u32) kbg;
    hit.x28 = (u32) wbk;
    hit.x2C = (u32) bkb;
    save_pct = fp->dmg.x1830_percent;
    save_tmp = fp->dmg.x1838_percentTemp;
    save_kb = fp->dmg.kb_applied;
    if (pct.f >= 0.0f) {
        fp->dmg.x1830_percent = pct.f;
    }
    fp->dmg.x1838_percentTemp = (float) hit.unk_count;
    atk = at != NULL ? Player_GetAttackRatio(at->player_id) : 1.0f;
    kb = ftColl_80079AB0(fp, &hit, hit.unk_count, gm_8016B248(), atk,
                         Player_GetDefenseRatio(fp->player_id), fp->co_attrs.weight);
    if (post) {
        fp->dmg.kb_applied = kb;
        ftCo_Damage_CalcKnockback(fp);
        kb = fp->dmg.kb_applied;
    }
    fp->dmg.x1830_percent = save_pct;
    fp->dmg.x1838_percentTemp = save_tmp;
    fp->dmg.kb_applied = save_kb;
    return kb;
}

/* ftCo_8008D8E8: the knockback level (0-3, 3 = tumble) of a knockback value, from its hitstun. */
int ScriptGame_LabKbLevel(int kb_bits)
{
    union {
        int i;
        float f;
    } kb;
    kb.i = kb_bits;
    return (int) ftCo_8008D8E8(kb.f * p_ftCommonData->x154);
}

/* The ftCommonData constants the launch uses (PlCo.dat as loaded), and the stage's blast zones. */
float ScriptGame_LabCommonF(int which)
{
    ftCommonData* d = p_ftCommonData;
    switch (which) {
    case LAB_C_KB_SPEED:
        return d->x100;
    case LAB_C_KB_MAX:
        return d->x108;
    case LAB_C_ANGLE_AIR_361:
        return d->x144_radians;
    case LAB_C_ANGLE_GROUND_MAX:
        return d->x148;
    case LAB_C_ANGLE_GROUND_KB0:
        return d->x14C;
    case LAB_C_ANGLE_GROUND_KB1:
        return d->x150;
    case LAB_C_HITSTUN_MUL:
        return d->x154;
    case LAB_C_DI_DEGREES:
        return d->x1A8;
    case LAB_C_KB_DECAY:
        return d->x204_knockbackFrameDecay;
    case LAB_C_SQUAT_MUL:
        return d->kb_squat_mul;
    case LAB_C_BLAST_LEFT:
        return Stage_GetBlastZoneLeftOffset();
    case LAB_C_BLAST_RIGHT:
        return Stage_GetBlastZoneRightOffset();
    case LAB_C_BLAST_TOP:
        return Stage_GetBlastZoneTopOffset();
    case LAB_C_BLAST_BOTTOM:
        return Stage_GetBlastZoneBottomOffset();
    case LAB_C_LCANCEL_WINDOW:
        return (float) d->xE4;
    case LAB_C_LCANCEL_DIV:
        return d->xE8;
    case LAB_C_STICK_SMASH_DZ:
        return d->horizontal_stick_smash_deadzone;
    case LAB_C_TUMBLE_WIGGLE:
        return d->x210;
    case LAB_C_TUMBLE_WINDOW:
        return (float) d->x214;
    case LAB_C_TECH_ROLL_STICK:
        return d->x254;
    case LAB_C_SDI_MIN:
        return d->sdi_min_stick_mag;
    case LAB_C_SDI_WINDOW:
        return (float) d->sdi_stick_window;
    case LAB_C_ASDI_SCALE:
        return d->x4BC;
    case LAB_C_STICK_DZ_X:
        return d->horizontal_stick_deadzone;
    case LAB_C_STICK_DZ_Y:
        return d->vertical_stick_deadzone;
    }
    return 0.0f;
}

/* Stage D. The first floor line under (x, y) within `depth` units: its y, or -1e6 when there is
 * none. The dummy times its tech press with it. mpCheckFloor clears the collision joints'
 * bounding flags it sets, so this leaves no state behind (safe between frames, rewind-exact). */
float ScriptGame_LabFloorY(int x_bits, int y_bits, int depth_bits)
{
    union {
        int i;
        float f;
    } x, y, d;
    Vec3 pos;
    x.i = x_bits;
    y.i = y_bits;
    d.i = depth_bits;
    if (!mpCheckFloor(x.f, y.f, x.f, y.f - d.f, 0.0f, &pos, NULL, NULL, NULL, -1, -1, -1, NULL, NULL)) {
        return -1000000.0f;
    }
    return pos.y;
}

/* Stage D, the dummy's infinite shield: a gameplay write (offline; gw_script.c forks the Lab's
 * timeline first, as for set_percent). */
int ScriptGame_LabSetShield(int slot, int health_bits)
{
    Fighter* fp = script_fighter(slot);
    union {
        int i;
        float f;
    } h;
    if (fp == NULL) {
        return -1;
    }
    h.i = health_bits;
    fp->shield_health = h.f;
    return 0;
}

/* The victim's own numbers the flight uses: gravity, terminal velocity, aerial friction,
 * facing, grounded. */
float ScriptGame_LabFlightF(int slot, int which)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL) {
        return 0.0f;
    }
    switch (which) {
    case 0:
        return fp->co_attrs.gravity;
    case 1:
        return fp->co_attrs.terminal_velocity;
    case 2:
        return fp->co_attrs.aerial_friction;
    case 3:
        return fp->facing_dir;
    case 4:
        return fp->ground_or_air == GA_Ground ? 1.0f : 0.0f;
    case 5:
        return fp->co_attrs.weight;
    case 6:
        return fp->dmg.x1830_percent;
    }
    return 0.0f;
}

/* ---- Stage E: name a SyncTest mismatch inside a Fighter (the rollback visualiser) ---------- */
#define LAB_FF(name, field) { name, (int) __builtin_offsetof(Fighter, field), (int) sizeof(((Fighter*) 0)->field) }
static const struct {
    const char* name;
    int off, size;
} lab_ffields[] = {
    LAB_FF("kind", kind),
    LAB_FF("motion_id", motion_id),
    LAB_FF("anim_id", anim_id),
    LAB_FF("facing_dir", facing_dir),
    LAB_FF("x34_scale", x34_scale),
    LAB_FF("x44_mtx", x44_mtx),
    LAB_FF("x74_self_accel", x74_self_accel),
    LAB_FF("self_vel", self_vel),
    LAB_FF("x8c_kb_vel", x8c_kb_vel),
    LAB_FF("x98_atk_shield_kb", x98_atk_shield_kb),
    LAB_FF("cur_pos", cur_pos),
    LAB_FF("prev_pos", prev_pos),
    LAB_FF("pos_delta", pos_delta),
    LAB_FF("ground_or_air", ground_or_air),
    LAB_FF("gr_vel", gr_vel),
    LAB_FF("xF0_ground_kb_vel", xF0_ground_kb_vel),
    LAB_FF("input", input),
    LAB_FF("co_attrs", co_attrs),
    LAB_FF("coll_data", coll_data),
    LAB_FF("ecb_lock", ecb_lock),
    LAB_FF("cur_anim_frame", cur_anim_frame),
    LAB_FF("frame_speed_mul", frame_speed_mul),
    LAB_FF("x8B0", x8B0),
    LAB_FF("x914 (hitboxes)", x914),
    LAB_FF("xDF4", xDF4),
    LAB_FF("x1064_thrownHitbox", x1064_thrownHitbox),
    LAB_FF("hurt_capsules", hurt_capsules),
    LAB_FF("x1614", x1614),
    LAB_FF("dmg", dmg),
    LAB_FF("x1968_jumpsUsed", x1968_jumpsUsed),
    LAB_FF("x2064_ledgeCooldown", x2064_ledgeCooldown),
    LAB_FF("mv (state vars)", mv),
};
#define LAB_NFF ((int) (sizeof lab_ffields / sizeof lab_ffields[0]))

/* (player << 16) | offset when `va` lies inside a live Fighter struct, else -1 */
int ScriptGame_LabFighterAt(u32 va)
{
    HSD_GObj* g;
    for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
        Fighter* f = GET_FIGHTER(g);
        u32 base = (u32) (uintptr_t) f;
        if (f != NULL && va >= base && va < base + (u32) sizeof(Fighter)) {
            return ((int) f->player_id << 16) | (int) (va - base);
        }
    }
    return -1;
}

static int lab_ffield(int off)
{
    int i;
    for (i = 0; i < LAB_NFF; ++i) {
        if (off >= lab_ffields[i].off && off < lab_ffields[i].off + lab_ffields[i].size) {
            return i;
        }
    }
    return -1;
}
const char* ScriptGame_LabFighterFieldName(int off)
{
    int i = lab_ffield(off);
    return i >= 0 ? lab_ffields[i].name : NULL;
}
int ScriptGame_LabFighterFieldBase(int off)
{
    int i = lab_ffield(off);
    return i >= 0 ? lab_ffields[i].off : off;
}

/* ---- largemap: isolated area, capacity and bounds API ---- */
#include "script_largemap.inc"
