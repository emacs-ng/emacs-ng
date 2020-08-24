macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(::std::ptr::null() as *const $ty)).$field as *const _ as usize
    };
}

/// Equivalent to PSEUDOVECSIZE in C
macro_rules! pseudovecsize {
    ($ty: ty, $field: ident) => {
        ((offset_of!($ty, $field) - *crate::vectors::HEADER_SIZE) / *crate::vectors::WORD_SIZE)
    };
}

/// Equivalent to VECSIZE in C
macro_rules! vecsize {
    ($ty: ty) => {
        ((::std::mem::size_of::<$ty>() - *crate::vectors::HEADER_SIZE + *crate::vectors::WORD_SIZE
            - 1)
            / *crate::vectors::WORD_SIZE)
    };
}
