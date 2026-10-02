/* Included in script_game.c: explicit offline inspection of the loaded model,
 * scalar bridge only. */
#include "script_parts.h"
#include <melee/ft/ftparts.h>
#include <sysdolphin/baselib/displayfunc.h>
#include <sysdolphin/baselib/pobj.h>
#define PART_FLOAT_MAX 3.402823466e38f
#define part_finite(x)                                                        \
    ((x) == (x) && (x) <= PART_FLOAT_MAX && (x) >= -PART_FLOAT_MAX)

#define PART_MAX 512
#define PART_COLORS 1024
typedef struct {
    HSD_DObj* dobj;
    HSD_JObj* joint;
    int owner_joint, material, material_data, triangles, status;
    int source, item_kind, item_ordinal, path;
    float area, lo[3], hi[3], region[PART_REGIONS];
} LabPart;
static LabPart lab_parts[PART_MAX];
static int lab_parts_n, lab_parts_slot = -1;
static int lab_part_colors_hi;
static u32 lab_parts_hash[2];
static struct {
    HSD_DObj* dobj;
    u32 rgb;
    int owner, mode;
} lab_part_colors[PART_COLORS];
static struct {
    int active, owner, inverted, count;
    HSD_DObj* dobj[PART_MAX];
} lab_part_ids;

static void part_hash_byte(u8 b)
{
    lab_parts_hash[0] = (lab_parts_hash[0] ^ b) * 16777619u;
    lab_parts_hash[1] = (lab_parts_hash[1] ^ b) * 2246822519u;
}
static void part_hash_int(u32 v)
{
    part_hash_byte((u8) (v >> 24));
    part_hash_byte((u8) (v >> 16));
    part_hash_byte((u8) (v >> 8));
    part_hash_byte((u8) v);
}
static void part_hash_float(float f)
{
    part_hash_int((u32) script_mesh_bits(f));
}

void ScriptGame_PartsReset(int owner)
{
    int i;
    for (i = 0; i < PART_COLORS; ++i) {
        if (owner < 0 || lab_part_colors[i].owner == owner) {
            memset(&lab_part_colors[i], 0, sizeof lab_part_colors[i]);
        }
    }
    while (lab_part_colors_hi > 0 &&
           lab_part_colors[lab_part_colors_hi - 1].dobj == NULL)
    {
        --lab_part_colors_hi;
    }
    if (owner < 0 || lab_part_ids.owner == owner) {
        memset(&lab_part_ids, 0, sizeof lab_part_ids);
    }
    if (owner < 0) {
        lab_parts_n = 0;
        lab_parts_slot = -1;
    }
}

void ScriptParts_Forget(HSD_DObj* dobj)
{
    int i;
    for (i = 0; i < PART_COLORS; ++i) {
        if (lab_part_colors[i].dobj == dobj) {
            lab_part_colors[i].dobj = NULL;
        }
    }
    for (i = 0; i < lab_part_ids.count; ++i) {
        if (lab_part_ids.dobj[i] == dobj) {
            lab_part_ids.dobj[i] = NULL;
        }
    }
    /* Inspection records are snapshots; retired parts can never be targeted.
     */
    for (i = 0; i < lab_parts_n; ++i) {
        if (lab_parts[i].dobj == dobj) {
            lab_parts[i].dobj = NULL;
        }
    }
}

/* Two palettes are captured at the SAME frozen pose. Inversion validates
 * pixels against HUD, particles and antialiasing. Colors are spaced so
 * boundary pixels can be rejected. */
static u32 part_id_rgb(int i, int inverted)
{
    u32 n = (u32) i + 1;
    u32 rgb = ((16u + (n & 7u) * 32u) << 16) |
              ((16u + ((n >> 3) & 7u) * 32u) << 8) |
              (16u + ((n >> 6) & 7u) * 32u);
    return inverted ? rgb ^ 0xFFFFFFu : rgb;
}

