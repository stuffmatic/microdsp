#[macro_use]
extern crate lazy_static;

pub mod mpm;
pub mod snov;

#[no_mangle]
pub extern "C" fn allocate_f32_array(size: usize) -> *mut f32 {
    let mut buf = Vec::<f32>::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as *mut f32
}
