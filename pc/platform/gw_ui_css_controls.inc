/* CSS controls are a modal menu, not pad simulation. Persist through the host
 * callbacks; match setup freezes these choices before the first logic frame. */
int at_css_controls_rows(const AtCss *c)
{
    return (c->controls_layouts > 0) + 1 + c->controls_extras + 1;
}

unsigned at_css_controls_open(AtCss *c, int port)
{
    int n;
    if (c->controls_port >= 0 || !at_css_zone_ok(c, port, port, AT_ZONE_CONTROLS)) return 0;
    c->controls_port = port;
    c->controls_ck = c->p[port].ck;
    c->controls_row = 0;
    n = c->ops && c->ops->controls_count ? c->ops->controls_count(c->ops->user, c->controls_ck, 0) : 0;
    c->controls_layouts = n < 0 ? 0 : n > 16 ? 16 : n;
    n = c->ops && c->ops->controls_count ? c->ops->controls_count(c->ops->user, c->controls_ck, 1) : 0;
    c->controls_extras = n < 0 ? 0 : n > 8 ? 8 : n;
    c->controls_choice = c->ops && c->ops->controls_get ? c->ops->controls_get(c->ops->user, port, c->controls_ck) : 0;
    /* Stale saved options cannot select a missing layout or enable a new extra. */
    c->controls_choice &= 0x3Fu | (((1u << c->controls_extras) - 1u) << 8);
    if ((c->controls_choice & 3u) > 2u) c->controls_choice &= ~3u;
    if ((int) ((c->controls_choice >> 2) & 15u) >= c->controls_layouts) c->controls_choice &= ~0x3Cu;
    return AT_CE_FORWARD;
}

unsigned at_css_controls_point(AtCss *c, int port, int row, int action)
{
    int jump = c->controls_layouts > 0, rows = at_css_controls_rows(c), dir = action < 0 ? -1 : 1;
    unsigned choice = c->controls_choice;
    if (port < 0 || port > 3 || c->controls_port != port) return 0;
    if (action == 3) { c->controls_port = -1; c->controls_block_frame = -2; return AT_CE_BACK; }
    if (row < 0 || row >= rows) return 0;
    c->controls_row = row;
    if (action == 0) return 0;
    if (row == rows - 1) {
        if (action == 2) { c->controls_port = -1; c->controls_block_frame = -2; return AT_CE_BACK; }
        return 0;
    }
    if (jump && row == 0) {
        int layout = (int) ((choice >> 2) & 15u);
        layout = (layout + dir + c->controls_layouts) % c->controls_layouts;
        choice = (choice & ~0x3Cu) | ((unsigned) layout << 2);
    } else if (row == jump) {
        choice = (choice & ~3u) | (unsigned) (((int) (choice & 3u) + dir + 3) % 3);
    } else {
        choice ^= 1u << (8 + row - jump - 1);
    }
    if (choice == c->controls_choice) return 0;
    if (c->ops && c->ops->controls_set && !c->ops->controls_set(c->ops->user, port, c->controls_ck, choice)) {
        toast(c, "Controls could not be saved.");
        return AT_CE_BACK | AT_CE_TOAST;
    }
    c->controls_choice = choice;
    return AT_CE_MOVE;
}

static unsigned controls_step(AtCss *c, int port, AtCssIn in)
{
    int rows, row;
    if (c->controls_port != port) return 0;
    if (c->p[port].kind != AT_CSS_HMN || c->p[port].ck != c->controls_ck) {
        c->controls_port = -1;
        return AT_CE_BACK;
    }
    row = c->controls_row;
    rows = at_css_controls_rows(c);
    if (in.trig & AT_CI_B) return at_css_controls_point(c, port, row, 3);
    if (in.rep & AT_CI_UP) { c->controls_row = (row + rows - 1) % rows; return AT_CE_MOVE; }
    if (in.rep & AT_CI_DOWN) { c->controls_row = (row + 1) % rows; return AT_CE_MOVE; }
    if (in.rep & AT_CI_LEFT) return at_css_controls_point(c, port, row, -1);
    if (in.rep & AT_CI_RIGHT) return at_css_controls_point(c, port, row, 1);
    if (in.trig & AT_CI_A) return at_css_controls_point(c, port, row, 2);
    return 0;
}
