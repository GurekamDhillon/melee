/* Shared by gen/kinds/tu/<stem>.c (included after mtu_common.h): deep traces of Vec3* / SpawnItem* arguments.
 * Mirrored by tvec / tspawn in tests/kinds_harness/mod.rs. */
static void TVEC(const Vec3* v)
{
    if (v) { T(B(v->x)); T(B(v->y)); T(B(v->z)); }
}
/* the item attribute views upstream dropped (the port still reads them): the one generator's legacy declarations */
#define PORT_LEGACY_ITEMS_ONLY
#include "../../../../melee_types/gen/port_legacy.h"
#define TFL(f) do { T((f).x0.b0); T((f).x0.b1); T((f).x0.b2); T((f).x0.b3); T((f).x0.b4); T((f).x0.b5); T((f).x0.b6); T((f).x0.b7); } while (0)  /* UnkFlagStruct.x0 since upstream named the bits' struct */
static void TSPAWN(const SpawnItem* s)
{
    if (!s) return;
    T(PI(s->x0_parent_gobj)); T(PI(s->x4_parent_gobj2)); T((u32) s->kind); T((u32) s->hold_kind); T((u32) s->x10);
    TVEC(&s->pos); TVEC(&s->prev_pos); TVEC(&s->vel);
    T(B(s->facing_dir)); T((u32) s->x3C_damage); T((u32) s->x3E); T((u32) s->x40);
    TFL(s->x44_flag); TFL(s->x45_flag); TFL(s->x46_flag); TFL(s->x47_flag);
    T((u32) s->x48_ground_or_air);
}

static void TECB(const itECB* e)
{
    if (e) { T(B(e->top)); T(B(e->bottom)); T(B(e->right)); T(B(e->left)); }
}
