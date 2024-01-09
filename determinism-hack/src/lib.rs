// ssize_t getrandom(void buf[.buflen], size_t buflen, unsigned int flags);

use libc::{c_uint, c_void, memset, size_t, ssize_t};

/// # Safety
/// Totally unsafe.
#[allow(unsafe_code)]
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: size_t, _flags: c_uint) -> ssize_t {
    unsafe { memset(buf, 0, buflen) };
    buflen.try_into().unwrap()
}
