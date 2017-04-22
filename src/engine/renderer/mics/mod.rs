// Simple offset_of macro akin to C++ offsetof
#[macro_export]
macro_rules! offset_of{
    ($base: path, $field: ident) => {
        {
            #[allow(unused_unsafe)]
            unsafe{
                let b: $base = mem::uninitialized();
                (&b.$field as *const _ as isize) - (&b as *const _ as isize)
            }
        }
    }
}