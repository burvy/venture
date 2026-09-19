use std::sync::Arc;

use pixels::{Pixels, PixelsBuilder, SurfaceTexture, wgpu::Backends};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{Fullscreen, Window, WindowId},
};

pub struct Graphics {
    pub window: Arc<Window>,
    pub pixels: Pixels<'static>,
}

#[derive(Default)]
pub struct App {
    pub graphics: Option<Graphics>,
    pub proxy: Option<EventLoopProxy<Graphics>>,
    pub canvas_parent: Option<String>,
}

impl ApplicationHandler<Graphics> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }
        let window_attributes = Window::default_attributes()
            .with_title("venture")
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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
        let Some(graphics) = self.graphics.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
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
                for pixel in graphics.pixels.frame_mut().chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[16, 212, 48, 255]);
                }
                if let Err(err) = graphics.pixels.render() {
                    eprintln!("render failed: {err}");
                    event_loop.exit();
                }
            }
            _ => (),
        }
    }
}
