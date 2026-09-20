use std::sync::Arc;

use pixels::{Pixels, PixelsBuilder, SurfaceTexture, wgpu::Backends};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::{graphics, systems};

pub struct Graphics {
    pub window: Arc<Window>,
    pub pixels: Pixels<'static>,
}

#[derive(Default)]
pub struct App {
    pub game_state: Option<systems::GameState>,
    pub graphics: Option<Graphics>,
    pub music: Option<systems::BGMusicPlayer>,
    pub proxy: Option<EventLoopProxy<Graphics>>,
    pub canvas_parent: Option<String>,
    pub cursor_pos: (f64, f64),
}

impl App {
    fn handle_tap(&mut self, x: u32, y: u32) {
        let Some(game_state) = self.game_state.as_mut() else {
            return;
        };
        for button in graphics::buttons(game_state) {
            if button.contains(x, y) {
                (button.on_click)(game_state);
                return; // early return, can't use functional tools
            }
        }
        let pos = systems::Position { x, y };

        if let Some(entity) = graphics::troop_at(&game_state.world, x, y) {
            if game_state.deleting {
                game_state.world.despawn(entity);
                return;
            }
        } else {
            systems::spawn_troop(&mut game_state.world, pos, game_state.team_mode);
        }
    }
}

impl ApplicationHandler<Graphics> for App {
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(music) = self.music.as_mut() {
            music.update(); // update every loop iteration
        }
        if let Some(graphics) = self.graphics.as_ref() {
            graphics.window.request_redraw();
        }
    }
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }
        let window_attributes = Window::default_attributes()
            .with_title("venture")
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.music = Some(systems::BGMusicPlayer::new());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas().expect("winit gave no canvas");
            canvas.style().set_property("width", "100%").unwrap();
            canvas.style().set_property("height", "100%").unwrap();
            canvas.style().set_property("display", "block").unwrap();
            canvas.set_tab_index(0);
            let doc = web_sys::window().unwrap().document().unwrap();
            let parent = doc
                .get_element_by_id(self.canvas_parent.as_ref().expect("couldn't find parent"))
                .unwrap_or_else(|| doc.body().unwrap().into());
            parent.append_child(&canvas).unwrap();
            let _ = canvas.focus();
        }

        let proxy = self.proxy.clone().expect("proxy not set before resumed");
        let build = async move {
            let size = window.inner_size();
            let (w, h) = (size.width.max(1), size.height.max(1));
            let surface_texture = SurfaceTexture::new(w, h, window.clone());
            let pixels = PixelsBuilder::new(w, h, surface_texture)
                .wgpu_backend(Backends::GL)
                .build_async()
                .await
                .expect("pixels build failed");
            let graphics = Graphics { window, pixels };
            let _ = proxy.send_event(graphics);
        };

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(build);
        #[cfg(not(target_arch = "wasm32"))]
        pollster::block_on(build);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
        self.graphics = Some(graphics);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.graphics.is_none() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let graphics = self.graphics.as_mut().unwrap();
                if size.width > 0 && size.height > 0 {
                    graphics
                        .pixels
                        .resize_surface(size.width, size.height)
                        .unwrap();
                    graphics
                        .pixels
                        .resize_buffer(size.width, size.height)
                        .unwrap();
                    graphics.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                {
                    let graphics = self.graphics.as_mut().unwrap();
                    for pixel in graphics.pixels.frame_mut().chunks_exact_mut(4) {
                        pixel.copy_from_slice(&[16, 212, 48, 255]);
                    }
                }

                graphics::draw_fn(self);

                let graphics = self.graphics.as_mut().unwrap();
                if let Err(err) = graphics.pixels.render() {
                    eprintln!("render failed: {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && !event.repeat
                    && event.physical_key == PhysicalKey::Code(KeyCode::KeyM)
                {
                    if let Some(game_state) = self.game_state.as_mut() {
                        game_state.change_teams()
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && button == MouseButton::Left {
                    let (x, y) = (self.cursor_pos.0 as u32, self.cursor_pos.1 as u32);
                    self.handle_tap(x, y);
                }
            }
            WindowEvent::Touch(touch) => {
                if touch.phase == TouchPhase::Started {
                    self.handle_tap(touch.location.x as u32, touch.location.y as u32);
                }
            }
            _ => (),
        }
    }
}
