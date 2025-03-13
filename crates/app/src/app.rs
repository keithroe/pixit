use eframe::egui_wgpu;
//use eframe::{wgpu::Device, wgpu::Queue};
use egui_flex::Flex;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to gltf/glb file to be rendered
    #[arg(short, long, default_value = "assets/Avocado.glb")]
    gltf: String,
}

struct RenderViewport {
    renderer: render::Renderer,
    render_texture: egui::load::SizedTexture,
}

impl RenderViewport {
    fn new(wgpu_render_state: &egui_wgpu::RenderState, mesh: &model::Model) -> Self {
        let renderer = render::Renderer::new(
            &[App::VIEWPORT_WIDTH, App::VIEWPORT_HEIGHT],
            wgpu_render_state.device.clone(),
            wgpu_render_state.queue.clone(),
            mesh,
        );

        let render_texture_id = wgpu_render_state.renderer.write().register_native_texture(
            &wgpu_render_state.device,
            renderer.get_render_texture_view(),
            wgpu::FilterMode::Nearest,
        );

        let render_texture = egui::load::SizedTexture {
            size: egui::Vec2::new(App::VIEWPORT_WIDTH as f32, App::VIEWPORT_HEIGHT as f32),
            id: render_texture_id,
        };
        Self {
            render_texture,
            renderer,
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.renderer.render();
        let image = egui::Image::from_texture(self.render_texture)
            .sense(egui::Sense::drag())
            .max_size(egui::Vec2::new(512.0, 512.0));
        let response = ui.add(image);

        if response.dragged() {
            let egui_drag_begin = response.interact_pointer_pos().unwrap() - response.rect.min;
            let egui_drag_end = egui_drag_begin + response.drag_motion();

            let drag_begin = glam::Vec2::new(egui_drag_begin.x, egui_drag_begin.y);
            let drag_end = glam::Vec2::new(egui_drag_end.x, egui_drag_end.y);
            let modifiers = event::Modifiers::default(); // TODO: handle modifiers
            let button = if response.dragged_by(egui::PointerButton::Primary) {
                event::MouseButton::Primary
            } else if response.dragged_by(egui::PointerButton::Secondary) {
                event::MouseButton::Secondary
            } else {
                // if response.dragged_by(egui::PointerButton::Middle) {
                event::MouseButton::Middle
            };

            self.renderer.handle_event(event::Event::Drag {
                button,
                modifiers,
                drag_begin,
                drag_end,
            });
        }
    }
}

pub struct App {
    num_frames: i32,
    cur_frame: i32,

    render_viewport: RenderViewport,
}

impl App {
    const VIEWPORT_WIDTH: u32 = 512;
    const VIEWPORT_HEIGHT: u32 = 512;

    /// Called once before the first frame.
    pub fn new(args: Args, cc: &eframe::CreationContext<'_>) -> Self {
        //let model = model::Model::from_gltf("assets/Fox.glb");
        let model = model::Model::from_gltf(&args.gltf);

        App {
            num_frames: 60, // TODO: connect this value
            cur_frame: 0,
            render_viewport: RenderViewport::new(cc.wgpu_render_state.as_ref().unwrap(), &model),
        }
    }

    fn render_left_viewport(&mut self, ui: &mut egui::Ui) {
        self.render_viewport.draw(ui);
    }

    fn render_right_viewport(&mut self, ui: &mut egui::Ui) {
        let image = egui::Image::new(egui::include_image!("../../../assets/monkey_pixel.png"))
            .max_size(egui::Vec2::new(512.0, 512.0));
        ui.add(image);
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::style::Visuals) -> [f32; 4] {
        [0.05, 0.05, 0.05, 1.0]
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
