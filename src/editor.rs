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

/**
 * ゆめかわ
 */
fn get_yumekawa_color(t: f64, offset: f64) -> Color32 {
    let wave = ((t * 0.2 + offset).sin() * 0.5 + 0.5) as f32;
    let hue = 0.55 + (0.4 * wave);

    egui::ecolor::Hsva::new(hue, 0.35, 1.0, 1.0).into()
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
    let center_color = Color32::WHITE.linear_multiply(0.8).gamma_multiply(0.5);

    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: Pos2::ZERO,
        color: center_color,
    });

    // 外周の頂点を作る
    let num_points = 32; // 頂点数
    for i in 0..=num_points {
        // 0.0 ~ 2π
        let angle = (i as f64 / num_points as f64) * std::f64::consts::TAU;

        // ノイズ生成
        let noise = (angle * 2.0 + time * 1.5 + seed).sin() * 0.15
            + (angle * 3.0 - time * 1.0 + seed).cos() * 0.1;

        let r = radius * (1.0 + noise as f32);

        // 極座標から直交座標に変換
        let pos = center + Vec2::angled(angle as f32) * r;

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
    let mut visuals = Visuals::light(); // ベースはライトモード

    // ウィジェットの色をパステル系に
    // 通常時の背景
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(245, 235, 240);
    // ホバー時の背景
    visuals.widgets.hovered.bg_fill = Color32::WHITE;
    // クリック時の背景
    visuals.widgets.active.bg_fill = Color32::from_rgb(255, 200, 220);

    // スライダーの中身や枠線
    visuals.selection.bg_fill = Color32::from_rgb(255, 150, 180); // 濃いピンク
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 100, 150));

    // 文字色
    let text_color = Color32::from_rgb(80, 60, 90);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_color);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_color);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, text_color);

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

            egui::CentralPanel::default().show(egui_ctx, |ui| {
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
                        let mut particle_color = get_yumekawa_color(time, seed);

                        // すけすけ
                        let alpha = (clean_amp * 200.0).clamp(0.0, 150.0) as u8;

                        // 色に透明度を追加
                        particle_color = Color32::from_rgba_premultiplied(
                            particle_color.r(),
                            particle_color.g(),
                            particle_color.b(),
                            alpha
                        );

                        // アメーバ
                        paint_amoeba(ui, Pos2::new(x, y), size, particle_color, time, seed);
                    }
                }

                /* UIパネル */
                ui.vertical_centered(|ui| {
                    // 画面の中央に配置
                    ui.add_space(30.0);

                    // 透明パネル
                    let panel_response = egui::Frame::default()
                        .fill(Color32::from_white_alpha(80))  // 半透明の白でガラス感
                        .inner_margin(30.0) // 内側の余白
                        .outer_margin(10.0) // 外側の余白
                        .show(ui, |ui| {
                            // パネル内にガラスっぽいグラデーションを重ねる
                            let panel_rect = ui.max_rect();

                            // 上部が明るく、下部が暗いグラデーション
                            let top_color = Color32::from_white_alpha(60);
                            let bottom_color = Color32::from_white_alpha(20);

                            let mut glass_mesh = Mesh::default();
                            glass_mesh.vertices.push(egui::epaint::Vertex {
                                pos: panel_rect.min,
                                uv: Pos2::ZERO,
                                color: top_color
                            });
                            glass_mesh.vertices.push(egui::epaint::Vertex {
                                pos: Pos2::new(panel_rect.max.x, panel_rect.min.y),
                                uv: Pos2::ZERO,
                                color: top_color
                            });
                            glass_mesh.vertices.push(egui::epaint::Vertex {
                                pos: panel_rect.max,
                                uv: Pos2::ZERO,
                                color: bottom_color
                            });
                            glass_mesh.vertices.push(egui::epaint::Vertex {
                                pos: Pos2::new(panel_rect.min.x, panel_rect.max.y),
                                uv: Pos2::ZERO,
                                color: bottom_color
                            });
                            glass_mesh.add_triangle(0, 1, 2);
                            glass_mesh.add_triangle(0, 2, 3);
                            ui.painter().add(Shape::mesh(glass_mesh));

                            // タイトル
                            ui.heading(
                                egui::RichText::new("Tape Stop")
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
                                    ui.label(label("Curve"));
                                    let mut selected_curve = params.curve.value();
                                    egui::ComboBox::new("curve", "")
                                        .selected_text(format!("{:?}", selected_curve))
                                        .width(130.0)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut selected_curve, TapeCurve::Linear, "Linear");
                                            ui.selectable_value(&mut selected_curve, TapeCurve::Smooth, "Smooth");
                                            ui.selectable_value(&mut selected_curve, TapeCurve::SlowStart, "SlowStart");
                                            ui.selectable_value(&mut selected_curve, TapeCurve::QuickCut, "QuickCut");
                                        });
                                    if selected_curve != params.curve.value() {
                                        setter.begin_set_parameter(&params.curve);
                                        setter.set_parameter(&params.curve, selected_curve);
                                        setter.end_set_parameter(&params.curve);
                                    }
                                    ui.end_row();

                                    // BPM Sync
                                    ui.label(label("Sync"));
                                    let mut use_sync = params.use_sync.value();
                                    let sync_text = if use_sync { "ON ♪" } else { "OFF" };
                                    if ui.checkbox(&mut use_sync, sync_text).changed() {
                                        setter.begin_set_parameter(&params.use_sync);
                                        setter.set_parameter(&params.use_sync, use_sync);
                                        setter.end_set_parameter(&params.use_sync);
                                    }
                                    ui.end_row();

                                    // スライダー
                                    ui.label(label("Stop Time"));
                                    ui.add(widgets::ParamSlider::for_param(&params.stop_time, setter).with_width(140.0));
                                    ui.end_row();

                                    ui.label(label("Start Time"));
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