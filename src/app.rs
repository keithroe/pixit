use egui_flex::Flex;

use crate::model;
use crate::render;

// TODO:
//     * decide on app-restore or not

pub struct App {
    num_frames: i32,
    cur_frame: i32,

    renderer: render::Renderer,
    render_texture: egui::load::SizedTexture,
}

impl App {
    const VIEWPORT_WIDTH: u32 = 512;
    const VIEWPORT_HEIGHT: u32 = 512;

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let model = model::Model::load("assets/Fox.glb");

        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        let renderer = render::Renderer::new(
            App::VIEWPORT_WIDTH,
            App::VIEWPORT_HEIGHT,
            wgpu_render_state.device.clone(),
            wgpu_render_state.queue.clone(),
            &model,
        );

        let render_texture_id = wgpu_render_state.renderer.write().register_native_texture(
            &wgpu_render_state.device,
            renderer.get_render_texture_view(),
            wgpu::FilterMode::Nearest,
        );
        App {
            num_frames: 60, // TODO: connect this value
            cur_frame: 0,
            renderer,
            render_texture: egui::load::SizedTexture {
                size: egui::Vec2::new(App::VIEWPORT_WIDTH as f32, App::VIEWPORT_HEIGHT as f32),
                id: render_texture_id,
            },
        }
    }

    fn render_left_viewport(&mut self, ui: &mut egui::Ui) {
        self.renderer.render();
        let image = egui::Image::from_texture(self.render_texture)
            .sense(egui::Sense::drag())
            .max_size(egui::Vec2::new(512.0, 512.0));
        let response = ui.add(image);
        if response.dragged() {
            println!(
                "dragged {} from {:?}",
                response.drag_motion(),
                response.interact_pointer_pos().unwrap() - response.rect.min,
            );
        }
    }

    fn render_right_viewport(&mut self, ui: &mut egui::Ui) {
        let image = egui::Image::new(egui::include_image!("../assets/monkey_pixel.png"))
            .max_size(egui::Vec2::new(512.0, 512.0));
        ui.add(image);
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::style::Visuals) -> [f32; 4] {
        [0.1, 0.1, 0.2, 0.5]
    }
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
