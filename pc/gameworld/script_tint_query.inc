/* Included after script_parts.inc. Registry read only: no model reinspection. */
int ScriptGame_TintCount(int slot)
{
    int i, n=0;
    if (slot<0 || slot>=6) return 0;
    for (i=0;i<lab_part_colors_hi;++i)
        if (lab_part_colors[i].dobj && lab_part_colors[i].slot==slot &&
            lab_part_colors[i].mode==PART_CTL_TINT) ++n;
    return n;
}
