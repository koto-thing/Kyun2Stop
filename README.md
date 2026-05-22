# Tape Stopの仕組み

このプロジェクトにはバグ修正パッチを適用した `nih-plug` が含まれています。
テープストップエフェクトの実装は、以下のような仕組みで行われています。

* リングバッファを用いて、今流れている音声データを保存する。
* テープストップがトリガーされると、保存された音声データを逆再生しながらピッチを下げていく。
* ピッチの変化は、選択されたカーブに基づいて計算される。
* ピッチが0になると音声出力を停止し、テープが完全に止まった状態を再現する。
* テープが再び動き出すと、保存された音声データを正再生しながらピッチを上げていく。
* ピッチが元の値に戻ると、通常の音声出力に切り替わる。

The rest of this README was written by AI.

# Kyun'Stop VST3

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Format](https://img.shields.io/badge/format-VST3-green.svg)

**キュンとストップ** は、Rustと `nih-plug` で開発された、オーディオリアクティブなビジュアルを持つテープストップ・エフェクトプラグインです。

<img width="599" height="396" alt="image" src="https://github.com/user-attachments/assets/9c624926-a3ec-44c8-b30b-2371974ef716" />

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

### 🔧 パッチ適用済みの `nih-plug` について (Patched nih-plug)

本プロジェクトでは、一部のホストDAW（Cubaseなど、トランスポート停止時にオーディオ処理スレッドを停止するDAW）において、トランスポート停止中にGUIのパラメータ値（Tap To Stopボタンなど）が正常に更新・保持されないバグを修正するため、ローカルのパッチ版 `nih-plug`（`nih-plug-patched`）を使用しています。

このローカルパッチは `Cargo.toml` の `[patch]` セクションで自動的に適用されるため、通常のビルド手順（`build_vst3.ps1` の実行など）をそのまま行うだけで問題ありません。また、`nih-plug-patched` 内のソースコードも Git で追跡されているため、別途 submodule 等のセットアップは不要です。

1. **リポジトリのクローン**
   ```bash
   git clone https://github.com/koto-thing/Kyun2Stop.git
   cd Kyun2Stop
   ```

2. **ビルドとパッケージング (Windows)**

   Windows環境では、ビルド後に正しくVST3のディレクトリ構成（バンドル形式）にする必要があります。単に `.dll` を `.vst3` にリネームして配置しただけでは、Cubaseなどの厳格なDAWでロードに失敗します。
   
   リポジトリ内のスクリプトを使用することで、自動でビルド・パッケージングを行えます。

   * **ビルドとパッケージングのみ実行する場合:**
     ```powershell
     powershell -ExecutionPolicy Bypass -File .\scripts\build_vst3.ps1
     ```
     ビルド成果物は `target\bundled\Kyun2Stop.vst3` に出力されます。

   * **ビルド後にシステムVST3フォルダ（`C:\Program Files\Common Files\VST3\`）へのインストールまで行う場合:**
     （※ 管理者権限で起動したPowerShellで実行してください）
     ```powershell
     powershell -ExecutionPolicy Bypass -File .\scripts\build_vst3.ps1 -Install
     ```

## C++から使うためのFFI

VST3エクスポートとは別に、C/C++向け `extern "C"` APIを追加しています。
公開ヘッダは `include/kyun2stop_ffi.h` です。

### ヘッダ生成（cbindgen）

`cbindgen` を使ってヘッダを再生成できます。

```powershell
cargo install cbindgen
powershell -ExecutionPolicy Bypass -File .\scripts\generate_ffi_header.ps1
```

### FFIビルド成果物

```powershell
cargo build --release
```

`target\release\` に `Kyun2Stop.dll` と `Kyun2Stop.lib`（環境依存）が出力されます。

### FFI APIの基本フロー

1. `k2s_create()` でエンジンハンドルを作成
2. `k2s_process_interleaved_f32()` で `float` インターリーブ音声を処理
3. `k2s_destroy()` でハンドルを破棄

`k2s_process_interleaved_f32()` の `frames` はフレーム数です。
バッファ長は `frames * channels` を確保してください。

### C++サンプル

`examples/cpp/main.cpp` に最小サンプルがあります。
RAIIラッパは `examples/cpp/k2s_raii.hpp` です。

```powershell
Set-Location examples\cpp
cmake -S . -B build
cmake --build build --config Release
.\build\Release\k2s_ffi_example.exe
```

### FFIのRustテスト

```powershell
cargo test ffi::tests::ffi_matches_direct_engine_for_interleaved_stereo -- --nocapture
```

