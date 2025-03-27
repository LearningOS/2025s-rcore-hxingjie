//! Process management syscalls
//! 
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};

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
    //trace!("kernel: sys_get_time");
    //-1
    
    // my code
    let us = crate::timer::get_time_us();
    let timeval = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let mut ptr = &timeval as *const TimeVal as usize;

    let mut buffers = crate::mm::translated_byte_buffer(crate::task::current_user_token(), ts as *const u8, core::mem::size_of::<TimeVal>());
    for buffer in buffers.iter_mut() {
        let data = unsafe {core::slice::from_raw_parts(ptr as *const u8, buffer.len()) };
        ptr += buffer.len();

        buffer.copy_from_slice(data);
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/*
    如果 trace_request 为 0，则 id 应被视作 *const u8 ，
        表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。
    如果 trace_request 为 1，则 id 应被视作 *const u8 ，
        表示写入 data （作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为0。
    如果 trace_request 为 2，
        表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计。
    否则，忽略其他参数，返回值为 -1。
*/
use crate::mm::PTEFlags;
use crate::mm::{VirtAddr, VirtPageNum};
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    //trace!("kernel: sys_trace");
    //-1
    match trace_request {
        0 => {
            let r_addr = id as *const u8 ;
            let page_table = crate::mm::PageTable::from_token(crate::task::current_user_token());
            
            let vpn: VirtPageNum = VirtAddr::from(r_addr as usize).floor();

            if let Some(pte) = page_table.translate(vpn) {
                if (pte.flags() & PTEFlags::U) != PTEFlags::empty() && pte.readable() {
                    let buffers = 
                        crate::mm::translated_byte_buffer(
                        crate::task::current_user_token(), 
                        r_addr, 
                        core::mem::size_of::<u8>());

                    buffers[0][0] as isize
                } else {
                    -1 // this page is no readable
                }
            } else {
                -1 // pte no valid
            }
        },
        1 => {
            let w_addr = id as *const u8 ;
            let page_table = crate::mm::PageTable::from_token(crate::task::current_user_token());
            
            let vpn: VirtPageNum = VirtAddr::from(w_addr as usize).floor();

            if let Some(pte) = page_table.translate(vpn) {
                if (pte.flags() & PTEFlags::U) != PTEFlags::empty() && pte.writable() {
                    let mut buffers = 
                        crate::mm::translated_byte_buffer(
                        crate::task::current_user_token(), 
                        w_addr, 
                        core::mem::size_of::<u8>());

                    buffers[0][0] = data as u8;
                    
                    0
                } else {
                    -1 // this page is no writable
                }
            } else {
                -1 // pte no valid
            }
        },
        2 => {
            crate::task::get_sysinfo(id) as isize
        },
        _ => {-1}
    }
}

// YOUR JOB: Implement mmap.

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    // trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    // -1

    crate::task::mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    //trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    //-1

    crate::task::munmap(start, len)
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
