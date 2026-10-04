extern int gw_ScriptGame_CameraParamsTest(void);
static int test_script_camera_params(void) {
    char out[2048];
    int saved_active=gs.match_active, rc;
    if (gw_ScriptGame_CameraParamsTest()) {
        gw_test_fail("camera params partial update, owner, ranges or restore fixture failed"); return 1;
    }
    gs.match_active=1;
    rc=t_exec("assert(type(gd.camera_params)=='function'); "
               "local function reject(v,s) local ok,e=pcall(gd.camera_params,v); "
               "assert(not ok and tostring(e):find(s,1,true),tostring(e)) end; "
               "reject({fov=0},'fov must be'); reject({fov=0/0},'fov must be'); "
               "reject({yaw_gain=math.huge},'yaw_gain must be'); "
               "reject({min_dist=200,max_depth=100},'min_dist must not exceed'); "
               "reject({fov=25,tilt=30},'FOV alias'); reject({typo=1},'unknown camera')", out, sizeof out);
    gs.match_active=saved_active;
    if (rc) {
        gw_test_fail("camera params Lua validation: %s", out); return 1;
    }
    return 0;
}
