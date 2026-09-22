/* gmfrontend_kitlist.inc - included by gmfrontend.c (TARGET_PC only), after the menus.
 *
 * ---- TOOLKIT SCREENS ON THE KIT: MATCH SETUP and ONLINE PLAY ------------------------------
 *
 * The toolkit (gmfrontend.c: FrontendItem rows - actions, choices, sliders, toggles, read-only
 * status rows - with their get/set/visible/call functions and input handling) stays exactly as
 * it is; this draws it with brief section 1 instead of the old art pack:
 *
 *   chrome   chrome_layout.json in the Versus section: header "MATCH SETUP" / "ONLINE PLAY",
 *            breadcrumb VERSUS / MELEE / MATCH SETUP [/ ONLINE PLAY], description strip (the
 *            row's help), footer hints A Select, D-pad Change, B Back
 *   rows     list_layout.json's row template, nine visible and scrolling; the continue action
 *            is the first row (the cursor starts there, as it started on the button)
 *   widgets  widgets_layout.json in the value slot (x 244, row middle): choice (arrows and a
 *            centred value), slider (track, fill, knob, readout), toggle (track, knob, OFF/ON),
 *            and read-only readouts (the online codes); colours per row state ng / sel
 *   motion   kit_motion.json: row_select / row_deselect on a cursor move, choice_step,
 *            slider_step, toggle_set on a change, bump at a slider's end - with the kit's value
 *            expressions ("60*dir", "130*value/max-130") evaluated by the player
 *   status   while ONLINE PLAY connects, the connection status is the kit's toast over the
 *            description strip (ok accent, danger when it failed), sliding in and out with
 *            toast_in / toast_out
 *
 * The screen is rebuilt when the visible rows change (a value can show or hide rows, and Play
 * As swaps the host's and the guest's rows); cursor and values carry over. */

static void fe_value_string(const FrontendItem* it, char* out);

enum { FKW_NONE, FKW_CHOICE, FKW_SLIDER, FKW_TOGGLE, FKW_READOUT };

#define FK_MAX_ROWS FM_MAX_VIS

static struct {
    bool on;
    const FrontendScreen* screen;
    int nrows;
    int item[FK_MAX_ROWS];
    u8 widget[FK_MAX_ROWS];
    int tile[FK_MAX_ROWS];
    int q_cv[FK_MAX_ROWS], q_cvo[FK_MAX_ROWS], q_read[FK_MAX_ROWS];
    int q_w[FK_MAX_ROWS][8]; ///< every widget quad, for recolouring by row state
    const char* q_wcol[FK_MAX_ROWS][8];
    int nq_w[FK_MAX_ROWS];
    int q_off[FK_MAX_ROWS], q_on[FK_MAX_ROWS];
    int j_sf[FK_MAX_ROWS], j_sk[FK_MAX_ROWS], j_tk[FK_MAX_ROWS], j_cv[FK_MAX_ROWS],
        j_cvo[FK_MAX_ROWS];
    float ox[FK_MAX_ROWS]; ///< value slot origin x (unsheared)
    char val[FK_MAX_ROWS][FE_STR];
    int cursor;
    int dir;
    int scroll;
    int rows_joint;
    int toast_joint, q_toast_text, q_toast_accent;
    bool toast_up;
    char toast_str[FE_STR];
} fk;

static u8 fk_widget_of(const FrontendItem* it)
{
    switch (it->kind) {
    case FE_CHOICE: return FKW_CHOICE;
    case FE_TOGGLE: return FKW_TOGGLE;
    case FE_SLIDER: return it->set != NULL ? FKW_SLIDER : FKW_READOUT;
    case FE_ACTION: return it->format != NULL ? FKW_READOUT : FKW_NONE; /* an action with a status */
    default: return FKW_NONE;
    }
}

/* Set a text quad's string again: fitted to max_w and anchored left (0), centre (1) or
 * right (2) of x. */
