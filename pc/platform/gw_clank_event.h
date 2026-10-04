/* Scalar observation of a resolved hitbox clash; no simulation-owned state. */
#ifndef GW_CLANK_EVENT_H
#define GW_CLANK_EVENT_H
#define GS_EV_CLANK 11
typedef struct {
    int port_a,port_b,item,item_kind,cancel_a,cancel_b,entity_a,entity_b;
    int rebound_a,rebound_b,finalized;
    float x,y,z,damage_a,damage_b,hitlag_a,hitlag_b;
} GsClank;
#endif
