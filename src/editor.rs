use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use nih_plug_egui::egui::{Color32, Mesh, Vec2, Pos2, Shape, Stroke, Visuals};

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::params::{TapeStopParams, TapeCurve};

fn get_pastel_color(t: f64, offset: f64) -> Color32 {
    let hue = ((t * 0.05 + offset) % 1.0) as f32;
    egui::ecolor::Hsva::new(hue, 0.4, 0.9, 1.0).into()
}

fn get_kawaii_color(t: f64, offset: f64) -> Color32 {
    let hue = ((t * 0.1 + offset) % 1.0) as f32;
    egui::ecolor::Hsva::new(hue, 0.55, 1.0, 0.5).into()
}

fn paint_amoeba(ui: &mut egui::Ui, center: Pos2, radius: f32, color: Color32, time: f64, seed: f64) {
    let mut mesh = Mesh::default();
    let center_color = color.linear_multiply(1.2);
    mesh.vertices.push(egui::epaint::Vertex { pos: center, uv: Pos2::ZERO, color: center_color });
    let num_points = 64;
    for i in 0..=num_points {
        let angle = (i as f64 / num_points as f64) * std::f64::consts::TAU;
        let noise = (angle * 2.0 + time * 0.5 + seed).sin() * 0.15
            + (angle * 3.0 - time * 0.3 + seed).cos() * 0.1;
        let r = radius * (1.0 + noise as f32);
        let pos = center + Vec2::angled(angle as f32) * r;
        mesh.vertices.push(egui::epaint::Vertex { pos, uv: Pos2::ZERO, color });

        if i < num_points {
            mesh.add_triangle(0, (i + 1) as u32, (i + 2) as u32);
        }
    }

    ui.painter().add(Shape::mesh(mesh));
}

