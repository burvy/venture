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
    pub pan_up: bool,
    pub pan_down: bool,
    pub pan_left: bool,
    pub pan_right: bool,
    pub is_mobile: bool,
    pub mouse_down: bool,
    pub last_drag_pos: Option<(i32, i32)>,
    pub dpad_touch: Option<u64>,
}

impl App {
    fn set_pan(&mut self, direction: systems::PanDirection) {
        self.pan_up = matches!(direction, systems::PanDirection::UP);
        self.pan_down = matches!(direction, systems::PanDirection::DOWN);
        self.pan_left = matches!(direction, systems::PanDirection::LEFT);
        self.pan_right = matches!(direction, systems::PanDirection::RIGHT);
    }

    fn clear_pan(&mut self) {
        self.pan_up = false;
        self.pan_down = false;
        self.pan_left = false;
        self.pan_right = false;
    }
    /// determines what happens if you drag
    fn handle_drag(&mut self, x: u32, y: u32) {
        let Some(game_state) = self.game_state.as_mut() else {
            return;
        };
        let world_x = x as i32 + game_state.camera.x;
        let world_y = y as i32 + game_state.camera.y;
        match game_state.mode {
            systems::Mode::PAINT => {
                // if last drag pos not found then just lerp from the same point to itself
                let (from_x, from_y) = self.last_drag_pos.unwrap_or((world_x, world_y));
                for (pixi, pixj) in systems::lerp_points(from_x, from_y, world_x, world_y) {
                    systems::paint_at(&mut game_state.world, pixi, pixj, systems::BRUSH_RADIUS);
                }
                self.last_drag_pos = Some((world_x, world_y));
            }
            systems::Mode::ERASE => {
                let (from_x, from_y) = self.last_drag_pos.unwrap_or((world_x, world_y));
                for (pixi, pixj) in systems::lerp_points(from_x, from_y, world_x, world_y) {
                    systems::erase_at(&mut game_state.world, pixi, pixj, systems::BRUSH_RADIUS);
                }
                self.last_drag_pos = Some((world_x, world_y));
            }
            systems::Mode::DEPLOY => {
                let entity: Option<systems::Entity> =
                    graphics::troop_at(&game_state.world, world_x, world_y);
                let pos = systems::Position {
                    x: world_x,
                    y: world_y,
                };
                if entity.is_none() {
                    systems::spawn_troop(&mut game_state.world, pos, game_state.team_mode);
                }
            }
        }
    }
    /// Determines what happens if you interact with something
    /// at these x and y coordinates.
    fn handle_tap(&mut self, x: u32, y: u32) {
        let Some(game_state) = self.game_state.as_mut() else {
            return;
        };
        for button in graphics::buttons(game_state) {
            // if x and y of the interaction fall within the
            // bounds of the button
            if button.contains(x, y) {
                (button.on_click)(game_state);
                return; // early return, can't use functional tools
            }
        }

        if self.is_mobile {
            let size = self.graphics.as_ref().unwrap().pixels.texture().size();
            // cancel everything else out when moving dpad like the other buttons
            if graphics::dpad_hit(size.width, size.height, x, y).is_some() {
                return;
            }
        }

        let world_x = x as i32 + game_state.camera.x;
        let world_y = y as i32 + game_state.camera.y;
        let entity: Option<systems::Entity> =
            graphics::troop_at(&game_state.world, world_x, world_y);
        let pos = systems::Position {
            x: world_x,
            y: world_y,
        };
        match game_state.mode {
            systems::Mode::DEPLOY => {
                if entity.is_none() {
                    systems::spawn_troop(&mut game_state.world, pos, game_state.team_mode);
                }
            }
            systems::Mode::PAINT => {
                systems::paint_at(
                    &mut game_state.world,
                    world_x,
                    world_y,
                    systems::BRUSH_RADIUS,
                );
            }
            systems::Mode::ERASE => {
                systems::erase_at(
                    &mut game_state.world,
                    world_x,
                    world_y,
                    systems::BRUSH_RADIUS,
                );
            }
        }
        self.last_drag_pos = Some((world_x, world_y));
    }
}

impl ApplicationHandler<Graphics> for App {
    /// Tick loop
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(music) = self.music.as_mut() {
            music.update();
        }
        if let Some(game_state) = self.game_state.as_mut() {
            systems::pan_camera(
                &mut game_state.camera,
                self.pan_up,
                self.pan_down,
                self.pan_left,
                self.pan_right,
            );
            if !game_state.paused {
                systems::update_troops(&mut game_state.world);
            }
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
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        self.pan_up = event.state == ElementState::Pressed
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        self.pan_left = event.state == ElementState::Pressed
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        self.pan_down = event.state == ElementState::Pressed
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        self.pan_right = event.state == ElementState::Pressed
                    }
                    _ => {}
                }
                if let Some(game_state) = self.game_state.as_mut()
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyM) => game_state.change_teams(),
                        PhysicalKey::Code(KeyCode::KeyE) => game_state.toggle_erase(),
                        PhysicalKey::Code(KeyCode::KeyO) => game_state.toggle_paint(),
                        PhysicalKey::Code(KeyCode::KeyP) => game_state.toggle_pause(),
                        PhysicalKey::Code(KeyCode::Space) => game_state.toggle_pause(),
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
                if self.mouse_down {
                    self.handle_drag(position.x as u32, position.y as u32);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.mouse_down = state == ElementState::Pressed;
                    if state == ElementState::Pressed {
                        let (x, y) = (self.cursor_pos.0 as u32, self.cursor_pos.1 as u32);
                        self.handle_tap(x, y);
                    } else {
                        // clear the last drag pos so a new stroke doesn't
                        // have a line connecting to the last one
                        self.last_drag_pos = None;
                    }
                }
            }
            WindowEvent::Touch(touch) => {
                self.is_mobile = true;
                let (x, y) = (touch.location.x as u32, touch.location.y as u32);
                match touch.phase {
                    TouchPhase::Started => {
                        let size = self.graphics.as_ref().unwrap().pixels.texture().size();
                        match graphics::dpad_hit(size.width, size.height, x, y) {
                            Some(direction) => {
                                self.dpad_touch = Some(touch.id);
                                self.set_pan(direction);
                            }
                            None => self.handle_tap(x, y),
                        }
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        if self.dpad_touch == Some(touch.id) {
                            self.dpad_touch = None;
                            self.clear_pan();
                        }

                        self.last_drag_pos = None;
                    }
                    TouchPhase::Moved => {
                        if self.dpad_touch != Some(touch.id) {
                            self.handle_drag(x, y);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
