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

use crate::display_info::DisplayInfoRef;
use crate::font::FontRef;
use crate::lisp::ExternalPtr;

pub type OutputRef = ExternalPtr<Output>;

pub trait OutputExtWindowSystem {
    fn display_info(&self) -> DisplayInfoRef;
    fn set_font(&mut self, font: FontRef);
    fn set_fontset(&mut self, fontset: i32);
}

impl OutputExtWindowSystem for OutputRef {
    fn set_font(&mut self, mut font: FontRef) {
        self.font = font.as_mut();
    }

    fn set_fontset(&mut self, fontset: i32) {
        self.fontset = fontset;
    }
    fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.display_info as *mut _)
    }
}
