extern crate libc;

use libc::{c_int, fd_set, select, timeval, FD_ZERO};

pub struct Select {
    fds: Vec<c_int>,
    read_fds: fd_set,
    index: usize,
}

impl Select {
    pub fn new(fds: Vec<c_int>) -> Result<Self, std::io::Error> {
        let mut read_fds: fd_set = unsafe { std::mem::zeroed() };
        unsafe { FD_ZERO(&mut read_fds) };

        for &fd in fds.iter() {
            unsafe { libc::FD_SET(fd, &mut read_fds) };
        }

        let max_fd = *fds.iter().max().unwrap_or(&-1);
        let mut timeout = timeval {
            tv_sec: 1,
            tv_usec: 0,
        };

        let select_result = unsafe {
            select(
                max_fd + 1,
                &mut read_fds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeout,
            )
        };

        if select_result == -1 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self {
            fds,
            read_fds,
            index: 0,
        })
    }
}

impl Iterator for Select {
    type Item = c_int;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.fds.len() {
            let fd = self.fds[self.index];
            self.index += 1;

            if unsafe { libc::FD_ISSET(fd, &self.read_fds) } {
                return Some(fd);
            }
        }

        None
    }
}
