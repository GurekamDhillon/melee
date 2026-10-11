/* Compact CSS overlay: the same bevels, port colour and caption roles as the
 * player panels. All underlying targets are removed while it is open. */
static void csv_draw_controls(const AtScreen *sc, const AtTextOps *o, const AtSink *s, HitCtx *hc, float canvas_w)
{
    const AtCssControlsView *v = &sc->css_controls;
    int i, n = v->n < AT_CSS_CONTROL_ROWS ? v->n : AT_CSS_CONTROL_ROWS;
    float w = 376.0f, h = 86.0f + 26.0f * (float) n;
    AtRect box = { (canvas_w - w) * 0.5f, (480.0f - h) * 0.5f, w, h };
    unsigned col = at_part_port_colour(v->port);
    char title[32];
    at_poly_rect(s, 0.0f, 0.0f, canvas_w, 480.0f, 0x080C14B8u);
    at_plate(s, (AtRect){ box.x + 4.0f, box.y + 5.0f, box.w, box.h }, AT_C_INK, AT_C_INK, 0.0f, 6.0f);
    at_plate(s, box, AT_C_PLATE2, AT_C_LINE2, 2.0f, 6.0f);
    at_plate(s, (AtRect){ box.x + 3.0f, box.y + 3.0f, box.w - 6.0f, 24.0f }, col, col, 0.0f, 4.0f);
    snprintf(title, sizeof title, "P%d  /  CONTROLS", v->port + 1);
    at_text(s, o, AT_R_CAP12, title, box.x + 14.0f, box.y + 19.0f, AT_C_INK, AT_ALIGN_LEFT, 0.0f);
    csv_text(s, o, AT_R_ROW16, v->title, box.x + 16.0f, box.y + 47.0f, AT_C_IVORY, AT_ALIGN_LEFT, box.w - 32.0f);
    for (i = 0; i < n; i++) {
        AtRect row = { box.x + 12.0f, box.y + 58.0f + (float) i * 26.0f, box.w - 24.0f, 23.0f };
        int focus = v->row == i, done = i == n - 1;
        at_plate(s, row, focus ? AT_C_LIFT : AT_C_PLATE, focus ? col : AT_C_EDGE, focus ? 2.0f : 1.0f, 3.0f);
        if (done) {
            at_text(s, o, AT_R_CAP12, "DONE", row.x + row.w * 0.5f, row.y + 16.0f, AT_C_IVORY, AT_ALIGN_CENTER, 0.0f);
        } else {
            csv_text(s, o, AT_R_CAP12, v->label[i], row.x + 10.0f, row.y + 16.0f, AT_C_IVORY, AT_ALIGN_LEFT, row.w * 0.50f - 14.0f);
            csv_text(s, o, AT_R_CAP12, v->value[i], row.x + row.w * 0.76f, row.y + 16.0f, AT_C_IVORY, AT_ALIGN_CENTER, row.w * 0.43f - 36.0f);
            at_text(s, o, AT_R_CAP12, "<", row.x + row.w * 0.52f, row.y + 16.0f, col, AT_ALIGN_CENTER, 0.0f);
            at_text(s, o, AT_R_CAP12, ">", row.x + row.w - 12.0f, row.y + 16.0f, col, AT_ALIGN_CENTER, 0.0f);
        }
        hit_add(hc, row, AT_HIT_CONTROLS, i, 2);
        if (!done) {
            hit_add(hc, (AtRect){ row.x + row.w * 0.48f, row.y, 27.0f, row.h }, AT_HIT_CONTROLS, i, -1);
            hit_add(hc, (AtRect){ row.x + row.w - 28.0f, row.y, 28.0f, row.h }, AT_HIT_CONTROLS, i, 1);
        }
    }
    at_text(s, o, AT_R_CAP12, "A  CHANGE     B  CLOSE", box.x + box.w * 0.5f, box.y + box.h - 9.0f, AT_C_MUTED, AT_ALIGN_CENTER, 0.0f);
}
