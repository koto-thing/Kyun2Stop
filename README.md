This Readme is wirtten by AI.

# Kyun'Stop VST3

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Format](https://img.shields.io/badge/format-VST3-green.svg)

**キュンとストップ** は、Rustと `nih-plug` で開発された、オーディオリアクティブなビジュアルを持つテープストップ・エフェクトプラグインです。

<img width="597" height="400" alt="image" src="https://github.com/user-attachments/assets/8c6e7019-c63c-45c2-a6b4-40354bedc6eb" />

## ✨ 特徴 (Features)

### 🎛 DSP & 機能
* **Tape Stop / Start**: レコードやテープが止まる/動き出すようなピッチ変化を再現。
* **BPM Sync**: ホストDAWのテンポに同期した停止時間設定が可能（1/8, 1/4, 1Barなど）。
* **Curve Control**: 4種類の減衰カーブを選択可能。
    * `Linear`: 直線的な変化
    * `Smooth`: 滑らかなS字カーブ
    * `SlowStart`: 慣性を再現（ゆっくり落ち始め、急に止まる）
    * `QuickCut`: 急激に落ちる
* **Auto Filter**: テープ速度の低下に合わせて、自動的にローパスフィルターを適用し、こもった音を演出。

### 🎨 ビジュアル (GUI)
* **Yumekawa Theme**: パステルカラーの動くグラデーション背景。
* **Audio Reactive Particles**: 出力音量に反応して弾む、半透明のアメーバ状パーティクル。音が止まると消滅します。

## 📦 ビルド方法 (Build)

このプラグインをビルドするには、Rustのツールチェーンが必要です。

1. **リポジトリのクローン**
   ```bash
   git clone [https://github.com/YourUsername/YourProjectName.git](https://github.com/YourUsername/YourProjectName.git)
   cd YourProjectName