static void fk_retext(int q, const char* s, float x, int align, float max_w)
{
    FpQuad* d;
    float w;
    if (q < 0) {
        return;
    }
    d = &fp.q[q];
    if (d->ff_role >= 0) {
        int role = ff_role(d->ff_role >= 0 ? ff.r[d->ff_role].name : "row");
        /* start from the slot's own role again, so a shorter string gets its size back */
        (void) role;
        d->ff_role = (s8) ff_fit(ff_role(kit.row_role), s, max_w, d->textbuf,
                                 (int) sizeof d->textbuf);
        d->text = d->textbuf;
        w = ff_width(d->ff_role, d->text);
    } else {
        fp_copy(d->textbuf, s, (int) sizeof d->textbuf);
        d->text = d->textbuf;
        w = (float) strlen(s) * d->text_h * FM_SIS_ADV;
    }
    d->ax = align == 0 ? x : align == 1 ? x - w * 0.5F : x - w;
}

static int fk_wtext(int row, const char* id, float x, float y, int align, float max_w,
                    const char* s, const char* colour)
{
    float rgb[3];
    int q;
    kit_widget_state_colour(0, colour, rgb);
    q = fm_text(id, FPR_OTHER, fp.nt - 1, kit.row_role, x, y, s, rgb, max_w);
    if (q >= 0) {
        fk_retext(q, s, x, align, max_w);
        fp.tile[fk.tile[row]].extra[fp.tile[fk.tile[row]].nextra++] = q;
        fk.q_w[row][fk.nq_w[row]] = q;
        fk.q_wcol[row][fk.nq_w[row]++] = colour;
    }
    return q;
}

static int fk_wflat(int row, const char* id, float x0, float y0, float x1, float y1,
                    const char* colour)
{
    float rgb[3];
    int q;
    kit_widget_state_colour(0, colour, rgb);
    q = fm_flat(id, FPR_OTHER, fp.nt - 1, x0, y0, x1, y1, rgb);
    if (q >= 0) {
        fp.tile[fk.tile[row]].extra[fp.tile[fk.tile[row]].nextra++] = q;
        fk.q_w[row][fk.nq_w[row]] = q;
        fk.q_wcol[row][fk.nq_w[row]++] = colour;
    }
    return q;
}

static int fk_joint(const char* pat, int row, int parent, const float* pivot)
{
    char nm[FP_ID];
    fp_subst(nm, pat, fp.tile[fk.tile[row]].id);
    return fp_add_joint(nm, parent, pivot);
}

static float fk_frac(const FrontendItem* it)
{
    if (it->get == NULL || it->max <= it->min) {
        return 0.0F;
    }
    return (float) (it->get() - it->min) / (float) (it->max - it->min);
}

/* Row states: recolour a row's widget quads (ng / sel) from the frame f on. */
static void fk_row_state(int row, int state, int f)
{
    int k, c;
    for (k = 0; k < fk.nq_w[row]; k++) {
        float rgb[3];
        const char* col = fk.q_wcol[row][k];
        int q = fk.q_w[row][k];
        const FrontendItem* it = &fk.screen->items[fk.item[row]];
        /* the toggle's active half takes knob_text */
        if (fk.widget[row] == FKW_TOGGLE && it->get != NULL &&
            ((q == fk.q_on[row] && it->get() != 0) || (q == fk.q_off[row] && it->get() == 0)))
        {
            col = "knob_text";
        }
        kit_widget_state_colour(state, col, rgb);
        for (c = 0; c < 3; c++) {
            fp_set(fp_mch(q, c), f, rgb[c]);
        }
    }
}