int ScriptParts_Draw(HSD_DObj* dobj)
{
    int i, active = 0;
    u32 rgb = 0;
    HSD_TevDesc tev;
    GXColor color;
    if (lab_part_ids.active) {
        active = 1;
        for (i = 0; i < lab_part_ids.count; ++i) {
            if (lab_part_ids.dobj[i] == dobj) {
                rgb = part_id_rgb(i, lab_part_ids.inverted);
                break;
            }
        }
        /* Isolate the target geometry: a stage wall must not hide it from a
         * side view. Static non-HSD backgrounds are rejected by the
         * complementary capture. */
        if (i == lab_part_ids.count) {
            return 2;
        }
    } else {
        for (i = 0; i < lab_part_colors_hi; ++i) {
            if (lab_part_colors[i].dobj == dobj) {
                if (lab_part_colors[i].mode == PART_CTL_TINT) return 0;
                rgb = lab_part_colors[i].rgb;
                active = 1;
                break;
            }
        }
    }
    if (!active) {
        return 0;
    }
    /* Exact constant output, independent of textures and lighting; normal
     * polygon transforms and depth testing still run. Alpha texture cutouts
     * are deliberately measured as opaque geometry, flagged in metadata. This
     * mode never mutates/recompiles the source MObj. */
    HSD_StateInitTev();
    memset(&tev, 0, sizeof tev);
    tev.flags = TEVCONF_MODE;
    tev.stage = HSD_StateAssignTev();
    tev.coord = GX_TEXCOORD_NULL;
    tev.map = GX_TEXMAP_NULL;
    tev.color = GX_COLOR_NULL;
    tev.u.tevconf.clr_op = GX_TEV_ADD;
    tev.u.tevconf.clr_a = tev.u.tevconf.clr_b = tev.u.tevconf.clr_c =
        GX_CC_ZERO;
    tev.u.tevconf.clr_d = GX_CC_C0;
    tev.u.tevconf.clr_clamp = GX_ENABLE;
    tev.u.tevconf.clr_out_reg = GX_TEVPREV;
    tev.u.tevconf.alpha_op = GX_TEV_ADD;
    tev.u.tevconf.alpha_a = tev.u.tevconf.alpha_b = tev.u.tevconf.alpha_c =
        GX_CA_ZERO;
    tev.u.tevconf.alpha_d = GX_CA_A0;
    tev.u.tevconf.alpha_clamp = GX_ENABLE;
    tev.u.tevconf.alpha_out_reg = GX_TEVPREV;
    HSD_SetupTevStage(&tev);
    HSD_SetupRenderMode(RENDER_CONSTANT);
    color.r = (u8) (rgb >> 16);
    color.g = (u8) (rgb >> 8);
    color.b = (u8) rgb;
    color.a = 255;
    GXSetTevColor(GX_TEVREG0, color);
    return 1;
}

/* Append a final RGB modulation stage to the normal material. Alpha and its
 * cutout test remain untouched. An unused konst register avoids disturbing
 * earlier compiled texture/lighting stages; unsupported materials stay original. */
static int part_tint_register(HSD_DObj* dobj)
{
    HSD_TevDesc* d;
    unsigned used = 0;
    int stages = 0, k;
    if (dobj == NULL || dobj->mobj == NULL || dobj->mobj->tevdesc == NULL) return -1;
    for (d = &dobj->mobj->tevdesc->desc; d != NULL; d = d->next) {
        int c, a;
        if (++stages >= 16) return -1;
        if (!(d->flags & TEVCONF_MODE)) continue;
        c = d->u.tevconf.kcsel;
        a = d->u.tevconf.kasel;
        if (c >= 12 && c <= 15) used |= 1u << (c - 12);
        else if (c >= 16 && c <= 31) used |= 1u << ((c - 16) % 4);
        if (a >= 16 && a <= 31) used |= 1u << ((a - 16) % 4);
    }
    for (k = 0; k < 4; ++k) if (!(used & (1u << k))) return k;
    return -1;
}

int ScriptParts_Tint(HSD_DObj* dobj)
{
    int i, reg, stage;
    GXColor color;
    u32 rgb;
    if (lab_part_ids.active) return 0;
    for (i = 0; i < lab_part_colors_hi; ++i)
        if (lab_part_colors[i].dobj == dobj && lab_part_colors[i].mode == PART_CTL_TINT) break;
    /* MObj setup already committed the material and reset the allocation
     * counter to zero. Appending at that counter would overwrite stage 0 and
     * multiply an uninitialized PREV, turning selected surfaces black. */
    stage = HSD_StateGetCommittedTevStages();
    if (i == lab_part_colors_hi || stage < 1 || stage >= 16) return 0;
    reg = part_tint_register(dobj);
    if (reg < 0) return 0;
    rgb = lab_part_colors[i].rgb;
    color.r = (u8)(rgb >> 16); color.g = (u8)(rgb >> 8); color.b = (u8)rgb; color.a = 255;
    GXSetTevKColor(reg, color);
    GXSetTevOrder(stage, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR_NULL);
    GXSetTevKColorSel(stage, GX_TEV_KCSEL_K0 + reg);
    GXSetTevColorIn(stage, GX_CC_ZERO, GX_CC_CPREV, GX_CC_KONST, GX_CC_ZERO);
    GXSetTevColorOp(stage, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_ENABLE, GX_TEVPREV);
    GXSetTevAlphaIn(stage, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_APREV);
    GXSetTevAlphaOp(stage, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, GX_ENABLE, GX_TEVPREV);
    GXSetTevSwapMode(stage, GX_TEV_SWAP0, GX_TEV_SWAP0);
    GXSetNumTevStages(stage + 1);
    return 1;
}

