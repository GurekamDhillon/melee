#ifndef GW_SCRIPT_SPINE_EVENT_H
#define GW_SCRIPT_SPINE_EVENT_H
/* Observation metadata only, never read by simulation or snapshot authority. */
typedef struct {int frame,phase;unsigned epoch,sequence,cause;char actor[80],target[80];} GsSpineEvent;
#endif
