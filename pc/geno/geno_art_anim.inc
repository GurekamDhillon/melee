/* One key value of an FObj stream (fobj.c's own reader, restated): the format is the top 3 bits
 * of `frac`, the fixed-point shift the low 5. */
static f32 genoArt_FObjValue(u8** p, u8 frac)
{
    u8* s = *p;
    f32 scale = (f32) (1 << (frac & 0x1F));
    switch (frac & 0xE0) {
    case 0x00: {
        union {
            u32 u;
            f32 f;
        } v;
        v.u = ((u32) s[0] << 24) | ((u32) s[1] << 16) | ((u32) s[2] << 8) | s[3];
        *p = s + 4;
        return v.f;
    }
    case 0x20:
        *p = s + 2;
        return (f32) (s16) ((s[0] << 8) | s[1]) / scale;
    case 0x40:
        *p = s + 2;
        return (f32) (u16) ((s[0] << 8) | s[1]) / scale;
    case 0x60:
        *p = s + 1;
        return (f32) (s8) s[0] / scale;
    default:
        *p = s + 1;
        return (f32) s[0] / scale;
    }
}

static u32 genoArt_FObjPacked(u8** p)
{
    u8* s = *p;
    u32 v = *s & 0x7F;
    int shift = 7;
    while (*s++ & 0x80) {
        v |= (u32) (*s & 0x7F) << shift;
        shift += 7;
    }
    *p = s;
    return v;
}

/* Walk the texture-image track (HSD_A_T_TIMG) of `tobj`'s animation. Returns true when a key
 * sits exactly on `frame` (a frame with no key of its own just holds the previous name), and
 * stores the largest image index any key selects in *max_img (-1 when there is no track). */
static bool genoArt_ImageKeyAt(HSD_TObj* tobj, int frame, int* max_img)
{
    HSD_FObj* fo;
    bool hit = false;
    *max_img = -1;
    if (tobj == NULL || tobj->aobj == NULL) {
        return false;
    }
    for (fo = tobj->aobj->fobj; fo != NULL; fo = fo->next) {
        u8* p;
        u8* end;
        f32 t;
        if (fo->obj_type != HSD_A_T_TIMG || fo->ad_head == NULL) {
            continue;
        }
        p = fo->ad_head;
        end = p + fo->length;
        t = (f32) fo->startframe;
        while (p < end) {
            u8 b = *p++;
            int op = b & 0xF;
            int n = (b >> 4) & 7;
            int shift = 3;
            while (b & 0x80) {
                b = *p++;
                n += (b & 0x7F) << shift;
                shift += 7;
            }
            for (n += 1; n > 0; n--) {
                f32 v = 0.0F;
                u32 wait = 0;
                switch (op) {
                case HSD_A_OP_CON:
                case HSD_A_OP_LIN:
                case HSD_A_OP_SPL0:
                    v = genoArt_FObjValue(&p, fo->frac_value);
                    wait = genoArt_FObjPacked(&p);
                    break;
                case HSD_A_OP_SPL:
                    v = genoArt_FObjValue(&p, fo->frac_value);
                    genoArt_FObjValue(&p, fo->frac_slope);
                    wait = genoArt_FObjPacked(&p);
                    break;
                case HSD_A_OP_SLP:
                    genoArt_FObjValue(&p, fo->frac_slope);
                    continue;
                case HSD_A_OP_KEY:
                    v = genoArt_FObjValue(&p, fo->frac_value);
                    break;
                default:
                    return hit;
                }
                if ((int) t == frame) {
                    hit = true;
                }
                if ((int) v > *max_img) {
                    *max_img = (int) v;
                }
                t += (f32) wait;
            }
        }
    }
    return hit;
}