static int part_joint_region(Fighter* fp, HSD_JObj* joint)
{
    static const int role[] = {
        FtPart_HeadN,  FtPart_NeckN,      FtPart_LHandN, FtPart_RHandN,
        FtPart_LFootJ, FtPart_RFootJ,     FtPart_LArmJ,  FtPart_LShoulderJ,
        FtPart_RArmJ,  FtPart_RShoulderJ, FtPart_LLegJ,  FtPart_LKneeJ,
        FtPart_RLegJ,  FtPart_RKneeJ,     FtPart_BustN,  FtPart_WaistN,
        FtPart_HipN
    };
    static const int region[] = { 1, 1, 5, 6, 9, 10, 3, 3, 4,
                                  4, 7, 7, 8, 8, 2,  2, 2 };
    int k, depth = 0, count = lab_joint_count(fp);
    FighterPartsTable* table;
    if (ftPartsTable == NULL || fp->kind < 0 || fp->kind >= 256 ||
        (table = ftPartsTable[fp->kind]) == NULL ||
        table->part_to_joint == NULL)
    {
        return 0;
    }
    while (joint != NULL && depth++ < MAX_FT_PARTS) {
        int ji = lab_joint_index(fp, joint);
        for (k = 0; k < (int) (sizeof role / sizeof role[0]); ++k) {
            int mapped = table->part_to_joint[role[k]];
            if (mapped != FTPART_INVALID && mapped < count && mapped == ji) {
                return region[k];
            }
        }
        joint = joint->parent;
    }
    return 0;
}

typedef struct {
    float pos[3], region[PART_REGIONS];
} PartVertex;
typedef struct {
    Mtx m;
    float region[PART_REGIONS];
} PartEnvelope;
static void part_transform(Mtx m, const float* v, float* out)
{
    int k;
    for (k = 0; k < 3; ++k) {
        out[k] = m[k][0] * v[0] + m[k][1] * v[1] + m[k][2] * v[2] + m[k][3];
    }
}

static int part_envelopes(Fighter* fp, LabPart* part, HSD_PObj* p,
                          PartEnvelope* out)
{
    HSD_SList* list;
    int n = 0, k, r, c;
    Mtx node, tmp, composed;
    MtxPtr right = _HSD_mkEnvelopeModelNodeMtx(part->joint, node);
    for (list = p->u.envelope_list; list != NULL && n < 64;
         list = list->next, ++n)
    {
        HSD_Envelope* e = list->data;
        float total = 0;
        int guard = 0;
        memset(&out[n], 0, sizeof out[n]);
        if (e == NULL) {
            return -1;
        }
        for (; e != NULL && guard++ < MAX_FT_PARTS; e = e->next) {
            MtxPtr m;
            float weight = e->weight;
            if (e->jobj == NULL || !part_finite(weight) || weight < 0) {
                return -1;
            }
            HSD_JObjSetupMatrix(e->jobj);
            m = e->jobj->mtx;
            if (list->data != e || e->next != NULL || weight < 1.0f ||
                right != NULL)
            {
                if (e->jobj->envelopemtx == NULL) {
                    return -1;
                }
                PSMTXConcat(m, e->jobj->envelopemtx, tmp);
                m = tmp;
            }
            for (r = 0; r < 3; ++r) {
                for (c = 0; c < 4; ++c) {
                    out[n].m[r][c] += weight * m[r][c];
                }
            }
            k = part->source ? 11 : part_joint_region(fp, e->jobj);
            part_hash_int((u32) lab_joint_index(fp, e->jobj));
            part_hash_float(weight);
            out[n].region[k] += weight;
            total += weight;
        }
        if (total < 0.00001f || guard >= MAX_FT_PARTS) {
            return -1;
        }
        for (k = 0; k < PART_REGIONS; ++k) {
            out[n].region[k] /= total;
        }
        if (right != NULL) {
            PSMTXConcat(out[n].m, right, composed);
            PSMTXCopy(composed, out[n].m);
        }
    }
    return list != NULL ? -1 : n;
}

