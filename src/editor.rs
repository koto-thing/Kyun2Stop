use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use nih_plug_egui::egui::{Color32, Mesh, Vec2, Pos2, Shape, Stroke, Visuals};

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::params::{TapeStopParams, TapeCurve};

/**
 * パステル調の色を時間に応じて取得
 */
fn get_pastel_color(t: f64, offset: f64) -> Color32 {
    let hue = ((t * 0.05 + offset) % 1.0) as f32;
    egui::ecolor::Hsva::new(hue, 0.4, 0.9, 1.0).into()
}

fn get_kawaii_color(t: f64, offset: f64) -> Color32 {
    let hue = ((t * 0.1 + offset) % 1.0) as f32;

    egui::ecolor::Hsva::new(
        hue,
        0.55,
        1.0,
        0.5
    ).into()
}

/**
 * ぐにゃぐにゃアメーバ
 */
fn paint_amoeba(
    ui: &mut egui::Ui,
    center: Pos2,
    radius: f32,
    color: Color32,
    time: f64,
    seed: f64
) {
    let mut mesh = Mesh::default();

    // 中心点をつくる
    let center_color = color.linear_multiply(1.2);

    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: Pos2::ZERO,
        color: center_color,
    });

    // 外周の頂点を作る
    let num_points = 64; // 頂点数
    for i in 0..=num_points {
        // 0.0 ~ 2π
        let angle = (i as f64 / num_points as f64) * std::f64::consts::TAU;

        // ノイズ生成
        let noise = (angle * 2.0 + time * 0.5 + seed).sin() * 0.15
            + (angle * 3.0 - time * 0.3 + seed).cos() * 0.1;

        // 頂点位置計算
        let r = radius * (1.0 + noise as f32);
        let pos = center + Vec2::angled(angle as f32) * r;

        // 頂点色
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: Pos2::ZERO,
            color,
        });

        // 三角形を作る
        if i < num_points {
            mesh.add_triangle(0, (i + 1) as u32, (i + 2) as u32);
        }
    }

    // 描画登録
    ui.painter().add(Shape::mesh(mesh));
}

fn set_kawaii_style(ctx: &egui::Context) {
    let mut visuals = Visuals::light();

    let pink_pale = Color32::from_rgb(255, 235, 245);
    let pink_hover = Color32::from_rgb(255, 245, 250);
    let pink_strong = Color32::from_rgb(255, 160, 200);
    let pink_stroke = Color32::from_rgb(255, 120, 170);
    let text_color = Color32::from_rgb(110, 80, 100);

    // 全体の透明化
    visuals.window_fill = Color32::TRANSPARENT;
    visuals.panel_fill = Color32::TRANSPARENT;

    // 通常時
    visuals.widgets.inactive.bg_fill = pink_pale;
    visuals.widgets.inactive.weak_bg_fill = pink_pale;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, pink_stroke);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);

    // ホバー時
    visuals.widgets.hovered.bg_fill = pink_hover;
    visuals.widgets.hovered.weak_bg_fill = pink_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, pink_stroke);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text_color);
    visuals.widgets.hovered.expansion = 1.0;

    // クリック/アクティブ時
    visuals.widgets.active.bg_fill = pink_strong;
    visuals.widgets.active.weak_bg_fill = pink_strong;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, pink_stroke);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    // 選択・入力済み部分
    visuals.selection.bg_fill = pink_strong;
    visuals.selection.stroke = Stroke::new(1.0, pink_stroke);

    // コンボボックスのメニューなど
    visuals.window_fill = Color32::from_rgb(255, 240, 250);

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 5],
        blur: 15,
        spread: 2,
        color: Color32::from_rgb(200, 150, 180).linear_multiply(0.4),
    };

    ctx.set_visuals(visuals);
}

