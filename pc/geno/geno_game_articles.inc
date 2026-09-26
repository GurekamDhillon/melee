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

Article Geno_ArtDesc[GENO_MAX_PROFILES][GENO_MAX_ARTICLES];
ItemAttr Geno_ArtAttr[GENO_MAX_PROFILES][GENO_MAX_ARTICLES];
ItemModelDesc Geno_ArtModel[GENO_MAX_PROFILES][GENO_MAX_ARTICLES];
ItemStateArray Geno_ArtStates; /* never read: the one state has no animation (anim_id -1) */

/* The per-item variables, in the item's per-kind block (Item.xDD4_itemVar). */
typedef struct GenoArtVars {
    s32 profile;
    s32 article;
    s32 frame;  /* frames alive (1 on the first) */
    s32 why;    /* the despawn reason (event 29) */
    f32 vx, vy; /* the travel velocity the phys callback writes to x40_vel */
    Vec3 prev;  /* the position before this frame's move (the stage ray test) */
    f32 spin;   /* the model's roll (visual) */
    s8 cur[GENO_ART_HITBOXES]; /* v5.1: the hitbox entry each Melee slot holds now, -1 off */
} GenoArtVars;

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
    if (kind < GENO_ART_KIND_BASE || kind >= GENO_ART_KIND_END) {
        return 0;
    }
    if (kind < GENO_ART_KIND_BASE2) {
        *p = (kind - GENO_ART_KIND_BASE) / GENO_ART_PER_RANGE;
        *a = (kind - GENO_ART_KIND_BASE) % GENO_ART_PER_RANGE;
    } else {
        *p = (kind - GENO_ART_KIND_BASE2) / GENO_ART_PER_RANGE;
        *a = GENO_ART_PER_RANGE + (kind - GENO_ART_KIND_BASE2) % GENO_ART_PER_RANGE;
    }
    return *a < Geno_ArticleCount(*p);
}

/* The descriptor, from the registry (idempotent: the same values at every spawn). */
static void geno_art_build(int p, int a)
{
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

static ItemLogicTable geno_art_logic = {
    geno_art_states,
    NULL,               /* spawned */
    NULL,               /* destroyed */
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
    if (ip->facing_dir != ip->xC68) {
        ip->facing_dir = ip->xC68;
        HSD_JObjSetRotationY(GET_JOBJ(gobj), (f32) (M_PI / 2) * ip->facing_dir);
    }
    v->vx = (v->vx < 0.0f ? -v->vx : v->vx) * ip->facing_dir;
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
    if (Geno_ArticleParam(p, a, GENO_AP_EFFECT) > 0) {
        /* attached to the article's root joint, as Mario's fireball carries its flame */
        efAsync_Spawn(ig, &ip->xBC0, 1, (u32) Geno_ArticleParam(p, a, GENO_AP_EFFECT), GET_JOBJ(ig));
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
