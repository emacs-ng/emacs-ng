#[macro_export]
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {{
        use std::ptr;
        &(*(ptr::null() as *const $ty)).$field as *const _ as usize
    }};
}

/// Equivalent to VECSIZE in C
#[macro_export]
macro_rules! vecsize {
    ($ty: ty) => {
        ((::std::mem::size_of::<$ty>() - *$crate::vector::HEADER_SIZE + *$crate::vector::WORD_SIZE
            - 1)
            / *$crate::vector::WORD_SIZE)
    };
}
