/* Shared by gen/kinds/tu/<stem>.c (included after mtu_common.h): deep traces of Vec3* / SpawnItem* arguments.
 * Mirrored by tvec / tspawn in tests/kinds_harness/mod.rs. */
static void TVEC(const Vec3* v)
{
    if (v) { T(B(v->x)); T(B(v->y)); T(B(v->z)); }
}
#define TFL(f) do { T((f).b0); T((f).b1); T((f).b2); T((f).b3); T((f).b4); T((f).b5); T((f).b6); T((f).b7); } while (0)
static void TSPAWN(const SpawnItem* s)
{
    if (!s) return;
    T(PI(s->x0_parent_gobj)); T(PI(s->x4_parent_gobj2)); T((u32) s->kind); T((u32) s->hold_kind); T((u32) s->x10);
    TVEC(&s->pos); TVEC(&s->prev_pos); TVEC(&s->vel);
    T(B(s->facing_dir)); T((u32) s->x3C_damage); T((u32) s->x3E); T((u32) s->x40);
    TFL(s->x44_flag); TFL(s->x45_flag); TFL(s->x46_flag); TFL(s->x47_flag);
    T((u32) s->x48_ground_or_air);
}