static void fk_build(bool animate_in)
{
    static const char* const hg[3] = { "glyph_a", "glyph_dpad", "glyph_b" };
    static const char* const ht[3] = { "Select", "Change", "Back" };
    const char* crumbs[4] = { "VERSUS", "MELEE", "MATCH SETUP", "ONLINE PLAY" };
    void *lt, *lp, *mt, *mp;
    const FjNode* L;
    const FjNode* M;
    int section = KS_VERSUS, r, i;
    kit_load();
    ff_load();
    kit_widgets_load();
    kw_section = section;
    fk.screen = fe.screen;
    fp_free_textures();
    fp_reset();
    L = fj_load("hub_layout.json", &lt, &lp);
    if (L != NULL) {
        fp_read_states(L);
    }
    if (lt != NULL) HSD_Free(lt);
    if (lp != NULL) HSD_Free(lp);
    fm_default_states(section);
    fp.sel_offset[0] = kit.row_sel_offset[0];
    fp.sel_offset[1] = kit.row_sel_offset[1];
    fp.recolour = true;
    fk.rows_joint = fp_add_joint("rows", -1, NULL);
    fm_chrome_with(fe.screen->subtitle, crumbs, fe.screen == &fe_screen_online ? 4 : 3, section,
                   3, hg, ht);

    fk.nrows = fe.n_vis < FK_MAX_ROWS ? fe.n_vis : FK_MAX_ROWS;
    for (r = 0; r < fk.nrows; r++) {
        const FrontendItem* it = &fe.screen->items[fe.vis[r]];
        FeMenuItem tmp;
        float y = kit.rows_top + r * kit.pitch, x0 = kit.row_x0, oy = y + 15.0F, ox;
        char id[FP_ID];
        u8 w = fk_widget_of(it);
        memset(&tmp, 0, sizeof tmp);
        tmp.label = it->label;
        tmp.desc = it->help != NULL ? it->help : "";
        fk.item[r] = fe.vis[r];
        fk.widget[r] = w;
        fk.nq_w[r] = 0;
        fk.q_cv[r] = fk.q_cvo[r] = fk.q_read[r] = fk.q_off[r] = fk.q_on[r] = -1;
        fk.j_sf[r] = fk.j_sk[r] = fk.j_tk[r] = fk.j_cv[r] = fk.j_cvo[r] = -1;
        fk.tile[r] = fp.nt;
        fm_tile(r, &tmp, section, r, x0, y, kit.row_x1, y + kit.row_h, false, true);
        if (w != FKW_NONE) { /* a label beside a widget keeps to the template's width */
            FpQuad* lq = &fp.q[fp.tile[fk.tile[r]].q_label];
            fk_retext(fp.tile[fk.tile[r]].q_label, it->label, lq->ax, 0, kit.row_label_max);
        }
        ox = x0 + 244.0F;
        fk.ox[r] = ox;
        fe_value_string(it, fk.val[r]);
        switch (w) {
        case FKW_CHOICE:
            sprintf(id, "%s_cal", fp.tile[fk.tile[r]].id);
            fk_wtext(r, id, ox, oy + 5.28F, 0, 12, "\x02", "aux");
            sprintf(id, "%s_cvo", fp.tile[fk.tile[r]].id);
            fk.q_cvo[r] = fk_wtext(r, id, ox + 95, oy + 5.28F, 1, 122.4F, fk.val[r], "text");
            sprintf(id, "%s_cv", fp.tile[fk.tile[r]].id);
            fk.q_cv[r] = fk_wtext(r, id, ox + 95, oy + 5.28F, 1, 122.4F, fk.val[r], "text");
            sprintf(id, "%s_car", fp.tile[fk.tile[r]].id);
            fk_wtext(r, id, ox + 190, oy + 5.28F, 2, 12, "\x04", "aux"); /* U+2192 */
            break;
        case FKW_SLIDER:
            sprintf(id, "%s_trk", fp.tile[fk.tile[r]].id);
            fk_wflat(r, id, ox, oy - 3, ox + 130, oy + 3, "track");
            sprintf(id, "%s_fill", fp.tile[fk.tile[r]].id);
            fk_wflat(r, id, ox, oy - 3, ox + 130, oy + 3, "fill");
            sprintf(id, "%s_knob", fp.tile[fk.tile[r]].id);
            fk_wflat(r, id, ox + 126, oy - 9, ox + 134, oy + 9, "knob");
            sprintf(id, "%s_read", fp.tile[fk.tile[r]].id);
            fk.q_read[r] = fk_wtext(r, id, ox + 190, oy + 5.28F, 2, 50, fk.val[r], "text");
            break;
        case FKW_TOGGLE:
            sprintf(id, "%s_trk", fp.tile[fk.tile[r]].id);
            fk_wflat(r, id, ox, oy - 10, ox + 120, oy + 10, "track");
            sprintf(id, "%s_knob", fp.tile[fk.tile[r]].id);
            fk_wflat(r, id, ox + 62, oy - 8, ox + 118, oy + 8, "toggle_knob");
            sprintf(id, "%s_off", fp.tile[fk.tile[r]].id);
            fk.q_off[r] = fk_wtext(r, id, ox + 30, oy + 5.28F, 1, 28.2F, "OFF", "track_text");
            sprintf(id, "%s_on", fp.tile[fk.tile[r]].id);
            fk.q_on[r] = fk_wtext(r, id, ox + 90, oy + 5.28F, 1, 28.2F, "ON", "track_text");
            break;
        case FKW_READOUT:
            sprintf(id, "%s_read", fp.tile[fk.tile[r]].id);
            fk.q_read[r] = fk_wtext(r, id, ox + 190, oy + 5.28F, 2, 190, fk.val[r], "text");
            break;
        }
    }
    /* the toast (dialog_layout.json): over the description strip, drawn last */
    {
        float ink[3], ok[3], bone[3];
        int q;
        fm_tok("ink", section, ink);
        fm_tok("ok", section, ok);
        fm_tok("bone", section, bone);
        q = fm_flat("toast_plate", FPR_CHROME, -1, kit.desc_strip[0], kit.desc_strip[1],
                    kit.desc_strip[2], kit.desc_strip[3], ink);
        if (q >= 0) fp.q[q].late = true;
        fk.q_toast_accent = fm_flat("toast_accent", FPR_CHROME, -1, kit.desc_strip[0],
                                    kit.desc_strip[1], kit.desc_strip[0] + 6, kit.desc_strip[3],
                                    ok);
        if (fk.q_toast_accent >= 0) fp.q[fk.q_toast_accent].late = true;
        fk.q_toast_text = fm_text("toast_text", FPR_CHROME, -1, kit.desc_role,
                                  kit.desc_anchor[0], kit.desc_anchor[1], " ", bone,
                                  kit.desc_max);
        if (fk.q_toast_text >= 0) fp.q[fk.q_toast_text].late = true;
    }

    M = fj_load("hub_motion.json", &mt, &mp);
    if (M != NULL) {
        fp_read_motion(M, true);
    }
    if (mt != NULL) HSD_Free(mt);
    if (mp != NULL) HSD_Free(mp);
    kit_read_motion(section);
    /* the widget materials the kit's events name */
    {
        static const char* const mk[4][2] = {
            { "cv_<id>", "<id>_cv" }, { "cvo_<id>", "<id>_cvo" },
            { "cal_<id>", "<id>_cal" }, { "car_<id>", "<id>_car" },
        };
        for (i = 0; i < 4 && fp.nmat < 16; i++, fp.nmat++) {
            fp_copy(fp.mat_key[fp.nmat], mk[i][0], FP_ID);
            fp_copy(fp.mat_val[fp.nmat], mk[i][1], FP_ID);
        }
    }
    /* chrome on the header / footer joints, rows under the scroll joint */
    {
        int hj = fp_find_joint("header"), fj2 = fp_find_joint("footer");
        for (i = 0; i < fp.nq; i++) {
            const char* id = fp.q[i].id;
            if (fp.q[i].tile >= 0 || fp.q[i].joint >= 0) {
                continue;
            }
            if (strncmp(id, "crumb", 5) == 0 || strncmp(id, "header", 6) == 0) {
                fp.q[i].joint = (s8) hj;
            } else if (strncmp(id, "hint", 4) == 0 || strncmp(id, "desc_strip", 10) == 0 ||
                       strncmp(id, "footer", 6) == 0)
            {
                fp.q[i].joint = (s8) fj2;
            }
        }
        for (i = 0; i < fp.nt; i++) {
            if (fp.tile[i].j_tile >= 0) {
                fp.j[fp.tile[i].j_tile].parent = (s8) fk.rows_joint;
            }
        }
    }
    /* widget joints, each under its row's lift so it rises with the row */
    for (r = 0; r < fk.nrows; r++) {
        FpTile* t = &fp.tile[fk.tile[r]];
        char q[FP_ID];
        float oy = kit.rows_top + r * kit.pitch + 15.0F;
        float piv[2];
        piv[0] = fk.ox[r] + (fp.shear_y0 - oy) * fp.shear_s;
        piv[1] = oy;
        switch (fk.widget[r]) {
        case FKW_CHOICE:
            fk.j_cv[r] = fk_joint("cv_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_cv", t->id);
            fp_joint_draws(fk.j_cv[r], q);
            fk.j_cvo[r] = fk_joint("cvo_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_cvo", t->id);
            fp_joint_draws(fk.j_cvo[r], q);
            i = fk_joint("cal_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_cal", t->id);
            fp_joint_draws(i, q);
            i = fk_joint("car_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_car", t->id);
            fp_joint_draws(i, q);
            break;
        case FKW_SLIDER:
            i = fk_joint("trk_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_trk", t->id);
            fp_joint_draws(i, q);
            sprintf(q, "%s_read", t->id);
            fp_joint_draws(i, q);
            fk.j_sf[r] = fk_joint("sf_<id>", r, t->j_lift, piv);
            sprintf(q, "%s_fill", t->id);
            fp_joint_draws(fk.j_sf[r], q);
            fk.j_sk[r] = fk_joint("sk_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_knob", t->id);
            fp_joint_draws(fk.j_sk[r], q);
            break;
        case FKW_TOGGLE:
            i = fk_joint("trk_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_trk", t->id);
            fp_joint_draws(i, q);
            sprintf(q, "%s_off", t->id);
            fp_joint_draws(i, q);
            sprintf(q, "%s_on", t->id);
            fp_joint_draws(i, q);
            fk.j_tk[r] = fk_joint("tk_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_knob", t->id);
            fp_joint_draws(fk.j_tk[r], q);
            break;
        case FKW_READOUT:
            i = fk_joint("rd_<id>", r, t->j_lift, NULL);
            sprintf(q, "%s_read", t->id);
            fp_joint_draws(i, q);
            break;
        }
    }
    fk.toast_joint = fp_add_joint("toast", -1, NULL);
    fp_joint_draws(fk.toast_joint, "toast_plate");
    fp_joint_draws(fk.toast_joint, "toast_accent");
    fp_joint_draws(fk.toast_joint, "toast_text");
    for (i = 0; i < fp.nq; i++) { /* atlas pages now, not mid-draw */
        if (fp.q[i].kind == FPK_TEXT && fp.q[i].ff_role >= 0) {
            int p;
            for (p = 0; p < ff.r[fp.q[i].ff_role].npages; p++) {
                fp_tex(ff.r[fp.q[i].ff_role].page[p]);
            }
        }
    }

    fp_reset_channels();
    fp.ok = true;
    fp.frame = 0;
    /* rest values: slider fill/knob, toggle knob, the outgoing choice text, the toast */
    for (r = 0; r < fk.nrows; r++) {
        const FrontendItem* it = &fe.screen->items[fk.item[r]];
        if (fk.j_sf[r] >= 0) fp_jch(fk.j_sf[r], FJC_SCA_X)->rest = fk_frac(it);
        if (fk.j_sk[r] >= 0) fp_jch(fk.j_sk[r], FJC_TRA_X)->rest = 130.0F * fk_frac(it) - 130.0F;
        if (fk.j_tk[r] >= 0 && it->get != NULL) {
            fp_jch(fk.j_tk[r], FJC_TRA_X)->rest = it->get() ? 0.0F : -60.0F;
        }
        if (fk.q_cvo[r] >= 0) fp_mch(fk.q_cvo[r], FMC_ALPHA)->rest = 0.0F;
        fk_row_state(r, 0, 0);
    }
    fp_jch(fk.toast_joint, FJC_TRA_X)->rest = 600.0F;
    fk.toast_up = false;
    fk.toast_str[0] = '\0';
    fk.cursor = fe.cursor < fk.nrows ? fe.cursor : 0;
    fk.scroll = 0;
    if (fk.cursor >= kit.visible - 1 && fk.nrows > kit.visible) {
        fk.scroll = fk.cursor - (kit.visible - 2);
        if (fk.scroll > fk.nrows - kit.visible) fk.scroll = fk.nrows - kit.visible;
        fp_set(fp_jch(fk.rows_joint, FJC_TRA_Y), 0, -kit.pitch * fk.scroll);
    }
    if (fk.nrows > 0) {
        fp_select_initial(fk.tile[fk.cursor], 0);
        fk_row_state(fk.cursor, 1, 0);
    }
    if (animate_in) {
        fp_screen_in(0);
    }
    fk.on = true;
    OSReport("frontend: %s on the kit - %d rows\n", fe.screen->subtitle, fk.nrows);
}

