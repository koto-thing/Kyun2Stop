use nih_plug::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use nih_plug_egui::EguiState;

mod params;
mod dsp;
mod editor;

use params::TapeStopParams;
use dsp::engine::TapeStopEngine;

struct TapeStop {
    params: Arc<TapeStopParams>,
    dsp: Option<TapeStopEngine>,
    editor_state: Arc<EguiState>,
    peak_meter: Arc<AtomicU32>,
}

impl Default for TapeStop {
    fn default() -> Self {
        Self {
            params: Arc::new(TapeStopParams::default()),
            dsp: None,
            editor_state: EguiState::from_size(600, 400),
            peak_meter: Arc::new(AtomicU32::new(0f32.to_bits())),
        }
    }
}

impl Plugin for TapeStop {
    const NAME: &'static str = "Kyun'Stop";
    const VENDOR: &'static str = "Goto Kenta";
    const URL: &'static str = "https://koto-thing.github.io/MyWebsite/";
    const EMAIL: &'static str = "gotoukenta62@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // 音声入出力の設定
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.dsp = Some(TapeStopEngine::new(
            buffer_config.sample_rate,
            3.0, // 最大遅延時間 3 秒
            2,      // ステレオ対応
        ));
        true
    }

    fn reset(&mut self) {
        if let Some(engine) = &mut self.dsp {
            engine.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let engine = match &mut self.dsp {
            Some(engine) => engine,
            None => return ProcessStatus::Normal,
        };

        // BPMをホストから取得
        let bpm = context.transport().tempo.unwrap_or(120.0);
        let mut max_amplitude: f32 = 0.0;

        // バッファ内の全サンプルを処理する
        for (_sample_id, mut channel_samples) in buffer.iter_samples().enumerate() {
            // パラメータをとってくる
            let trigger = self.params.trigger.value();
            let stop_time = self.params.stop_time.value();
            let start_time = self.params.start_time.value();
            let curve = self.params.curve.value();
            let use_sync = self.params.use_sync.value();
            let sync_beat = self.params.sync_beat.value();
            let enable_filter = self.params.enable_filter.value();

            let num_channels = channel_samples.len();

            // 入力を一時的にコピーしておく
            let mut input_frame = [0.0f32; 2];
            for (i, sample) in channel_samples.iter_mut().enumerate() {
                if i < 2 {
                    input_frame[i] = *sample;
                }
            }

            let mut output_frame = [0.0f32; 2];

            // DSPエンジンで処理
            engine.process(
                &input_frame,
                &mut output_frame,
                trigger,
                stop_time,
                start_time,
                curve,
                use_sync,
                sync_beat,
                bpm,
                enable_filter
            );

            // 最大振幅を計算
            for sample in output_frame.iter() {
                let abs = sample.abs();
                if abs > max_amplitude {
                    max_amplitude = abs;
                }
            }

            // 結果をバッファに書き戻す
            for (i, sample) in channel_samples.iter_mut().enumerate() {
                if i < 2 {
                    *sample = output_frame[i];
                }
            }
        }

        self.peak_meter.store(max_amplitude.to_bits(), Ordering::Relaxed);

        ProcessStatus::Normal
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.editor_state.clone(),
        )
    }
}

impl Vst3Plugin for TapeStop {
    const VST3_CLASS_ID: [u8; 16] = *b"TapeStopPlugin12";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

nih_export_vst3!(TapeStop);