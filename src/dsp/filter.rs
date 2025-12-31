#[derive(Clone, Copy)]
pub struct OnePoleLowpass {
    prev_output: f32,
    alpha: f32,
}

impl OnePoleLowpass {
    pub fn new () -> Self {
        Self {
            prev_output: 0.0,
            alpha: 1.0,
        }
    }

    /**
     * カットオフ周波数を設定する
     */
    pub fn set_cutoff(&mut self, cutoff_hz: f32, sample_rate: f32) {
        let y = 1.0 - (-2.0 * std::f32::consts::PI * cutoff_hz / sample_rate).exp();
        self.alpha = y.clamp(0.0, 1.0);
    }

    /**
     * フィルター処理を行う
     */
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.prev_output + self.alpha * (input - self.prev_output);
        self.prev_output = output;
        output
    }

    /**
     * フィルター状態をリセット
     */
    pub fn reset(&mut self) {
        self.prev_output = 0.0;
        self.alpha = 1.0;
    }
}