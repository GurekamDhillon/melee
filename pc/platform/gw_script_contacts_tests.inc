extern int gw_ScriptGame_ContactsTest(void);
/* Fake pictures exercise the same observer and watcher used by FramePost. */
static int test_script_contacts(void) {
    GsContactPicture a = {0}, b;
    char out[2048];
    GsContactSurface s = {0};
    GsContactWait w = {0};
    char text[256];
    int i, failed = 0;
#define CONTACT_CHECK(v) do { if (!(v)) { gw_test_fail("contact fixture line %d", __LINE__); ++failed; } } while (0)
    CONTACT_CHECK(gw_ScriptGame_ContactsTest()==0);
    CONTACT_CHECK(t_exec("assert(type(gd.contacts)=='function');assert(type(gd.contact_events)=='function');"
        "assert(not pcall(gd.contacts,0));assert(not pcall(gd.contacts,4294967297));"
        "assert(not pcall(gd.wait_until,{port=4294967297,x=1}));"
        "assert(not pcall(gd.wait_until,{x=0/0}));assert(not pcall(gd.wait_until,{timeout=0,x=1}));"
        "assert(not pcall(gd.wait_until,{state='',x=1}));assert(not pcall(gd.wait_until,{typo=1}));"
        "assert(not pcall(gd.wait_until,{}));assert(not pcall(gd.contact_overlay,1));"
        "assert(not pcall(gd.contact_events,-1))",out,sizeof out)==0);
    CONTACT_CHECK(gs_contact_label_put(87654,9,"Floor_4m.003"));
    CONTACT_CHECK(!strcmp(gs_contact_label(87654),"Floor_4m.003"));
    CONTACT_CHECK(!gs_contact_label_put(87654,8,"wrong owner"));
    CONTACT_CHECK(!strcmp(gs_contact_label(0),""));
    gs_contacts_reset("test reset");CONTACT_CHECK(!strcmp(gs_contact_label(87654),"Floor_4m.003"));
    memset(gs_contact_labels,0,sizeof gs_contact_labels);
    s.present=1; s.line=7; s.owner=123; s.part=2;
    s.x0=10; s.y0=0; s.x1=-10; s.y1=0;
    gs_contact_project(&s, 0, 0);
    CONTACT_CHECK(s.x0==-10 && s.offset==10 && s.t==0.5f);
    s.x0=0; s.y0=0; s.x1=3; s.y1=4;
    gs_contact_project(&s, 1.5f, 2);
    CONTACT_CHECK(s.offset==2.5f && s.t==0.5f);
    s.x0=0; s.y0=10; s.x1=0; s.y1=0;
    gs_contact_project(&s, 0, 4);
    CONTACT_CHECK(s.y0==0 && s.offset==4 && s.t==0.4f);
    gs_contacts_reset("test reset");
    a.present=1; a.state=14; strcpy(a.state_name,"Wait"); a.surface[0]=s;
    gs_contact_observe(0,&a,1);
    CONTACT_CHECK(gs_contact_count==1 && !strcmp(gs_contact_ring[0].kind,"land"));
    gs_contact_observe(0,&a,2); CONTACT_CHECK(gs_contact_count==1);
    b=a; b.airborne=1; b.surface[0].present=0; b.jump=1;
    gs_contact_observe(0,&b,3);
    CONTACT_CHECK(!strcmp(gs_contact_ring[1].kind,"leave") && !strcmp(gs_contact_ring[1].reason,"jumped"));
    b=a; b.surface[0].present=0; b.x=-1; a.surface[0].x0=0;
    CONTACT_CHECK(!strcmp(gs_contact_leave(&a,&b),"ran_off_left"));
    b.x=11; a.surface[0].x1=10; CONTACT_CHECK(!strcmp(gs_contact_leave(&a,&b),"ran_off_right"));
    b.pass=1; CONTACT_CHECK(!strcmp(gs_contact_leave(&a,&b),"dropped_through"));
    b.pass=0; b.knocked=1; CONTACT_CHECK(!strcmp(gs_contact_leave(&a,&b),"knocked"));
    gs_contacts_reset("test reset");
    a.surface[0].present=0; a.surface[1]=s; a.surface[3]=s; a.surface[4]=s;
    gs_contact_observe(0,&a,4);
    CONTACT_CHECK(gs_contact_count==3);
    b=a; b.surface[1].present=b.surface[3].present=b.surface[4].present=0;
    gs_contact_observe(0,&b,5);
    CONTACT_CHECK(gs_contact_count==5); /* ceiling release is intentionally not an event */
    b.dead=1; gs_contact_observe(0,&b,6); CONTACT_CHECK(!strcmp(gs_contact_ring[5].kind,"ko"));
    b.dead=0; b.rebirth=1; gs_contact_observe(0,&b,7); CONTACT_CHECK(!strcmp(gs_contact_ring[6].kind,"respawn"));
    gs_contact_state_changes=1; b.state=15; gs_contact_observe(0,&b,8);
    CONTACT_CHECK(!strcmp(gs_contact_ring[7].kind,"state_change")); gs_contact_state_changes=0;
    for(i=0;i<GS_CONTACT_CAP+9;++i) gs_contact_emit("land","",0,&a,&s,i);
    CONTACT_CHECK(gs_contact_count==GS_CONTACT_CAP && gs_contact_dropped==17);
    CONTACT_CHECK(t_exec("local e,n,d=gd.contact_events(0);assert(#e==512 and d==17);"
        "assert(e[512].seq-e[1].seq==511);local empty,next_seq=gd.contact_events(n);"
        "assert(#empty==0 and next_seq==n);assert(e[1].contact.owner==123)",out,sizeof out)==0);
    w.used=1; w.timeout=2; w.has_x=1; w.x=20; w.radius=1;
    a.x=20; CONTACT_CHECK(gs_contact_matches(&w,&a));
    w.has_state=1; strcpy(w.state,"Fall"); CONTACT_CHECK(!gs_contact_matches(&w,&a));
    strcpy(w.state,"Wait"); CONTACT_CHECK(gs_contact_matches(&w,&a));
    w.has_on=1; w.on=123; a.surface[0]=s; CONTACT_CHECK(gs_contact_matches(&w,&a));
    w.on=124; CONTACT_CHECK(!gs_contact_matches(&w,&a));
    w.has_on=0; w.has_air=1; w.airborne=1; CONTACT_CHECK(!gs_contact_matches(&w,&a));
    {GsContactWait initial={0};GsContactPicture initial_picture=a;
     initial_picture.airborne=1;gs_contact_wait_seed(&initial,&initial_picture);
     CONTACT_CHECK(initial.saw_air && initial.last_on.present);
     initial.has_air=1;initial.airborne=1;initial_picture.airborne=0;initial_picture.surface[0].present=0;
     gs_contact_failure(&initial,&initial_picture,text,sizeof text);CONTACT_CHECK(strcmp(text,"never left the ground")!=0);}
    w.saw_air=0; gs_contact_failure(&w,&a,text,sizeof text);
    CONTACT_CHECK(!strcmp(text,"never left the ground"));
    w.has_air=0; a.surface[1]=s; strcpy(a.surface[1].label,"Wall_Solid.014");
    gs_contact_failure(&w,&a,text,sizeof text); CONTACT_CHECK(strstr(text,"stopped at wall \"Wall_Solid.014\" (left side)")!=NULL);
    a.surface[1].present=0;a.surface[2]=s;strcpy(a.surface[2].label,"Wall_Right");
    gs_contact_failure(&w,&a,text,sizeof text);CONTACT_CHECK(strstr(text,"(right side)")!=NULL);
    a.surface[2].present=0; a.surface[0]=s; strcpy(a.surface[0].label,"Ramp.002");
    gs_contact_failure(&w,&a,text,sizeof text); CONTACT_CHECK(strstr(text,"last on \"Ramp.002\"")!=NULL);
    a.surface[0].present=0;w.last_on=s;strcpy(w.last_on.label,"Ramp.002");
    gs_contact_failure(&w,&a,text,sizeof text);CONTACT_CHECK(strstr(text,"last on \"Ramp.002\"")!=NULL);
    w.last_on.present=0;gs_contact_failure(&w,&a,text,sizeof text);CONTACT_CHECK(strstr(text,"last at x=")!=NULL);
    a.surface[0]=s;strcpy(a.surface[0].label,"Ramp.002");
    strcpy(w.drop_label,"Floor_4m.003"); gs_contact_failure(&w,&a,text,sizeof text);
    CONTACT_CHECK(!strcmp(text,"fell through \"Floor_4m.003\""));
    gs_contact_overlay_text(0,&a,text,sizeof text); CONTACT_CHECK(strstr(text,"P1 Wait  on Ramp.002")!=NULL);
    {GsContactEvent e={0};e.port=1;e.fighter=a;e.surface=s;strcpy(e.kind,"land");
     gs_contact_event_text(&e,text,sizeof text);CONTACT_CHECK(strstr(text,"P1 landed on")!=NULL);}
    gs_contacts_reset("test reset");
    a.surface[0]=s; a.surface[0].passthrough=1;
    a.surface[1].present=a.surface[2].present=a.surface[3].present=a.surface[4].present=0;
    gs_contact_observe(0,&a,10);
    b=a;b.surface[0].present=0;b.pass=1;
    gs_contact_observe(0,&b,11);
    CONTACT_CHECK(gs_contact_count==3 && !strcmp(gs_contact_ring[2].kind,"drop_through"));
    gs_contact_observe(0,&b,12);CONTACT_CHECK(gs_contact_count==3);
    b.surface[2]=s;gs_contact_observe(0,&b,13);
    CONTACT_CHECK(!strcmp(gs_contact_ring[3].kind,"wall_touch"));
    gs_contact_observe(0,&b,14);CONTACT_CHECK(gs_contact_count==4);
    b.surface[2].present=0;gs_contact_observe(0,&b,15);
    CONTACT_CHECK(!strcmp(gs_contact_ring[4].kind,"wall_release"));
    b.state=16;gs_contact_observe(0,&b,16);CONTACT_CHECK(gs_contact_count==5); /* off by default */
    memset(&gs_contact_waits[0],0,sizeof gs_contact_waits[0]);
    gs_contact_waits[0].used=1;gs_contact_waits[0].has_x=1;gs_contact_waits[0].x=21;
    gs_contact_waits[0].radius=0.1f;gs_contact_waits[0].timeout=2;
    gs_contact_wait_tick(0,&a);CONTACT_CHECK(!gs_contact_waits[0].done && gs_contact_waits[0].frames==1);
    a.x=21;gs_contact_wait_tick(0,&a);CONTACT_CHECK(gs_contact_waits[0].done && gs_contact_waits[0].ok);
    gs_contact_waits[0].done=gs_contact_waits[0].ok=gs_contact_waits[0].frames=0;
    a.x=30;gs_contact_wait_tick(0,&a);gs_contact_wait_tick(0,&a);
    CONTACT_CHECK(gs_contact_waits[0].done && !gs_contact_waits[0].ok && strstr(gs_contact_waits[0].reason,"timeout 2 frames"));
    gs_contact_waits[0].done=0;strcpy(gs_contact_waits[0].drop_label,"old drop");
    gs_contact_emit("land","",0,&a,&s,17);CONTACT_CHECK(!gs_contact_waits[0].drop_label[0]);
    {uint64_t before=gs_contact_next;gs_contacts_reset("state loaded or rewound; watcher cancelled");
     CONTACT_CHECK(gs_contact_next>before && !gs_contact_count && gs_contact_waits[0].done && !gs_contact_waits[0].ok);}
    memset(&gs_contact_waits[0],0,sizeof gs_contact_waits[0]);
    {FILE* f=tmpfile();CONTACT_CHECK(f!=NULL);if(f) {
        gs_contact_json_string(f,"Ramp\"\\\n");fflush(f);rewind(f);
        memset(text,0,sizeof text);fread(text,1,sizeof text-1,f);fclose(f);
        CONTACT_CHECK(!strcmp(text,"\"Ramp\\\"\\\\\\u000a\""));
    }}
    gs_contacts_reset("test reset");
#ifdef _WIN32
    /* Rotation uses a unique fixture-owned path, never a user's trace file. */
    {char dir[MAX_PATH],path[MAX_PATH],rotated[MAX_PATH+4];GsContactEvent e={0};FILE* f;
     CONTACT_CHECK(gs_contact_file==NULL);
     if(!gs_contact_file && GetTempPathA(sizeof dir,dir) && GetTempFileNameA(dir,"gct",0,path)) {
        snprintf(rotated,sizeof rotated,"%s.1",path);
        snprintf(gs_contact_path,sizeof gs_contact_path,"%s",path);
        gs_contact_file=fopen(path,"wb");CONTACT_CHECK(gs_contact_file!=NULL);
        if(gs_contact_file) {
            fputs("fixture seed\n",gs_contact_file);fflush(gs_contact_file);
            gs_contact_bytes=GS_CONTACT_TRACE_MAX;
            strcpy(e.kind,"land");e.fighter=a;e.surface=s;e.seq=123;
            gs_contact_trace_event(&e);
            CONTACT_CHECK(gs_contact_file!=NULL && gs_contact_bytes<GS_CONTACT_TRACE_MAX);
            f=fopen(rotated,"rb");CONTACT_CHECK(f!=NULL);if(f){memset(text,0,sizeof text);fread(text,1,sizeof text-1,f);fclose(f);CONTACT_CHECK(!strcmp(text,"fixture seed\n"));}
            gs_contact_bytes=GS_CONTACT_TRACE_MAX;gs_contact_trace_event(&e);
            f=fopen(rotated,"rb");CONTACT_CHECK(f!=NULL);if(f){memset(text,0,sizeof text);fread(text,1,sizeof text-1,f);fclose(f);CONTACT_CHECK(strstr(text,"\"seq\":123")!=NULL);}
            gs_contact_trace_close();
        }
        remove(path);remove(rotated);gs_contact_path[0]=0;gs_contact_bytes=0;
     }else CONTACT_CHECK(0);
    }
#endif
#undef CONTACT_CHECK
    return failed ? 1 : 0;
}
