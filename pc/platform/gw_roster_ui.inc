/* Wide catalogue entry points. The legacy UI_Fighter API remains unchanged.
 * Art consumers use SourceIconAt, while names/selections retain the wide IdAt.
 */
int gw_UI_RosterCount(void)
{
    return gw_UI_FighterCount() + gw_Roster_NativeCount();
}

int gw_UI_RosterIdAt(int index)
{
    int legacy = gw_UI_FighterCount();
    GwUiFighter row;
    if (index < 0) return -1;
    if (index < legacy) return gw_UI_FighterAt(index, &row) && row.has_file ? row.kind : -1;
    return gw_Roster_IdAt(index - legacy);
}

int gw_UI_RosterSourceIconAt(int index)
{
    int legacy = gw_UI_FighterCount(), id, source, i;
    GwUiFighter row;
    if (index < 0 || index >= gw_UI_RosterCount()) return -1;
    if (index < legacy) return index;
    id = gw_Roster_IdAt(index - legacy); source = gw_Roster_SourceCK(id);
    if (source < 0) return -1;
    for (i = 0; i < legacy; ++i) {
        if (gw_UI_FighterAt(i, &row) && row.has_file && row.kind == source) return i;
    }
    return -1;
}
