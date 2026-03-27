use std::os::raw::c_int;

use crate::{
    WebViewEvent, cef_impl,
    shared::{with_gl, with_renderer_read},
    webview::SENDER,
};

fn get_scale() -> f32 {
    std::env::var("STREMIO_SCALE_FACTOR")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(1.0)
}

cef_impl!(
    prefix = "WebView",
    name = RenderHandler,
    sys_type = cef_dll_sys::cef_render_handler_t,
    {
        fn screen_info(
            &self,
            _browser: Option<&mut Browser>,
            screen_info: Option<&mut ScreenInfo>,
        ) -> ::std::os::raw::c_int {
            if let Some(screen_info) = screen_info {
                screen_info.device_scale_factor = get_scale();
                return true as _;
            }
            false as _
        }

        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            with_renderer_read(|renderer| {
                if let Some(rect) = rect {
                    *rect = Rect {
                        x: 0,
                        y: 0,
                        width: renderer.width,
                        height: renderer.height,
                    };
                }
            });
        }

        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: PaintElementType,
            _dirty_rects_count: usize,
            dirty_rects: Option<&Rect>,
            buffer: *const u8,
            width: c_int,
            height: c_int,
        ) {
            with_gl(|_, _| {
                with_renderer_read(|renderer| {
                    if renderer.width == width && renderer.height == height {
                        if let Some(dirty) = dirty_rects {
                            renderer.paint(
                                dirty.x,
                                dirty.y,
                                dirty.width,
                                dirty.height,
                                buffer,
                                width,
                            );
                        } else {
                            renderer.paint(0, 0, width, height, buffer, width);
                        }

                        if let Some(sender) = SENDER.get() {
                            sender.send(WebViewEvent::Paint).ok();
                            crate::shared::wake_event_loop();
                        }
                    } else if let Some(sender) = SENDER.get() {
                        sender.send(WebViewEvent::Resized).ok();
                    }
                });
            });
        }
    }
);
