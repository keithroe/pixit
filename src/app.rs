use egui_flex::Flex;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::CentralPanel::default().show(ctx, |ui| {
            Flex::vertical()
                .w_full()
                .h_full()
                .align_items(egui_flex::FlexAlign::Stretch)
                .align_items_content(egui::Align2::CENTER_CENTER)
                .wrap(false)
                .show(ui, |flex| {
                    flex.add_flex(
                        egui_flex::item().grow(2.0),
                        // We need the FlexAlignContent::Stretch to make the buttons fill the space
                        Flex::horizontal()
                            .w_full()
                            .h_full()
                            .align_items(egui_flex::FlexAlign::Stretch)
                            .align_items_content(egui::Align2::CENTER_CENTER)
                            .wrap(false),
                        |flex| {
                            flex.add_ui(
                                egui_flex::FlexItem::default()
                                    .grow(1.0)
                                    .basis(0.0)
                                    .align_self(egui_flex::FlexAlign::Stretch)
                                    .frame(egui::Frame::group(flex.ui().style())),
                                |ui| {
                                    egui::Frame::none().fill(egui::Color32::RED).show(ui, |ui| {
                                        ui.label("Label with red background");
                                    });
                                },
                            );
                            flex.add_ui(
                                egui_flex::FlexItem::default()
                                    .grow(1.0)
                                    .basis(0.0)
                                    .align_self(egui_flex::FlexAlign::Stretch)
                                    .frame(egui::Frame::group(flex.ui().style())),
                                |ui| {
                                    egui::Frame::canvas(ui.style())
                                        .fill(egui::Color32::RED)
                                        .show(ui, |ui| {});
                                },
                            );
                        },
                    );

                    flex.add_flex(
                        egui_flex::item().grow(1.0),
                        // We need the FlexAlignContent::Stretch to make the buttons fill the space
                        Flex::horizontal()
                            .w_full()
                            .h_full()
                            .align_items(egui_flex::FlexAlign::Stretch)
                            .align_items_content(egui::Align2::CENTER_CENTER)
                            .wrap(false),
                        |flex| {
                            flex.add_ui(
                                egui_flex::FlexItem::default()
                                    .grow(1.0)
                                    .frame(egui::Frame::group(flex.ui().style())),
                                |ui| {
                                    ui.label("controls: ");
                                },
                            );
                        },
                    );

                    /*
                                        Flex::horizontal()
                                            .w_full()
                                            .align_items(egui_flex::FlexAlign::Center)
                                            .align_items_content(egui::Align2::CENTER_CENTER)
                                            .wrap(false)
                                            .show(ui, |flex| {
                                                flex.add_ui(
                                                    egui_flex::FlexItem::default()
                                                        .grow(1.0)
                                                        .frame(egui::Frame::group(flex.ui().style())),
                                                    |ui| {
                                                        ui.label("left viewport: ");
                                                    },
                                                );
                                                flex.add_ui(
                                                    egui_flex::FlexItem::default()
                                                        .grow(1.0)
                                                        .frame(egui::Frame::group(flex.ui().style())),
                                                    |ui| {
                                                        ui.label("right viewport: ");
                                                    },
                                                );
                                            });
                    */
                })
        });

        /*
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("pixit");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        */
    }
}
