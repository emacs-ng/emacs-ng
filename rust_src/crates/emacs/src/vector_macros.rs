/// Equivalent to VECSIZE in C
#[macro_export]
macro_rules! vecsize {
    ($ty: ty) => {
        ((::std::mem::size_of::<$ty>() - *$crate::vector::HEADER_SIZE + *$crate::vector::WORD_SIZE
            - 1)
            / *$crate::vector::WORD_SIZE)
    };
}
