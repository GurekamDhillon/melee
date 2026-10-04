/* Host presentation deadlines, never simulation state. Frames mean 1/60 second.
 * This timer does not alter manual pause/step state or resume another owner. */
#ifndef GW_PRESENTATION_STATE_H
#define GW_PRESENTATION_STATE_H
typedef struct { int owner, used; double start, end; } GwPresentationTimer;
static inline void gw_presentation_start(GwPresentationTimer* t,int owner,double now,int frames)
{ t->used=1;t->owner=owner;t->start=now;t->end=now+frames*(1000.0/60.0); }
static inline int gw_presentation_live(const GwPresentationTimer* t,double now)
{ return t->used && now<t->end; }
static inline double gw_presentation_progress(const GwPresentationTimer* t,double now)
{ double p=t->end>t->start ? (now-t->start)/(t->end-t->start):1;return p<0?0:p>1?1:p; }
static inline int gw_presentation_cancel(GwPresentationTimer* t,int owner)
{ if(!t->used || t->owner!=owner)return 0;t->used=0;return 1; }
#endif
