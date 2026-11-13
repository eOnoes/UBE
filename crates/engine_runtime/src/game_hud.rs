use crate::game::GameHud;

/// Immediate-mode presentation for renderer-neutral application HUD data.
pub struct GameHudPresenter {
    context: egui::Context,
}

impl Default for GameHudPresenter {
    fn default() -> Self {
        let context = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "IBM Plex Mono".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/IBMPlexMono-Regular.ttf"
            ))),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, "IBM Plex Mono".into());
        }
        context.set_fonts(fonts);
        Self { context }
    }
}

impl GameHudPresenter {
    pub fn frame(
        &mut self,
        hud: &GameHud,
        width: u32,
        height: u32,
    ) -> (egui::Context, egui::FullOutput) {
        let width = width.max(1) as f32;
        let height = height.max(1) as f32;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            ..Default::default()
        };
        let output = self.context.run_pass(input, |context| {
            let panel = egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(8, 18, 24, 220))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(72, 112, 112),
                ))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(10));

            egui::Area::new(egui::Id::unique("game_status"))
                .fixed_pos(egui::pos2(18.0, 18.0))
                .show(context, |ui| {
                    panel.show(ui, |ui| {
                        ui.set_width(260.0);
                        ui.label(
                            egui::RichText::new(&hud.title)
                                .size(22.0)
                                .strong()
                                .color(egui::Color32::from_rgb(239, 202, 112)),
                        );
                        ui.label(
                            egui::RichText::new(&hud.subtitle)
                                .size(12.0)
                                .color(egui::Color32::from_rgb(151, 202, 195)),
                        );
                        ui.add_space(6.0);
                        meter(
                            ui,
                            "HEALTH",
                            hud.health,
                            egui::Color32::from_rgb(221, 88, 76),
                        );
                        meter(
                            ui,
                            "HUNGER",
                            hud.hunger,
                            egui::Color32::from_rgb(232, 177, 74),
                        );
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&hud.objective).size(12.0));
                    });
                });

            egui::Area::new(egui::Id::unique("game_resources"))
                .fixed_pos(egui::pos2((width - 292.0).max(18.0), 18.0))
                .show(context, |ui| {
                    panel.show(ui, |ui| {
                        ui.set_width(250.0);
                        ui.label(
                            egui::RichText::new(&hud.compass)
                                .size(13.0)
                                .strong()
                                .color(egui::Color32::from_rgb(151, 202, 195)),
                        );
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&hud.resources).size(13.0));
                    });
                });

            if hud.crosshair {
                let painter = context.layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::unique("game_crosshair"),
                ));
                let center = egui::pos2(width * 0.5, height * 0.5);
                let color = egui::Color32::from_rgba_unmultiplied(255, 244, 205, 230);
                painter.line_segment(
                    [center - egui::vec2(8.0, 0.0), center - egui::vec2(3.0, 0.0)],
                    egui::Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [center + egui::vec2(3.0, 0.0), center + egui::vec2(8.0, 0.0)],
                    egui::Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [center - egui::vec2(0.0, 8.0), center - egui::vec2(0.0, 3.0)],
                    egui::Stroke::new(2.0, color),
                );
                painter.line_segment(
                    [center + egui::vec2(0.0, 3.0), center + egui::vec2(0.0, 8.0)],
                    egui::Stroke::new(2.0, color),
                );
            }

            egui::Area::new(egui::Id::unique("game_prompt"))
                .fixed_pos(egui::pos2(width * 0.5 - 230.0, height * 0.5 + 42.0))
                .show(context, |ui| {
                    ui.set_width(460.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&hud.prompt)
                                .size(15.0)
                                .strong()
                                .background_color(egui::Color32::from_rgba_unmultiplied(
                                    8, 18, 24, 220,
                                ))
                                .color(egui::Color32::from_rgb(245, 232, 184)),
                        );
                        if !hud.ammo.is_empty() {
                            ui.label(
                                egui::RichText::new(&hud.ammo)
                                    .size(14.0)
                                    .background_color(egui::Color32::from_rgba_unmultiplied(
                                        8, 18, 24, 210,
                                    ))
                                    .color(egui::Color32::from_rgb(219, 235, 218)),
                            );
                        }
                    });
                });

            let slot_width = 112.0;
            let bar_width = (hud.slots.len() as f32 * slot_width).max(220.0);
            egui::Area::new(egui::Id::unique("game_hotbar"))
                .fixed_pos(egui::pos2((width - bar_width) * 0.5, height - 150.0))
                .show(context, |ui| {
                    ui.horizontal(|ui| {
                        for slot in &hud.slots {
                            let fill = if slot.selected {
                                egui::Color32::from_rgba_unmultiplied(86, 101, 65, 240)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(8, 18, 24, 225)
                            };
                            egui::Frame::new()
                                .fill(fill)
                                .stroke(egui::Stroke::new(
                                    if slot.selected { 2.0 } else { 1.0 },
                                    if slot.selected {
                                        egui::Color32::from_rgb(239, 202, 112)
                                    } else {
                                        egui::Color32::from_rgb(72, 112, 112)
                                    },
                                ))
                                .corner_radius(egui::CornerRadius::same(5))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.set_width(94.0);
                                    ui.set_height(48.0);
                                    ui.label(egui::RichText::new(&slot.label).strong().size(13.0));
                                    ui.label(
                                        egui::RichText::new(&slot.detail)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(192, 214, 200)),
                                    );
                                });
                        }
                    });
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&hud.status)
                                .size(13.0)
                                .background_color(egui::Color32::from_rgba_unmultiplied(
                                    8, 18, 24, 205,
                                ))
                                .color(egui::Color32::from_rgb(239, 202, 112)),
                        );
                        ui.label(
                            egui::RichText::new(&hud.controls)
                                .size(10.0)
                                .background_color(egui::Color32::from_rgba_unmultiplied(
                                    8, 18, 24, 205,
                                ))
                                .color(egui::Color32::from_rgb(192, 214, 200)),
                        );
                    });
                });

            if hud.intro_seconds > 0.0 {
                let alpha = (hud.intro_seconds.min(2.0) * 110.0) as u8;
                egui::Area::new(egui::Id::unique("game_intro"))
                    .fixed_pos(egui::pos2(width * 0.5 - 250.0, height * 0.25))
                    .show(context, |ui| {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(
                                8,
                                18,
                                24,
                                210 + alpha / 5,
                            ))
                            .stroke(egui::Stroke::new(
                                2.0,
                                egui::Color32::from_rgb(239, 202, 112),
                            ))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(20))
                            .show(ui, |ui| {
                                ui.set_width(460.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("SURVIVING ORNITH")
                                            .size(32.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(239, 202, 112)),
                                    );
                                    ui.label(
                                        egui::RichText::new("ORNITH ISLAND  •  UBE nerd 0.1")
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(151, 202, 195)),
                                    );
                                    ui.add_space(8.0);
                                    ui.label("Gather. Hunt. Tame. Build a place to survive.");
                                    ui.label("Right-click to look around  •  V changes camera");
                                });
                            });
                    });
            }
        });
        (self.context.clone(), output)
    }
}

fn meter(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    let value = value.clamp(0.0, 100.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(11.0));
        ui.add_space(6.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(140.0, 12.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(34, 45, 47));
        let filled = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width() * value / 100.0, rect.height()),
        );
        ui.painter().rect_filled(filled, 3.0, color);
        ui.label(egui::RichText::new(format!("{value:.0}")).size(11.0));
    });
}
