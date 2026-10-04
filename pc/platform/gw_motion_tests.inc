// Real core fixtures for the headless suite; no GX device or game-state writes.
static int test_motion_history(void) {
    using namespace gw_motion;
    History<std::vector<uint8_t>,32> h;
    std::vector<uint8_t> pose{1,2,3,4};
    h.push(10,pose);pose[0]=99;
    if(!h.age(11,1)||h.age(11,1)->at(0)!=1){gw_test_fail("pose bytes were not retained");return 1;}
    if(h.push(10,pose)){gw_test_fail("pause duplicated pose");return 1;}
    h.push(9,pose);
    if(h.age(11,1)){gw_test_fail("rewind retained future pose");return 1;}
    for(int f=10;f<50;++f)h.push(f,pose);
    if(h.size()!=32||h.age(50,33)){gw_test_fail("history capacity");return 1;}
    h.clear();if(h.size()){gw_test_fail("history cleanup");return 1;}
    Matrix a=identity(),inverse_a;a[3]=5;a[7]=-3;a[0]=2;
    if(!inverse(a,inverse_a)){gw_test_fail("camera inverse");return 1;}
    auto product=multiply(a,inverse_a),id=identity();
    for(int k=0;k<12;++k)if(std::abs(product[k]-id[k])>1e-5f){gw_test_fail("camera reprojection");return 1;}
    return 0;
}
static int test_motion_contract(void) {
    GwMotionOptions o;gw_motion_defaults(&o,GW_MOTION_TRACER);
    if(!valid(o)){gw_test_fail("defaults refused");return 1;}
    o.width=NAN;if(valid(o)){gw_test_fail("nonfinite width");return 1;}
    gw_motion_defaults(&o,GW_MOTION_AFTERIMAGE);o.copies=GW_MOTION_MAX_COPIES+1;
    if(valid(o)){gw_test_fail("copy capacity");return 1;}
    if(gw_motion::fade(8,8,1)!=0){gw_test_fail("lifetime");return 1;}
    return 0;
}
static int test_motion_fallbacks(void){
    if(!aurora::gx::motion::contract_test()){gw_test_fail("motion material/fog/palette/counter contract");return 1;}
    GwMotionOptions o;gw_motion_defaults(&o,GW_MOTION_AFTERIMAGE);
    o.copies=12;o.lifetime=60;if(!valid(o)){gw_test_fail("expanded afterimage limits");return 1;}
    gw_motion_defaults(&o,GW_MOTION_TRACER);o.width=24;o.length=60;if(!valid(o)){gw_test_fail("expanded tracer limits");return 1;}
    for(unsigned i=0;i<GW_MOTION_STATS_COUNT;++i)if(!strcmp(gw_motion_stat_name(i),"invalid")){gw_test_fail("missing diagnostic stat name");return 1;}
    return 0;
}
extern "C" void gw_motion_register_tests(void){
    gw_test_register("motion_pose_history",test_motion_history);
    gw_test_register("motion_limits",test_motion_contract);
    gw_test_register("motion_capture_fallbacks",test_motion_fallbacks);
}
