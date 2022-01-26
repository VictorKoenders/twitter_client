#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc::{Receiver, Sender, channel};
use sixtyfps::Weak;

fn main() {
    let window = MainWindow::new();
    let background = Background::spawn(window.as_weak());
    {
        let background = background.clone();
        window.on_arrowDownPressed(move || background.send(ToBackground::ArrowDown));
    }
    {
        let background = background.clone();
        window.on_arrowUpPressed(move|| background.send(ToBackground::ArrowUp));
    }
    window.run();
}
sixtyfps::include_modules!();

struct Background {
    ui: Weak<MainWindow>,
    receiver: Receiver<ToBackground>,
}

impl Background {
    fn spawn(ui: Weak<MainWindow>) -> BackgroundHandle {
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            Background { ui, receiver }.run();
        });
        BackgroundHandle { sender }
    }

    fn run(self) {
        while let Ok(item) = self.receiver.recv() {
            match item {
                ToBackground::ArrowDown => self.ui.clone().upgrade_in_event_loop(|window| {
                    window.set_activeId(9);
                }),
                ToBackground::ArrowUp => self.ui.clone().upgrade_in_event_loop(|window| {
                    window.set_activeId(1);
                }),
            }
        }
    }
}

#[derive(Clone)]
struct BackgroundHandle {
    sender: Sender<ToBackground>,
}

impl BackgroundHandle {
    fn send(&self, b: ToBackground) {
        if let Err(e) = self.sender.send(b) {
            eprintln!("Could not send to background: {:?}", e);
        }
    }
}

enum ToBackground {
    ArrowUp,
    ArrowDown,
}

// mod background;
// mod image;
// mod ui;

// use background::{Background, ToUI};
// use egui_glium::egui_winit::WindowSettings;
// use epi::{file_storage::FileStorage, Storage};
// use glium::glutin::{
//     self,
//     event::{ElementState, Event, WindowEvent},
//     event_loop::{ControlFlow, EventLoop},
//     window::Window,
// };
// use std::{
//     sync::Arc,
//     time::{Duration, Instant},
// };

// struct RepaintSignal {
//     proxy: std::sync::Mutex<glutin::event_loop::EventLoopProxy<ToUI>>,
// }

// impl epi::backend::RepaintSignal for RepaintSignal {
//     fn request_repaint(&self) {
//         self.proxy
//             .lock()
//             .unwrap()
//             .send_event(ToUI::Repaint)
//             .unwrap();
//     }
// }

// fn main() {
//     let _ = dotenv::dotenv();
//     pretty_env_logger::init();

//     let title = "Rusty twitter client";
//     let mut persistence = Persistence::from_app_name(title);
//     let event_loop = EventLoop::with_user_event();
//     let background = background::spawn(event_loop.create_proxy());
//     let display = create_display(&persistence, &event_loop, title);

//     let mut integration = Integration::new(
//         title,
//         egui_glium::EguiGlium::new(&display),
//         ui::State::default(),
//         Arc::new(RepaintSignal {
//             proxy: std::sync::Mutex::new(event_loop.create_proxy()),
//         }),
//         background,
//     );

//     let mut last_image_cleanup = Instant::now();

//     event_loop.run(move |event, _, control_flow| {
//         let mut redraw = || {
//             if last_image_cleanup.elapsed().as_secs() >= 1 {
//                 image::cleanup(&integration.frame);
//                 last_image_cleanup = Instant::now();
//             }
//             let (needs_repaint, mut tex_allocation_data, shapes) =
//                 integration.update(display.gl_window().window());
//             let clipped_meshes = integration.egui_glium.egui_ctx.tessellate(shapes);

//             let painter = &mut integration.egui_glium.painter;

//             for (id, image) in tex_allocation_data.creations {
//                 painter.set_texture(&display, id, &image);
//             }
//             {
//                 use glium::Surface as _;
//                 let mut target = display.draw();
//                 let color: f32 = 3.0 / 255.0;
//                 target.clear_color(color, color, color, 1.0);

//                 painter.paint_meshes(
//                     &display,
//                     &mut target,
//                     integration.egui_glium.egui_ctx.pixels_per_point(),
//                     clipped_meshes,
//                     &integration.egui_glium.egui_ctx.font_image(),
//                 );

//                 target.finish().unwrap();
//             }

//             for id in tex_allocation_data.destructions.drain(..) {
//                 log::info!(target: "image", "Destroying texture {}", id);
//                 painter.free_texture(id);
//             }

//             *control_flow = if !integration.app.is_running() {
//                 ControlFlow::Exit
//             } else if needs_repaint {
//                 display.gl_window().window().request_redraw();
//                 ControlFlow::Poll
//             } else {
//                 ControlFlow::Wait
//             };
//         };

//         match event {
//             // Platform-dependent event handlers to workaround a winit bug
//             // See: https://github.com/rust-windowing/winit/issues/987
//             // See: https://github.com/rust-windowing/winit/issues/1619
//             Event::RedrawEventsCleared if cfg!(windows) => redraw(),
//             Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

//             Event::WindowEvent { event, .. } => {
//                 if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
//                     *control_flow = glutin::event_loop::ControlFlow::Exit;
//                 }

