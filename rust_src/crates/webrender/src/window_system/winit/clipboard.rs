use crate::window_system::api::event_loop::EventLoop;

#[cfg(use_tao)]
pub type Clipboard = crate::window_system::api::clipboard::Clipboard;
#[cfg(use_winit)]
pub type Clipboard = Box<dyn copypasta::ClipboardProvider>;

pub trait ClipboardExt {
    fn build(event_loop: &EventLoop<i32>) -> Self;
    fn write(&mut self, content: String);
    fn read(&mut self) -> String;
}

#[cfg(use_tao)]
impl ClipboardExt for Clipboard {
    fn build(_: &EventLoop<i32>) -> Self {
        crate::window_system::api::clipboard::Clipboard::new()
    }
    fn write(&mut self, content: String) {
        self.write_text(content);
    }

    fn read(&mut self) -> String {
        match &self.read_text() {
            Some(s) => s.to_string(),
            None => String::from(""),
        }
    }
}

#[cfg(use_winit)]
impl ClipboardExt for Clipboard {
    fn build(_event_loop: &EventLoop<i32>) -> Self {
        #[cfg(free_unix)]
        {
            use crate::window_system::api::platform::wayland::EventLoopWindowTargetExtWayland;
            if _event_loop.is_wayland() {
                let wayland_display = _event_loop
                    .wayland_display()
                    .expect("Fetch Wayland display failed");
                let (_, clipboard) = unsafe {
                    copypasta::wayland_clipboard::create_clipboards_from_external(wayland_display)
                };
                Box::new(clipboard)
            } else {
                #[cfg(x11_platform)]
                {
                    return Box::new(
                        copypasta::x11_clipboard::X11ClipboardContext::<
                            copypasta::x11_clipboard::Clipboard,
                        >::new()
                        .unwrap(),
                    );
                }
                #[cfg(not(x11_platform))]
                panic!("No clipboard impl avaiable")
            }
        }
        #[cfg(windows_platform)]
        {
            use copypasta::windows_clipboard::WindowsClipboardContext;
            return Box::new(WindowsClipboardContext::new().unwrap());
        }
        #[cfg(macos_platform)]
        {
            use copypasta::osx_clipboard::OSXClipboardContext;
            return Box::new(OSXClipboardContext::new().unwrap());
        }
    }

    fn write(&mut self, content: String) {
        if let Err(err) = self.set_contents(content) {
            log::error!("Failed to write to clipboard {err:?}")
        };
    }

    fn read(&mut self) -> String {
        match &self.get_contents() {
            Ok(s) => s.to_string(),
            Err(_) => String::from(""),
        }
    }
}
