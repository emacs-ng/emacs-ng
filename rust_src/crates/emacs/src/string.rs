#[macro_export]
macro_rules! new_unibyte_string {
    ($str:expr) => {{
        let strg = ::std::ffi::CString::new($str).unwrap();
        unsafe {
            $crate::bindings::make_unibyte_string(strg.as_ptr(), strg.as_bytes().len() as isize)
        }
    }};
}
