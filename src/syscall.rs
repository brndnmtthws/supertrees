use std::io;

pub fn syscall(r: libc::c_int) -> io::Result<libc::c_int> {
    match r {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(r),
    }
}
