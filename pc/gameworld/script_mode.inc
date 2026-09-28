/* Included in script_game.c: its BSS is already in gw_snap.c's game set.
 * Like ScriptGame_ModelField, access is scalar; no native pointer enters MEM1. */
static unsigned int script_mode[SCRIPT_MODE_SLOTS][SCRIPT_MODE_WORDS];

int ScriptGame_ModeRead(int slot, int word)
{
    if (slot < 0 || slot >= SCRIPT_MODE_SLOTS || word < 0 ||
        word >= SCRIPT_MODE_WORDS) return 0;
    return (int) script_mode[slot][word];
}

void ScriptGame_ModeWrite(int slot, int word, int value)
{
    if (slot < 0 || slot >= SCRIPT_MODE_SLOTS || word < 0 ||
        word >= SCRIPT_MODE_WORDS) return;
    script_mode[slot][word] = (unsigned int) value;
}

int ScriptGame_ModeActive(void)
{
    int i;
    for (i = 0; i < SCRIPT_MODE_SLOTS; ++i) {
        if (script_mode[i][0]) return 1;
    }
    return 0;
}
