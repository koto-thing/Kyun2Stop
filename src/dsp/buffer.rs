pub struct DelayLine {
    data: Vec<f32>,
    mask: usize,
}

impl DelayLine {
    pub fn new(max_seconds: f32, sample_rate: f32) -> Self {
        let size = (max_seconds * sample_rate) as usize;
        let power_of_two_size = size.next_power_of_two();

        Self {
            data: vec![0.0; power_of_two_size],
            mask: power_of_two_size - 1,
        }
    }

    /**
    * バッファをリセット
    */
    pub fn reset(&mut self) {
        self.data.fill(0.0);
    }

    /**
    * 新しいサンプルをバッファに追加
    */
    #[inline]
    pub fn write(&mut self, index: usize, value: f32) {
        self.data[index & self.mask] = value;
    }

    /**
    * 現在の書き込み位置から指定された遅延サンプル数だけ前のサンプルを取得
    */
    #[inline]
    pub fn read(&self, index: f64) -> f32 {
        let len = self.data.len();
        let mask = self.mask;

        // 整数部と小数部
        let idx_i = index.floor() as usize;
        let frac = (index - idx_i as f64) as f32;

        // 4点のサンプルを取得
        let p0 = idx_i.wrapping_sub(1) & mask;
        let p1 = idx_i & mask;
        let p2 = idx_i.wrapping_add(1) & mask;
        let p3 = idx_i.wrapping_add(2) & mask;

        let s0 = self.data[p0];
        let s1 = self.data[p1];
        let s2 = self.data[p2];
        let s3 = self.data[p3];

        // 4点エルミート補間の公式
        let c0 = s1;
        let c1 = 0.5 * (s2 - s0);
        let c2 = s0 - 2.5 * s1 + 2.0 * s2 - 0.5 * s3;
        let c3 = 0.5 * (s3 - s0) + 1.5 * (s1 - s2);

        ((c3 * frac + c2) * frac + c1) * frac + c0
    }
}