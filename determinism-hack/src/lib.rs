// ssize_t getrandom(void buf[.buflen], size_t buflen, unsigned int flags);

use libc::{c_uint, c_void, memset, size_t, ssize_t};

/// # Safety
/// Totally unsafe.
#[allow(unsafe_code)]
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut u8, buflen: size_t, _flags: c_uint) -> ssize_t {
    #[allow(clippy::cast_possible_truncation)]
    unsafe {
        for i in 0..buflen {
            *(buf.add(i)) = (buf as usize + 33 + 1) as u8;
        }
    };
    buflen.try_into().unwrap()
}
