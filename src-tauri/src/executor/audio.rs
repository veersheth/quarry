use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

pub(super) fn play_timer_sound(sound: &str) {
    match sound {
        "beep" => play_beep_classic(),
        _ => play_beep(), // "synth" and anything unrecognised
    }
}

/// 4 beeps: first three warm at 880 Hz, last one higher at 1320 Hz.
fn play_beep_classic() {
    use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
    use std::f32::consts::TAU;

    let Ok((_stream, handle)) = OutputStream::try_default() else {
        return;
    };
    let Ok(sink) = Sink::try_new(&handle) else {
        return;
    };

    let sample_rate: u32 = 44100;
    let beep_dur = 0.12f32;
    let gap_dur = 0.10f32;
    let beep_n = (sample_rate as f32 * beep_dur) as usize;
    let gap_n = (sample_rate as f32 * gap_dur) as usize;
    let fade = (sample_rate as f32 * 0.008) as usize; // 8 ms fade to avoid clicks

    // First 3 beeps: 880 Hz warm (fundamental + sub-octave + 2nd harmonic)
    // Last beep: 1320 Hz (E6, a fifth above) — same warmth, clearly higher
    let freqs: [f32; 4] = [987.77, 987.77, 987.77, 1318.51];

    let mut samples = Vec::with_capacity((beep_n + gap_n) * 4);

    for &freq in &freqs {
        for i in 0..beep_n {
            let t = i as f32 / sample_rate as f32;
            let env = if i < fade {
                i as f32 / fade as f32
            } else if i > beep_n - fade {
                (beep_n - i) as f32 / fade as f32
            } else {
                1.0
            };
            samples.push((TAU * freq * t).sin() * 0.55 * env);
        }
        samples.extend(std::iter::repeat_n(0.0f32, gap_n));
    }

    let source = SamplesBuffer::new(1, sample_rate, samples);
    sink.append(source);
    sink.sleep_until_end();
}

/// Slow synth build-up — original sound.
fn play_beep() {
    use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
    use std::f32::consts::{FRAC_PI_2, TAU};

    let Ok((_stream, handle)) = OutputStream::try_default() else {
        return;
    };
    let Ok(sink) = Sink::try_new(&handle) else {
        return;
    };

    let sample_rate: u32 = 44100;
    let duration = 2.2f32;
    let n = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(n);

    let harmonics: &[(f32, f32)] = &[
        (130.8, 0.13),
        (196.0, 0.09),
        (261.6, 0.18),
        (329.6, 0.13),
        (392.0, 0.09),
        (523.2, 0.07),
    ];

    let attack = 0.14f32;
    let sustain = 0.25f32;
    let release = duration - attack - sustain;

    for i in 0..n {
        let t = i as f32 / sample_rate as f32;

        let env = if t < attack {
            ((t / attack) * FRAC_PI_2).sin()
        } else if t < attack + sustain {
            1.0
        } else {
            let r = (t - attack - sustain) / release;
            ((1.0 - r).max(0.0) * FRAC_PI_2).sin().powf(1.8)
        };

        let mut s = 0.0f32;
        for &(freq, amp) in harmonics {
            s += (TAU * freq * t).sin() * amp;
        }
        samples.push(s * env * 0.65);
    }

    let source = SamplesBuffer::new(1, sample_rate, samples);
    sink.append(source);
    sink.sleep_until_end();
}

// Shared flag: true = pink noise should be playing.
pub static PINK_NOISE_RUNNING: AtomicBool = AtomicBool::new(false);

pub(crate) fn toggle_pink_noise() {
    // If already running, signal it to stop.
    if PINK_NOISE_RUNNING
        .compare_exchange(true, false, AtomicOrdering::SeqCst, AtomicOrdering::SeqCst)
        .is_ok()
    {
        return;
    }
    // Not running - start it.
    PINK_NOISE_RUNNING.store(true, AtomicOrdering::SeqCst);
    std::thread::spawn(|| {
        play_pink_noise();
        PINK_NOISE_RUNNING.store(false, AtomicOrdering::SeqCst);
    });
}