static int part_component_bytes(int type)
{
    return type == GX_U8 || type == GX_S8     ? 1
           : type == GX_U16 || type == GX_S16 ? 2
           : type == GX_F32                   ? 4
                                              : 0;
}
static int part_attr_bytes(HSD_VtxDescList* v)
{
    if (v->attr_type == GX_NONE) {
        return 0;
    }
    if (v->attr_type == GX_INDEX8 || v->attr_type == GX_INDEX16) {
        int size = v->attr_type == GX_INDEX8 ? 1 : 2;
        return size * ((v->attr == GX_VA_NRM || v->attr == GX_VA_NBT) &&
                               v->comp_cnt == GX_NRM_NBT3
                           ? 3
                           : 1);
    }
    if (v->attr_type != GX_DIRECT) {
        return -1;
    }
    if (v->attr <= GX_VA_TEX7MTXIDX) {
        return 1;
    }
    if (v->attr == GX_VA_CLR0 || v->attr == GX_VA_CLR1) {
        static const int sizes[] = { 2, 3, 4, 2, 3, 4 };
        return (int) v->comp_type < 6 ? sizes[v->comp_type] : -1;
    }
    if (v->attr == GX_VA_POS) {
        return part_component_bytes(v->comp_type) *
               (v->comp_cnt == GX_POS_XY ? 2 : 3);
    }
    if (v->attr == GX_VA_NRM || v->attr == GX_VA_NBT) {
        return part_component_bytes(v->comp_type) *
               (v->comp_cnt == GX_NRM_XYZ ? 3 : 9);
    }
    if (v->attr >= GX_VA_TEX0 && v->attr <= GX_VA_TEX7) {
        return part_component_bytes(v->comp_type) *
               (v->comp_cnt == GX_TEX_S ? 1 : 2);
    }
    return -1;
}
static float part_component(const u8* p, int type, int frac)
{
    int val;
    if (type == GX_F32) {
        union {
            u32 u;
            float f;
        } x;
        x.u =
            ((u32) p[0] << 24) | ((u32) p[1] << 16) | ((u32) p[2] << 8) | p[3];
        return x.f;
    }
    val = type == GX_U8   ? p[0]
          : type == GX_S8 ? (signed char) p[0]
                          : (p[0] << 8) | p[1];
    if (type == GX_S16) {
        val = (short) val;
    }
    return (float) val / (float) (1u << frac);
}

static int part_vertex(Fighter* fp, LabPart* part, HSD_PObj* p, const u8* data,
                       PartEnvelope* env, int ne, int palette, PartVertex* out)
{
    HSD_VtxDescList* v;
    float local[3] = { 0, 0, 0 };
    int found = 0, matrix = 0, attributes = 0, k;
    memset(out, 0, sizeof *out);
    for (v = p->verts; v->attr != GX_VA_NULL && attributes++ < 32; ++v) {
        int bytes = part_attr_bytes(v);
        if (bytes < 0) {
            return 0;
        }
        if (v->attr == GX_VA_PNMTXIDX && v->attr_type == GX_DIRECT) {
            matrix = data[0];
        }
        if (v->attr == GX_VA_POS && bytes > 0) {
            const u8* src = data;
            int count = v->comp_cnt == GX_POS_XY ? 2 : 3,
                width = part_component_bytes(v->comp_type);
            if (width == 0 || v->frac > 30) {
                return 0;
            }
            if (v->attr_type == GX_INDEX8 || v->attr_type == GX_INDEX16) {
                int index = v->attr_type == GX_INDEX8
                                ? data[0]
                                : (data[0] << 8) | data[1];
                if (v->vertex == NULL || v->stride < count * width) {
                    return 0;
                }
                src = (u8*) v->vertex + index * v->stride;
            }
            for (k = 0; k < count; ++k) {
                local[k] =
                    part_component(src + k * width, v->comp_type, v->frac);
                part_hash_float(local[k]);
            }
            found = 1;
        }
        data += bytes;
    }
    if (!found || attributes >= 32) {
        return 0;
    }
    part_hash_int((u32) matrix);
    if (pobj_type(p) == POBJ_ENVELOPE) {
        int slot = palette ? matrix : matrix / 3;
        if ((!palette && matrix % 3 != 0) || slot >= ne || slot < 0) {
            return 0;
        }
        part_transform(env[slot].m, local, out->pos);
        memcpy(out->region, env[slot].region, sizeof out->region);
    } else {
        HSD_JObj* joint = part->joint;
        if (p->u.jobj != NULL && matrix == 3) {
            joint = p->u.jobj;
        } else if (matrix != 0) {
            return 0;
        }
        if (joint == NULL) {
            return 0;
        }
        HSD_JObjSetupMatrix(joint);
        part_transform(joint->mtx, local, out->pos);
        out->region[part->source ? 11 : part_joint_region(fp, joint)] = 1;
    }
    return part_finite(out->pos[0]) && part_finite(out->pos[1]) &&
           part_finite(out->pos[2]);
}

