/* geno_game_articles.inc - Geno v5: articles (projectiles), the on_hit dispatch point and counter
 * windows. Included at the end of geno_game.c (one translation unit: it shares that file's statics).
 * Design and data format: docs/geno.md section 19.
 *
 * ARTICLES ARE MELEE ITEMS. Article a of profile p is item kind GENO_ART_KIND_BASE + p * 8 + a.
 * Item_80267978 (item.c) asks Geno_ArticleDesc / Geno_ArticleLogic for its descriptor and logic
 * table instead of a vanilla / m-ex table, and Item_802674AC gives it the character-article hold
 * kind, so from there it is an ordinary item: Item_80268B18 spawns it, the engine's item procs move
 * it (x40_vel), draw it (the generic item renderer, it_8026EECC), run its hitboxes against fighters,
 * shields, reflectors, absorbers and counters, and destroy it when a callback says so. What Geno adds
 * is the descriptor (built from geno.json at spawn), one item state with three native callbacks
 * (life / hitbox windows, motion, stage contact) and the reaction callbacks (hit, shield, clank,
 * reflect, absorb). Reused as-is: the spawn path Fox's laser uses (it_8029C504: SpawnItem,
 * Item_80268B18, Item_80268E5C, db_80225DD8), the item hitbox setup the item script's hitbox
 * command performs (it_802790C0: it_8026FCF8, it_80272460, it_80275594, it_8027129C), the laser's
 * stage ray test (it_8026E9A4) and its reference cleanup (it_8026B894).
 *
 * ROLLBACK. The descriptors, attributes and model descs are game globals written only with values
 * from the registry (idempotent, the same bytes every time). The model's joint tree lives in Geno's
 * withheld guest region, loaded once and never written. Everything that changes per frame is in the
 * item (game heap, snapshotted with every item): its position, velocity and hitboxes, and the
 * GenoArtVars below in its per-kind variable block. No host state, time or randomness.
 */

#include <melee/db/db.h>
#include <melee/ef/efasync.h>
#include <melee/lb/lb_00B0.h>
#include <sysdolphin/baselib/generator.h>
#include <sysdolphin/baselib/particle.h>
#include <sysdolphin/baselib/psstructs.h>
#include <sysdolphin/baselib/cobj.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjgxlink.h>
#include <melee/ef/forward.h>

extern HSD_Particle* hsd_804D0908[16]; /* particle.c: the live particles, one list per link */
#include <melee/it/it_26B1.h>
#include <melee/it/it_2725.h>
#include <melee/it/itcoll.h>
#include <melee/it/itgroundcoll.h>
#include <melee/it/ithitbox.h>
#include <melee/it/item.h>
#include <melee/it/kinds/inlines.h>
#include <melee/it/kinds/types.h>
#include <melee/it/types.h>
#include <sysdolphin/baselib/gobjproc.h>
#include <sysdolphin/baselib/jobj.h>

extern int Geno_ArticleCount(int p);
extern int Geno_ArticleParam(int p, int a, int id);
extern int Geno_ArticleHitCount(int p, int a);
extern int Geno_ArticleHitParam(int p, int a, int h, int id);
extern int Geno_ArticleJoint(int p, int a);
extern int Geno_StateCounter(int p, int s, int field);
extern int Geno_StateHasCounter(int p, int s);
extern int Geno_SpecialSelect(int p, int which, int i);

/* ---- game globals: one descriptor per (profile, article) ------------------------------------- */

Article** Geno_ArtDesc;
ItemAttr** Geno_ArtAttr;
ItemModelDesc** Geno_ArtModel;
int Geno_ArtProfileCapacity;
ItemStateArray Geno_ArtStates; /* never read: the one state has no animation (anim_id -1) */

/* The per-item variables, in the item's per-kind block (Item.xDD4_itemVar). */
static GenoArtVars* geno_art_vars(Item* ip)
{
    return (GenoArtVars*) &ip->xDD4_itemVar;
}

static f32 geno_art_f(int p, int a, int id)
{
    union {
        u32 u;
        f32 f;
    } b;
    b.u = (u32) Geno_ArticleParam(p, a, id);
    return b.f;
}

static f32 geno_art_hf(int p, int a, int h, int id)
{
    union {
        u32 u;
        f32 f;
    } b;
    b.u = (u32) Geno_ArticleHitParam(p, a, h, id);
    return b.f;
}

/* An item kind -> (profile, article); 0 when it is not a Geno article. */
static int geno_art_of_kind(int kind, int* p, int* a)
{
    if (!GENO_ART_IS_KIND(kind)) {
        return 0;
    }
    if (kind >= GENO_ART_EXTRA_BASE) {
        *p = 32 + (kind - GENO_ART_EXTRA_BASE) / 16;
        *a = (kind - GENO_ART_EXTRA_BASE) % 16;
    } else if (kind < GENO_ART_KIND_BASE2) {
        *p = (kind - GENO_ART_KIND_BASE) / GENO_ART_PER_RANGE;
        *a = (kind - GENO_ART_KIND_BASE) % GENO_ART_PER_RANGE;
    } else {
        *p = (kind - GENO_ART_KIND_BASE2) / GENO_ART_PER_RANGE;
        *a = GENO_ART_PER_RANGE + (kind - GENO_ART_KIND_BASE2) % GENO_ART_PER_RANGE;
    }
    return *a < Geno_ArticleCount(*p);
}

/* v5.5: ftcoll.c - an item's hitbox `hit` hit fighter `vic`: a Geno article's hitbox entry gives its "stun" bonus */
void Geno_ArticleStunBonus(Item* item, HitCapsule* hit, Fighter* vic)
{
    int p, a, slot, h;
    GenoArtVars* v;
    if (item == NULL || hit == NULL || !geno_art_of_kind(item->kind, &p, &a)) {
        return;
    }
    for (slot = 0; slot < 4 && &item->x5D4_hitboxes[slot].hit != hit; slot++) {
    }
    if (slot >= 4 || slot >= GENO_ART_HITBOXES) {
        return;
    }
    v = geno_art_vars(item);
    h = v->cur[slot];
    if (h < 0) {
        return;
    }
    Geno_GiveStunBonus(vic, Geno_ArticleHitParam(p, a, h, GENO_AH_STUN));
}

