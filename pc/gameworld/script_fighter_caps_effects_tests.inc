/* Pure fixtures: no live fighter mutation, usable from native suite scalar thunk. */
int ScriptGame_CapsEffectsTest(void)
{
    int failed = 0;
    struct script_caps_effect timer;
    memset(&timer,0,sizeof timer);
    timer.last_timer=30;timer.baseline=20;
    script_caps_effect_observe(&timer,30);
    if (timer.baseline!=20 || timer.foreign) ++failed; /* hitlag freezes retail timer */
    script_caps_effect_observe(&timer,28);
    if (timer.baseline!=18 || timer.foreign) ++failed;
    script_caps_effect_observe(&timer,35);
    if (!timer.foreign || timer.baseline!=18) ++failed; /* newer retail extension */
    script_caps_effect_observe(&timer,34);
    if (!timer.foreign || timer.baseline!=17) ++failed;
    static Fighter fp;
    union { float f; int i; } selected;
    memset(&fp,0,sizeof fp);
    selected.f = 4.0f;
    fp.dmg.x186c = selected.i;
    fp.dmg.x1838_percentTemp = 20.0f; /* concurrent hits still add all percent */
    fp.dmg.kb_applied = 40.0f;
    if (!script_caps_hit_threshold(&fp,5.0f,0.0f)) ++failed;
    selected.f = 8.0f;
    fp.dmg.x186c = selected.i;
    fp.dmg.x1838_percentTemp = 1.0f; /* aggregate is deliberately irrelevant */
    if (script_caps_hit_threshold(&fp,5.0f,0.0f)) ++failed;
    if (!script_caps_threshold(4.0f, 30.0f, 5.0f, 0.0f)) ++failed;
    if (script_caps_threshold(5.0f, 30.0f, 5.0f, 0.0f)) ++failed;
    if (!script_caps_threshold(20.0f, 29.0f, 0.0f, 30.0f)) ++failed;
    if (script_caps_threshold(20.0f, 30.0f, 0.0f, 30.0f)) ++failed;
    if (script_caps_threshold(1.0f, 1.0f, 0.0f, 0.0f)) ++failed;
    if (!script_caps_effect_valid(0, 1.0f, 1)) ++failed;
    if (script_caps_effect_valid(3, 0.0f, 60)) ++failed;
    if (script_caps_effect_valid(3, 0.0f/0.0f, 60)) ++failed;
    if (script_caps_effect_valid(0, 1.0f, 0)) ++failed;
    if (script_caps_effect_valid(0, 2.0f, 60)) ++failed;
    if (script_caps_effect_valid(0, 1.0f, 3601)) ++failed;
    if (!script_caps_effect_valid(3, 4.0f, 3600)) ++failed;
    if (script_caps_holdable(It_Kind_Heart) || script_caps_holdable(It_Kind_Box)) ++failed;
    if (!script_caps_holdable(It_Kind_Sword) || !script_caps_holdable(It_Kind_M_Ball)) ++failed;
    return failed;
}
