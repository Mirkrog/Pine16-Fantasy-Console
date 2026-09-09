use pixels::{Pixels, SurfaceTexture};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::process;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
use std::{sync::Arc, thread::sleep};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(windows)]
use winit::platform::windows::{Color, WindowAttributesExtWindows};
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use futures::channel::oneshot::{Receiver, Sender, channel};
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use crate::console::Console;

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<Pixels<'static>>>,
    #[cfg(target_arch = "wasm32")]
    receiver: Option<Receiver<Vec<u8>>>,
    window: Option<Arc<Window>>,
    console: Console,
    #[cfg(not(target_arch = "wasm32"))]
    cart_path: PathBuf,
    #[cfg(target_arch = "wasm32")]
    last_step: Instant,
}

impl App {
    pub fn new(
        #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<Pixels<'static>>,
        #[cfg(not(target_arch = "wasm32"))] cart_path: PathBuf,
        console_guardrails: bool,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy,
            #[cfg(target_arch = "wasm32")]
            receiver: None,
            window: None,
            console: Console::new(console_guardrails),
            #[cfg(not(target_arch = "wasm32"))]
            cart_path,
            #[cfg(target_arch = "wasm32")]
            last_step: Instant::now(),
        }
    }
}

impl ApplicationHandler<Pixels<'static>> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(not(target_arch = "wasm32"))]
        let window_attributes = window_attributes.with_title("Pine16VirtualConsole");

        #[cfg(target_os = "windows")]
        let window_attributes =
            window_attributes.with_title_background_color(Some(Color::from_rgb(128, 128, 128)));

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::UnwrapThrowExt;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            let surface_texture = SurfaceTexture::new(
                window.inner_size().width,
                window.inner_size().height,
                window.clone(),
            );

            self.console.resume_renderer(
                Pixels::new(
                    crate::renderer::CANVAS_WIDTH as u32,
                    crate::renderer::CAVAS_HEIGHT as u32,
                    surface_texture,
                )
                .unwrap(),
            );
        }
        #[cfg(target_arch = "wasm32")]
        {
            let surface_texture = SurfaceTexture::new(40, 40, window.clone());

            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    use pixels::PixelsBuilder;

                    assert!(
                        proxy
                            .send_event(
                                PixelsBuilder::new(
                                    crate::renderer::CANVAS_WIDTH as u32,
                                    crate::renderer::CAVAS_HEIGHT as u32,
                                    surface_texture,
                                )
                                .wgpu_backend(wgpu::Backends::GL)
                                .build_async()
                                .await
                                .unwrap()
                            )
                            .is_ok()
                    )
                });
            }
        }

        self.window = Some(window);
    }
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Pixels<'static>) {
        let _ = event;
        #[cfg(target_arch = "wasm32")]
        {
            self.console.resume_renderer(event);
            self.console.resize_renderer(
                self.window
                    .as_ref()
                    .expect("Window disappeared")
                    .inner_size(),
            );
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::debug!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.console.render();
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let _ = device_id;
                let _ = is_synthetic;
                if event.state.is_pressed() {
                    self.console.key_pressed(event.logical_key);
                } else {
                    self.console.key_released(event.logical_key);
                }
            }
            WindowEvent::Resized(size) => {
                self.console.resize_renderer(size);
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;

        #[cfg(not(target_arch = "wasm32"))]
        {
            if !self.console.is_rom_loaded() {
                use std::{fs::File, io::Read};
                let mut bytes = Vec::new();
                File::open(&self.cart_path)
                    .unwrap_or_else(|e| {
                        log::error!(
                            "Encountered error while opening {:?}: {e}",
                            &self.cart_path.file_name()
                        );
                        process::exit(1)
                    })
                    .read_to_end(&mut bytes)
                    .unwrap_or_else(|e| {
                        log::error!(
                            "Encountered error while reading {:?}: {e}",
                            &self.cart_path.file_name()
                        );
                        process::exit(1)
                    });
                self.console
                    .load_rom_from_bytes(bytes)
                    .unwrap_or_else(|_| process::exit(1));
            }

            let now = Instant::now();
            self.console.step();

            if let Some(window) = &self.window {
                window.request_redraw();
            }

            sleep(Duration::from_millis(
                ((1.0 / 30.0) * 1000.0) as u64 - now.elapsed().subsec_millis() as u64,
            ));
        }
        #[cfg(target_arch = "wasm32")]
        {
            if !self.console.is_rom_loaded() {
                if let Some(rx) = &mut self.receiver {
                    match rx.try_recv() {
                        Ok(Some(bytes)) => {
                            println!("Bytes arrived! Size: {}", bytes.len());

                            self.console.load_rom_from_bytes(bytes).unwrap()
                        }
                        Ok(None) | Err(_) => {
                            return;
                        }
                    }
                } else {
                    let (tx, rx) = channel::<Vec<u8>>();
                    self.receiver = Some(rx);

                    wasm_bindgen_futures::spawn_local(async move {
                        let bytes = get_bytes_from_url("./AUTOLOAD.pinecart".to_string()).await;
                        match bytes {
                            Some(bytes) => {
                                tx.send(bytes).unwrap();
                            }
                            None => drop(tx),
                        }
                    });
                    return;
                }
            }

            if self.last_step.elapsed().subsec_millis() as u64 > ((1.0 / 30.0) * 1000.0) as u64 {
                self.console.step();

                self.last_step = Instant::now();
            }

            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn get_bytes_from_url(url: String) -> Option<Vec<u8>> {
    use gloo_net::http::Request;

    let response = Request::get(&url).send().await.unwrap();

    if !response.ok() {
        return None;
    }

    Some(response.binary().await.unwrap())
}

pub fn run(
    #[cfg(not(target_arch = "wasm32"))] cart_path: PathBuf,
    console_guardrails: bool,
) -> anyhow::Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen::UnwrapThrowExt::unwrap_throw(console_log::init_with_level(log::Level::Info));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new(
            #[cfg(not(target_arch = "wasm32"))]
            cart_path,
            console_guardrails,
        );
        event_loop.run_app(&mut app)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let app = App::new(&event_loop, console_guardrails);
        event_loop.spawn_app(app);
    }

    Ok(())
}