/* The descriptor, from the registry (idempotent: the same values at every spawn). */
static void geno_art_prepare(int p)
{
    int n = Geno_ProfileCount(), i;
    if (n > Geno_ArtProfileCapacity) {
        Article** ds = geno_profile_alloc(n * sizeof(*ds));
        ItemAttr** as = geno_profile_alloc(n * sizeof(*as));
        ItemModelDesc** ms = geno_profile_alloc(n * sizeof(*ms));
        for (i = 0; i < Geno_ArtProfileCapacity; ++i) {
            ds[i] = Geno_ArtDesc[i]; as[i] = Geno_ArtAttr[i]; ms[i] = Geno_ArtModel[i];
        }
        Geno_ArtDesc = ds; Geno_ArtAttr = as; Geno_ArtModel = ms; Geno_ArtProfileCapacity = n;
    }
    if (!Geno_ArtDesc[p]) Geno_ArtDesc[p] = geno_profile_alloc(GENO_MAX_ARTICLES * sizeof(Article));
    if (!Geno_ArtAttr[p]) Geno_ArtAttr[p] = geno_profile_alloc(GENO_MAX_ARTICLES * sizeof(ItemAttr));
    if (!Geno_ArtModel[p]) Geno_ArtModel[p] = geno_profile_alloc(GENO_MAX_ARTICLES * sizeof(ItemModelDesc));
}
static void geno_art_build(int p, int a)
{
    geno_art_prepare(p);
    Article* d = &Geno_ArtDesc[p][a];
    ItemAttr* at = &Geno_ArtAttr[p][a];
    ItemModelDesc* m = &Geno_ArtModel[p][a];
    geno_zero(at, sizeof(*at));
    at->x60_scale = geno_art_f(p, a, GENO_AP_SCALE);
    if (at->x60_scale <= 0.0f) {
        at->x60_scale = 1.0f;
    }
    geno_zero(m, sizeof(*m));
    m->x0_joint = (HSD_Joint*) Geno_ArticleJoint(p, a);
    geno_zero(d, sizeof(*d));
    d->x0_common_attr = at;
    d->x4_specialAttributes = NULL;
    d->x8_hurtbones = NULL; /* articles are not hittable (no hurtbox), like Fox's laser */
    d->xC_itemStates = &Geno_ArtStates;
    d->x10_modelDesc = m;
    d->x14_dynamics = NULL;
}

/* Item_80267978 (item.c): the descriptor and logic table of a Geno article kind. */
static bool geno_art_anim(Item_GObj* gobj);
static void geno_art_phys(Item_GObj* gobj);
static bool geno_art_coll(Item_GObj* gobj);
static bool geno_art_hit(Item_GObj* gobj);
static bool geno_art_shield(Item_GObj* gobj);
static bool geno_art_clank(Item_GObj* gobj);
static bool geno_art_reflected(Item_GObj* gobj);
static bool geno_art_absorbed(Item_GObj* gobj);
static void geno_art_unref(Item_GObj* gobj, HSD_GObj* ref)
{
    it_8026B894(gobj, ref);
}

static ItemStateTable geno_art_states[1] = {
    { -1, geno_art_anim, geno_art_phys, geno_art_coll },
};

/* Every way an article goes (hit, timeout, stage, blast zone, absorbed): stop the particle
 * generators that follow its root joint (a "FollowJointPos" m-ex generator keeps the item's JObj;
 * efLib_DestroyAll only removes effect MODELS). hsd_8039D5DC is what grlib uses for a stage part:
 * the emitted particles live on, the generators stop. */
/* Geno effects (gw_fx.c, native): ints across; the JObj is a guest address, its world matrix at +0x44. */
extern int Fx_Attach(int pkg, int owner, int mtx_off, int frame, int facing);
extern void Fx_Detach(int owner);
extern void Fx_SetOwnerPort(int owner, int port);
extern void Fx_Frame(int frame);
static int geno_fx_frame; /* the last frame Geno_FxFramePost saw (seeds attaches) */
/* one number per game frame for the hitstun bonus (game memory; hits and the damage state of one frame agree) */
int geno_stun_frame(void)
{
    return geno_fx_frame;
}

static void geno_art_destroyed(Item_GObj* gobj)
{
    hsd_8039D5DC(GET_JOBJ(gobj));
    Fx_Detach((int) GET_JOBJ(gobj)); /* its emitters stop; live particles finish */
}

/* hide every DObj of an article's joint tree (depth first: child, then next) */
static void geno_art_hide_model(HSD_JObj* j)
{
    for (; j != NULL; j = j->next) {
        HSD_DObj* d;
        for (d = (j->flags & (JOBJ_PTCL | JOBJ_SPLINE | JOBJ_INSTANCE)) ? NULL : j->u.dobj; d != NULL; d = d->next) {
            HSD_DObjSetFlags(d, DOBJ_HIDDEN);
        }
        geno_art_hide_model(j->child);
    }
}

/* Once per scene frame, after the scripts (gmscene.c): the scene's frame counter is game memory, so a
 * rollback or LAB rewind brings it back and gw_fx re-simulates from there. */
extern void Fx_Draw(int view_mtx);
extern int Fx_Stat(int what);
extern void Fx_SetOwner(int handle, int kind, int port);
static HSD_GObj* geno_fx_gobj; /* game memory: a rollback restores it with the GObj heap */

/* The effects draw inside the camera that draws gx link 8 (Melee's own particles'), after them (priority 255),
 * in the translucent pass (render code 2 = HSD_GObj_804085F0's XLU): depth-tested against the world, under the
 * HUD's camera. The native side draws once per frame, with this camera's viewing matrix. */
extern void diag_fx_cam(int cobj, int code, int main_cobj, float ex, float ey, float ez, float mx, float my,
                        float mz);
