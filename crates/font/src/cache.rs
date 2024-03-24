use emacs_sys::lisp::ExternalPtr;
use emacs_sys::terminal::TerminalRef;
use font_index::FontCache;

pub type FontCacheRef = ExternalPtr<FontCache>;

pub trait TerminalExtFontIndex {
    fn font_cache(&mut self) -> FontCacheRef;
}

impl TerminalExtFontIndex for TerminalRef {
    fn font_cache(&mut self) -> FontCacheRef {
        if self.font_index_cache.is_null() {
            let font_cache = Box::new(FontCache::default());
            self.font_index_cache = Box::into_raw(font_cache) as *mut libc::c_void;
        }
        FontCacheRef::new(self.font_index_cache as *mut FontCache)
    }
}
