use arch::{VirtAddr, VirtPage, PAGE_SIZE};
use executor::current_task;
use frame_allocator::ceil_div;
use log::debug;

use crate::syscall::consts::from_vfs;
use crate::syscall::consts::MapFlags;
use crate::syscall::consts::ProtFlags;
use crate::syscall::consts::UserRef;

use super::consts::LinuxError;

pub async fn sys_brk(addr: isize) -> Result<usize, LinuxError> {
    debug!("sys_brk @ increment: {:#x}", addr);
    let user_task = current_task().as_user_task().unwrap();
    if addr == 0 {
        Ok(user_task.heap())
    } else {
        debug!("alloc pos: {}", addr - user_task.heap() as isize);
        Ok(user_task.sbrk(addr - user_task.heap() as isize))
    }
}

pub async fn sys_mmap(
    start: usize,
    len: usize,
    prot: usize,
    flags: usize,
    fd: usize,
    off: usize,
) -> Result<usize, LinuxError> {
    let flags = MapFlags::from_bits_truncate(flags as _);
    debug!(
        "sys_mmap @ start: {:#x}, len: {:#x}, prot: {}, flags: {:?}, fd: {}, offset: {}",
        start, len, prot, flags, fd as isize, off
    );
    let user_task = current_task().as_user_task().unwrap();
    let file = user_task.get_fd(fd);

    let addr = user_task.get_last_free_addr();

    let addr = if start == 0 {
        if usize::from(addr) >= 0x4000_0000 {
            addr
        } else {
            VirtAddr::from(0x4000_0000)
        }
    } else {
        VirtAddr::new(start)
    };

    debug!("sys_mmap @ free addr: {}", addr);

    if flags.contains(MapFlags::MAP_SHARED) {
        match file.clone() {
            Some(file) => user_task.map_frames(
                VirtPage::from_addr(addr.into()),
                executor::MemType::ShareFile,
                (len + PAGE_SIZE - 1) / PAGE_SIZE,
                Some(file),
                usize::from(addr),
                len,
            ),
            None => user_task.frame_alloc(
                VirtPage::from_addr(addr.into()),
                executor::MemType::Shared,
                (len + PAGE_SIZE - 1) / PAGE_SIZE,
            ),
        };
    } else {
        user_task.frame_alloc(
            VirtPage::from_addr(addr.into()),
            executor::MemType::Mmap,
            ceil_div(len, PAGE_SIZE),
        );
    }
    let mut buffer = UserRef::<u8>::from(addr).slice_mut_with_len(len);

    if let Some(file) = file {
        let offset = file.seek(fs::SeekFrom::CURRENT(0)).map_err(from_vfs)?;
        file.seek(fs::SeekFrom::SET(off)).map_err(from_vfs)?;
        let len = file.read(&mut buffer).map_err(from_vfs)?;
        file.seek(fs::SeekFrom::SET(offset)).map_err(from_vfs)?;
        debug!("read len: {:#x}", len);
    }
    Ok(addr.into())
}

pub async fn sys_munmap(start: usize, len: usize) -> Result<usize, LinuxError> {
    debug!("sys_munmap @ start: {:#x}, len: {}", start, len);
    let current_task = current_task().as_user_task().unwrap();

    current_task.inner_map(|x| {
        x.memset.iter_mut().for_each(|mem_area| {
            mem_area
                .mtrackers
                .drain_filter(|x| (start..start + len).contains(&x.vpn.to_addr()));
        })
    });
    Ok(0)
}

pub async fn sys_mprotect(start: usize, len: usize, prot: u32) -> Result<usize, LinuxError> {
    let prot = ProtFlags::from_bits_truncate(prot);
    debug!(
        "sys_mprotect @ start: {:#x}, len: {:#x}, prot: {:?}",
        start, len, prot
    );
    // Err(LinuxError::EPERM)
    Ok(0)
}