extern HSD_CObj* Camera_800310B8(void);
static void geno_fx_render(HSD_GObj* gobj, int code)
{
    HSD_CObj* cobj;
    {
        /* MELEE_FX_CAMLOG: every call, any code: which camera is current, and the main camera's eye */
        HSD_CObj* cur = HSD_CObjGetCurrent();
        HSD_CObj* mc = Camera_800310B8();
        Vec3 e = { 0, 0, 0 }, m = { 0, 0, 0 };
        if (cur != NULL) {
            HSD_CObjGetEyePosition(cur, &e);
        }
        if (mc != NULL) {
            HSD_CObjGetEyePosition(mc, &m);
        }
        diag_fx_cam((int) cur, code, (int) mc, e.x, e.y, e.z, m.x, m.y, m.z);
    }
    if (code != 2) {
        return;
    }
    cobj = HSD_CObjGetCurrent();
    if (cobj != NULL) {
        Fx_Draw((int) HSD_CObjGetViewingMtxPtr(cobj)); /* computed (LookAt when dirty), as psdisp.c: the Direct field can be stale */
    }
}

/* v5.5: fighters whose profile names "fx_bindings" attach their states' effects (gw_fx.c keeps what happened in
 * each state with its own rollback state; this only reports where each fighter is). */
extern int Geno_FxBindings(int p);
extern void Fx_Drive(int set, int owner, int motion, int anim, float anim_frame, int time, int airborne, int facing, int port,
                     int parts, int stride, int nparts, int joint_to_part, int root, int frame);
extern HSD_GObj* Player_GetEntity(s32 slot);
extern int Fx_Owner(int i);
static void geno_fx_setup_tree(HSD_JObj* j)
{
    for (; j != NULL; j = j->next) {
        HSD_JObjSetupMatrix(j);
        geno_fx_setup_tree(j->child);
    }
}

static void geno_fx_drive(int frame)
{
    int slot;
    for (slot = 0; slot < 6; slot++) {
        HSD_GObj* g = Player_GetEntity(slot);
        Fighter* fp;
        int p, set;
        if (g == NULL) {
            continue;
        }
        fp = GET_FIGHTER(g);
        p = Geno_ProfileForKind(fp->kind);
        set = p >= 0 ? Geno_FxBindings(p) : -1;
        if (set < 0 || fp->parts == NULL || ftPartsTable[fp->kind] == NULL) {
            continue;
        }
        geno_fx_setup_tree(GET_JOBJ(g)); /* this frame's joint matrices (HSD computes them lazily at display) */
        Fx_Drive(set, (int) fp, (int) fp->motion_id, fp->anim_id, fp->cur_anim_frame, (int) geno_state(fp)->action_time,
                 fp->ground_or_air == GA_Air, fp->facing_dir < 0.0f ? -1 : 1, slot + 1, (int) fp->parts,
                 (int) sizeof(FighterBone), (int) ftPartsTable[fp->kind]->parts_num,
                 (int) ftPartsTable[fp->kind]->joint_to_part, (int) GET_JOBJ(g), frame);
    }
}

void Geno_FxFramePost(int frame)
{
    if (frame <= 1) {
        geno_fx_gobj = NULL; /* a new scene: the last scene's GObjs are gone */
    }
    geno_fx_frame = frame;
    geno_fx_drive(frame);
    {
        /* every JObj an effect follows: this frame's world matrix before the step reads it */
        int i, owner;
        for (i = 0; i < 256; i++) {
            if ((owner = Fx_Owner(i)) != 0) {
                HSD_JObjSetupMatrix((HSD_JObj*) owner);
            }
        }
    }
    Fx_Frame(frame);
    /* only once something is live: a match without Geno effects never has this GObj */
    if (geno_fx_gobj == NULL && Fx_Stat(0) > 0) {
        geno_fx_gobj = GObj_Create(HSD_GOBJ_CLASS_EFFECT, 0xC, 0);
        if (geno_fx_gobj != NULL) {
            GObj_SetupGXLink(geno_fx_gobj, geno_fx_render, 7, 255); /* link 7: the world particles (the main camera); 8 is the screen-space camera */
        }
    }
}

static ItemLogicTable geno_art_logic = {
    geno_art_states,
    NULL,               /* spawned */
    geno_art_destroyed, /* destroyed */
    NULL,               /* picked_up */
    NULL,               /* dropped */
    NULL,               /* thrown */
    geno_art_hit,       /* dmg_dealt */
    NULL,               /* dmg_received */
    NULL,               /* entered_air */
    geno_art_reflected, /* reflected */
    geno_art_clank,     /* clanked */
    geno_art_absorbed,  /* absorbed */
    geno_art_shield,    /* shield_bounced */
    geno_art_shield,    /* hit_shield */
    geno_art_unref,     /* evt_unk: drop references to a fighter that is going away */
};

void* Geno_ArticleDesc(int kind)
{
    int p, a;
    if (!geno_art_of_kind(kind, &p, &a)) {
        return NULL; /* Item_80267978's "no descriptor" report follows */
    }
    geno_art_build(p, a);
    return &Geno_ArtDesc[p][a];
}

void* Geno_ArticleLogic(int kind)
{
    return &geno_art_logic;
}

/* Live Geno articles owned by `owner` (article index `article`, or any when -1). */
static int geno_art_live(HSD_GObj* owner, int article)
{
    HSD_GObj* cur;
    int n = 0, p, a;
    if (HSD_GObjPLinkHead == NULL || owner == NULL) {
        return 0;
    }
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; cur != NULL; cur = cur->next) {
        Item* ip = GET_ITEM(cur);
        if (ip == NULL || ip->owner != owner || !geno_art_of_kind(ip->kind, &p, &a)) {
            continue;
        }
        if (article < 0 || a == article) {
            n++;
        }
    }
    return n;
}

/* ---- hitboxes: what the item script's hitbox command (it_802790C0) does, from geno.json -------- */

