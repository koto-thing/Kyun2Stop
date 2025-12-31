use nih_plug::prelude::*;

// カーブの種類
#[derive(Enum, PartialEq, Clone, Copy, Debug)]
pub enum TapeCurve {
    Linear,      // 直線的
    Smooth,      // 滑らか
    SlowStart,   // 慣性あり
    QuickCut,    // 急に落ち始め、最後にゆっくり
}

// BPM同期用の拍数定義
#[derive(Enum, PartialEq, Clone, Copy)]
pub enum SyncBeat {
    #[name = "1/8"]
    Eight,
    #[name = "1/4"]
    Quarter,
    #[name = "1/2"]
    Half,
    #[name = "1 Bar"]
    OneBar,
    #[name = "2 Bars"]
    TwoBars,
}

#[derive(Params)]
pub struct TapeStopParams {
    #[id = "trigger"]
    pub trigger: BoolParam, // テープストップトリガー

    #[id = "use_sync"]
    pub use_sync: BoolParam, // BPM同期を使うかどうか

    #[id = "stop_time"]
    pub stop_time: FloatParam, // 秒数指定

    #[id = "sync_beat"]
    pub sync_beat: EnumParam<SyncBeat>, // 拍数指定

    #[id = "start_time"]
    pub start_time: FloatParam, // 再生開始までの時間

    #[id = "curve"]
    pub curve: EnumParam<TapeCurve>, // カーブの種類

    #[id = "enable_filter"]
    pub enable_filter: BoolParam, // ローパスフィルター
}

impl Default for TapeStopParams {
    fn default() -> Self {
        Self {
            trigger: BoolParam::new("Trigger", false),
            use_sync: BoolParam::new("BPM Sync", false),
            stop_time: FloatParam::new("Stop Time (Sec)", 0.5, FloatRange::Linear { min: 0.1, max: 2.0 }),
            sync_beat: EnumParam::new("Stop Beat", SyncBeat::Quarter),
            start_time: FloatParam::new("Start Time", 0.5, FloatRange::Linear { min: 0.1, max: 2.0 }),
            curve: EnumParam::new("Curve", TapeCurve::Linear),
            enable_filter: BoolParam::new("Low-pass Effect", true),
        }
    }
}