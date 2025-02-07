use std::fmt::write;

use eframe::egui_wgpu;
use egui_flex::Flex;

use crate::render;

#[derive(Default)]
pub struct Model {}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    model: Model,

    #[serde(skip)] // This how you opt-out of serialization of a field
    fps: i32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    num_frames: i32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    cur_frame: i32,

    #[serde(skip)] // This how you opt-out of serialization of a field
    renderer: Option<render::Renderer>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    frame_state: render::FrameState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            model: Model {},
            fps: 60,
            num_frames: 30,
            cur_frame: 1,
            frame_state: render::FrameState::default(),
            renderer: None,
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
        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        */

        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let mut app = App::default();
        app.renderer = Some(render::Renderer::new(wgpu_render_state));
        app
    }

    fn render_left_viewport(&mut self, ui: &mut egui::Ui) {
        Flex::vertical().grow_items(1.0).show(ui, |flex| {
            flex.add_ui(
                egui_flex::FlexItem::default()
                    .grow(1.0)
                    .frame(egui::Frame::group(flex.ui().style())),
                |ui| {
                    //egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    //egui::ScrollArea::both().auto_shrink(true).show(ui, |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(egui::Vec2::splat(512.0), egui::Sense::drag());

                    self.frame_state.angle += response.drag_motion().x * 0.01;
                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        render::RenderCallback {
                            frame_state: render::FrameState {
                                angle: self.frame_state.angle,
                            },
                        },
                    ));
                    /*
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.custom_painting(ui);
                    });
                    ui.label("Drag to rotate!");
                    */
                    //})
                },
            );
        });
        /*
        Flex::vertical()
            .grow_items(1.0)
            .align_items(egui_flex::FlexAlign::Stretch)
            .show(ui, |flex| {
                //ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let image = egui::Image::new(egui::include_image!("../assets/monkey_orig.png"))
                    .max_size(egui::Vec2::new(512.0, 512.0));
                //ui.add(egui_flex::item(), image);
                flex.add(egui_flex::item(), image);
            });
        */

        /*
        egui::ScrollArea::both()
             .auto_shrink(false)
             .show(ui, |ui| {
                 ui.horizontal(|ui| {
                     ui.spacing_mut().item_spacing.x = 0.0;
                     ui.label("The triangle is being painted using ");
                     ui.hyperlink_to("WGPU", "https://wgpu.rs");
                     ui.label(" (Portable Rust graphics API awesomeness)");
                 });
                 ui.label("It's not a very impressive demo, but it shows you can embed 3D inside of egui.");

                 egui::Frame::canvas(ui.style()).show(ui, |ui| {
                     self.custom_painting(ui);
                 });
                 ui.label("Drag to rotate!");
             });
        */
    }

    fn render_right_viewport(&mut self, ui: &mut egui::Ui) {
        let image = egui::Image::new(egui::include_image!("../assets/monkey_pixel.png"))
            .max_size(egui::Vec2::new(512.0, 512.0));
        //ui.add(egui_flex::item(), image);
        ui.add(image);
    }
}

impl eframe::App for App {
    /*
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.image(egui::include_image!("../assets/monkey_orig.png"))
                    .on_hover_text_at_pointer("Svg");
            });
        });
    }
    */

    /// Called by the frame work to save state before shutdown.
    /*
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    */

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::CentralPanel::default().show(ctx, |ui| {
            Flex::vertical()
                .w_full()
                .h_full()
                .align_items(egui_flex::FlexAlign::Stretch)
                .align_content(egui_flex::FlexAlignContent::Stretch)
                .align_content(egui_flex::FlexAlignContent::Stretch)
                .show(ui, |flex| {
                    flex.add_ui(
                        egui_flex::FlexItem::default()
                            .grow(1.0)
                            .frame(egui::Frame::group(flex.ui().style())),
                        |ui| {
                            Flex::horizontal()
                                .w_full()
                                .justify(egui_flex::FlexJustify::SpaceAround)
                                .show(ui, |flex| {
                                    flex.add_ui(
                                        egui_flex::FlexItem::default()
                                            .grow(1.0)
                                            .frame(egui::Frame::group(flex.ui().style())),
                                        |ui| {
                                            self.render_left_viewport(ui);
                                        },
                                    );
                                    flex.add_ui(
                                        egui_flex::FlexItem::default()
                                            .grow(1.0)
                                            .frame(egui::Frame::group(flex.ui().style())),
                                        |ui| {
                                            self.render_right_viewport(ui);
                                        },
                                    );

                                    /*
                                    let image = egui::Image::new(egui::include_image!(
                                        "../assets/monkey_pixel.png"
                                    ))
                                    .max_size(egui::Vec2::new(512.0, 512.0));
                                    flex.add(egui_flex::item(), image);
                                    */
                                })
                        },
                    );
                    flex.add_ui(
                        egui_flex::FlexItem::default()
                            .grow(1.0)
                            .frame(egui::Frame::group(flex.ui().style())),
                        |ui| {
                            ui.scope(|ui| {
                                ui.spacing_mut().slider_width = ui.available_width() - 48.0;
                                Flex::vertical()
                                    .grow_items(1.0)
                                    .align_items(egui_flex::FlexAlign::Stretch)
                                    .show(ui, |flex| {
                                        //ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                        let timeline = egui::Slider::new(
                                            &mut self.cur_frame,
                                            1..=self.num_frames,
                                        );
                                        flex.add(egui_flex::item().grow(1.0).basis(0.0), timeline);
                                    });
                            });
                        },
                    );
                });
        });
    }
}
