/* Snapshot-safe overlays: no fighter attributes or retail jump counter edits. */
int ScriptGame_CapsMovementTest(void)
{
    ScriptCapsMovement saved[12], snapshot[12];
    static Fighter fake, fake_sub;
    Fighter* fp = &fake;
    Fighter* sub = &fake_sub;
    int rc = 0, retail, used;
    memcpy(saved, script_caps_movement, sizeof saved);
    memset(script_caps_movement, 0, sizeof script_caps_movement);
    memset(fp, 0, sizeof *fp);
    memset(sub, 0, sizeof *sub);
    fp->co_attrs.max_jumps = 2;
    sub->co_attrs.max_jumps = 6;
    fp->x1968_jumpsUsed = 1;
    retail = fp->co_attrs.max_jumps; used = fp->x1968_jumpsUsed;
    if (!ScriptGame_CapsMovement(1, 1, 8, 31) ||
        script_caps_max_jumps_value(fp, 1) != 9 ||
        script_caps_action_allowed_value(1, 1) ||
        ScriptGame_CapsMovementRead(1, 1) != 8) rc++;
    if (sub && script_caps_max_jumps_value(sub, 7) != sub->co_attrs.max_jumps) rc++;
    memcpy(snapshot, script_caps_movement, sizeof snapshot);
    ScriptGame_CapsMovement(1, 1, 0, 0);
    memcpy(script_caps_movement, snapshot, sizeof snapshot);
    if (memcmp(script_caps_movement, snapshot, sizeof snapshot) ||
        script_caps_max_jumps_value(fp, 1) != 9 ||
        script_caps_action_allowed_value(1, 16)) rc++;
    if (ScriptGame_CapsMovement(1, 2, 2, 0) ||
        ScriptGame_CapsMovement(1, 1, 9, 0) ||
        ScriptGame_CapsMovement(1, 1, -2, 0) ||
        ScriptGame_CapsMovement(1, 1, 1, 32)) rc++;
    ScriptGame_CapsMovementRelease(2);
    if (script_caps_max_jumps_value(fp, 1) != 9) rc++;
    if (!ScriptGame_CapsMovement(1, 1, 0, 2) ||
        script_caps_max_jumps_value(fp, 1) != 1 ||
        !script_caps_action_allowed_value(1, 1) ||
        script_caps_action_allowed_value(1, 2)) rc++;
    if (!ScriptGame_CapsMovement(1, 1, -1, 0) ||
        script_caps_max_jumps_value(fp, 1) != retail ||
        ScriptGame_CapsMovementRead(1, 0) != 0) rc++;
    if (!ScriptGame_CapsMovement(7, 2, 3, 8) ||
        script_caps_max_jumps_value(sub, 7) != 4 ||
        script_caps_max_jumps_value(fp, 1) != retail) rc++;
    ScriptGame_CapsMovementRelease(2);
    ScriptGame_CapsMovementRelease(1);
    if (ScriptGame_CapsMovementRead(1, 0) ||
        ScriptGame_CapsMovementRead(1, 1) != -1 ||
        script_caps_max_jumps_value(fp, 1) != retail ||
        fp->co_attrs.max_jumps != retail || fp->x1968_jumpsUsed != used) rc++;
    memcpy(script_caps_movement, saved, sizeof saved);
    return rc;
}
