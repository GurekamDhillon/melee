"""Source integration audit, not a substitute for native/GPU acceptance.

Checks that regression fixes are wired into the actual capture, warm, assembly,
Lua stats and retained-only draw paths. Run from the workspace root.
"""
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
def source(path):return (ROOT/path).read_text()

class MotionCaptureWiring(unittest.TestCase):
    def test_camera_fix_preserved(self):
        s=source('pc/gameworld/script_motion_draw.h')
        self.assertIn('camera!=Camera_PCMainCObj()',s)
        self.assertNotIn('camera!=cm_804D6464',s)
        world=source('src/melee/cm/camera.c')
        self.assertIn('MotionWorldDraw((int)&HSD_CObjGetCurrent()->view_mtx)',world)

    def test_sampler_fallback_and_fog(self):
        s=source('extern/aurora/lib/gx/motion_capture.hpp')
        self.assertIn('sampled_copy(original_info',s)
        self.assertIn('sampledIndTextures.test(i)',s)
        self.assertIn('const bool flat=textureless||copied_texture',s)
        self.assertIn('fogType=GX_FOG_NONE',s)
        self.assertIn('fogRangeEnabled=false',s)
        self.assertIn('const auto uniform=build_uniform(info)',s)
        self.assertNotIn('if(config.shaderConfig.fogRangeEnabled){capture->failed',s)

    def test_warm_variants_are_real(self):
        s=source('extern/aurora/lib/gx/motion_capture.hpp')
        self.assertIn('for(int flat=textureless?1:0;flat<2;++flat)',s)
        self.assertIn('pnPalette=!c.shaderConfig.pnPalette',s)
        self.assertIn('gfx::pipeline_warm_record(ref)',s)
        cp=source('extern/aurora/lib/gx/command_processor.cpp')
        self.assertIn('motion::warm(cache.config)',cp)
        self.assertIn('SurfaceBegin(index + 1)',source('pc/gameworld/script_warm.inc'))

    def test_item_and_all_passes_are_assembled(self):
        s=source('pc/platform/gw_fx_motion.cpp')
        self.assertIn('h.pending.draws.push_back(pose)',s)
        self.assertIn('h.poses.push(h.pending_frame,std::move(h.pending))',s)
        self.assertIn('GXAuroraMotionCallback(finish_callback',s)
        self.assertIn('gw_ScriptGame_MotionHeldDraw(draw_port-1,draw_sub)',s)
        self.assertNotIn('if(b.held){history.poses.clear()',s)
        cp=source('extern/aurora/lib/gx/command_processor.cpp')
        self.assertIn('if(!motion::suppress_live())gfx::push_draw_command(motionDraw)',cp)
        self.assertIn('!motion::active() && !motion::suppress_live()',cp)
        item=source('pc/gameworld/script_motion.inc')
        self.assertIn('item->render_cb(item,pass)',item)
        self.assertIn('HSD_CObjGetCurrent()==main',item)

    def test_diagnostics_and_rebasing(self):
        s=source('extern/aurora/lib/gx/motion_replay.inc')
        self.assertIn('relocate_array(original,block.old,fresh',s)
        self.assertIn('_pad=fresh.offset/4',s)
        self.assertIn('fail_once(sample,Failure(n),emit)',s)
        self.assertIn('for(size_t i=0;i<FailureCount;++i)',s)
        lua=source('pc/platform/gw_script_motion.inc')
        self.assertIn('gw_motion_stat_name((unsigned)k)',lua)
        self.assertIn('GW_MOTION_STATS_COUNT',lua)

if __name__=='__main__':unittest.main()
