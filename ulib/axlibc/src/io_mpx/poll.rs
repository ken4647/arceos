use crate::{
    ctypes,
    fd_ops::get_file_like,
};
use axerrno::{LinuxError, LinuxResult};
use axhal::time::current_time;

use core::{ffi::c_int, time::Duration};

fn poll_all(fds: &mut [ctypes::pollfd]) -> LinuxResult<usize> {
    let mut events_num = 0;

    for pollfd_item in fds.iter_mut() {
        let intfd = pollfd_item.fd;
        let events = pollfd_item.events;
        let revents = &mut pollfd_item.revents;
        match get_file_like(intfd as c_int)?.poll() {
            Err(_) => {
                if (events & ctypes::EPOLLERR as i16) != 0 {
                    *revents |= ctypes::EPOLLERR as i16;
                }
            }
            Ok(state) => {
                if state.readable && (events & ctypes::EPOLLIN as i16 != 0) {
                    *revents |= ctypes::EPOLLIN as i16;
                }

                if state.writable && (events & ctypes::EPOLLOUT as i16 != 0) {
                    *revents |= ctypes::EPOLLOUT as i16;
                }                
            }
        }
        events_num += 1;
    }
    Ok(events_num)    
}

// used to monitor multiple file descriptors for events
#[no_mangle]
pub unsafe extern "C" fn ax_poll(
    fds: *mut ctypes::pollfd,
    nfds: ctypes::nfds_t,
    timeout: c_int
) -> c_int{
    debug!("ax_poll <= nfds: {} timeout: {}", nfds, timeout);
    ax_call_body!(ax_poll, {
        if nfds <= 0 {
            return Err(LinuxError::EINVAL);
        }
        let fds = core::slice::from_raw_parts_mut(fds, nfds as usize);
        let deadline = (!timeout.is_negative())
            .then(|| current_time() + Duration::from_millis(timeout as u64));
        loop {
            #[cfg(feature = "net")]
            axnet::poll_interfaces();
            let fds_num = poll_all(fds)?;
            if fds_num > 0 {
                return Ok(fds_num as c_int);
            }

            if deadline.map_or(false, |ddl| current_time() >= ddl) {
                debug!("    timeout!");
                return Ok(0);
            }
            axstd::thread::yield_now();
        }
    })
}