static void part_triangle(LabPart* part, PartVertex* a, PartVertex* b,
                          PartVertex* c)
{
    float u[3], v[3], cross[3], area;
    int k;
    for (k = 0; k < 3; ++k) {
        u[k] = b->pos[k] - a->pos[k];
        v[k] = c->pos[k] - a->pos[k];
    }
    cross[0] = u[1] * v[2] - u[2] * v[1];
    cross[1] = u[2] * v[0] - u[0] * v[2];
    cross[2] = u[0] * v[1] - u[1] * v[0];
    area = 0.5f * sqrtf(cross[0] * cross[0] + cross[1] * cross[1] +
                        cross[2] * cross[2]);
    if (!part_finite(area) || area < 0.0000001f) {
        return;
    }
    ++part->triangles;
    part->area += area;
    for (k = 0; k < PART_REGIONS; ++k) {
        part->region[k] +=
            area * (a->region[k] + b->region[k] + c->region[k]) / 3;
    }
}

static void part_geometry(Fighter* fp, LabPart* part)
{
    HSD_PObj* p;
    int polys = 0, k;
    for (k = 0; k < 3; ++k) {
        part->lo[k] = PART_FLOAT_MAX;
        part->hi[k] = -PART_FLOAT_MAX;
    }
    for (p = part->dobj->pobj; p != NULL && polys++ < 4096; p = p->next) {
        HSD_VtxDescList* v;
        const u8 *at, *end;
        int stride = 0, attrs = 0, ne = 0, palette = 0;
        PartEnvelope envelopes[64];
        if (pobj_type(p) == POBJ_SHAPEANIM) {
            part->status |= PART_STATUS_SHAPE;
            continue;
        }
        if (p->verts == NULL || p->display == NULL) {
            part->status |= PART_STATUS_DECODE;
            continue;
        }
        if (HSD_CLASS_METHOD(p)->head.class_name != NULL &&
            strcmp(HSD_CLASS_METHOD(p)->head.class_name, "geno_pal_pobj_v1") ==
                0)
        {
            palette = 1;
        }
        if (palette) {
            part->status |= PART_STATUS_PALETTE;
        }
        for (v = p->verts; v->attr != GX_VA_NULL && attrs++ < 32; ++v) {
            int bytes = part_attr_bytes(v);
            if (bytes < 0) {
                break;
            }
            stride += bytes;
        }
        if (v->attr != GX_VA_NULL || stride <= 0 || attrs >= 32) {
            part->status |= PART_STATUS_DECODE;
            continue;
        }
        if (pobj_type(p) == POBJ_ENVELOPE) {
            ne = part_envelopes(fp, part, p, envelopes);
            if (ne <= 0) {
                part->status |= PART_STATUS_DECODE;
                continue;
            }
        }
        at = p->display;
        end = at + ((u32) p->n_display << 5);
        while (at + 3 <= end && *at != 0) {
            int op = *at & 0xF8, n = (at[1] << 8) | at[2], i;
            PartVertex first, previous, previous2;
            at += 3;
            if (n <= 0 || (u32) (end - at) < (u32) n * (u32) stride ||
                (op != GX_TRIANGLES && op != GX_TRIANGLESTRIP &&
                 op != GX_TRIANGLEFAN && op != GX_QUADS))
            {
                part->status |= PART_STATUS_DECODE;
                break;
            }
            memset(&first, 0, sizeof first);
            memset(&previous, 0, sizeof previous);
            memset(&previous2, 0, sizeof previous2);
            for (i = 0; i < n; ++i, at += stride) {
                PartVertex current;
                if (!part_vertex(fp, part, p, at, envelopes, ne, palette,
                                 &current))
                {
                    part->status |= PART_STATUS_DECODE;
                    break;
                }
                for (k = 0; k < 3; ++k) {
                    if (current.pos[k] < part->lo[k]) {
                        part->lo[k] = current.pos[k];
                    }
                    if (current.pos[k] > part->hi[k]) {
                        part->hi[k] = current.pos[k];
                    }
                }
                if (i == 0 || (op == GX_QUADS && i % 4 == 0)) {
                    first = current;
                }
                if (op == GX_TRIANGLES && i % 3 == 2) {
                    part_triangle(part, &previous2, &previous, &current);
                }
                if (op == GX_TRIANGLESTRIP && i >= 2) {
                    part_triangle(part, &previous2, &previous, &current);
                }
                if (op == GX_TRIANGLEFAN && i >= 2) {
                    part_triangle(part, &first, &previous, &current);
                }
                if (op == GX_QUADS && (i % 4 == 2 || i % 4 == 3)) {
                    part_triangle(part, &first, &previous, &current);
                }
                previous2 = previous;
                previous = current;
            }
            if (i < n) {
                break;
            }
        }
    }
    if (polys >= 4096) {
        part->status |= PART_STATUS_DECODE;
    }
    for (k = 0; k < 3; ++k) {
        if (part->lo[k] == PART_FLOAT_MAX) {
            part->lo[k] = part->hi[k] = 0;
        }
    }
    if (part->area > 0) {
        for (k = 0; k < PART_REGIONS; ++k) {
            part->region[k] /= part->area;
        }
    }
    if (part->region[0] > 0.001f) {
        part->status |= PART_STATUS_MAPPING;
    }
}

