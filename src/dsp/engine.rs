use super::buffer::DelayLine;
use super::filter::OnePoleLowpass;
use crate::params::{TapeCurve, SyncBeat};

pub struct TapeStopEngine {
    buffers: Vec<DelayLine>,      // チャンネルごとの遅延バッファ
    filters: Vec<OnePoleLowpass>, // チャンネルごとのフィルタ
    sample_rate: f32,             // サンプルレート

    write_pos: usize, // 書き込み位置
    read_pos: f64,    // 読み込み位置

    phase: f64,          // 1.0 -> 0.0の進行度
    current_speed: f64,  // phaseとcurveから計算された実際の速度
    crossfade_gain: f32, // テープ音とリアルタイム音のクロスフェードゲイン
}

impl TapeStopEngine {
    pub fn new(sample_rate: f32, max_seconds: f32, channels: usize) -> Self {
        let buffers = (0..channels).map(|_| DelayLine::new(max_seconds, sample_rate)).collect();
        let filters = (0..channels).map(|_| OnePoleLowpass::new()).collect();

        Self {
            buffers,
            filters,
            sample_rate,
            write_pos: 0,
            read_pos: 0.0,
            phase: 1.0,
            current_speed: 1.0,
            crossfade_gain: 1.0,
        }
    }

    /**
     * エンジンの状態をリセット
     */
    pub fn reset(&mut self) {
        for buffer in &mut self.buffers {
            buffer.reset();
        }
        for filter in &mut self.filters {
            filter.reset();
        }
        self.write_pos = 0;
        self.read_pos = 0.0;
        self.phase = 1.0;
        self.current_speed = 1.0;
        self.crossfade_gain = 1.0;
    }

    /**
     * テープストップエフェクトを処理
     * - input 入力バッファ
     * - output 出力バッファ
     * -  trigger テープストップトリガー
     * - stop_time_sec 停止時間（秒）
     * - start_time_sec 再生開始時間（秒）
     * - curve_type カーブの種類
     * - use_sync BPM同期を使うかどうか
     * - sync_beat BPM同期時の拍数指定
     * - bpm ホストからのBPM情報
     * - enable_filter ローパスフィルターを有効にするかどう
     */
    pub fn process(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        trigger: bool,
        stop_time_sec: f32,
        start_time_sec: f32,
        curve_type: TapeCurve,
        use_sync: bool,
        sync_beat: SyncBeat,
        bpm: f64,
        enable_filter: bool,
    ) {
        // 時間の決定
        let actual_stop_time = if use_sync {
            // BPM同期
            let current_bpm = bpm; // BPMが取れないときは120
            let beats = match sync_beat {
                SyncBeat::Eight => 0.5,
                SyncBeat::Quarter => 1.0,
                SyncBeat::Half => 2.0,
                SyncBeat::OneBar => 4.0,
                SyncBeat::TwoBars => 8.0,
            };
            // 時間 = (60 / BPM) * 拍数
            (60.0 / current_bpm as f32) * beats
        } else {
            stop_time_sec
        };

        // 変化量の計算
        let stop_step = 1.0 / (actual_stop_time * self.sample_rate) as f64;
        let start_step = 1.0 / (start_time_sec * self.sample_rate) as f64;
        let xfade_step = 1.0 / (0.1 * self.sample_rate);

        // 進行度の更新
        if trigger {
            // Phaseを 1.0 -> 0.0 へ減らす
            self.phase -= stop_step;
            if self.phase < 0.0 { self.phase = 0.0; }
            self.crossfade_gain = 0.0;
        } else {
            // Phaseを 0.0 -> 1.0 へ増やす
            if self.phase < 1.0 {
                self.phase += start_step;
                if self.phase > 1.0 { self.phase = 1.0; }
                self.crossfade_gain = 0.0;
            } else {
                // Phaseが1.0に戻ったらクロスフェードで復帰
                if self.crossfade_gain < 1.0 {
                    self.crossfade_gain += xfade_step;
                    if self.crossfade_gain >= 1.0 {
                        self.crossfade_gain = 1.0;
                        self.read_pos = self.write_pos as f64; // 同期
                    }
                }
            }
        }

        // Curve適用
        let t = self.phase;
        self.current_speed = match curve_type {
            TapeCurve::Linear => t,
            TapeCurve::Smooth => t * t * (3.0 - 2.0 * t),
            TapeCurve::SlowStart => 1.0 - (1.0 - t).powi(2),
            TapeCurve::QuickCut => t.powi(3),
        };

        // フィルター係数の計算
        if enable_filter {
            // 速度に応じてカットオフを変化させる
            // 速度が低いほどこもらせる
            let min_cut: f32 = 200.0;
            let max_cut: f32 = 20000.0;
            let cutoff = min_cut * (max_cut / min_cut).powf(self.current_speed as f32);

            for f in &mut self.filters {
                f.set_cutoff(cutoff, self.sample_rate);
            }
        }

        // 音声処理
        for (ch, (&in_sample, out_sample)) in input.iter().zip(output.iter_mut()).enumerate() {
            if ch >= self.buffers.len() { break; }

            // 書き込み
            self.buffers[ch].write(self.write_pos, in_sample);

            // 読み込み
            let mut tape_sound = self.buffers[ch].read(self.read_pos);

            // フィルター適用
            if enable_filter {
                tape_sound = self.filters[ch].process(tape_sound);
            }

            // クロスフェード出力
            *out_sample = tape_sound * (1.0 - self.crossfade_gain) + in_sample * self.crossfade_gain;
        }

        // ヘッド進行
        self.write_pos = self.write_pos.wrapping_add(1);
        self.read_pos += self.current_speed;
    }
}