//! Security keys coming and going while the sign-in page is open
//! (docs/ACCOUNTS.md › Passkeys › Where the ceremony runs › Linux): the
//! kernel's uevents for `hidraw` devices, read from a netlink socket on a
//! thread of its own. Enumerating when the form opens stays the fallback;
//! this only says when to enumerate again.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A running watch. Dropping it stops the thread within a quarter second.
pub struct Watch {
    stop: Arc<AtomicBool>,
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

/// Whether a kernel uevent (`action@devpath` and NUL-separated `KEY=VALUE`
/// lines) is a `hidraw` device being added or removed: a security key, or
/// another HID device, which enumeration then sorts out.
pub fn is_key_event(message: &[u8]) -> bool {
    let mut subsystem = false;
    let mut action = false;
    for field in message.split(|byte| *byte == 0) {
        match field {
            b"SUBSYSTEM=hidraw" => subsystem = true,
            b"ACTION=add" | b"ACTION=remove" => action = true,
            _ => {}
        }
    }
    subsystem && action
}

/// Calls `changed` after each `hidraw` add or remove, until the returned
/// [`Watch`] is dropped. `None` where there are no uevents to read (not
/// Linux, or the socket was refused); the page then relies on enumerating
/// when the form opens.
pub fn watch(changed: impl Fn() + Send + 'static) -> Option<Watch> {
    let stop = Arc::new(AtomicBool::new(false));
    platform::spawn(stop.clone(), Box::new(changed))?;
    Some(Watch { stop })
}

#[cfg(target_os = "linux")]
mod platform {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    /// The kernel's own multicast group (udevd's is 2, with its own header).
    const KERNEL_GROUP: u32 = 1;
    /// udev sets the device node's permissions just after the kernel's
    /// event; enumeration waits for it.
    const SETTLE: Duration = Duration::from_millis(400);

    struct Socket(libc::c_int);

    impl Drop for Socket {
        fn drop(&mut self) {
            // SAFETY: the descriptor is ours and closed once.
            unsafe { libc::close(self.0) };
        }
    }

    fn open() -> Option<Socket> {
        // SAFETY: plain socket calls on a descriptor this function owns.
        unsafe {
            let fd = libc::socket(
                libc::AF_NETLINK,
                libc::SOCK_DGRAM | libc::SOCK_CLOEXEC,
                libc::NETLINK_KOBJECT_UEVENT,
            );
            if fd < 0 {
                return None;
            }
            let socket = Socket(fd);
            let mut address: libc::sockaddr_nl = std::mem::zeroed();
            address.nl_family = libc::AF_NETLINK as libc::sa_family_t;
            address.nl_groups = KERNEL_GROUP;
            let bound = libc::bind(
                socket.0,
                (&address as *const libc::sockaddr_nl).cast::<libc::sockaddr>(),
                std::mem::size_of::<libc::sockaddr_nl>() as libc::socklen_t,
            );
            (bound == 0).then_some(socket)
        }
    }

    pub(super) fn spawn(stop: Arc<AtomicBool>, changed: Box<dyn Fn() + Send>) -> Option<()> {
        let socket = open()?;
        std::thread::Builder::new()
            .name("kanso-security-keys".into())
            .spawn(move || {
                let mut buffer = vec![0u8; 8192];
                while !stop.load(Ordering::Acquire) {
                    let mut poll = libc::pollfd {
                        fd: socket.0,
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    // SAFETY: one pollfd, valid for the call.
                    let ready = unsafe { libc::poll(&mut poll, 1, 250) };
                    if ready <= 0 {
                        continue;
                    }
                    // SAFETY: the buffer is valid for its length.
                    let read = unsafe {
                        libc::recv(socket.0, buffer.as_mut_ptr().cast(), buffer.len(), 0)
                    };
                    if read <= 0 {
                        continue;
                    }
                    if super::is_key_event(&buffer[..read as usize]) {
                        std::thread::sleep(SETTLE);
                        if !stop.load(Ordering::Acquire) {
                            changed();
                        }
                    }
                }
            })
            .ok()?;
        Some(())
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    pub(super) fn spawn(_stop: Arc<AtomicBool>, _changed: Box<dyn Fn() + Send>) -> Option<()> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_hidraw_adds_and_removes_count() {
        let event = |action: &str, subsystem: &str| {
            format!(
                "{action}@/devices/pci0000:00/usb1/1-1/1-1:1.0/0003:1050:0407.0001/hidraw/hidraw3\0ACTION={action}\0DEVPATH=/devices/x\0SUBSYSTEM={subsystem}\0DEVNAME=hidraw3\0SEQNUM=4242\0"
            )
            .into_bytes()
        };
        assert!(is_key_event(&event("add", "hidraw")));
        assert!(is_key_event(&event("remove", "hidraw")));
        assert!(!is_key_event(&event("change", "hidraw")));
        assert!(!is_key_event(&event("add", "usb")));
        assert!(!is_key_event(b""));
        // A value that only starts like the field doesn't count.
        assert!(!is_key_event(b"ACTION=add\0SUBSYSTEM=hidraw2\0"));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn there_is_nothing_to_watch_off_linux() {
        assert!(watch(|| {}).is_none());
    }
}
