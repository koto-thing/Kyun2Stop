use std::slice;

use crate::dsp::engine::TapeStopEngine;
use crate::params::{SyncBeat, TapeCurve};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct K2sConfig {
    pub sample_rate: f32,
    pub max_seconds: f32,
    pub channels: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum K2sCurve {
    Linear = 0,
    Smooth = 1,
    SlowStart = 2,
    QuickCut = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum K2sSyncBeat {
    Eight = 0,
    Quarter = 1,
    Half = 2,
    OneBar = 3,
    TwoBars = 4,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct K2sProcessParams {
    pub trigger: bool,
    pub stop_time_sec: f32,
    pub start_time_sec: f32,
    pub curve: K2sCurve,
    pub use_sync: bool,
    pub sync_beat: K2sSyncBeat,
    pub bpm: f64,
    pub enable_filter: bool,
}

struct K2sHandle {
    engine: TapeStopEngine,
    channels: usize,
    frame_in: Vec<f32>,
    frame_out: Vec<f32>,
}

#[repr(C)]
pub struct K2sOpaqueHandle {
    _private: [u8; 0],
}

impl K2sCurve {
    fn to_internal(self) -> TapeCurve {
        match self {
            K2sCurve::Linear => TapeCurve::Linear,
            K2sCurve::Smooth => TapeCurve::Smooth,
            K2sCurve::SlowStart => TapeCurve::SlowStart,
            K2sCurve::QuickCut => TapeCurve::QuickCut,
        }
    }
}

impl K2sSyncBeat {
    fn to_internal(self) -> SyncBeat {
        match self {
            K2sSyncBeat::Eight => SyncBeat::Eight,
            K2sSyncBeat::Quarter => SyncBeat::Quarter,
            K2sSyncBeat::Half => SyncBeat::Half,
            K2sSyncBeat::OneBar => SyncBeat::OneBar,
            K2sSyncBeat::TwoBars => SyncBeat::TwoBars,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn k2s_create(config: K2sConfig) -> *mut K2sOpaqueHandle {
    if config.sample_rate <= 0.0 || config.max_seconds <= 0.0 || config.channels == 0 {
        return core::ptr::null_mut();
    }

    let handle = K2sHandle {
        engine: TapeStopEngine::new(config.sample_rate, config.max_seconds, config.channels as usize),
        channels: config.channels as usize,
        frame_in: vec![0.0_f32; config.channels as usize],
        frame_out: vec![0.0_f32; config.channels as usize],
    };

    Box::into_raw(Box::new(handle)) as *mut K2sOpaqueHandle
}

#[unsafe(no_mangle)]
pub extern "C" fn k2s_destroy(handle: *mut K2sOpaqueHandle) {
    if handle.is_null() {
        return;
    }

    // SAFETY: `handle` was created by `k2s_create()` and is consumed exactly once here.
    unsafe {
        drop(Box::from_raw(handle as *mut K2sHandle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn k2s_reset(handle: *mut K2sOpaqueHandle) -> bool {
    if handle.is_null() {
        return false;
    }

    // SAFETY: Null has been checked, and caller guarantees a valid mutable handle.
    let state = unsafe { &mut *(handle as *mut K2sHandle) };
    state.engine.reset();
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn k2s_process_interleaved_f32(
    handle: *mut K2sOpaqueHandle,
    input: *const f32,
    output: *mut f32,
    frames: usize,
    params: K2sProcessParams,
) -> bool {
    if handle.is_null() || input.is_null() || output.is_null() {
        return false;
    }

    // SAFETY: Pointers are non-null, and lengths are derived from caller-provided frame count.
    let state = unsafe { &mut *(handle as *mut K2sHandle) };
    let channels = state.channels;
    let total_samples = match frames.checked_mul(channels) {
        Some(v) => v,
        None => return false,
    };

    // SAFETY: Caller provides valid buffers with at least `total_samples` elements.
    let in_buf = unsafe { slice::from_raw_parts(input, total_samples) };
    // SAFETY: Caller provides valid output buffer with at least `total_samples` elements.
    let out_buf = unsafe { slice::from_raw_parts_mut(output, total_samples) };

    let curve = params.curve.to_internal();
    let sync_beat = params.sync_beat.to_internal();

    for frame in 0..frames {
        let start = frame * channels;
        let end = start + channels;

        state.frame_in.copy_from_slice(&in_buf[start..end]);
        state.engine.process(
            &state.frame_in,
            &mut state.frame_out,
            params.trigger,
            params.stop_time_sec,
            params.start_time_sec,
            curve,
            params.use_sync,
            sync_beat,
            params.bpm,
            params.enable_filter,
        );
        out_buf[start..end].copy_from_slice(&state.frame_out);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_matches_direct_engine_for_interleaved_stereo() {
        let frames = 64usize;
        let channels = 2usize;
        let cfg = K2sConfig {
            sample_rate: 48_000.0,
            max_seconds: 3.0,
            channels: channels as u32,
        };

        let mut input = vec![0.0f32; frames * channels];
        for frame in 0..frames {
            let x = (frame as f32) * 0.01;
            input[frame * channels] = x.sin();
            input[frame * channels + 1] = x.cos() * 0.5;
        }

        let params = K2sProcessParams {
            trigger: true,
            stop_time_sec: 0.5,
            start_time_sec: 0.5,
            curve: K2sCurve::Smooth,
            use_sync: false,
            sync_beat: K2sSyncBeat::Quarter,
            bpm: 120.0,
            enable_filter: true,
        };

        let mut expected = vec![0.0f32; frames * channels];
        let mut engine = TapeStopEngine::new(cfg.sample_rate, cfg.max_seconds, channels);
        let mut frame_in = vec![0.0f32; channels];
        let mut frame_out = vec![0.0f32; channels];

        for frame in 0..frames {
            let start = frame * channels;
            let end = start + channels;
            frame_in.copy_from_slice(&input[start..end]);
            engine.process(
                &frame_in,
                &mut frame_out,
                params.trigger,
                params.stop_time_sec,
                params.start_time_sec,
                params.curve.to_internal(),
                params.use_sync,
                params.sync_beat.to_internal(),
                params.bpm,
                params.enable_filter,
            );
            expected[start..end].copy_from_slice(&frame_out);
        }

        let mut actual = vec![0.0f32; frames * channels];
        let handle = k2s_create(cfg);
        assert!(!handle.is_null());

        let ok = k2s_process_interleaved_f32(handle, input.as_ptr(), actual.as_mut_ptr(), frames, params);
        assert!(ok);

        for (a, b) in actual.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1.0e-6);
        }

        k2s_destroy(handle);
    }
}