/* Entry h into Melee hitbox slot `slot`. A slot that is already on is re-set IN PLACE (Ultimate's
 * ATTACK re-issued on the same id: new numbers, the victim list stays), as the item script's hitbox
 * command does when the hit group is unchanged. */
static void geno_art_hitbox_on(Item_GObj* gobj, Item* ip, int p, int a, int h, int slot)
{
    ItemHitbox* hb = &ip->x5D4_hitboxes[slot];
    HitCapsule* hit = &hb->hit;
    u32 fl = (u32) Geno_ArticleHitParam(p, a, h, GENO_AH_FLAGS);
    f32 dmg = geno_art_hf(p, a, h, GENO_AH_DAMAGE);
    if (hit->state == HitCapsule_Disabled) {
        hit->x4 = 0;
        hit->state = HitCapsule_Enabled;
        ip->xDC8_word.flags.x16 = 1;
        ip->xDAA_flag.b2 = 0;
        it_8026FCF8(ip, hit);
    }
    hit->jobj = gobj->hsd_obj;
    ScriptGame_HitRuleForget(hit);
    it_80272460(hit, (u32) (ip->xC3C * ((dmg + 0.5f) * ip->xC40)), gobj);
    hit->scale = geno_art_hf(p, a, h, GENO_AH_SIZE);
    ip->x3C = hit->scale;
    it_80275594(gobj, slot, 1.0f / ip->scl);
    hit->b_offset.x = geno_art_hf(p, a, h, GENO_AH_OFF_X);
    hit->b_offset.y = geno_art_hf(p, a, h, GENO_AH_OFF_Y);
    hit->b_offset.z = geno_art_hf(p, a, h, GENO_AH_OFF_Z);
    hit->kb_angle = Geno_ArticleHitParam(p, a, h, GENO_AH_ANGLE);
    hit->x24 = (u32) Geno_ArticleHitParam(p, a, h, GENO_AH_KBG);
    hit->x28 = (u32) Geno_ArticleHitParam(p, a, h, GENO_AH_WKB);
    hit->x43_b1 = 0;
    hit->x2C = (u32) Geno_ArticleHitParam(p, a, h, GENO_AH_BKB);
    hit->element = (u32) Geno_ArticleHitParam(p, a, h, GENO_AH_ELEMENT);
    hit->x40_b0 = 0;
    hit->x40_b1 = 0;
    hit->x34 = Geno_ArticleHitParam(p, a, h, GENO_AH_SHIELD_DAMAGE);
    hit->sfx_severity = Geno_ArticleHitParam(p, a, h, GENO_AH_SFX_SEVERITY);
    hit->sfx_kind = Geno_ArticleHitParam(p, a, h, GENO_AH_SFX_KIND);
    hit->x40_b2 = (fl & GENO_AHF_AIR) != 0;    /* hits airborne fighters (ftcoll.c) */
    hit->x40_b3 = (fl & GENO_AHF_GROUND) != 0; /* hits grounded fighters */
    hit->x40_b4 = 0;
    hit->x41_b4 = 0;
    hit->x41_b5 = 0;
    hit->x41_b6 = 0;
    hit->x41_b7 = (fl & GENO_AHF_REFLECT) != 0; /* a reflector can take it */
    hit->x42_b0 = (fl & GENO_AHF_ABSORB) != 0;  /* an absorber can take it */
    hit->x42_b1 = (fl & GENO_AHF_COUNTER) != 0; /* shield-bubble counters (Marth's) can take it */
    hit->x42_b2 = 0;                            /* 1 would hit only fighters facing away */
    hit->x42_b3 = 0;
    hit->x42_b4 = 1; /* reflect / absorb checks apply to it at all */
    hit->x42_b5 = 1; /* it hits fighters */
    hit->x42_b6 = 0; /* 1 would hit only grabbable hurtboxes */
    hit->x42_b7 = 0;
    hit->x43_b0 = 0;
    hb->x138 = 1;
    hit->x43_b2 = 0;
    ScriptGame_HitRuleItemCreate(ip, hit);
    if (HSD_GObj_CurrentInvokedProc != NULL && HSD_GObj_CurrentInvokedProc->s_link > 11) {
        it_8027129C(gobj, slot);
    }
}

/* Each entry is on during [start, end] of the article's life (end 0 = to the end), in its slot; the
 * last listed entry whose window holds the frame wins its slot. A slot whose entry changes hands over
 * in place; a slot with no entry is switched off. */
static void geno_art_hitboxes(Item_GObj* gobj, Item* ip, GenoArtVars* v)
{
    int n = Geno_ArticleHitCount(v->profile, v->article), h, sl;
    int want[GENO_ART_HITBOXES];
    for (sl = 0; sl < GENO_ART_HITBOXES; sl++) {
        want[sl] = -1;
    }
    for (h = 0; h < n && h < GENO_ART_HIT_ENTRIES; h++) {
        int start = Geno_ArticleHitParam(v->profile, v->article, h, GENO_AH_START);
        int end = Geno_ArticleHitParam(v->profile, v->article, h, GENO_AH_END);
        if (v->frame >= start && (end <= 0 || v->frame <= end)) {
            want[Geno_ArticleHitParam(v->profile, v->article, h, GENO_AH_SLOT) & (GENO_ART_HITBOXES - 1)] = h;
        }
    }
    for (sl = 0; sl < GENO_ART_HITBOXES; sl++) {
        if (want[sl] < 0) {
            ip->x5D4_hitboxes[sl].hit.state = HitCapsule_Disabled;
        } else if (want[sl] != v->cur[sl] || ip->x5D4_hitboxes[sl].hit.state == HitCapsule_Disabled) {
            geno_art_hitbox_on(gobj, ip, v->profile, v->article, want[sl], sl);
        }
        v->cur[sl] = (s8) want[sl];
    }
}

/* ---- motion: pure, so the headless tests run it without an item ------------------------------ */

/* One frame of an article's travel. (vx, vy) the velocity, (px, py) its position, (tx, ty) the
 * homing target (has_target 0 = none), frame the article's frame. Writes the new velocity. */
