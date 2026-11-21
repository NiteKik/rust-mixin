//! Windows process counters. Snapshots, not allocator attribution.
use serde::Serialize;
use std::ffi::c_void;

#[repr(C)]
struct Counters {
    cb: u32,
    page_fault_count: u32,
    // PROCESS_MEMORY_COUNTERS_EX, in Windows SDK field order.
    sizes: [usize; 9],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn K32GetProcessMemoryInfo(process: *mut c_void, counters: *mut Counters, cb: u32) -> i32;
}

#[derive(Serialize)]
pub struct Snapshot {
    pub working_set_bytes: usize,
    pub private_bytes: usize,
    pub peak_working_set_bytes: usize,
}

pub fn snapshot() -> Result<Snapshot, String> {
    let mut counters = Counters {
        cb: std::mem::size_of::<Counters>() as u32,
        page_fault_count: 0,
        sizes: [0; 9],
    };
    // SAFETY: properly sized SDK-compatible buffer and current-process handle.
    let success =
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    if success == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(Snapshot {
        working_set_bytes: counters.sizes[1],
        private_bytes: counters.sizes[8],
        peak_working_set_bytes: counters.sizes[0],
    })
}
