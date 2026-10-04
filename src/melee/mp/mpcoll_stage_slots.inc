/* Script slots reserve one backend joint per line. Ledge occlusion must compare
 * original DAT groups, not those allocation records. Other collision is unchanged. */
static bool mpColl_stage_same_joint(int a, int b)
{
#ifdef TARGET_PC
    extern int ScriptGame_StageSlotSameJoint(int, int);
    int same = ScriptGame_StageSlotSameJoint(a, b);
    if (same >= 0) return same != 0;
#endif
    return mpJointFromLine(a) == mpJointFromLine(b);
}