void GenoGame_ArticleStep(int p, int a, int frame, f32* vx, f32* vy, f32 px, f32 py, int has_target,
                          f32 tx, f32 ty)
{
    f32 grav = geno_art_f(p, a, GENO_AP_GRAVITY), mf = geno_art_f(p, a, GENO_AP_MAX_FALL);
    f32 acc = geno_art_f(p, a, GENO_AP_ACCEL), ms = geno_art_f(p, a, GENO_AP_MAX_SPEED);
    f32 turn = geno_art_f(p, a, GENO_AP_HOMING_TURN), range = geno_art_f(p, a, GENO_AP_HOMING_RANGE);
    f32 delay = geno_art_f(p, a, GENO_AP_HOMING_DELAY);
    f32 speed = sqrtf(*vx * *vx + *vy * *vy);
    if (acc != 0.0f && speed > 0.0001f) {
        f32 ns = speed + acc, mn = geno_art_f(p, a, GENO_AP_MIN_SPEED);
        if (acc < 0.0f && ns < mn) {
            ns = speed < mn ? speed : mn; /* a brake stops at min_speed (Ultimate's stable_speed) */
        }
        if (ns < 0.0f) {
            ns = 0.0f;
        }
        *vx *= ns / speed;
        *vy *= ns / speed;
        speed = ns;
    }
    if (turn > 0.0f && has_target && (f32) frame > delay && speed > 0.0001f) {
        f32 dx = tx - px, dy = ty - py;
        if (range <= 0.0f || dx * dx + dy * dy <= range * range) {
            f32 cur = atan2f(*vy, *vx), want = atan2f(dy, dx), d = want - cur;
            f32 lim = turn * (f32) M_PI / 180.0f;
            while (d > (f32) M_PI) {
                d -= 2.0f * (f32) M_PI;
            }
            while (d < -(f32) M_PI) {
                d += 2.0f * (f32) M_PI;
            }
            d = d > lim ? lim : d < -lim ? -lim : d;
            cur += d;
            *vx = speed * cosf(cur);
            *vy = speed * sinf(cur);
        }
    }
    *vy -= grav;
    if (mf > 0.0f && *vy < -mf) {
        *vy = -mf;
    }
    if (ms > 0.0f) {
        speed = sqrtf(*vx * *vx + *vy * *vy);
        if (speed > ms) {
            *vx *= ms / speed;
            *vy *= ms / speed;
        }
    }
}

/* The nearest fighter that is not the owner (its body: position + 5 up). */
static int geno_art_target(Item* ip, f32* tx, f32* ty)
{
    HSD_GObj* cur;
    f32 best = -1.0f;
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; cur != NULL; cur = cur->next) {
        Fighter* fp = GET_FIGHTER(cur);
        f32 dx, dy, d;
        if (fp == NULL || cur == ip->owner ||
            (ip->owner != NULL && fp->player_id == GET_FIGHTER(ip->owner)->player_id))
        {
            continue;
        }
        dx = fp->cur_pos.x - ip->pos.x;
        dy = fp->cur_pos.y + 5.0f - ip->pos.y;
        d = dx * dx + dy * dy;
        if (best < 0.0f || d < best) {
            best = d;
            *tx = fp->cur_pos.x;
            *ty = fp->cur_pos.y + 5.0f;
        }
    }
    return best >= 0.0f;
}

/* ---- the state's callbacks --------------------------------------------------------------------- */

static void geno_art_gone(Item* ip, GenoArtVars* v, int why)
{
    HSD_GObj* o = ip->owner;
    v->why = why;
    Geno_Event(29, o != NULL ? GET_FIGHTER(o)->kind : -1, o != NULL ? GET_FIGHTER(o)->player_id : -1,
               v->article, why);
}

static Item_GObj* geno_art_spawn(HSD_GObj* owner, int p, int a, int variant, f32 angle, Vec3* base,
                                  f32 facing);

/* The article model's joint `k` in depth-first order (0 = the root), NULL past the end. */
static HSD_JObj* geno_art_joint(Item_GObj* gobj, int k)
{
    HSD_JObj* j = GET_JOBJ(gobj);
    while (j != NULL && k > 0) {
        if (HSD_JObjGetChild(j) != NULL) {
            j = HSD_JObjGetChild(j);
        } else {
            while (j != NULL && HSD_JObjGetNext(j) == NULL) {
                j = HSD_JObjGetParent(j);
            }
            j = j != NULL ? HSD_JObjGetNext(j) : NULL;
        }
        k--;
    }
    return j;
}

/* Attach the article's effects whose frame is `frame` ("effects", v5.2 / v5.3). An id 5000..8999 is the
 * owner fighter's own m-ex effect (its bank's models 5000+ / particle generators 6000+), queued as an
 * m-ex article's item script does (iteffect.c) and flushed at once; any other id is Melee's. Returns
 * how many it attached. */
static int geno_art_effects(Item_GObj* ig, Item* ip, int p, int a, int frame, f32 facing)
{
    static const Vec3 zero = { 0.0f, 0.0f, 0.0f };
    int e, n = 0;
    for (e = -1; e < 8; e++) {
        int id = Geno_ArticleParam(p, a, e < 0 ? GENO_AP_EFFECT : GENO_AP_EFFECTS + e);
        int at = e < 0 ? 0 : Geno_ArticleParam(p, a, GENO_AP_EFFECT_AT + e);
        HSD_JObj* j;
        int c, count = ((u32) at >> 24) + 1;
        if (id <= 0 || (at & 0xFFFF) != frame) {
            continue;
        }
        j = geno_art_joint(ig, (at >> 16) & 0xFF);
        if (j == NULL) {
            j = GET_JOBJ(ig);
        }
        for (c = 0; c < count; c++) {
            Vec3 off = zero;
            if (EF_MEX_IS_CUSTOM(id + c)) {
                efAsync_MexSpawn(ig, &ip->xBC0, id + c, j, &off, facing, 0.0f);
            } else {
                efAsync_Spawn(ig, &ip->xBC0, 1, (u32) (id + c), j);
            }
            n++;
        }
    }
    if (n > 0) {
        efAsync_QueueFlush(ig, &ip->xBC0);
        Geno_Event(37, a, n, frame, 0);
    }
    return n;
}

