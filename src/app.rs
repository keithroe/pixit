use eframe::egui_wgpu;
use egui_flex::Flex;

use crate::render;

// TODO:
//     * use render-to-tex instead of paint callback for better separation of egui/wgpu
//     * decide on app-restore or not

pub struct App {
    num_frames: i32,
    cur_frame: i32,

    frame_state: render::FrameState,
    render_state_idx: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            num_frames: 30,
            cur_frame: 1,
            frame_state: render::FrameState::default(),
            render_state_idx: 0,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        let render_state_idx = render::init(wgpu_render_state);
        App {
            render_state_idx,
            ..Default::default()
        }
    }

    fn render_left_viewport(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(512.0), egui::Sense::drag());

        self.frame_state.angle += response.drag_motion().x * 0.01;
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            render::RenderCallback {
                resource_idx: self.render_state_idx,
                frame_state: render::FrameState {
                    angle: self.frame_state.angle,
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
        egui::CentralPanel::default().show(ctx, |ui| {
            Flex::vertical()
                .w_full()
                .h_full()
                .align_items(egui_flex::FlexAlign::Stretch)
                .align_content(egui_flex::FlexAlignContent::Stretch)
                .align_content(egui_flex::FlexAlignContent::Stretch)
                .show(ui, |flex| {
                    // Top pane containing rendered images
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
                    // Botom pane containing controls
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