static void fk_scroll_to_cursor(void)
{
    int want = fk.scroll;
    FpKey k[2];
    if (fk.nrows <= kit.visible) {
        return;
    }
    if (fk.cursor < want + 1) want = fk.cursor - 1;
    if (fk.cursor > want + kit.visible - 2) want = fk.cursor - (kit.visible - 2);
    if (want < 0) want = 0;
    if (want > fk.nrows - kit.visible) want = fk.nrows - kit.visible;
    if (want == fk.scroll) {
        return;
    }
    fk.scroll = want;
    k[0].frame = 0;
    k[0].interp = FPI_SPL;
    k[0].from_current = 1;
    k[0].v = 0.0F;
    k[0].slope = 0.0F;
    k[1] = k[0];
    k[1].frame = 6;
    k[1].from_current = 0;
    k[1].v = -kit.pitch * (float) fk.scroll;
    fp_start(fp_jch(fk.rows_joint, FJC_TRA_Y), fp.frame, k, 2);
}

/* fe_change reports a slider held at its end: the kit's bump. */
static void fk_bump(int dir)
{
    int r = fk.cursor;
    if (!fk.on || r >= fk.nrows || fk.widget[r] != FKW_SLIDER) {
        return;
    }
    fp.v_dir = (float) dir;
    fp_copy(fp.n_value, "sk", 8);
    fp_copy(fp.n_dir, dir > 0 ? "r" : "l", 4);
    fp_event("bump", fk.tile[r], fp.frame, -1);
}