/* v5.1 "children": an article spawns articles (a cloud's bolts) at frame, every k frames, n times,
 * from its own position and facing, owned by its fighter. */
static void geno_art_children(Item_GObj* gobj, Item* ip, GenoArtVars* v)
{
    int c;
    for (c = 0; c < GENO_ART_CHILDREN; c++) {
        int base = GENO_AP_CHILD + 5 * c;
        int art = Geno_ArticleParam(v->profile, v->article, base);
        int f0 = Geno_ArticleParam(v->profile, v->article, base + 1);
        int every = Geno_ArticleParam(v->profile, v->article, base + 2);
        int count = Geno_ArticleParam(v->profile, v->article, base + 3);
        int k;
        if (art < 0 || v->frame < f0 || ip->owner == NULL) {
            continue;
        }
        k = every > 0 ? (v->frame - f0) / every : 0;
        if ((every > 0 ? (v->frame - f0) % every == 0 : v->frame == f0) && k < count) {
            geno_art_spawn(ip->owner, v->profile, art, Geno_ArticleParam(v->profile, v->article, base + 4),
                           0.0f, &ip->pos, ip->facing_dir);
        }
    }
}

/* Diagnostics (event 38/39, at article frames 5, 10, 20 and 30): the particle generators attached to
 * this article's root joint, and the live particles of their bank per generator (matched by the
 * generator's command list). Reads only; the log is how the particle pilot is checked. */
static void geno_art_fx_census(Item_GObj* gobj, GenoArtVars* v)
{
    HSD_Generator* g;
    HSD_JObj* j0 = GET_JOBJ(gobj);
    Item* ip = GET_ITEM(gobj);
    u8* lists[32];
    int ngen = 0, per[32], total = 0, i, k;
    f32 sdx = 0.0f, sdy = 0.0f, svx = 0.0f;
    for (k = 0; k < 32; k++) {
        per[k] = 0;
    }
    /* the generators on this article or its joints, in creation order (the list is newest first) */
    for (g = hsd_804D78FC; g != NULL; g = g->next) {
        HSD_JObj* gj = g->jobj;
        while (gj != NULL && gj != j0) {
            gj = HSD_JObjGetParent(gj);
        }
        if (gj == j0 && g->jobj != NULL && ngen < 32) {
            for (k = ngen; k > 0; k--) {
                lists[k] = lists[k - 1];
            }
            lists[0] = g->cmdList;
            ngen++;
        }
    }
    for (i = 0; i < 16; i++) {
        HSD_Particle* pp;
        for (pp = hsd_804D0908[i]; pp != NULL; pp = pp->next) {
            for (k = 0; k < ngen; k++) {
                if (pp->cmdList == lists[k]) {
                    per[k]++;
                    total++;
                    sdx += (pp->pos.x - ip->pos.x) * ip->facing_dir;
                    sdy += pp->pos.y - ip->pos.y;
                    svx += pp->vel.x * ip->facing_dir;
                    break;
                }
            }
        }
    }
    Geno_Event(38, v->article, v->frame, ngen, total);
    /* per group of 4 generators (one Ultimate emitter each, trail_vfx_melee.py v2) */
    Geno_Event(39, (per[0] + per[1] + per[2] + per[3]) << 8 | (per[4] + per[5] + per[6] + per[7]),
               (per[8] + per[9] + per[10] + per[11]) << 8 | (per[12] + per[13] + per[14] + per[15]),
               (per[16] + per[17] + per[18] + per[19]) << 8 | (per[20] + per[21] + per[22] + per[23]),
               (per[24] + per[25] + per[26] + per[27]) << 8 | (per[28] + per[29] + per[30] + per[31]));
    if (total > 0) {
        Geno_Event(41, (int) (100.0f * sdx / total), (int) (100.0f * sdy / total), (int) (1000.0f * svx / total),
                   total);
    }
}

static bool geno_art_anim(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    GenoArtVars* v = geno_art_vars(ip);
    f32 spin = geno_art_f(v->profile, v->article, GENO_AP_SPIN);
    v->frame++;
    if ((f32) v->frame > geno_art_f(v->profile, v->article, GENO_AP_LIFETIME)) {
        geno_art_gone(ip, v, 0);
        return true;
    }
    geno_art_hitboxes(gobj, ip, v);
    geno_art_children(gobj, ip, v);
    geno_art_effects(gobj, ip, v->profile, v->article, v->frame, ip->facing_dir);
    {
        /* "spins": joints of the model turning about their own Z (a billboarded joint: in the screen
           plane), angle = frame x speed - derived from the frame, so a rollback needs nothing */
        int k;
        for (k = 0; k < 4; k++) {
            f32 sp = geno_art_f(v->profile, v->article, GENO_AP_SPINS + 2 * k + 1);
            HSD_JObj* sj;
            if (sp == 0.0f) {
                continue;
            }
            sj = geno_art_joint(gobj, Geno_ArticleParam(v->profile, v->article, GENO_AP_SPINS + 2 * k));
            if (sj != NULL) {
                HSD_JObjSetRotationZ(sj, sp * (f32) v->frame);
            }
        }
    }
    if (v->frame == 5 || v->frame == 10 || v->frame == 20 || v->frame == 30) {
        geno_art_fx_census(gobj, v);
    }
    if (spin != 0.0f) {
        v->spin += spin * (f32) M_PI / 180.0f;
        HSD_JObjSetRotationX(GET_JOBJ(gobj), v->spin);
    }
    return false;
}

