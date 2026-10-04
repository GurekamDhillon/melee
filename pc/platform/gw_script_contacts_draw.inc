/* Uses the exact safe-area canvas of gd.safe_area(), with no work when disabled. */
static void gs_contact_draw(void)
{
    int port,row=0;
    float width,bottom,left;
    if(!gs_contact_overlay || !gs.match_active)return;
    l_safe_area(gs.L);
    lua_getfield(gs.L,-1,"w");width=(float)lua_tonumber(gs.L,-1);lua_pop(gs.L,1);
    lua_getfield(gs.L,-1,"bottom");bottom=(float)lua_tonumber(gs.L,-1);lua_pop(gs.L,1);
    lua_getfield(gs.L,-1,"x");left=(float)lua_tonumber(gs.L,-1);lua_pop(gs.L,2);
    for(port=0;port<6;++port) {
        GsContactPicture p;GwScriptDraw* d;char text[256];
        if(gw_ScriptGame_FighterI(port,SI_SLOT_TYPE)!=0)continue; /* humans only */
        gs_contact_picture(port,&p);if(!p.present)continue;
        gs_contact_overlay_text(port,&p,text,sizeof text);
        d=gs_draw_new(GW_SDRAW_TEXT);
        if(d){d->x=left+12;d->y=bottom-18*(++row)-10;d->rgba=0xF7E7CDFFu;d->size=fminf(0.85f,(width-24)/(7.0f*fmaxf(1,(float)strlen(text))));snprintf(d->text,sizeof d->text,"%s",text);}
        {int side;for(side=0;side<5;++side)if(p.surface[side].present) {
            GsContactSurface* s=&p.surface[side];float ax,ay,bx,by,cx,cy,depth;
            if(gs_project(s->x0,s->y0,0,&ax,&ay,&depth) && gs_project(s->x1,s->y1,0,&bx,&by,&depth))
                gs_stage_draw_segment(ax,ay,bx,by,0x62DDD5FFu);
            if(gs_project(s->x0+(s->x1-s->x0)*s->t,s->y0+(s->y1-s->y0)*s->t,0,&cx,&cy,&depth)) {
                gs_stage_draw_segment(cx-3,cy,cx+3,cy,0xFFE095FFu);gs_stage_draw_segment(cx,cy-3,cx,cy+3,0xFFE095FFu);
            }
        }}
    }
}