static void part_add(Fighter* fp, HSD_DObj* d, HSD_JObj* joint, int source,
                     int kind, int ordinal, int path)
{
    LabPart* part;
    int i;
    if (lab_parts_n >= PART_MAX || d == NULL || joint == NULL) {
        return;
    }
    part = &lab_parts[lab_parts_n];
    memset(part, 0, sizeof *part);
    part->dobj = d;
    part->joint = joint;
    part->owner_joint = source ? -1 : lab_joint_index(fp, joint);
    part->source = source;
    part->item_kind = kind;
    part->item_ordinal = ordinal;
    part->path = path;
    part->material = part->material_data = lab_parts_n;
    for (i = 0; i < lab_parts_n; ++i) {
        if (d->mobj != NULL && d->mobj == lab_parts[i].dobj->mobj) {
            part->material = lab_parts[i].material;
        }
        if (d->mobj != NULL && lab_parts[i].dobj->mobj != NULL &&
            d->mobj->mat == lab_parts[i].dobj->mobj->mat)
        {
            part->material_data = lab_parts[i].material_data;
        }
    }
    if (d->mobj != NULL) {
        /* Render/texture flags include dirty/cache bits changed by a draw or
         * animation. Asset hashes identify authored materials; this signature
         * identifies geometry. */
        if ((d->mobj->rendermode & RENDER_XLU) ||
            (d->mobj->mat && d->mobj->mat->alpha < 1.0f))
        {
            part->status |= PART_STATUS_ALPHA;
        }
    }
    part_hash_int((u32) source);
    part_hash_int((u32) part->owner_joint);
    part_geometry(fp, part);
    ++lab_parts_n;
}

static void part_item_walk(Fighter* fp, HSD_JObj* j, int kind, int ordinal,
                           int* path, int depth)
{
    for (; j != NULL && depth < MAX_FT_PARTS; j = j->next) {
        HSD_DObj* d;
        int ji = (*path)++, di = 0;
        if (union_type_dobj(j)) {
            for (d = j->u.dobj; d != NULL && di < 124; d = d->next, ++di) {
                part_add(fp, d, j, 1, kind, ordinal, ji * 128 + di);
            }
        }
        if (j->child != NULL) {
            part_item_walk(fp, j->child, kind, ordinal, path, depth + 1);
        }
    }
}