static void geno_art_phys(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    GenoArtVars* v = geno_art_vars(ip);
    f32 tx = 0.0f, ty = 0.0f;
    int has = geno_art_f(v->profile, v->article, GENO_AP_HOMING_TURN) > 0.0f && geno_art_target(ip, &tx, &ty);
    v->prev = ip->pos;
    GenoGame_ArticleStep(v->profile, v->article, v->frame, &v->vx, &v->vy, ip->pos.x, ip->pos.y, has,
                         tx, ty);
    ip->x40_vel.x = v->vx;
    ip->x40_vel.y = v->vy;
    ip->x40_vel.z = 0.0f;
}

static bool geno_art_coll(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    GenoArtVars* v = geno_art_vars(ip);
    if ((Geno_ArticleParam(v->profile, v->article, GENO_AP_DESPAWN) & GENO_ART_DESPAWN_STAGE) &&
        it_8026E9A4(gobj, &v->prev, &ip->pos, NULL))
    {
        Geno_Event(35, v->article, (int) (v->prev.x * 100.0f), (int) (v->prev.y * 100.0f),
                   (int) (ip->pos.y * 100.0f));
        geno_art_gone(ip, v, 3);
        return true; /* Item_80269978 destroys it */
    }
    return false;
}

static bool geno_art_react(Item_GObj* gobj, u32 bit, int why)
{
    Item* ip = GET_ITEM(gobj);
    GenoArtVars* v = geno_art_vars(ip);
    if (Geno_ArticleParam(v->profile, v->article, GENO_AP_DESPAWN) & bit) {
        geno_art_gone(ip, v, why);
        return true;
    }
    return false;
}

static bool geno_art_hit(Item_GObj* gobj)
{
    return geno_art_react(gobj, GENO_ART_DESPAWN_HIT, 1);
}

static bool geno_art_shield(Item_GObj* gobj)
{
    return geno_art_react(gobj, GENO_ART_DESPAWN_SHIELD, 2);
}

static bool geno_art_clank(Item_GObj* gobj)
{
    return geno_art_react(gobj, GENO_ART_DESPAWN_CLANK, 4);
}

/* Reflected: the engine already made the reflector the owner and stored the new facing in xC68 (as
 * for Fox's laser); the travel turns around with it. */
static bool geno_art_reflected(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    GenoArtVars* v = geno_art_vars(ip);
    if (ip->owner != NULL) Fx_SetOwnerPort((int) GET_JOBJ(gobj), GET_FIGHTER(ip->owner)->player_id + 1);
    if (ip->facing_dir != ip->xC68) {
        ip->facing_dir = ip->xC68;
        HSD_JObjSetRotationY(GET_JOBJ(gobj), (f32) (M_PI / 2) * ip->facing_dir);
    }
    /* Item_80269F14 leaves the speed multiplier to the projectile callback. Update the
       snapshotted travel velocity, otherwise the next phys callback would undo it. */
    v->vx = (v->vx < 0.0f ? -v->vx : v->vx) * ip->facing_dir * ip->xC70;
    v->vy *= ip->xC70;
    return false;
}

static bool geno_art_absorbed(Item_GObj* gobj)
{
    geno_art_gone(GET_ITEM(gobj), geno_art_vars(GET_ITEM(gobj)), 1);
    return true;
}

/* ---- spawning (hook 5 "geno.article.spawn", arg = article index) ------------------------------ */

/* The spawner. `base` = where the offsets apply from (the owner's position or bone, or a parent
 * article's position), `facing` the direction; the velocity is (velocity) turned up by the article's
 * "angle" + `angle` degrees. Returns the item, NULL when refused (logged). */
static Item_GObj* geno_art_spawn(HSD_GObj* owner, int p, int a, int variant, f32 angle, Vec3* base,
                                  f32 facing)
{
    Fighter* fp = GET_FIGHTER(owner);
    int kind, max, nv;
    SpawnItem spawn;
    Vec3 pos;
    Item_GObj* ig;
    Item* ip;
    GenoArtVars* v;
    f32 fwd, up, vf, vu, rad, c, sn;
    if (p < 0 || p >= GENO_MAX_PROFILES || a < 0 || a >= Geno_ArticleCount(p)) {
        Geno_Event(30, fp->kind, fp->player_id, a, 0);
        return NULL;
    }
    max = Geno_ArticleParam(p, a, GENO_AP_MAX_LIVE);
    if (max > 0 && geno_art_live(owner, a) >= max) {
        Geno_Event(30, fp->kind, fp->player_id, a, 1);
        return NULL;
    }
    geno_art_build(p, a);
    kind = GENO_ART_KIND(p, a);
    nv = Geno_ArticleParam(p, a, GENO_AP_SPAWN_N);
    if (variant > 0 && variant < nv) {
        fwd = geno_art_f(p, a, GENO_AP_SPAWN_V + 2 * variant);
        up = geno_art_f(p, a, GENO_AP_SPAWN_V + 2 * variant + 1);
    } else if (nv > 0) {
        fwd = geno_art_f(p, a, GENO_AP_SPAWN_V);
        up = geno_art_f(p, a, GENO_AP_SPAWN_V + 1);
    } else {
        fwd = geno_art_f(p, a, GENO_AP_SPAWN_FWD);
        up = geno_art_f(p, a, GENO_AP_SPAWN_UP);
    }
    pos.x = base->x + facing * fwd;
    pos.y = base->y + up;
    pos.z = 0.0f;
    spawn.kind = kind;
    Item_InitSpawnPositionFromParent(&spawn, owner, &pos);
    Item_InitSpawnCommonFields(&spawn, owner, facing, true);
    ig = Item_80268B18(&spawn);
    if (ig == NULL) {
        Geno_Event(30, fp->kind, fp->player_id, a, 2);
        return NULL;
    }
    ip = GET_ITEM(ig);
    v = geno_art_vars(ip);
    geno_zero(v, sizeof(*v));
    v->profile = p;
    v->article = a;
    for (nv = 0; nv < GENO_ART_HITBOXES; nv++) {
        v->cur[nv] = -1;
    }
    vf = geno_art_f(p, a, GENO_AP_VEL_FWD);
    vu = geno_art_f(p, a, GENO_AP_VEL_UP);
    rad = (geno_art_f(p, a, GENO_AP_ANGLE) + angle) * (f32) M_PI / 180.0f;
    c = cosf(rad);
    sn = sinf(rad);
    v->vx = facing * (vf * c - vu * sn);
    v->vy = vf * sn + vu * c;
    v->prev = ip->pos;
    Item_80268E5C(ig, 0, ITEM_ANIM_UPDATE);
    ip->x40_vel.x = v->vx;
    ip->x40_vel.y = v->vy;
    ip->x40_vel.z = 0.0f;
    db_80225DD8(ig, owner);
    geno_art_effects(ig, ip, p, a, 0, facing); /* the frame-0 effects; later ones from the anim callback */
    if (Geno_ArticleParam(p, a, GENO_AP_FX) > 0) {
        HSD_JObjSetTranslate(GET_JOBJ(ig), &ip->pos); /* the spawn position, before the effect reads the matrix */
        HSD_JObjSetupMatrix(GET_JOBJ(ig));
        {
            int h = Fx_Attach(Geno_ArticleParam(p, a, GENO_AP_FX) - 1, (int) GET_JOBJ(ig), 0x44,
                              geno_fx_frame, facing < 0.0f ? -1 : 1);
            if (h > 0) Fx_SetOwner(h, 2, fp->player_id + 1);
        }
        /* the package is the article's look: the model's display objects (a placeholder, e.g. trail_magic_models.py's
         * sphere) are hidden; the joints keep being set up, so the effects still follow the article */
        geno_art_hide_model(GET_JOBJ(ig));
    }
    geno_state(fp)->art_spawned++;
    Geno_Event(28, fp->kind, fp->player_id, a, kind);
    {
        /* what will be drawn: the joints and display objects the item built from the model */
        HSD_JObj* j = GET_JOBJ(ig);
        int nj = 0, nd = 0;
        while (j != NULL && nj < 64) {
            HSD_DObj* d = HSD_JObjGetDObj(j);
            nj++;
            for (; d != NULL; d = d->next) {
                nd++;
            }
            j = HSD_JObjGetChild(j) != NULL ? HSD_JObjGetChild(j) : HSD_JObjGetNext(j);
        }
        Geno_Event(36, a, nj, nd, (int) (ip->scl * 100.0f));
    }
    Geno_Event(34, a, (int) (pos.x * 100.0f), (int) (pos.y * 100.0f), (int) (v->vx * 1000.0f));
    return ig;
}

