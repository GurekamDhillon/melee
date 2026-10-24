mod state_tev_backend;
use melee_runtime::tu::state::*;
use state_tev_backend::Trace;

#[test]
fn repeated_channel_control_is_cached() {
    let mut render = HsdRender::new();
    let mut gpu = Trace::default();
    let mut ch = HsdChan::zeroed();
    ch.chan = 4;
    ch.amb_color = gxcolor([1, 2, 3, 4]);
    ch.mat_color = gxcolor([5, 6, 7, 8]);
    ch.enable = 1; ch.light_mask = 3; ch.diff_fn = 2; ch.attn_fn = 1;
    render.setup_channel(&mut gpu, &ch);
    gpu.words.clear();
    render.setup_channel(&mut gpu, &ch);
    // tev.c compares all six cached control fields before GXSetChanCtrl.
    assert!(gpu.words.is_empty(), "redundant GX calls: {:?}", gpu.words);
}