static int part_refresh(int slot)
{
    Fighter* fp = script_fighter(slot);
    int d, j, count;
    HSD_GObj* g;
    lab_parts_n = 0;
    lab_parts_slot = slot;
    lab_parts_hash[0] = 2166136261u;
    lab_parts_hash[1] = 0x9E3779B9u;
    if (fp == NULL) {
        return 0;
    }
    count = lab_joint_count(fp);
    ScriptGame_JointsSetup(slot);
    for (d = 0; d < (int) fp->dobj_list.count; ++d) {
        HSD_DObj* dobj = lab_dobj(fp, d);
        HSD_JObj* owner = NULL;
        for (j = 0; j < count && owner == NULL; ++j) {
            HSD_JObj* joint = fp->parts[j].joint;
            HSD_DObj* cur;
            if (joint == NULL || !union_type_dobj(joint)) {
                continue;
            }
            for (cur = joint->u.dobj; cur != NULL; cur = cur->next) {
                if (cur == dobj) {
                    owner = joint;
                    break;
                }
            }
        }
        if (owner == NULL) {
            lab_parts_slot = -1;
            return -1;
        } /* never silently renumber the fighter's list */
        part_add(fp, dobj, owner, 0, -1, -1, d);
    }
    {
        u32 core_hash[2];
        core_hash[0] = lab_parts_hash[0];
        core_hash[1] = lab_parts_hash[1];
        if (HSD_GObjPLinkHead != NULL) {
            int ordinal = 0;
            for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; g != NULL;
                 g = g->next)
            {
                Item* item = GET_ITEM(g);
                if (item != NULL && item->owner == Player_GetEntity(slot)) {
                    int path = 0;
                    part_item_walk(fp, GET_JOBJ(g), item->kind, ordinal++,
                                   &path, 0);
                }
            }
        }
        lab_parts_hash[0] = core_hash[0];
        lab_parts_hash[1] = core_hash[1];
    }
    return lab_parts_n;
}

