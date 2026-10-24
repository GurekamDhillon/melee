static void dump_HSD_TETev(const HSD_TETev* p)
{
    int i0, i1, i2, j0, j1, j2, j3;
    if (HDBG) printf("B HSD_TETev%c", 10);
    HM((u32) p->type);
    HM(PI((const void*) p->next));
    HM((u32) p->c_ref);
    HM((u32) p->c_dst);
    HM((u32) p->c_op);
    HM((u32) p->c_clamp);
    HM((u32) p->c_bias);
    HM((u32) p->c_scale);
    HM((u32) p->c_range);
    HM((u32) p->a_ref);
    HM((u32) p->a_dst);
    HM((u32) p->a_op);
    HM((u32) p->a_clamp);
    HM((u32) p->a_bias);
    HM((u32) p->a_scale);
    HM((u32) p->tex_swap);
    HM((u32) p->ras_swap);
    HM((u32) p->kcsel);
    HM((u32) p->kasel);
    for (i0 = 0; i0 < 4; i0++) {
    HM((u32) p->c_in[i0].type);
    HM((u32) p->c_in[i0].sel);
    HM((u32) p->c_in[i0].arg);
    HM(PI((const void*) p->c_in[i0].exp));
    }
    for (i0 = 0; i0 < 4; i0++) {
    HM((u32) p->a_in[i0].type);
    HM((u32) p->a_in[i0].sel);
    HM((u32) p->a_in[i0].arg);
    HM(PI((const void*) p->a_in[i0].exp));
    }
    HM(PI((const void*) p->tex));
    HM((u32) p->chan);
}

static void dump_HSD_TECnst(const HSD_TECnst* p)
{
    int i0, i1, i2, j0, j1, j2, j3;
    if (HDBG) printf("B HSD_TECnst%c", 10);
    HM((u32) p->type);
    HM(PI((const void*) p->next));
    HM(PI((const void*) p->val));
    HM((u32) p->comp);
    HM((u32) p->ctype);
    HM((u32) p->reg);
    HM((u32) p->idx);
    HM((u32) p->ref);
    HM((u32) p->range);
}

static void dump_HSD_TExpRes(const HSD_TExpRes* p)
{
    int i0, i1, i2, j0, j1, j2, j3;
    if (HDBG) printf("B HSD_TExpRes%c", 10);
    HM((u32) p->failed);
    HM((u32) p->texmap);
    HM((u32) p->cnst_remain);
    for (i0 = 0; i0 < 8; i0++) {
    HM((u32) p->reg[i0].color);
    HM((u32) p->reg[i0].alpha);
    }
    for (i0 = 0; i0 < 4; i0++) {
    HM((u32) p->c_ref[i0]);
    }
    for (i0 = 0; i0 < 4; i0++) {
    HM((u32) p->a_ref[i0]);
    }
    for (i0 = 0; i0 < 4; i0++) {
    HM((u32) p->c_use[i0]);
    }
    for (i0 = 0; i0 < 4; i0++) {
    HM((u32) p->a_use[i0]);
    }
}

static void dump_HSD_TevConf(const HSD_TevConf* p)
{
    int i0, i1, i2, j0, j1, j2, j3;
    if (HDBG) printf("B HSD_TevConf%c", 10);
    HM((u32) p->clr_op);
    HM((u32) p->clr_a);
    HM((u32) p->clr_b);
    HM((u32) p->clr_c);
    HM((u32) p->clr_d);
    HM((u32) p->clr_scale);
    HM((u32) p->clr_bias);
    HM((u32) p->clr_clamp);
    HM((u32) p->clr_out_reg);
    HM((u32) p->alpha_op);
    HM((u32) p->alpha_a);
    HM((u32) p->alpha_b);
    HM((u32) p->alpha_c);
    HM((u32) p->alpha_d);
    HM((u32) p->alpha_scale);
    HM((u32) p->alpha_bias);
    HM((u32) p->alpha_clamp);
    HM((u32) p->alpha_out_reg);
    HM((u32) p->mode);
    HM((u32) p->ras_swap);
    HM((u32) p->tex_swap);
    HM((u32) p->kcsel);
    HM((u32) p->kasel);
}
