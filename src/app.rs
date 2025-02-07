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
    renderer0: Option<render::Renderer>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    renderer1: Option<render::Renderer>,

    #[serde(skip)] // This how you opt-out of serialization of a field
    frame_state0: render::FrameState,

    #[serde(skip)] // This how you opt-out of serialization of a field
    frame_state1: render::FrameState,
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
            frame_state0: render::FrameState::default(),
            frame_state1: render::FrameState::default(),
            renderer0: None,
            renderer1: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();

        let mut app = App::default();
        app.renderer0 = Some(render::Renderer::new(wgpu_render_state));
        app.renderer1 = Some(render::Renderer::new(wgpu_render_state));
        app
    }

    fn render_left_viewport(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(512.0), egui::Sense::drag());

        self.frame_state0.angle += response.drag_motion().x * 0.01;
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            render::RenderCallback {
                resource_idx: 0,
                frame_state: render::FrameState {
                    angle: self.frame_state0.angle,
                },
            },
        ));
    }

    fn render_right_viewport(&mut self, ui: &mut egui::Ui) {
        let image = egui::Image::new(egui::include_image!("../assets/monkey_pixel.png"))
            .max_size(egui::Vec2::new(512.0, 512.0));
        ui.add(image);
    }
}

impl eframe::App for App {
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
