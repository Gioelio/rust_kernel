#[repr(C)]
struct MultiBootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_count: u32,
    mods_addr: u32,
    syms: [u32; 4],
    mmap_length: u32,
    mmap_addr: u32,
    drives_length: u32,
    drives_addr: u32,
    config_table: u32,
    boot_loader_name: u32,
    apm_table: u32,
    vbe_control_info: u32,
    vbe_mode_info: u32,
    vbe_mode: u32,
    vbe_interface_seg: u32,
    vbe_interface_off: u32,
    vbe_interface_len: u32,
}

/// Return memory available if MultiBoot provide such information
///
/// None if MultiBoot doesnt provide valid memory info
pub fn physical_mem_available_kib(mb_info_addr: u64) -> Option<u64> {
    let mb_info = unsafe { &*(mb_info_addr as *const MultiBootInfo) };

    // Bit 0 flags indicates if mem_lower and mem_upper are valid
    if mb_info.flags & (1 << 0) != 0 {
        let total_kib = mb_info.mem_lower as u64 + mb_info.mem_upper as u64;
        return Some(total_kib);
    }

    None
}
