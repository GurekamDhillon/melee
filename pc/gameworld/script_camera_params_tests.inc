/* Written before implementation. Fake stage only; preserve the running fixture. */
int ScriptGame_CameraParamsTest(void)
{
    StageCameraInfo saved = stage_info.cam_info;
    ScriptCameraParams state = script_camera_params;
    int rc = 0;
    memset(&script_camera_params, 0, sizeof script_camera_params);
    stage_info.cam_info.cam_zoom_rate = 80;
    stage_info.cam_info.cam_max_depth = 1000;
    stage_info.cam_info.cam_vertical_tilt = 30;
    stage_info.cam_info.x20 = 0.05f;
    stage_info.cam_info.x24 = 0.05f;
    stage_info.cam_info.cam_pan_degrees = -10;
    if (!ScriptGame_CameraParams(1,1 << 7,0,0,0,0,0,0,0,script_mesh_bits(15),0,0) ||
        stage_info.cam_info.cam_pan_degrees!=15) rc++;
    if (!ScriptGame_CameraParams(1, (1 << 8) | (1 << 9), 0,0,0,0,0,0,0,0,0,0)) rc++;
    if (stage_info.cam_info.x20 != 0 || stage_info.cam_info.x24 != 0 ||
        stage_info.cam_info.cam_zoom_rate != 80) rc++;
    if (ScriptGame_CameraParams(2, 0, 0,0,0,0,0,0,0,0,0,0)) rc++;
    if (ScriptGame_CameraParams(1, 3, script_mesh_bits(200),script_mesh_bits(100),0,0,0,0,0,0,0,0)) rc++;
    if (stage_info.cam_info.cam_zoom_rate != 80) rc++;
    if (!ScriptGame_CameraParams(1, 1 << 2, 0,0,script_mesh_bits(25),0,0,0,0,0,0,0)) rc++;
    if (ScriptGame_CameraFov(30) != 25) rc++;
    if (!ScriptGame_CameraParams(1,1 << 6,0,0,0,0,0,0,script_mesh_bits(20),0,0,0) ||
        ScriptGame_CameraFov(30)!=20 || stage_info.cam_info.cam_vertical_tilt!=20) rc++;
    if (ScriptGame_CameraParams(1,(1 << 2)|(1 << 6),0,0,script_mesh_bits(25),0,0,0,script_mesh_bits(20),0,0,0)) rc++;
    script_camera_params_release(2);
    if (ScriptGame_CameraFov(30) != 20) rc++;
    script_camera_params_release(1);
    if (stage_info.cam_info.x20 != 0.05f || stage_info.cam_info.x24 != 0.05f ||
        stage_info.cam_info.cam_pan_degrees != -10 ||
        stage_info.cam_info.cam_vertical_tilt != 30 || ScriptGame_CameraFov(30) != 30) rc++;
    stage_info.cam_info = saved;
    script_camera_params = state;
    return rc;
}