/* Hook 5: arg [7:0] article, [15:8] spawn variant, [31:16] signed extra angle (degrees). From the
 * fighter's position, or its part "bone" (world position) when the article names one. */
int GenoGame_ArticleSpawn(Fighter_GObj* gobj, int arg)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int a = arg & 0xFF, bone;
    Vec3 base = fp->cur_pos;
    if (st->profile < 0 || a >= Geno_ArticleCount(st->profile)) {
        Geno_Event(30, fp->kind, fp->player_id, a, 0);
        return 0;
    }
    bone = Geno_ArticleParam(st->profile, a, GENO_AP_BONE);
    if (bone >= 0 && bone < 128 && fp->parts != NULL && fp->parts[bone].joint != NULL) {
        lb_8000B1CC(fp->parts[bone].joint, NULL, &base);
        base.z = 0.0f;
    }
    return geno_art_spawn(gobj, st->profile, a, (arg >> 8) & 0xFF, (f32) (s16) ((u32) arg >> 16), &base,
                          fp->facing_dir) != NULL;
}

static int hook_article_spawn(Fighter_GObj* gobj, Fighter* fp, GenoState* st, s32 arg)
{
    if (geno_test_capture) {
        st->art_spawned++; /* the test fighter has no item heap: count the request */
        return 0;
    }
    GenoGame_ArticleSpawn(gobj, arg);
    return 0;
}

/* ---- on_hit and counter windows ------------------------------------------------------------------
 * Fighter_ProcessHit_8006D1EC (fighter.c), before Melee reacts to the hits this frame collected
 * (fp->dmg): a hit is "knockback to apply" or "damage to take". The context goes to the HIT_*
 * values first, so a counter target's entry and the on_hit hooks both see it. A hit in a counter
 * window (the fighter's current Geno state has a "counter" whose frames include this action frame)
 * with "negate" drops the knockback and the damage before Melee applies them - the hitlag both
 * sides already got stays, as with Marth's counter - and changes to the window's target. Returns 1
 * when the hit was countered. */
int Geno_OnHit(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int s, countered = 0;
    if (st->profile < 0 || (fp->dmg.kb_applied == 0.0f && fp->dmg.x1838_percentTemp <= 0.0f)) {
        return 0;
    }
    st->hit_count++;
    st->hit_damage = fp->dmg.x1838_percentTemp;
    st->hit_port = fp->dmg.x18c4_source_ply >= 0 && fp->dmg.x18c4_source_ply < 6
                       ? fp->dmg.x18c4_source_ply
                       : -1;
    st->hit_counter = 0;
    Geno_Event(31, fp->kind, fp->player_id, (int) (st->hit_damage * 100.0f), st->hit_port);
    s = geno_cur_state(fp, st);
    if (s >= 0 && Geno_StateHasCounter(st->profile, s) &&
        st->action_time >= Geno_StateCounter(st->profile, s, GENO_CTR_FROM) &&
        st->action_time <= Geno_StateCounter(st->profile, s, GENO_CTR_TO))
    {
        u32 t = (u32) Geno_StateCounter(st->profile, s, GENO_CTR_TARGET);
        countered = 1;
        st->hit_counter = 1;
        st->counters++;
        Geno_Event(32, fp->kind, fp->player_id, s, st->action_time);
        if (Geno_StateCounter(st->profile, s, GENO_CTR_NEGATE)) {
            fp->dmg.kb_applied = 0.0f;
            fp->dmg.x1838_percentTemp = 0.0f;
        }
        if (t != GENO_NONE_T) {
            geno_go(gobj, fp, st, t);
        }
    }
    geno_run_event(gobj, st, GENO_EV_HIT);
    return countered;
}
