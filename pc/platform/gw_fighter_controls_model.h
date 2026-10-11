/* Pure original-input mapping. No profile, registry, or game-memory access. */
#ifndef GW_FIGHTER_CONTROLS_MODEL_H
#define GW_FIGHTER_CONTROLS_MODEL_H
#include "gw_fighter_controls.h"
#include <string.h>

enum { GW_FC_BUTTONS=12, GW_FC_MAX_LAYOUTS=16, GW_FC_MAX_EXTRAS=8, GW_FC_NAME=32 };
#define GW_FC_BUTTON_MASK 0x1f7fu
static const unsigned short gw_fc_bits[GW_FC_BUTTONS] = {
    0x100,0x200,0x400,0x800,0x10,0x1000,0x40,0x20,8,4,1,2
};
static const char* const gw_fc_labels[GW_FC_BUTTONS] = {
    "A","B","X","Y","Z","START","L","R","Up","Down","Left","Right"
};
typedef struct GwFighterControlsMap {
    char name[GW_FC_NAME];
    unsigned specified;
    unsigned short dest[GW_FC_BUTTONS];
} GwFighterControlsMap;
typedef struct GwFighterControlsSchema {
    int layouts, extras;
    GwFighterControlsMap layout[GW_FC_MAX_LAYOUTS], extra[GW_FC_MAX_EXTRAS];
} GwFighterControlsSchema;

/* The registry lends this only while freezing a match or reading menu options. */
const GwFighterControlsSchema* gw_Geno_ControlsSchema(int ck);

static unsigned gw_fc_normalize(const GwFighterControlsSchema* s, unsigned choice)
{
    unsigned jump=choice&GW_FC_JUMP_MASK, layout=(choice&GW_FC_LAYOUT_MASK)>>2;
    unsigned extras=s && s->extras>0 ? (1u<<s->extras)-1u : 0u;
    if(jump>GW_FC_JUMP_Y) jump=GW_FC_JUMP_BOTH;
    if(!s || layout>=(unsigned)s->layouts) layout=0;
    return jump|(layout<<2)|(choice&(extras<<8));
}
static void gw_fc_overlay(unsigned short out[GW_FC_BUTTONS], const GwFighterControlsMap* m)
{
    int i;
    for(i=0;i<GW_FC_BUTTONS;++i) if(m->specified&(1u<<i)) out[i]=m->dest[i];
}
static void gw_fc_build(const GwFighterControlsSchema* s, unsigned choice,
                        unsigned short out[GW_FC_BUTTONS])
{
    unsigned jump, layout;
    int i;
    choice=gw_fc_normalize(s,choice);
    jump=choice&GW_FC_JUMP_MASK; layout=(choice&GW_FC_LAYOUT_MASK)>>2;
    memcpy(out,gw_fc_bits,sizeof gw_fc_bits);
    if(s && s->layouts>0) gw_fc_overlay(out,&s->layout[layout]);
    if(s) for(i=0;i<s->extras;++i) if(choice&(1u<<(8+i))) gw_fc_overlay(out,&s->extra[i]);
    /* Jump selection gates only these original sources' jump destinations.
     * Y->B still works with X jump; an explicit fighter A->X can still jump. */
    if(jump==GW_FC_JUMP_X) out[3]&=~0xc00u;
    if(jump==GW_FC_JUMP_Y) out[2]&=~0xc00u;
}
static int gw_fc_apply(const unsigned short map[GW_FC_BUTTONS], int buttons)
{
    unsigned raw=(unsigned)buttons, out=raw&~GW_FC_BUTTON_MASK;
    int i;
    /* Read raw, never out: X->Y,Y->A cannot recursively emit A from X. */
    for(i=0;i<GW_FC_BUTTONS;++i) if(raw&gw_fc_bits[i]) out|=map[i];
    return (int)out;
}
#endif