int ScriptGame_PartsI(int slot, int d, int field)
{
    LabPart* part;
    Fighter* fp = script_fighter(slot);
    if (field == PART_I_REFRESH) {
        return part_refresh(slot);
    }
    if (lab_parts_slot != slot || fp == NULL) {
        return -1;
    }
    if (field == PART_I_COUNT) {
        return lab_parts_n;
    }
    if (field == PART_I_HASH_LO) {
        return (int) lab_parts_hash[0];
    }
    if (field == PART_I_HASH_HI) {
        return (int) lab_parts_hash[1];
    }
    if (field == PART_I_MODELS) {
        return fp->x5AC.model_num;
    }
    if (field == PART_I_VIS_MODEL || field == PART_I_VIS_STATE) {
        int category, model, state, found = -1, mask = 0;
        for (category = 0; category < 5; ++category) {
            FtPartsVisLookup* lookup = fp->x5AC.xC[category];
            if (lookup == NULL || category == 2) {
                continue; /* category 2 indexes x203C, not this draw list */
            }
            for (model = 0; model < (int) fp->x5AC.model_num && model < 12;
                 ++model)
            {
                for (state = 0; state < lookup[model].x0; ++state) {
                    TempS* list = &lookup[model].x4[state];
                    int k;
                    for (k = 0; k < list->x0; ++k) {
                        if (list->x4[k] == d) {
                            if (found < 0) {
                                found = model;
                            }
                            if (model != found) {
                                return -2; /* multiple visibility models: no
                                              exclusivity claim */
                            }
                            if (state >= 31) {
                                return -2;
                            }
                            mask |= 1 << state;
                        }
                    }
                }
            }
        }
        return found < 0 ? -1 : field == PART_I_VIS_MODEL ? found : mask;
    }
    if (d < 0 || d >= lab_parts_n || lab_parts[d].dobj == NULL) {
        return -1;
    }
    part = &lab_parts[d];
    switch (field) {
    case PART_I_OWNER_JOINT:
        return part->owner_joint;
    case PART_I_MATERIAL:
        return part->material;
    case PART_I_MATERIAL_DATA:
        return part->material_data;
    case PART_I_TRIANGLES:
        return part->triangles;
    case PART_I_STATUS:
        return part->status;
    case PART_I_HIDDEN:
        return (part->dobj->flags & DOBJ_HIDDEN) != 0;
    case PART_I_SOURCE:
        return part->source;
    case PART_I_ITEM_KIND:
        return part->item_kind;
    case PART_I_ITEM_ORDINAL:
        return part->item_ordinal;
    case PART_I_PATH:
        return part->path;
    case PART_I_FLAGS:
        return part->joint->flags;
    case PART_I_BODY_JOINT: {
        int i;
        if (part->source || ftPartsTable == NULL ||
            ftPartsTable[fp->kind] == NULL ||
            ftPartsTable[fp->kind]->part_to_joint == NULL)
        {
            return 0;
        }
        for (i = 0; i <= FtPart_TransN2; ++i) {
            if (ftPartsTable[fp->kind]->part_to_joint[i] != FTPART_INVALID &&
                ftPartsTable[fp->kind]->part_to_joint[i] == part->owner_joint)
            {
                return 1;
            }
        }
        return 0;
    }
    }
    return -1;
}
float ScriptGame_PartsF(int slot, int d, int field)
{
    LabPart* part;
    if (lab_parts_slot != slot || d < 0 || d >= lab_parts_n ||
        lab_parts[d].dobj == NULL)
    {
        return 0;
    }
    part = &lab_parts[d];
    if (field == PART_F_AREA) {
        return part->area;
    }
    if (field >= PART_F_MIN_X && field <= PART_F_MIN_Z) {
        return part->lo[field - PART_F_MIN_X];
    }
    if (field >= PART_F_MAX_X && field <= PART_F_MAX_Z) {
        return part->hi[field - PART_F_MAX_X];
    }
    if (field >= PART_F_REGION && field < PART_F_REGION + PART_REGIONS) {
        return part->region[field - PART_F_REGION];
    }
    return 0;
}
int ScriptGame_PartsControl(int slot, int d, int what, int rgb, int owner)
{
    HSD_DObj* dobj;
    int i, empty = -1;
    if (what == PART_CTL_IDS_OFF) {
        if (lab_part_ids.active && lab_part_ids.owner != owner) {
            return -3;
        }
        memset(&lab_part_ids, 0, sizeof lab_part_ids);
        return 1;
    }
    if (what == PART_CTL_IDS) {
        if (lab_part_ids.active && lab_part_ids.owner != owner) {
            return -3;
        }
        if (lab_parts_slot != slot || script_fighter(slot) == NULL) {
            return -1;
        }
        lab_part_ids.active = 1;
        lab_part_ids.owner = owner;
        lab_part_ids.inverted = rgb != 0;
        lab_part_ids.count = lab_parts_n;
        for (i = 0; i < lab_parts_n; ++i) {
            lab_part_ids.dobj[i] = lab_parts[i].dobj;
        }
        return 1;
    }
    if (lab_parts_slot == slot && d >= 0 && d < lab_parts_n) {
        dobj = lab_parts[d].dobj;
        /* A valid snapshot entry may have been destroyed since inspection.
         * Report unavailable without disabling the script or retargeting a
         * different draw that happens to reuse this index. */
        if (dobj == NULL) {
            return 0;
        }
    } else {
        dobj = lab_dobj(script_fighter(slot), d);
    }
    if (dobj == NULL) {
        return -1;
    }
    for (i = 0; i < PART_COLORS; ++i) {
        if (lab_part_colors[i].dobj == dobj) {
            break;
        }
        if (empty < 0 && lab_part_colors[i].dobj == NULL) {
            empty = i;
        }
    }
    if (i < PART_COLORS && lab_part_colors[i].owner != owner) {
        return -3;
    }
    if (what == PART_CTL_CLEAR) {
        if (i < PART_COLORS) {
            memset(&lab_part_colors[i], 0, sizeof lab_part_colors[i]);
        }
        return 1;
    }
    if (what != PART_CTL_COLOR && what != PART_CTL_TINT) {
        return -1;
    }
    if (what == PART_CTL_TINT && part_tint_register(dobj) < 0) return 0;
    if (i == PART_COLORS) {
        i = empty;
    }
    if (i < 0) {
        return -2;
    }
    lab_part_colors[i].dobj = dobj;
    lab_part_colors[i].rgb = (u32) rgb & 0xFFFFFFu;
    lab_part_colors[i].owner = owner;
    lab_part_colors[i].mode = what;
    if (i >= lab_part_colors_hi) {
        lab_part_colors_hi = i + 1;
    }
    return 1;
}

/* Address stays inside native engine code; Lua only receives success/failure
 * for FX attachment. */
int ScriptGame_PartsJoint(int slot, int joint)
{
    Fighter* fp = script_fighter(slot);
    if (fp == NULL || joint < 0 || joint >= lab_joint_count(fp)) {
        return 0;
    }
    return (int) fp->parts[joint].joint;
}
