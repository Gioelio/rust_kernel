use bitfield_struct::bitfield;

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

#[bitfield(u64)]
pub struct PageTableEntry {
    /// The page is currently in memory
    present: bool,
    /// It's allowed to write to this page
    writable: bool,
    /// If not set, only kernel mode code can access this page
    user_accessible: bool,
    /// Writes go directly to memory
    write_through_caching: bool,
    /// No cache is used for this page
    disable_cache: bool,
    /// The CPU sets this bit when this page is used
    accessed: bool,
    /// The CPU sets this bit when a write to this page occurs
    dirty: bool,
    /// Must be 0 in P1 and P4, creates a 1 GiB page in P3, creates a 2MiB page in P2
    huge_page_null: bool,
    /// Page isn't flushed from caches on address space switch (PGE bit of CR4 register must be set)
    global: bool,
    /// can be used freely by the OS
    #[bits(3)]
    available: u8,
    /// The page aligned 52 bit physical address of the frame or the next page table
    #[bits(40)]
    physical_address: u64,
    /// can be used freely by the OS
    #[bits(11)]
    available1: u64,
    /// Forbid excetuting code on this page (the NXE bit in the EFER register must be set)
    no_execute: bool,
}
