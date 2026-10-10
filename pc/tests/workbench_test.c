/* Pure test of the workbench gate, frame JSON and camera orbit math. No game link. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "gw_workbench.h"

static int netplay_stub;
int gw_Wb_NetplayActive(void) { return netplay_stub; }

/* Resolving by joint id must survive compacted/sparse bone arrays. Orbit
 * conversion must keep the target fixed and scale the eye offset with distance. */
static void test_camera_orbit(void)
{
    GwWbFrame f, before;
    float eye[3], interest[3];
    memset(&f, 0, sizeof f);
    f.nbones = 2;
    f.bones[0].joint = 7;
    f.bones[0].x = 10; f.bones[0].y = 20; f.bones[0].z = 30;
    f.bones[1].joint = 42;
    f.bones[1].x = 14; f.bones[1].y = 28; f.bones[1].z = 38;
    before = f;
    assert(gw_Wb_CameraOrbit(&f, 7, 0, 0, 60, eye, interest) == 0);
    assert(interest[0] == 10 && interest[1] == 20 && interest[2] == 30);
    assert(fabsf(eye[0] - 10) < 0.001f && fabsf(eye[1] - 20) < 0.001f && fabsf(eye[2] - 90) < 0.001f);
    assert(gw_Wb_CameraOrbit(&f, 42, 90, 0, 30, eye, interest) == 0);
    assert(interest[0] == 14 && interest[1] == 28 && interest[2] == 38);
    assert(fabsf(eye[0] - 44) < 0.001f && fabsf(eye[1] - 28) < 0.001f && fabsf(eye[2] - 38) < 0.001f);
    assert(gw_Wb_CameraOrbit(&f, -1, 45, 15, 60, eye, interest) == 0);
    assert(interest[0] == 12 && interest[1] == 24 && interest[2] == 34);
    assert(fabsf(eye[0] - 52.980762f) < 0.001f && fabsf(eye[1] - 39.529143f) < 0.001f && fabsf(eye[2] - 74.980762f) < 0.001f);
    /* A vertical view must remain non-collinear with HSD's Y-up vector. */
    assert(gw_Wb_CameraOrbit(&f, 7, 0, 90, 60, eye, interest) == 0);
    assert(eye[1] > 79.99f && eye[2] > 30.01f);
    assert(gw_Wb_CameraOrbit(&f, 7, 0, -90, 60, eye, interest) == 0);
    assert(eye[1] < -39.99f && eye[2] > 30.01f);
    assert(gw_Wb_CameraOrbit(&f, 0, 0, 0, 60, eye, interest) == -1);
    assert(gw_Wb_CameraOrbit(&f, -2, 0, 0, 60, eye, interest) == -1);
    assert(gw_Wb_CameraOrbit(&f, 7, 0, 0, 0, eye, interest) == -1);
    assert(gw_Wb_CameraOrbit(&f, 7, NAN, 0, 60, eye, interest) == -1);
    assert(gw_Wb_CameraOrbit(&f, 7, 0, INFINITY, 60, eye, interest) == -1);
    assert(gw_Wb_CameraOrbit(&f, 7, 0, 0, INFINITY, eye, interest) == -1);
    assert(memcmp(&f, &before, sizeof f) == 0);
    f.nbones = 0;
    assert(gw_Wb_CameraOrbit(&f, -1, 0, 0, 60, eye, interest) == -1);
}

/* A lifted fighter's floor must be inside the default frustum, with room for
 * the stage edge. A 30-degree lens at distance 60 clips a floor 40 units below. */
static void test_camera_floor_fov(void)
{
    float eye[3] = {-60, 40, 60}, interest[3] = {-60, 40, 0};
    float fov = gw_Wb_CameraFloorFov(0, eye, interest, 30);
    assert(fov > 67 && fov < 75);
    assert(40 / (60 * tanf(fov * 0.00872664626f)) < 0.95f);
    eye[0] = 0; eye[2] = 0; /* side has the same vertical framing */
    assert(fabsf(gw_Wb_CameraFloorFov(0, eye, interest, 30) - fov) < 0.001f);
    assert(gw_Wb_CameraFloorFov(0, eye, interest, 90) == 90);
    eye[1] = interest[1] = 5;
    assert(gw_Wb_CameraFloorFov(0, eye, interest, 30) == 30);
    eye[1] = interest[1] = 61; /* distant floor does not shrink the fighter */
    assert(gw_Wb_CameraFloorFov(0, eye, interest, 30) == 30);
    eye[1] = interest[1] = 60;
    assert(gw_Wb_CameraFloorFov(0, eye, interest, 30) > 90);
    eye[1] = interest[1] = 140; /* stage height is not necessarily zero */
    assert(fabsf(gw_Wb_CameraFloorFov(100, eye, interest, 30) - fov) < 0.001f);
    eye[0] = -60; eye[1] = 100; eye[2] = 0.1f; interest[1] = 40;
    assert(gw_Wb_CameraFloorFov(0, eye, interest, 30) == 30); /* top */
}

