//! Process management syscalls
use crate::{
    mm::{PageTable, VirtAddr}, task::{current_user_token, exit_current_and_run_next, suspend_current_and_run_next}, timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;
    

    let time_val_vir_addr = VirtAddr::from(ts as usize);
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    
    // 拆分成单独字段处理
    let sec_vir_addr = time_val_vir_addr;
    let usec_vir_addr = VirtAddr::from((ts as usize) + core::mem::size_of::<usize>());
    
    // 获取sec字段的物理地址
    let sec_vpn = sec_vir_addr.floor();
    let sec_ppn = page_table.translate(sec_vpn).unwrap().ppn();
    let sec_offset = sec_vir_addr.page_offset();
    unsafe {
        *(sec_ppn.get_bytes_array().as_mut_ptr().add(sec_offset) as *mut usize) = sec;
    }
    // 获取usec字段的物理地址
    let usec_vpn = usec_vir_addr.floor();
    let usec_ppn = page_table.translate(usec_vpn).unwrap().ppn();
    let usec_offset = usec_vir_addr.page_offset();
    unsafe {
        *(usec_ppn.get_bytes_array().as_mut_ptr().add(usec_offset) as *mut usize) = usec;
    }
    
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            unsafe {
            let addr = _id as *const u8;
            let info = core::ptr::read_volatile(addr) as isize;
            info 
            }
        },
        1 => { 
            unsafe {
                let addr = _id as *mut u8;
                core::ptr::write_volatile(addr , _data as u8);
                0
            }
        },
        2 => {
                get_syscall_times(_id) as isize
             },
        _ => { -1 }
    }
}