fn play_pink_noise() {
    use rodio::{buffer::SamplesBuffer, OutputStream, Sink};

    let Ok((_stream, handle)) = OutputStream::try_default() else {
        return;
    };
    let Ok(sink) = Sink::try_new(&handle) else {
        return;
    };

    let sample_rate: u32 = 44100;
    let chunk_secs = 2u32;
    let chunk_len = (sample_rate * chunk_secs) as usize;

    // Voss-McCartney pink noise: sum of white noise sources updated at 1/2^k rate.
    let mut b = [0.0f32; 7];
    let mut rng_state: u64 = 0x853c49e6748fea9b;

    let white = |state: &mut u64| -> f32 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        (*state as i64 as f32) / (i64::MAX as f32)
    };

    loop {
        if !PINK_NOISE_RUNNING.load(AtomicOrdering::Relaxed) {
            break;
        }

        let mut samples = Vec::with_capacity(chunk_len);
        for i in 0..chunk_len {
            // Update one row of the Voss table per sample.
            let row = i.trailing_zeros() as usize;
            if row < b.len() {
                b[row] = white(&mut rng_state);
            }
            let pink: f32 = b.iter().sum::<f32>() / b.len() as f32;
            samples.push(pink * 0.18);
        }

        let source = SamplesBuffer::new(1, sample_rate, samples);
        sink.append(source);

        // Wait until this chunk is consumed before generating the next,
        // so we don't buffer unboundedly.
        while sink.len() > 1 {
            if !PINK_NOISE_RUNNING.load(AtomicOrdering::Relaxed) {
                sink.stop();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    sink.stop();
}

// Shared flag: true = rain noise should be playing.
pub static RAIN_NOISE_RUNNING: AtomicBool = AtomicBool::new(false);

pub(crate) fn toggle_rain_noise() {
    if RAIN_NOISE_RUNNING
        .compare_exchange(true, false, AtomicOrdering::SeqCst, AtomicOrdering::SeqCst)
        .is_ok()
    {
        return;
    }
    RAIN_NOISE_RUNNING.store(true, AtomicOrdering::SeqCst);
    std::thread::spawn(|| {
        play_rain_noise();
        RAIN_NOISE_RUNNING.store(false, AtomicOrdering::SeqCst);
    });
}

fn play_rain_noise() {
    use rodio::{buffer::SamplesBuffer, OutputStream, Sink};

    let Ok((_stream, handle)) = OutputStream::try_default() else {
        return;
    };
    let Ok(sink) = Sink::try_new(&handle) else {
        return;
    };

    const SR: u32 = 44100;
    const CHUNK: usize = (SR * 2) as usize;

    // Two independent XorShift64 states - one for drops, one for texture
    let mut rng_a: u64 = 0x9e3779b97f4a7c15;
    let mut rng_b: u64 = 0x6c62272e07bb0142;

    fn xs(s: &mut u64) -> f32 {
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        (*s as i64) as f32 / i64::MAX as f32
    }

    // Bandpass filter for the continuous wash: LP at ~4kHz then HP at ~200Hz.
    // One-pole LP: a = 1 - exp(-2π·fc/sr)
    let lp_a = 0.43f32; // fc ≈ 4 kHz
    let hp_a = 0.972f32; // fc ≈ 200 Hz  (α = R/(R+1), R = sr/(2π·fc))
    let mut lp = 0.0f32;
    let mut hp_x = 0.0f32;
    let mut hp_y = 0.0f32;

    // Each drop is an exponentially decaying burst of bandpass noise.
    struct Drop {
        amp: f32,  // current amplitude
        rate: f32, // per-sample decay multiplier  (close to 1.0)
        lp: f32,   // per-drop LP state for tonal variety
        lp_a: f32, // per-drop LP cutoff (varies drop-to-drop)
        rem: u32,  // samples remaining (hard cutoff)
    }
    let mut drops: Vec<Drop> = Vec::new();

    // Drop rate targets (events / second):
    //   fine drizzle : ~600/s  → short, quiet, bright
    //   medium drops : ~60/s   → medium length, moderate amp
    //   heavy drops  : ~8/s    → long tail, loud
    let p_fine: f32 = 600.0 / SR as f32;
    let p_medium: f32 = 60.0 / SR as f32;
    let p_heavy: f32 = 8.0 / SR as f32;

    loop {
        if !RAIN_NOISE_RUNNING.load(AtomicOrdering::Relaxed) {
            break;
        }

        let mut samples = Vec::with_capacity(CHUNK);

        for _ in 0..CHUNK {
            let w = xs(&mut rng_a); // white noise excitation

            // --- Continuous wash (bandpass-filtered noise) ---
            lp += lp_a * (w - lp);
            let bp = hp_a * (hp_y + lp - hp_x);
            hp_x = lp;
            hp_y = bp;

            // --- Spawn new drops ---
            let r = xs(&mut rng_b).abs();
            if r < p_fine {
                // Fine: 3–10 ms, high cutoff, quiet
                let dur = (SR as f32 * (0.003 + xs(&mut rng_b).abs() * 0.007)) as u32;
                drops.push(Drop {
                    amp: 0.04 + xs(&mut rng_b).abs() * 0.05,
                    rate: (-4.0 / dur as f32).exp(),
                    lp: 0.0,
                    lp_a: 0.55 + xs(&mut rng_b).abs() * 0.3, // 5–8 kHz
                    rem: dur,
                });
            }
            if r < p_medium {
                let dur = (SR as f32 * (0.015 + xs(&mut rng_b).abs() * 0.035)) as u32;
                drops.push(Drop {
                    amp: 0.10 + xs(&mut rng_b).abs() * 0.15,
                    rate: (-4.0 / dur as f32).exp(),
                    lp: 0.0,
                    lp_a: 0.30 + xs(&mut rng_b).abs() * 0.2, // 2–4 kHz
                    rem: dur,
                });
            }
            if r < p_heavy {
                let dur = (SR as f32 * (0.04 + xs(&mut rng_b).abs() * 0.08)) as u32;
                drops.push(Drop {
                    amp: 0.25 + xs(&mut rng_b).abs() * 0.25,
                    rate: (-4.0 / dur as f32).exp(),
                    lp: 0.0,
                    lp_a: 0.18 + xs(&mut rng_b).abs() * 0.1, // 1–2 kHz - more body
                    rem: dur,
                });
            }

            // --- Accumulate active drops ---
            // Each drop gets its own LP-filtered noise burst.
            let drop_noise = xs(&mut rng_a); // shared excitation for all active drops
            let mut drop_sum = 0.0f32;
            drops.retain_mut(|d| {
                d.lp += d.lp_a * (drop_noise - d.lp);
                drop_sum += d.amp * d.lp;
                d.amp *= d.rate;
                d.rem = d.rem.saturating_sub(1);
                d.rem > 0
            });

            let out = (bp * 0.18 + drop_sum * 0.55).clamp(-1.0, 1.0);
            samples.push(out);
        }

        sink.append(SamplesBuffer::new(1, SR, samples));

        while sink.len() > 1 {
            if !RAIN_NOISE_RUNNING.load(AtomicOrdering::Relaxed) {
                sink.stop();
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    sink.stop();
}
