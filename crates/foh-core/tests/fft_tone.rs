use foh_core::fft::rfft_mags;

#[test]
fn thousand_hertz_lands_in_the_1k_bin() {
    let sr = 48_000u32;
    let n = 4096usize;
    let freq = 1000.0f32;
    let mut x = vec![0.0f32; n];
    for i in 0..n {
        let t = i as f32 / sr as f32;
        x[i] = (2.0 * std::f32::consts::PI * freq * t).sin();
    }
    let mags = rfft_mags(&x);
    let hz_per = sr as f32 / n as f32;
    let (k, _) = mags
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    let hz = k as f32 * hz_per;
    assert!(
        (hz - 1000.0).abs() < hz_per * 1.5,
        "peak at {hz} Hz (bin {k}), expect ~1000"
    );
}

#[test]
fn identical_sines_correlate_near_one() {
    let n = 2048;
    let a: Vec<f32> = (0..n).map(|i| (i as f32 * 0.02).sin()).collect();
    let r = foh_core::corr::pearson(&a, &a);
    assert!(r > 0.999, "r={r}");
}

#[test]
fn channel_map_rejects_zero_and_parses() {
    let m = foh_core::ChannelMap::parse("vox 12 vocal\nkick 1 kick\n").unwrap();
    assert_eq!(m.desk_channel("vox"), Some(12));
    assert!(foh_core::ChannelMap::parse("vox 0 vocal").is_err());
}