//                 let egui_consumed = integration.egui_glium.on_event(&event);
//                 if !egui_consumed {
//                     match event {
//                         WindowEvent::KeyboardInput { input, .. } => {
//                             match (input.virtual_keycode, input.state) {
//                                 (Some(virtual_keycode), ElementState::Pressed) => integration
//                                     .app
//                                     .key_pressed(&mut integration.background, virtual_keycode),
//                                 (Some(virtual_keycode), ElementState::Released) => {
//                                     integration.app.key_released(virtual_keycode)
//                                 }
//                                 _ => {}
//                             }
//                         }
//                         e => log::trace!(target: "Event", "Unhandled {:?}", e),
//                     }
//                 }

//                 display.gl_window().window().request_redraw();
//             }

//             glutin::event::Event::UserEvent(ToUI::Repaint) => {
//                 display.gl_window().window().request_redraw();
//             }
//             glutin::event::Event::UserEvent(ToUI::ImageLoaded(img)) => {
//                 img.finish_load(&mut integration.frame);
//             }
//             glutin::event::Event::UserEvent(msg) => {
//                 integration.app.update(&mut integration.background, msg);
//                 display.gl_window().window().request_redraw();
//             }

//             _ => (),
//         }
//         persistence.maybe_autosave(&display);
//     });
// }

// pub struct Persistence {
//     storage: Option<FileStorage>,
//     last_auto_save: std::time::Instant,
// }

// impl Persistence {
//     const WINDOW_KEY: &'static str = "window";
//     const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(5 * 60);

//     pub fn from_app_name(app_name: &str) -> Self {
//         Self {
//             storage: FileStorage::from_app_name(app_name),
//             last_auto_save: std::time::Instant::now(),
//         }
//     }

//     pub fn save(&mut self, display: &glium::Display) {
//         if let Some(storage) = &mut self.storage {
//             epi::set_value(
//                 storage,
//                 Self::WINDOW_KEY,
//                 &WindowSettings::from_display(display.gl_window().window()),
//             );
//             storage.flush();
//         }
//     }

//     pub fn maybe_autosave(&mut self, display: &glium::Display) {
//         let now = std::time::Instant::now();
//         if now - self.last_auto_save > Self::AUTO_SAVE_INTERVAL {
//             self.save(display);
//             self.last_auto_save = now;
//         }
//     }

//     pub fn load_window_settings(&self) -> Option<crate::WindowSettings> {
//         epi::get_value(self.storage.as_ref()?, Self::WINDOW_KEY)
//     }
// }

// pub struct Context<'a> {
//     pub ctx: &'a egui::CtxRef,
//     pub frame: &'a epi::Frame,
//     pub background: &'a mut Background,
// }

// pub struct Integration {
//     frame: epi::Frame,
//     background: Background,
//     pub egui_glium: egui_glium::EguiGlium,
//     pub app: ui::State,
// }

// impl Integration {
//     fn new(
//         title: &'static str,
//         egui_glium: egui_glium::EguiGlium,
//         app: ui::State,
//         repaint_signal: Arc<dyn epi::backend::RepaintSignal>,
//         background: Background,
//     ) -> Self {
//         let frame = epi::Frame::new(epi::backend::FrameData {
//             info: epi::IntegrationInfo {
//                 name: title,
//                 web_info: None,
//                 prefer_dark_mode: Some(true),
//                 cpu_usage: None,
//                 native_pixels_per_point: Some(egui_glium.egui_winit.pixels_per_point()),
//             },
//             output: Default::default(),
//             repaint_signal,
//         });
//         Self {
//             frame,
//             egui_glium,
//             app,
//             background,
//         }
//     }

//     pub fn update(
//         &mut self,
//         window: &Window,
//     ) -> (
//         bool,
//         epi::backend::TexAllocationData,
//         Vec<egui::epaint::ClippedShape>,
//     ) {
//         let frame_start = std::time::Instant::now();

//         let raw_input = self.egui_glium.egui_winit.take_egui_input(window);
//         let (egui_output, shapes) = self.egui_glium.egui_ctx.run(raw_input, |egui_ctx| {
//             self.app.draw(&mut Context {
//                 ctx: egui_ctx,
//                 frame: &mut self.frame,
//                 background: &mut self.background,
//             });
//         });

//         let needs_repaint = egui_output.needs_repaint;
//         self.egui_glium
//             .egui_winit
//             .handle_output(window, &self.egui_glium.egui_ctx, egui_output);

//         let app_output = self.frame.take_app_output();
//         let tex_allocation_data = egui_glium::egui_winit::epi::handle_app_output(
//             window,
//             self.egui_glium.egui_ctx.pixels_per_point(),
//             app_output,
//         );

//         let frame_time = (std::time::Instant::now() - frame_start).as_secs_f64() as f32;
//         self.frame.lock().info.cpu_usage = Some(frame_time);

//         (needs_repaint, tex_allocation_data, shapes)
//     }
// }

// fn create_display(
//     persistence: &Persistence,
//     event_loop: &glutin::event_loop::EventLoop<ToUI>,
//     title: &str,
// ) -> glium::Display {
//     let window_settings = persistence.load_window_settings();
//     let window_builder = egui_glium::egui_winit::epi::window_builder(
//         &epi::NativeOptions {
//             maximized: true,
//             ..Default::default()
//         },
//         &window_settings,
//     )
//     .with_title(title);
//     let context_builder = glutin::ContextBuilder::new()
//         .with_depth_buffer(0)
//         .with_srgb(true)
//         .with_stencil_buffer(0)
//         .with_vsync(true);

//     glium::Display::new(window_builder, context_builder, event_loop).unwrap()
// }
