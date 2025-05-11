//! Process management syscalls
use crate::{
    config::PAGE_SIZE, mm::{free_frames_cnt, MapPermission, PageTable, VPNRange, VirtAddr,}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_times,
        suspend_current_and_run_next, TASK_MANAGER,
    }, timer::get_time_us
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
    let viradd = VirtAddr::from(_id);
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let vpn = viradd.floor();
    let offset = viradd.page_offset();
    match _trace_request {
        0 => {
            if let Some(pte) = page_table.translate(vpn) {
                if !pte.readable() || !pte.is_user() {
                    error!("sys_trace: read from non-readable or non-user page");
                    return -1;
                }
                let ppn = pte.ppn();
                unsafe {
                    let value = *(ppn.get_bytes_array().as_ptr().add(offset)) as isize;
                    value
                }
            } else {
                error!("sys_trace: read from non-mapped page");
                -1
            }
        }
        1 => {
            if let Some(pte) = page_table.translate(vpn) {
                if !pte.writable() || !pte.is_user() {
                    error!("sys_trace: write to non-writable or non-user page");
                    return -1;
                }
                let ppn = pte.ppn();
                unsafe {
                    *(ppn.get_bytes_array().as_mut_ptr().add(offset)) = _data as u8;
                    0
                }
            } else {
                error!("sys_trace: write to non-mapped page");
                -1
            }
        }
        2 => get_syscall_times(_id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if ! VirtAddr::from(_start).aligned(){
        return -1;
    }
    
    if _port & !0x7 != 0 || _port & 0x7 == 0 {
        error!("sys_mmap: invalid port");
        return -1;
    }

    if  free_frames_cnt() < (_len + PAGE_SIZE -1) / PAGE_SIZE {
        error!("sys_mmap: not enough free frames");
        return -1;  
    }

    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let cur = inner.current_task;
    let mem_set = &mut inner.tasks[cur].memory_set;
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    let range = VPNRange::new(start_vpn, end_vpn);
    
    for vpn in range {
        if let Some(pte) = mem_set.translate(vpn) {
            if pte.is_valid() {
                error!("sys_mmap: address already mapped");
                return -1;
            }
        }
    }

    let mut permission = MapPermission::U;
    if _port & 0x1 != 0 {
        permission |= MapPermission::R;
    }
    if _port & 0x2 != 0 {
        permission |= MapPermission::W; 
    }
    if _port & 0x4 != 0 {
        permission |= MapPermission::X; 
    }

    mem_set.insert_framed_area(
        start_vpn.into(),
        end_vpn.into(),
        permission,
    );
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if ! VirtAddr::from(_start).aligned(){
        return -1;
    }

    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let cur = inner.current_task;
    let mem_set = &mut inner.tasks[cur].memory_set;
    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();
    let range = VPNRange::new(start_vpn, end_vpn);

    for vpn in range {
        if let Some(pte) = mem_set.translate(vpn) {
            if !pte.is_valid() {
                error!("sys_munmap: address not mapped");
                return -1;
            }
        }
    }
    mem_set.delete_framed_area(_start.into(), (_start +_len).into());
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
