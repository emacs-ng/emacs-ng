use std::ffi::CStr;
use std::ffi::CString;

use core_foundation::bundle::CFBundle;

/// This function acts.
#[no_mangle]
#[allow(unused_doc_comments)]
pub extern "C" fn app_bundle_relocate(epath: *const ::libc::c_char) -> *const ::libc::c_char {
    let main_bundle = CFBundle::main_bundle();
    match main_bundle.path() {
        Some(path) => {
            let relocated_path: &CStr = unsafe { CStr::from_ptr(epath) };
            let relocated_path: &str = relocated_path.to_str().unwrap();

            if !std::path::Path::new(relocated_path).is_absolute() {
                let relocated_path = path.join(relocated_path).as_path().display().to_string();
                let relocated_path = CString::new(relocated_path).unwrap();
                relocated_path.into_raw()
            } else {
                epath
            }
        }
        None => epath,
    }
}