pub fn create(
    params: Arc<TapeStopParams>,
    peak_meter: Arc<AtomicU32>,
    editor_state: Arc<EguiState>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        editor_state,
        (),
        |ctx, _| { set_kawaii_style(ctx); },
        move |egui_ctx, setter, _state| {
            egui_ctx.request_repaint();

            let raw_bits = peak_meter.load(Ordering::Relaxed);
            let amplitude = f32::from_bits(raw_bits);
            let clean_amp = if amplitude < 0.001 { 0.0 } else { amplitude };
            let boost = clean_amp * 40.0;

            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(egui_ctx, |ui| {
                let rect = ui.max_rect();
                let time = ui.input(|i| i.time);

                /* 背景 */
                // 動くグラデーション
                let color_time = time + (clean_amp as f64 * 2.0);

                let c_tl = get_pastel_color(color_time, 0.0);
                let c_tr = get_pastel_color(color_time, 0.25);
                let c_br = get_pastel_color(color_time, 0.5);
                let c_bl = get_pastel_color(color_time, 0.75);

                let mut mesh = Mesh::default();
                mesh.vertices.push(egui::epaint::Vertex { pos: rect.min, uv: Pos2::ZERO, color: c_tl });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(rect.max.x, rect.min.y), uv: Pos2::ZERO, color: c_tr });
                mesh.vertices.push(egui::epaint::Vertex { pos: rect.max, uv: Pos2::ZERO, color: c_br });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(rect.min.x, rect.max.y), uv: Pos2::ZERO, color: c_bl });
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                ui.painter().add(Shape::mesh(mesh));

                // ぐにゃぐにゃアメーバ
                if clean_amp > 0.0 {
                    for i in 0..6 {
                        let seed = i as f64 * 123.45;

                        // ゆらゆら
                        let x = rect.min.x + rect.width() * (0.5 + 0.4 * (time * 0.15 + seed).sin() as f32);
                        let y = rect.min.y + rect.height() * (0.5 + 0.4 * (time * 0.2 + seed * 2.0).cos() as f32);

                        let base_size = 30.0 + 10.0 * (time * 0.5 + seed).sin() as f32;
                        let size = base_size + boost;

                        // ゆめかわ
                        let color = get_kawaii_color(time, seed * 0.1);

                        // アメーバ
                        paint_amoeba(ui, Pos2::new(x, y), size, color, time, seed);
                    }
                }

                /* UIパネル */
                ui.with_layout(
                    egui::Layout::top_down(egui::Align::Center)
                        .with_cross_align(egui::Align::Center),
                    |ui| {

                    // 透明パネル
                    egui::Frame::none()
                        .fill(Color32::from_white_alpha(15)) // ベースはかなり透明に
                        .rounding(20.0) // 角を丸くする
                        .stroke(Stroke::new(1.5, Color32::from_white_alpha(150))) // 縁取り（リムライト）を明るく入れる
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 0], // 影の位置ズレなし
                            blur: 20,               // ぼかしを強めに入れて「浮いている」感じに
                            spread: 5,              // 影を少し外側に広げる
                            color: Color32::from_black_alpha(40),
                        })
                        .inner_margin(30.0)
                        .outer_margin(30.0) // 左右に余白を持たせる
                        .show(ui, |ui| {
                            // タイトル
                            ui.heading(
                                egui::RichText::new("Kyun'Stop")
                                    .size(28.0)
                                    .strong()
                                    .color(Color32::from_rgb(200, 80, 120))
                            );
                            ui.add_space(15.0);

                            ui.add_space(5.0);

                            // グリッドレイアウト
                            egui::Grid::new("my_grid")
                                .num_columns(2)
                                .spacing([20.0, 15.0]) // 間隔を広めに
                                .show(ui, |ui| {
                                    let label = |text: &str| {
                                        egui::RichText::new(text).size(15.0).color(Color32::from_rgb(100, 90, 110))
                                    };

                                    // Curve
                                    ui.label(label("CURVE"));
                                    let mut selected_curve = params.curve.value();
                                    ui.scope(|ui| {
                                        let visuals = ui.visuals_mut();

                                        let pink_base   = Color32::from_rgb(255, 220, 240);
                                        let pink_hover  = Color32::from_rgb(255, 235, 250);
                                        let pink_active = Color32::from_rgb(255, 200, 230);
                                        let pink_select = Color32::from_rgb(255, 180, 210);
                                        let pink_stroke = Color32::from_rgb(255, 120, 170);

                                        // 通常時
                                        visuals.widgets.inactive.bg_fill = pink_base;
                                        visuals.widgets.inactive.weak_bg_fill = pink_base;

                                        // ホバー時
                                        visuals.widgets.hovered.bg_fill = pink_hover;
                                        visuals.widgets.hovered.weak_bg_fill = pink_hover;
                                        visuals.widgets.hovered.expansion = 2.0;

                                        // 開いている時
                                        visuals.widgets.open.bg_fill = pink_hover;
                                        visuals.widgets.open.weak_bg_fill = pink_hover;

                                        // 押したとき
                                        visuals.widgets.active.bg_fill = pink_active;
                                        visuals.widgets.active.weak_bg_fill = pink_active;

                                        // 選択部分
                                        visuals.selection.bg_fill = pink_select;
                                        visuals.selection.stroke = Stroke::new(1.0, pink_stroke);

                                        // 枠線と文字色
                                        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(120, 80, 100));
                                        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, pink_stroke);

                                        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(120, 80, 100));
                                        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, pink_stroke);

                                        visuals.widgets.open.bg_stroke = Stroke::new(1.0, pink_stroke);

                                        // コンボボックス描画
                                        egui::ComboBox::new("CURVE", "")
                                            .selected_text(format!("{:?}", selected_curve))
                                            .width(130.0)
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut selected_curve, TapeCurve::Linear, "Linear");
                                                ui.selectable_value(&mut selected_curve, TapeCurve::Smooth, "Smooth");
                                                ui.selectable_value(&mut selected_curve, TapeCurve::SlowStart, "SlowStart");
                                                ui.selectable_value(&mut selected_curve, TapeCurve::QuickCut, "QuickCut");
                                            });
                                    });

                                    if selected_curve != params.curve.value() {
                                        setter.begin_set_parameter(&params.curve);
                                        setter.set_parameter(&params.curve, selected_curve);
                                        setter.end_set_parameter(&params.curve);
                                    }
                                    ui.end_row();

                                    // BPM Sync
                                    ui.label(label("SYNC"));
                                    let mut use_sync = params.use_sync.value();
                                    let sync_text = if use_sync { "ON ♪" } else { "OFF" };
                                    if ui.checkbox(&mut use_sync, sync_text).changed() {
                                        setter.begin_set_parameter(&params.use_sync);
                                        setter.set_parameter(&params.use_sync, use_sync);
                                        setter.end_set_parameter(&params.use_sync);
                                    }
                                    ui.end_row();

                                    // スライダー
                                    ui.label(label("STOP TIME"));
                                    ui.add(widgets::ParamSlider::for_param(&params.stop_time, setter).with_width(140.0));
                                    ui.end_row();

                                    ui.label(label("START TIME"));
                                    ui.add(widgets::ParamSlider::for_param(&params.start_time, setter).with_width(140.0));
                                    ui.end_row();
                                });

                            ui.add_space(25.0);

                            // --- Big Kawaii Button Yeah ! ---
                            let mut trigger_val = params.trigger.value();
                            let btn_text = if trigger_val { "Stopping..." } else { "TAP TO STOP" };

                            // ボタンの色
                            let btn_fill = if trigger_val {
                                Color32::from_rgb(255, 120, 140)
                            } else {
                                Color32::from_rgb(120, 180, 255)
                            };

                            let btn = egui::Button::new(
                                egui::RichText::new(btn_text).size(20.0).color(Color32::WHITE).strong()
                            )
                                .min_size(Vec2::new(180.0, 50.0))
                                .fill(btn_fill);

                            if ui.add(btn).clicked() {
                                let new_val = !trigger_val;
                                setter.begin_set_parameter(&params.trigger);
                                setter.set_parameter(&params.trigger, new_val);
                                setter.end_set_parameter(&params.trigger);
                            }
                        });
                });
            });
        },
    )
}