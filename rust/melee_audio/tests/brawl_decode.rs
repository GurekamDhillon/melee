//! Decodes Brawl audio from the local Brawl folder (MELEE_BRAWL_LOCAL or `_build/brawl` walking up). Skipped (passes
//! with a note) when the files are absent: nothing from the disc is in the repo.

use melee_audio::brawl::reader::{decode_rstm, sound_dir, Rsar, ARCHIVE};

/// rms of the sample-to-sample difference over the rms: well under 1 for real audio, about 1.4 for noise.
fn rough(s: &[i16], ch: usize) -> f64 {
    let d: Vec<i16> = s.iter().zip(s.iter().skip(ch)).map(|(&a, &b)| ((b as i32 - a as i32) / 2) as i16).collect();
    2.0 * rms(&d) / rms(s).max(1.0)
}

fn rms(s: &[i16]) -> f64 {
    (s.iter().map(|&x| (x as f64).powi(2)).sum::<f64>() / s.len().max(1) as f64).sqrt()
}

#[test]
fn brawl_boss_stream_and_sfx_decode() {
    let Some(dir) = sound_dir() else {
        eprintln!("skipped: no local Brawl sound folder");
        return;
    };
    let a = Rsar::open(&dir.join(ARCHIVE)).expect("archive");
    let streams = a.sounds.iter().filter(|s| s.kind == 2).count();
    let waves = a.sounds.iter().filter(|s| s.kind == 3).count();
    eprintln!("sounds {} (streams {streams}, wave sounds {waves})", a.sounds.len());
    assert!(streams > 100 && waves > 1000);

    // Petey's boss fight: the archive's boss battle stream
    let s = a.find("snd_bgm_X18_BOSSBATTLE").expect("boss battle stream");
    let rel = a.stream_file(s).expect("stream file");
    assert!(rel.ends_with(".brstm"));
    let p = decode_rstm(&std::fs::read(dir.join(rel.trim_start_matches('/'))).unwrap()).expect("decode");
    eprintln!("boss stream: {} ch, {} Hz, {} frames, loop {:?}, rms {:.0}", p.channels, p.rate, p.frames(), p.loop_start, rms(&p.samples));
    assert_eq!(p.channels, 2);
    assert!(p.frames() > p.rate as usize * 30);
    assert!(p.loop_start.is_some());
    assert!(rms(&p.samples) > 500.0);
    assert!(rough(&p.samples, 2) < 0.7, "stream decodes to noise");

    // a Petey sound effect from the boss's group
    let e = a.find("snd_se_boss_Bosspackun_00").expect("petey sfx");
    let w = a.decode_wave_sound(e).expect("sfx decode");
    eprintln!("petey sfx 00: {} ch, {} Hz, {} frames, rms {:.0}", w.channels, w.rate, w.frames(), rms(&w.samples));
    eprintln!("roughness: stream {:.2}, sfx {:.2}", rough(&p.samples, 2), rough(&w.samples, w.channels));
    assert!(w.frames() > 100 && rms(&w.samples) > 100.0);
    assert!(rough(&w.samples, w.channels) < 0.9, "sfx decodes to noise");

    // a sample of all wave sounds decode
    let mut ok = 0;
    let mut tried = 0;
    for s in a.sounds.iter().filter(|s| s.kind == 3).step_by(97) {
        tried += 1;
        if a.decode_wave_sound(s).map(|p| p.frames() > 0 && (rms(&p.samples) < 1.0 || rough(&p.samples, p.channels) < 1.0)).unwrap_or(false) {
            ok += 1;
        }
    }
    eprintln!("wave sounds sampled: {ok}/{tried} decode");
    assert!(ok * 10 >= tried * 9);
}

#[test]
fn brawl_music_mixes_and_stops() {
    let mut out = [0i16; melee_audio::FRAME_SAMPLES * 2];
    melee_audio::brawl::mix_into(&mut out);
    assert!(out.iter().all(|&s| s == 0), "idle mix must add nothing");
    if sound_dir().is_none() {
        assert!(!melee_audio::brawl::play_music("snd_bgm_X18_BOSSBATTLE"));
        eprintln!("skipped: no local Brawl sound folder (silent fallback checked)");
        return;
    }
    assert!(melee_audio::brawl::play_music("snd_bgm_X18_BOSSBATTLE"));
    assert!(melee_audio::brawl::music_on());
    let mut heard = false;
    for _ in 0..400 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut o = [0i16; melee_audio::FRAME_SAMPLES * 2];
        melee_audio::brawl::mix_into(&mut o);
        if o.iter().any(|&s| s != 0) { heard = true; break; }
    }
    assert!(heard, "music never reached the mix");
    melee_audio::brawl::stop_all();
    let mut o = [0i16; melee_audio::FRAME_SAMPLES * 2];
    melee_audio::brawl::mix_into(&mut o);
    assert!(o.iter().all(|&s| s == 0) && !melee_audio::brawl::music_on());
}
