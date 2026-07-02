use std::alloc::LayoutError;

pub unsafe fn alloc_standard(bytes: usize, align: usize) -> Result<*mut u8, LayoutError> {
    let layout = std::alloc::Layout::from_size_align(bytes, align)?;
    unsafe { Ok(std::alloc::alloc(layout)) }
}

pub unsafe fn dealloc_standard<T>(ptr: *mut T, bytes: usize) -> Result<(), LayoutError> {
    let layout = std::alloc::Layout::from_size_align(bytes, align_of::<T>())?;
    unsafe { std::alloc::dealloc(ptr as *mut u8, layout) };
    Ok(())
}