int main(int argc, char **argv)
{
    char out[4096];
    GwWbFrame f;
    test_camera_orbit();
    test_camera_floor_fov();
#ifdef _WIN32
    _putenv("MELEE_WB_DIR=");
#else
    unsetenv("MELEE_WB_DIR");
#endif
    assert(!gw_Wb_Enabled());
    gw_Wb_SetEnabled(1);
    assert(gw_Wb_Enabled());
    netplay_stub = 1;
    assert(!gw_Wb_Enabled());
    netplay_stub = 0;
    gw_Wb_SetEnabled(0);
    assert(!gw_Wb_Enabled());

    memset(&f, 0, sizeof f);
    f.motion = 70; f.frame = 6; f.nbones = 1; f.nhits = 1;
    f.bones[0].joint = 3; f.bones[0].parent = 1;
    f.bones[0].x = 1.5f; f.bones[0].y = 2.0f; f.bones[0].z = -0.25f;
    f.hits[0].slot = 0; f.hits[0].joint = 3; f.hits[0].active = 1;
    f.hits[0].radius = 2.8f; f.hits[0].wy = 4.0f;
    f.lowest_y = -0.5f;
    assert(gw_Wb_FrameJson(&f, "dair", out, sizeof out) > 0);
    assert(strstr(out, "\"row\":\"dair\""));
    assert(strstr(out, "\"motion\":70"));
    assert(strstr(out, "\"joint\":3"));
    assert(strstr(out, "\"active\":true"));
    assert(strstr(out, "\"lowestY\":-0.5"));
    assert(strstr(out, "\"pos\":{\"x\":1.5,\"y\":2,\"z\":-0.25}"));
    assert(strstr(out, "\"world\":{\"x\":0,\"y\":4,\"z\":0}"));
    assert(strstr(out, "\"hurtboxes\":[]"));
    assert(strstr(out, "\"ecb\":{\"bottom\":{\"x\":0,\"y\":0},\"top\":{\"x\":0,\"y\":0}}"));
    assert(gw_Wb_FrameJson(&f, "dair", out, 20) == -1);
    assert(gw_Wb_FrameJson(&f, "a\"b", out, sizeof out) > 0 && strstr(out, "a\\\"b"));
    assert(gw_Wb_FrameJson(&f, "a\\b\n", out, sizeof out) > 0 && strstr(out, "a\\\\b\\n"));
    f.nhurts = 1; f.hurts[0].joint = 7; f.hurts[0].ay = 2;
    f.hurts[0].by = 3; f.hurts[0].radius = 0.5f;
    f.hits[0].active = 0;
    assert(gw_Wb_FrameJson(&f, "hurt", out, sizeof out) > 0);
    assert(strstr(out, "\"active\":false"));
    assert(strstr(out, "\"hurtboxes\":[{\"joint\":7,\"a\":{\"x\":0,\"y\":2,\"z\":0},\"b\":{\"x\":0,\"y\":3,\"z\":0},\"radius\":0.5}]"));
    { char p[256]; assert(gw_Wb_WritePayload("x.json", "{}", 2, p, sizeof p) != 0); }
    /* An optional scratch directory exercises publication on the host filesystem,
     * including replacing frame.json on Windows (CRT rename alone cannot do it). */
    if (argc > 1) {
        char env[1024], path[1024], previous[1024], text[4096];
        FILE *file;
        int length;
        assert(snprintf(env, sizeof env, "MELEE_WB_DIR=%s", argv[1]) < (int)sizeof env);
#ifdef _WIN32
        assert(_putenv(env) == 0);
#else
        assert(setenv("MELEE_WB_DIR", argv[1], 1) == 0);
#endif
        length = gw_Wb_FrameJson(&f, "hurt", out, sizeof out);
        assert(length > 0);
        assert(gw_Wb_WritePayload("frame.json", out, (size_t)length, path, sizeof path) == 0);
        strcpy(previous, path);
        assert(gw_Wb_WritePayload("frame.json", "{\"frame\":9}", 11, path, sizeof path) == 0);
        assert(strcmp(path, previous) == 0);
        file = fopen(path, "rb");
        assert(file != NULL);
        assert(fread(text, 1, sizeof text, file) == 11);
        assert(fclose(file) == 0);
        assert(memcmp(text, "{\"frame\":9}", 11) == 0);
        /* Failed publication does not replace the previous complete payload. */
        assert(gw_Wb_WritePayload("frame.json", "{}", 2, path, 3) != 0);
        file = fopen(previous, "rb");
        assert(file && fread(text, 1, sizeof text, file) == 11);
        assert(fclose(file) == 0);
        assert(memcmp(text, "{\"frame\":9}", 11) == 0);
        assert(remove(previous) == 0);
    }
    puts("workbench native test OK");
    return 0;
}