static void fk_note_dir(int dir)
{
    fk.dir = dir;
}

static bool fk_rows_changed(void)
{
    int r;
    if (fe.screen != fk.screen || fe.n_vis != fk.nrows) {
        return true;
    }
    for (r = 0; r < fk.nrows; r++) {
        if (fe.vis[r] != fk.item[r]) {
            return true;
        }
    }
    return false;
}

static void fk_frame(void)
{
    int r;
    if (!fk.on) {
        return;
    }
    if (fk_rows_changed()) {
        fk_build(fe.screen != fk.screen);
    }
    fp.frame++;
    /* cursor: the kit's list_cursor_move, with the hub's description fade in the strip */
    if (fe.cursor != fk.cursor && fe.cursor < fk.nrows) {
        int old = fk.cursor;
        fk.cursor = fe.cursor;
        fp.prev = fk.tile[old];
        fp.sel = fk.tile[fk.cursor];
        fp_event("row_deselect", fk.tile[old], fp.frame, -1);
        fp_event("desc_out", fk.tile[old], fp.frame, -1);
        fp_event("row_select", fk.tile[fk.cursor], fp.frame, -1);
        fp_event("desc_in", fk.tile[fk.cursor], fp.frame, -1);
        fk_row_state(old, 0, fp.frame);
        fk_row_state(fk.cursor, 1, fp.frame);
        fk_scroll_to_cursor();
    }
    /* values */
    for (r = 0; r < fk.nrows; r++) {
        const FrontendItem* it = &fe.screen->items[fk.item[r]];
        char now[FE_STR];
        int t = fk.tile[r];
        fe_value_string(it, now);
        if (strcmp(now, fk.val[r]) == 0) {
            continue;
        }
        fp.v_dir = (float) (fk.dir != 0 ? fk.dir : 1);
        fp_copy(fp.n_dir, fk.dir < 0 ? "l" : "r", 4);
        switch (fk.widget[r]) {
        case FKW_CHOICE:
            fk_retext(fk.q_cvo[r], fk.val[r], fk.ox[r] + 95, 1, 122.4F);
            fk_retext(fk.q_cv[r], now, fk.ox[r] + 95, 1, 122.4F);
            fp_copy(fp.n_value, "cv", 8);
            fp_event("choice_step", t, fp.frame, -1);
            break;
        case FKW_SLIDER:
            fk_retext(fk.q_read[r], now, fk.ox[r] + 190, 2, 50);
            fp.v_value = (float) (it->get() - it->min);
            fp.v_max = (float) (it->max - it->min);
            fp_event("slider_step", t, fp.frame, -1);
            break;
        case FKW_TOGGLE:
            fp.v_dir = it->get() ? 1.0F : -1.0F;
            /* the knob's rest is "on" (TRA_X 0): the event's 60*dir runs from off to on */
            {
                FpKey k[2];
                k[0].frame = 0;
                k[0].interp = FPI_SPL;
                k[0].from_current = 1;
                k[0].v = 0.0F;
                k[0].slope = 0.0F;
                k[1] = k[0];
                k[1].frame = 8;
                k[1].from_current = 0;
                k[1].v = it->get() ? 0.0F : -60.0F;
                fp_start(fp_jch(fk.j_tk[r], FJC_TRA_X), fp.frame, k, 2);
                fp_event("toggle_set", t, fp.frame, -1); /* the squash (SCA_X) */
                /* TRA_X from the event is relative to the travel; the explicit play above
                   decides the position, so drop the event's TRA_X */
                fp_start(fp_jch(fk.j_tk[r], FJC_TRA_X), fp.frame, k, 2);
            }
            fk_row_state(r, r == fk.cursor ? 1 : 0, fp.frame);
            break;
        case FKW_READOUT:
            fk_retext(fk.q_read[r], now, fk.ox[r] + 190, 2, 190);
            break;
        }
        fp_copy(fk.val[r], now, FE_STR);
    }
    /* the connection status as the kit's toast */
    if (fe.screen == &fe_screen_online || fe.screen == &fe_screen_wait ||
        fe.screen == &fe_screen_lobby)
    {
        bool want = fe_np_phase != FE_NP_IDLE || fe.screen == &fe_screen_lobby;
        if (want) {
            char st[FE_STR];
            if (fe.screen == &fe_screen_lobby) {
                fe_lb_instruction(st); /* what the lobby asks of this player now */
            } else {
                Netplay_MenuStatus(st, FE_STR);
            }
            if (strcmp(st, fk.toast_str) != 0) {
                fp_copy(fk.toast_str, st, FE_STR);
                fk_retext(fk.q_toast_text, st, kit.desc_anchor[0], 0, kit.desc_max);
            }
            if (fk.q_toast_accent >= 0) {
                float rgb[3];
                int c;
                fm_tok(fe_np_phase == FE_NP_FAILED ? "danger" : "ok", KS_VERSUS, rgb);
                for (c = 0; c < 3; c++) {
                    fp_set(fp_mch(fk.q_toast_accent, c), fp.frame, rgb[c]);
                }
            }
        }
        if (want && !fk.toast_up) {
            fp_event("toast_in", -1, fp.frame, fk.toast_joint);
            fk.toast_up = true;
        } else if (!want && fk.toast_up) {
            fp_event("toast_out", -1, fp.frame, fk.toast_joint);
            fk.toast_up = false;
        }
    }
    fp_evaluate(fp.frame);
    /* rows outside the nine visible */
    if (fk.nrows > kit.visible) {
        float top = kit.rows_top - kit.pitch * 0.5F;
        float bot = kit.rows_top + (kit.visible - 1) * kit.pitch + kit.pitch * 0.5F;
        float dy = fp_value(fp_jch(fk.rows_joint, FJC_TRA_Y), fp.frame);
        for (r = 0; r < fk.nrows; r++) {
            float y = kit.rows_top + r * kit.pitch + dy;
            if (y < top || y > bot) {
                FpTile* t = &fp.tile[fk.tile[r]];
                int e;
                fp.q[t->q_plate].wa = fp.q[t->q_face].wa = 0.0F;
                if (t->q_label >= 0) fp.q[t->q_label].wa = 0.0F;
                for (e = 0; e < t->nextra; e++) fp.q[t->extra[e]].wa = 0.0F;
            }
        }
    }
}

static void fk_exit(void)
{
    if (fk.on) {
        fp_free_textures();
        fp.ok = false;
        fk.on = false;
    }
}
