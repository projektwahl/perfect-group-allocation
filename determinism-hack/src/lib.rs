use core::slice;

use libc::{c_uint, size_t, ssize_t};
use rand::{RngCore, SeedableRng};

/// # Safety
/// Totally unsafe.
#[allow(unsafe_code)]
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut u8, buflen: size_t, _flags: c_uint) -> ssize_t {
    let slice = unsafe { slice::from_raw_parts_mut(buf, buflen) };
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    rng.fill_bytes(slice);
    buflen.try_into().unwrap()
}
