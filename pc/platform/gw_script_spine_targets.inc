/* Explicit compatibility allowlist. These are the existing implementations,
 * so offline/gameplay ownership gates and result signatures are retained. */
static lua_CFunction gs_spine_call_target(const char* name)
{
    static const luaL_Reg targets[]={
        {"fighter_caps",l_fighter_caps},{"fighter_effect",l_fighter_effect},
        {"fighter_armour",l_fighter_armour},{"fighter_armor",l_fighter_armor},
        {"fighter_timed_status",l_fighter_timed_status},{"give_item",l_give_item},
        {"nearest_opponent",l_nearest_opponent},{"opponents_in_radius",l_opponents_in_radius},
        {"fighter_status",l_fighter_status},{"hit_rule_add",l_hit_rule_add},
        {"hit_rules",l_hit_rules},{"hit_rules_clear",l_hit_rules_clear},
        {"fighter_history",l_fighter_history},{"echo_add",l_echo_add},
        {"echoes",l_echoes},{"zones_at",l_zones_at},{NULL,NULL}};
    int i;for(i=0;targets[i].name;++i)if(!strcmp(name,targets[i].name))return targets[i].func;
    return NULL;
}
