#[cfg(feature = "window-system-android")]
pub use crate::bindings::android_output as Output;
#[cfg(have_ns)]
pub use crate::bindings::ns_output as Output;
#[cfg(have_pgtk)]
pub use crate::bindings::pgtk_output as Output;
#[cfg(feature = "window-system-w32")]
pub use crate::bindings::w32_output as Output;
#[cfg(have_winit)]
pub use crate::bindings::winit_output as Output;
#[cfg(have_x11)]
pub use crate::bindings::x_output as Output;

#[cfg(have_window_system)]
use crate::display_info::DisplayInfoRef;
#[cfg(have_window_system)]
use crate::font::FontRef;
use crate::lisp::ExternalPtr;

pub type OutputRef = ExternalPtr<Output>;

impl OutputRef {
    #[cfg(have_window_system)]
    pub fn font(&self) -> FontRef {
        FontRef::new(self.font as *mut _)
    }
    #[cfg(have_window_system)]
    pub fn set_font(&mut self, mut font: FontRef) {
        self.font = font.as_mut();
    }

    #[cfg(have_window_system)]
    pub fn set_fontset(&mut self, fontset: i32) {
        self.fontset = fontset;
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.display_info as *mut _)
    }
}