fn set_kawaii_style(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    let pink_pale   = Color32::from_rgb(255, 235, 245);
    let pink_hover  = Color32::from_rgb(255, 245, 250);
    let pink_strong = Color32::from_rgb(255, 160, 200);
    let pink_stroke = Color32::from_rgb(255, 120, 170);
    let text_color  = Color32::from_rgb(110, 80, 100);

    visuals.window_fill = Color32::from_rgb(255, 240, 250);
    visuals.panel_fill  = Color32::from_rgb(255, 240, 250);
    visuals.widgets.inactive.bg_fill      = pink_pale;
    visuals.widgets.inactive.weak_bg_fill = pink_pale;
    visuals.widgets.inactive.bg_stroke    = Stroke::new(1.0, pink_stroke);
    visuals.widgets.inactive.fg_stroke    = Stroke::new(1.0, text_color);
    visuals.widgets.hovered.bg_fill       = pink_hover;
    visuals.widgets.hovered.weak_bg_fill  = pink_hover;
    visuals.widgets.hovered.bg_stroke     = Stroke::new(1.5, pink_stroke);
    visuals.widgets.hovered.fg_stroke     = Stroke::new(1.0, text_color);
    visuals.widgets.hovered.expansion     = 1.0;
    visuals.widgets.active.bg_fill        = pink_strong;
    visuals.widgets.active.weak_bg_fill   = pink_strong;
    visuals.widgets.active.bg_stroke      = Stroke::new(1.0, pink_stroke);
    visuals.widgets.active.fg_stroke      = Stroke::new(1.0, Color32::WHITE);
    visuals.selection.bg_fill             = pink_strong;
    visuals.selection.stroke              = Stroke::new(1.0, pink_stroke);
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
            egui_ctx.request_repaint_after(std::time::Duration::from_millis(8));

            let raw_bits  = peak_meter.load(Ordering::Relaxed);
            let amplitude = f32::from_bits(raw_bits);
            let clean_amp = if amplitude < 0.001 { 0.0 } else { amplitude };
            let boost     = clean_amp * 40.0;

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(Color32::from_rgb(255, 240, 250)))
                .show(egui_ctx, |ui| {
                    let rect       = ui.max_rect();
                    let time       = ui.input(|i| i.time);
                    let color_time = time + (clean_amp as f64 * 2.0);

                    // 背景グラデーション
                    let c_tl = get_pastel_color(color_time, 0.0);
                    let c_tr = get_pastel_color(color_time, 0.25);
                    let c_br = get_pastel_color(color_time, 0.5);
                    let c_bl = get_pastel_color(color_time, 0.75);
                    let mut mesh = Mesh::default();
                    mesh.vertices.push(egui::epaint::Vertex { pos: rect.min,                              uv: Pos2::ZERO, color: c_tl });
                    mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(rect.max.x, rect.min.y),    uv: Pos2::ZERO, color: c_tr });
                    mesh.vertices.push(egui::epaint::Vertex { pos: rect.max,                              uv: Pos2::ZERO, color: c_br });
                    mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(rect.min.x, rect.max.y),    uv: Pos2::ZERO, color: c_bl });
                    mesh.add_triangle(0, 1, 2);
                    mesh.add_triangle(0, 2, 3);
                    ui.painter().add(Shape::mesh(mesh));

                    // アメーバ
                    if clean_amp > 0.0 {
                        for i in 0..6 {
                            let seed = i as f64 * 123.45;
                            let x    = rect.min.x + rect.width()  * (0.5 + 0.4 * (time * 0.15 + seed).sin() as f32);
                            let y    = rect.min.y + rect.height() * (0.5 + 0.4 * (time * 0.2  + seed * 2.0).cos() as f32);
                            let size = 30.0 + 10.0 * (time * 0.5 + seed).sin() as f32 + boost;
                            let color = get_kawaii_color(time, seed * 0.1);
                            paint_amoeba(ui, Pos2::new(x, y), size, color, time, seed);
                        }
                    }

                    // UIパネル
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::Center)
                            .with_cross_align(egui::Align::Center),
                        |ui| {
                            egui::Frame::none()
                                .fill(Color32::from_white_alpha(15))
                                .rounding(20.0)
                                .stroke(Stroke::new(1.5, Color32::from_white_alpha(150)))
                                .shadow(egui::epaint::Shadow {
                                    offset: [0, 0],
                                    blur: 20,
                                    spread: 5,
                                    color: Color32::from_black_alpha(40),
                                })
                                .inner_margin(30.0)
                                .outer_margin(30.0)
                                .show(ui, |ui| {
                                    ui.heading(
                                        egui::RichText::new("Kyun'Stop")
                                            .size(28.0)
                                            .strong()
                                            .color(Color32::from_rgb(200, 80, 120))
                                    );
                                    ui.add_space(15.0);
                                    ui.add_space(5.0);

                                    egui::Grid::new("my_grid")
                                        .num_columns(2)
                                        .spacing([20.0, 15.0])
                                        .show(ui, |ui| {
                                            let label = |text: &str| {
                                                egui::RichText::new(text).size(15.0).color(Color32::from_rgb(100, 90, 110))
                                            };

                                            // CURVE
                                            ui.label(label("CURVE"));

                                            let mut selected_curve = params.curve.value();

                                            ui.scope(|ui| {
                                                let visuals = ui.visuals_mut();
                                                let pink_base   = Color32::from_rgb(255, 220, 240);
                                                let pink_hover  = Color32::from_rgb(255, 235, 250);
                                                let pink_active = Color32::from_rgb(255, 200, 230);
                                                let pink_select = Color32::from_rgb(255, 180, 210);
                                                let pink_stroke = Color32::from_rgb(255, 120, 170);
                                                visuals.widgets.inactive.bg_fill      = pink_base;
                                                visuals.widgets.inactive.weak_bg_fill = pink_base;
                                                visuals.widgets.hovered.bg_fill       = pink_hover;
                                                visuals.widgets.hovered.weak_bg_fill  = pink_hover;
                                                visuals.widgets.hovered.expansion     = 2.0;
                                                visuals.widgets.open.bg_fill          = pink_hover;
                                                visuals.widgets.open.weak_bg_fill     = pink_hover;
                                                visuals.widgets.active.bg_fill        = pink_active;
                                                visuals.widgets.active.weak_bg_fill   = pink_active;
                                                visuals.selection.bg_fill             = pink_select;
                                                visuals.selection.stroke              = Stroke::new(1.0, pink_stroke);
                                                visuals.widgets.inactive.fg_stroke    = Stroke::new(1.0, Color32::from_rgb(120, 80, 100));
                                                visuals.widgets.inactive.bg_stroke    = Stroke::new(1.0, pink_stroke);
                                                visuals.widgets.hovered.fg_stroke     = Stroke::new(1.0, Color32::from_rgb(120, 80, 100));
                                                visuals.widgets.hovered.bg_stroke     = Stroke::new(1.5, pink_stroke);
                                                visuals.widgets.open.bg_stroke        = Stroke::new(1.0, pink_stroke);

                                                let curve_response = egui::ComboBox::new("CURVE", "")
                                                    .selected_text(format!("{:?}", selected_curve))
                                                    .width(130.0)
                                                    .show_ui(ui, |ui| {
                                                        let mut changed = false;
                                                        changed |= ui.selectable_value(&mut selected_curve, TapeCurve::Linear,    "Linear").clicked();
                                                        changed |= ui.selectable_value(&mut selected_curve, TapeCurve::Smooth,    "Smooth").clicked();
                                                        changed |= ui.selectable_value(&mut selected_curve, TapeCurve::SlowStart, "SlowStart").clicked();
                                                        changed |= ui.selectable_value(&mut selected_curve, TapeCurve::QuickCut,  "QuickCut").clicked();
                                                        changed
                                                    });

                                                if curve_response.inner == Some(true) {
                                                    setter.begin_set_parameter(&params.curve);
                                                    setter.set_parameter(&params.curve, selected_curve);
                                                    setter.end_set_parameter(&params.curve);
                                                }
                                            });
                                            ui.end_row();

                                            // BPM SYNC
                                            ui.label(label("SYNC"));
                                            let mut use_sync  = params.use_sync.value();
                                            let sync_text = if use_sync { "ON ♪" } else { "OFF" };
                                            if ui.checkbox(&mut use_sync, sync_text).changed() {
                                                setter.begin_set_parameter(&params.use_sync);
                                                setter.set_parameter(&params.use_sync, use_sync);
                                                setter.end_set_parameter(&params.use_sync);
                                            }
                                            ui.end_row();

                                            // STOP TIME
                                            ui.label(label("STOP TIME"));
                                            ui.add(widgets::ParamSlider::for_param(&params.stop_time, setter).with_width(140.0));
                                            ui.end_row();

                                            // START TIME
                                            ui.label(label("START TIME"));
                                            ui.add(widgets::ParamSlider::for_param(&params.start_time, setter).with_width(140.0));
                                            ui.end_row();
                                        });

                                    ui.add_space(25.0);

                                    // TRIGGER BUTTON
                                    let trigger_val = params.trigger.value();
                                    let btn_text = if trigger_val { "Stopping..." } else { "TAP TO STOP" };
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

                                    let response = ui.add(btn);
                                    let clicked = response.clicked()
                                        || (response.hovered() && ui.input(|i| i.pointer.any_released()));

                                    if clicked {
                                        let new_val = !trigger_val;
                                        setter.begin_set_parameter(&params.trigger);
                                        setter.set_parameter(&params.trigger, new_val);
                                        setter.end_set_parameter(&params.trigger);
                                    }
                                });
                        },
                    );
                });
        },
    )
}