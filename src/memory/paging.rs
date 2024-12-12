//! Abstractions for page tables and page table entries.

#![cfg(target_pointer_width = "64")]
use core::fmt;
use core::ops::{Index, IndexMut};
use crate::memory::mapper::OffsetPageTable;

use bitflags::bitflags;

const PAGE_4KB_SIZE: u64 = 0x1000;
const PAGE_2MB_SIZE: u64 = 0x200000;
const PAGE_1GB_SIZE: u64 = 0x40000000;
const ADDRESS_SPACE_SIZE: u64 = 0x1_0000_0000_0000;

fn read_cr3() -> u64 {
    use core::arch::asm;
    unsafe {
        let mut frame: u64;
        asm!("mov {}, cr3", out(reg) frame, options(nomem, nostack, preserves_flags));
        frame &= 0x_000f_ffff_ffff_f000;
        frame.align_down(PAGE_4KB_SIZE)
    }
}

pub unsafe fn init(physical_memory_offset: u64) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub unsafe fn active_level_4_table(phys_mem_offset: u64) -> &'static mut PageTable {
    let phys = read_cr3();
    let virt = phys + phys_mem_offset;
    let page_table_ptr: *mut PageTable = virt as *mut PageTable;

    &mut *page_table_ptr
}

pub unsafe fn translate_addr(addr: u64, phys_mem_offset: u64) -> Option<u64> {
    inner_translate_addr(addr, phys_mem_offset)
}

pub unsafe fn inner_translate_addr(addr: u64, phys_mem_offset: u64) -> Option<u64> {
    let mut frame = read_cr3();
    let table_indexes = [
        addr.p1_index(), addr.p2_index(), addr.p3_index(), addr.p4_index()
    ];
    for &index in &table_indexes {
        // convert the frame into a page table reference
        let virt = phys_mem_offset + frame;
        let table_ptr: *const PageTable = virt as *const PageTable;
        let table = unsafe {&*table_ptr};

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame + u64::from(addr.page_offset()))
}

pub trait VirtAddr {
    fn page_offset(self) -> PageOffset;
    fn p4_index(self) -> PageTableIndex;
    fn p3_index(self) -> PageTableIndex;
    fn p2_index(self) -> PageTableIndex;
    fn p1_index(self) -> PageTableIndex;
    fn page_table_index(self, level: PageTableLevel) -> PageTableIndex;
    fn new_virt_truncate(addr: u64) -> u64;
    fn forward_checked_u64(start: Self, count: u64) -> Option<Self> where Self: Sized;
    fn forward_checked_impl(start: Self, count: usize) -> Option<Self> where Self: Sized;
}

impl VirtAddr for u64 {
    #[inline]
    fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self as u16)
    }
    #[inline]
    fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self >> 12) as u16)
    }

    /// Returns the 9-bit level 2 page table index.
    #[inline]
    fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self >> 12 >> 9) as u16)
    }

    /// Returns the 9-bit level 3 page table index.
    #[inline]
    fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self >> 12 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level 4 page table index.
    #[inline]
    fn p4_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self >> 12 >> 9 >> 9 >> 9) as u16)
    }
    #[inline]
    fn new_virt_truncate(addr: u64) -> u64 {
        // By doing the right shift as a signed operation (on a i64), it will
        // sign extend the value, repeating the leftmost bit.
        ((addr << 16) as i64 >> 16) as u64
    }

    fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        match level {
            PageTableLevel::One => self.p1_index(),
            PageTableLevel::Two => self.p2_index(),
            PageTableLevel::Three => self.p3_index(),
            PageTableLevel::Four => self.p4_index(),
        } 
    }

    #[inline]
    fn forward_checked_impl(start: Self, count: usize) -> Option<Self> {
        Self::forward_checked_u64(start, u64::try_from(count).ok()?)
    }

    /// An implementation of forward_checked that takes u64 instead of usize.
    #[inline]
    fn forward_checked_u64(start: Self, count: u64) -> Option<Self> {
        if count > ADDRESS_SPACE_SIZE {
            return None;
        }

        let mut addr = start.checked_add(count)?;

        match addr & 0xFFFF800000000000 {
            0x1 => {
                // Jump the gap by sign extending the 47th bit.
                addr |= 0x1ffff00000000000;
            }
            0x2 => {
                // Address overflow
                return None;
            }
            _ => {}
        }

        Some(addr)
    }
}

pub trait PhysAddr {
    fn is_aligned(&self, value: u64) -> bool;
    fn align_down(&self, value: u64) -> Self;
    fn new_truncate(addr: u64) -> Self;
}

impl PhysAddr for u64 {
    fn is_aligned(&self, value: u64) -> bool {
        self.align_down(value) == *self
    }

    fn align_down(&self, value: u64) -> Self {
        assert!(value.is_power_of_two(), "`align` must be a power of two");
        *self & !(value - 1)
    }

    fn new_truncate(addr: u64) -> Self {
        addr % (1 << 52)
    }
}

/// The error returned by the `PageTableEntry::frame` method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameError {
    /// The entry does not have the `PRESENT` flag set, so it isn't currently mapped to a frame.
    FrameNotPresent,
    /// The entry does have the `HUGE_PAGE` flag set. The `frame` method has a standard 4KiB frame
    /// as return type, so a huge frame can't be returned.
    HugeFrame,
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Creates an unused page table entry.
    #[inline]
    pub const fn new() -> Self {
        PageTableEntry { entry: 0 }
    }

    /// Returns whether this entry is zero.
    #[inline]
    pub const fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Sets this entry to zero.
    #[inline]
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    /// Returns the flags of this entry.
    #[inline]
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry)
    }

    /// Returns the physical address mapped by this entry, might be zero.
    #[inline]
    pub fn addr(&self) -> u64 {
        self.entry & 0x000f_ffff_ffff_f000
    }

    /// Returns the physical frame mapped by this entry.
    ///
    /// Returns the following errors:
    ///
    /// - `FrameError::FrameNotPresent` if the entry doesn't have the `PRESENT` flag set.
    /// - `FrameError::HugeFrame` if the entry has the `HUGE_PAGE` flag set (for huge pages the
    ///    `addr` function must be used)
    #[inline]
    pub fn frame(&self) -> Result<u64, FrameError> {
        if !self.flags().contains(PageTableFlags::PRESENT) {
            Err(FrameError::FrameNotPresent)
        } else if self.flags().contains(PageTableFlags::HUGE_PAGE) {
            Err(FrameError::HugeFrame)
        } else {
            Ok(self.addr().align_down(PAGE_4KB_SIZE))
        }
    }

    /// Map the entry to the specified physical address with the specified flags.
    #[inline]
    pub fn set_addr(&mut self, addr: u64, flags: PageTableFlags) {
        assert!(addr.is_aligned(PAGE_4KB_SIZE));
        self.entry = addr | flags.bits();
    }

    /// Map the entry to the specified physical frame with the specified flags.
    #[inline]
    pub fn set_frame(&mut self, frame: u64, flags: PageTableFlags) {
        assert!(!flags.contains(PageTableFlags::HUGE_PAGE));
        self.set_addr(frame, flags)
    }

    /// Sets the flags of this entry.
    #[inline]
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = self.addr() | flags.bits();
    }
}

impl Default for PageTableEntry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("addr", &format_args!("{:#x}", self.addr()));
        f.field("flags", &self.flags());
        f.finish()
    }
}

bitflags! {
    /// Possible flags for a page table entry.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageTableFlags: u64 {
        /// Specifies whether the mapped frame or page table is loaded in memory.
        const PRESENT =         1;
        /// Controls whether writes to the mapped frames are allowed.
        ///
        /// If this bit is unset in a level 1 page table entry, the mapped frame is read-only.
        /// If this bit is unset in a higher level page table entry the complete range of mapped
        /// pages is read-only.
        const WRITABLE =        1 << 1;
        /// Controls whether accesses from userspace (i.e. ring 3) are permitted.
        const USER_ACCESSIBLE = 1 << 2;
        /// If this bit is set, a “write-through” policy is used for the cache, else a “write-back”
        /// policy is used.
        const WRITE_THROUGH =   1 << 3;
        /// Disables caching for the pointed entry is cacheable.
        const NO_CACHE =        1 << 4;
        /// Set by the CPU when the mapped frame or page table is accessed.
        const ACCESSED =        1 << 5;
        /// Set by the CPU on a write to the mapped frame.
        const DIRTY =           1 << 6;
        /// Specifies that the entry maps a huge frame instead of a page table. Only allowed in
        /// P2 or P3 tables.
        const HUGE_PAGE =       1 << 7;
        /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
        /// the TLB on an address space switch.
        const GLOBAL =          1 << 8;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_9 =           1 << 9;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_10 =          1 << 10;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_11 =          1 << 11;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_52 =          1 << 52;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_53 =          1 << 53;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_54 =          1 << 54;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_55 =          1 << 55;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_56 =          1 << 56;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_57 =          1 << 57;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_58 =          1 << 58;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_59 =          1 << 59;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_60 =          1 << 60;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_61 =          1 << 61;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_62 =          1 << 62;
        /// Forbid code execution from the mapped frames.
        ///
        /// Can be only used when the no-execute page protection feature is enabled in the EFER
        /// register.
        const NO_EXECUTE =      1 << 63;
    }
}

/// The number of entries in a page table.
const ENTRY_COUNT: usize = 512;

/// Represents a page table.
///
/// Always page-sized.
///
/// This struct implements the `Index` and `IndexMut` traits, so the entries can be accessed
/// through index operations. For example, `page_table[15]` returns the 16th page table entry.
///
/// Note that while this type implements [`Clone`], the users must be careful not to introduce
/// mutable aliasing by using the cloned page tables.
#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Creates an empty page table.
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        PageTable {
            entries: [EMPTY; ENTRY_COUNT],
        }
    }

    /// Clears all entries.
    #[inline]
    pub fn zero(&mut self) {
        for entry in self.iter_mut() {
            entry.set_unused();
        }
    }

    /// Returns an iterator over the entries of the page table.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        (0..512).map(move |i| &self.entries[i])
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        // Note that we intentionally don't just return `self.entries.iter()`:
        // Some users may choose to create a reference to a page table at
        // `0xffff_ffff_ffff_f000`. This causes problems because calculating
        // the end pointer of the page tables causes an overflow. Therefore
        // creating page tables at that address is unsound and must be avoided.
        // Unfortunately creating such page tables is quite common when
        // recursive page tables are used, so we try to avoid calculating the
        // end pointer if possible. `core::slice::Iter` calculates the end
        // pointer to determine when it should stop yielding elements. Because
        // we want to avoid calculating the end pointer, we don't use
        // `core::slice::Iter`, we implement our own iterator that doesn't
        // calculate the end pointer. This doesn't make creating page tables at
        // that address sound, but it avoids some easy to trigger
        // miscompilations.
        let ptr = self.entries.as_mut_ptr();
        (0..512).map(move |i| unsafe { &mut *ptr.add(i) })
    }

    /// Checks if the page table is empty (all entries are zero).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter().all(|entry| entry.is_unused())
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTable {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}

/// A 9-bit index into a page table.
///
/// Can be used to select one of the 512 entries of a page table.
///
/// Guaranteed to only ever contain 0..512.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Creates a new index from the given `u16`. Panics if the given value is >=512.
    #[inline]
    pub const fn new(index: u16) -> Self {
        assert!((index as usize) < ENTRY_COUNT);
        Self(index)
    }

    /// Creates a new index from the given `u16`. Throws away bits if the value is >=512.
    #[inline]
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }

    #[inline]
    pub(crate) const fn into_u64(self) -> u64 {
        self.0 as u64
    }
}

impl From<PageTableIndex> for u16 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.0
    }
}

impl From<PageTableIndex> for u32 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        u32::from(index.0)
    }
}

impl From<PageTableIndex> for u64 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.into_u64()
    }
}

impl From<PageTableIndex> for usize {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        usize::from(index.0)
    }
}

/// A 12-bit offset into a 4KiB Page.
///
/// This type is returned by the `VirtAddr::page_offset` method.
///
/// Guaranteed to only ever contain 0..4096.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageOffset(u16);

impl PageOffset {
    /// Creates a new offset from the given `u16`. Panics if the passed value is >=4096.
    #[inline]
    pub fn new(offset: u16) -> Self {
        assert!(offset < (1 << 12));
        Self(offset)
    }

    /// Creates a new offset from the given `u16`. Throws away bits if the value is >=4096.
    #[inline]
    pub const fn new_truncate(offset: u16) -> Self {
        Self(offset % (1 << 12))
    }
}

impl From<PageOffset> for u16 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        offset.0
    }
}

impl From<PageOffset> for u32 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u32::from(offset.0)
    }
}

impl From<PageOffset> for u64 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u64::from(offset.0)
    }
}

impl From<PageOffset> for usize {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        usize::from(offset.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A value between 1 and 4.
pub enum PageTableLevel {
    /// Represents the level for a page table.
    One = 1,
    /// Represents the level for a page directory.
    Two,
    /// Represents the level for a page-directory pointer.
    Three,
    /// Represents the level for a page-map level-4.
    Four,
}

impl PageTableLevel {
    /// Returns the next lower level or `None` for level 1
    pub const fn next_lower_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => Some(PageTableLevel::Three),
            PageTableLevel::Three => Some(PageTableLevel::Two),
            PageTableLevel::Two => Some(PageTableLevel::One),
            PageTableLevel::One => None,
        }
    }

    /// Returns the next higher level or `None` for level 4
    pub const fn next_higher_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => None,
            PageTableLevel::Three => Some(PageTableLevel::Four),
            PageTableLevel::Two => Some(PageTableLevel::Three),
            PageTableLevel::One => Some(PageTableLevel::Two),
        }
    }

    /// Returns the alignment for the address space described by a table of this level.
    pub const fn table_address_space_alignment(self) -> u64 {
        1u64 << (self as u8 * 9 + 12)
    }

    /// Returns the alignment for the address space described by an entry in a table of this level.
    pub const fn entry_address_space_alignment(self) -> u64 {
        1u64 << (((self as u8 - 1) * 9) + 12)
    }
}


use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Trait for abstracting over the three possible page sizes on x86_64, 4KiB, 2MiB, 1GiB.
pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    /// The page size in bytes.
    const SIZE: u64;

    /// A string representation of the page size for debug output.
    const DEBUG_STR: &'static str;
}

/// This trait is implemented for 4KiB and 2MiB pages, but not for 1GiB pages.
pub trait NotGiantPageSize: PageSize {}

/// A standard 4KiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size4KiB {}

/// A “huge” 2MiB page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size2MiB {}

/// A “giant” 1GiB page.
///
/// (Only available on newer x86_64 CPUs.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size1GiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
    const DEBUG_STR: &'static str = "4KiB";
}

impl NotGiantPageSize for Size4KiB {}

impl PageSize for Size2MiB {
    const SIZE: u64 = Size4KiB::SIZE * 512;
    const DEBUG_STR: &'static str = "2MiB";
}

impl NotGiantPageSize for Size2MiB {}

impl PageSize for Size1GiB {
    const SIZE: u64 = Size2MiB::SIZE * 512;
    const DEBUG_STR: &'static str = "1GiB";
}

/// A virtual memory page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: u64,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// The page size in bytes.
    pub const SIZE: u64 = S::SIZE;

    /// Returns the page that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid page start).
    #[inline]
    pub fn from_start_address(address: u64) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned(S::SIZE) {
            return Err(AddressNotAligned);
        }
        Ok(Page::containing_address(address))
    }

    /// Returns the page that starts at the given virtual address.
    ///
    /// #Safety
    ///
    /// The address must be correctly aligned.
    #[inline]
    pub unsafe fn from_start_address_unchecked(start_address: u64) -> Self {
        Page {
            start_address,
            size: PhantomData,
        }
    }

    /// Returns the page that contains the given virtual address.
    #[inline]
    pub fn containing_address(address: u64) -> Self {
        Page {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the page.
    #[inline]
    pub fn start_address(self) -> u64 {
        self.start_address
    }

    /// Returns the size the page (4KB, 2MB or 1GB).
    #[inline]
    pub fn size(self) -> u64 {
        S::SIZE
    }

    /// Returns the level 4 page table index of this page.
    #[inline]
    pub fn p4_index(self) -> PageTableIndex {
        self.start_address().p4_index()
    }

    /// Returns the level 3 page table index of this page.
    #[inline]
    pub fn p3_index(self) -> PageTableIndex {
        self.start_address().p3_index()
    }

    /// Returns the table index of this page at the specified level.
    #[inline]
    pub fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        self.start_address().page_table_index(level)
    }

    /// Returns a range of pages, exclusive `end`.
    #[inline]
    pub fn range(start: Self, end: Self) -> PageRange<S> {
        PageRange { start, end }
    }

    /// Returns a range of pages, inclusive `end`.
    #[inline]
    pub fn range_inclusive(start: Self, end: Self) -> PageRangeInclusive<S> {
        PageRangeInclusive { start, end }
    }
}

impl<S: NotGiantPageSize> Page<S> {
    /// Returns the level 2 page table index of this page.
    #[inline]
    pub fn p2_index(self) -> PageTableIndex {
        self.start_address().p2_index()
    }
}

impl Page<Size1GiB> {
    /// Returns the 1GiB memory page with the specified page table indices.
    #[inline]
    pub fn from_page_table_indices_1gib(
        p4_index: PageTableIndex,
        p3_index: PageTableIndex,
    ) -> Self {
        let mut addr = 0;
        addr |= p4_index.into_u64() << 39;
        addr |= p3_index.into_u64() << 30;
        Page::containing_address(u64::new_virt_truncate(addr))
    }
}

impl Page<Size2MiB> {
    /// Returns the 2MiB memory page with the specified page table indices.
    #[inline]
    pub fn from_page_table_indices_2mib(
        p4_index: PageTableIndex,
        p3_index: PageTableIndex,
        p2_index: PageTableIndex,
    ) -> Self {
        let mut addr = 0;
        addr |= p4_index.into_u64() << 39;
        addr |= p3_index.into_u64() << 30;
        addr |= p2_index.into_u64() << 21;
        Page::containing_address(u64::new_virt_truncate(addr))
    }
}

impl Page<Size4KiB> {
    /// Returns the 4KiB memory page with the specified page table indices.
    #[inline]
    pub fn from_page_table_indices(
        p4_index: PageTableIndex,
        p3_index: PageTableIndex,
        p2_index: PageTableIndex,
        p1_index: PageTableIndex,
    ) -> Self {
        let mut addr = 0;
        addr |= p4_index.into_u64() << 39;
        addr |= p3_index.into_u64() << 30;
        addr |= p2_index.into_u64() << 21;
        addr |= p1_index.into_u64() << 12;
        Page::containing_address(u64::new_virt_truncate(addr))
    }

    /// Returns the level 1 page table index of this page.
    #[inline]
    pub fn p1_index(self) -> PageTableIndex {
        self.start_address.p1_index()
    }
}

impl<S: PageSize> fmt::Debug for Page<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Page[{}]({:#x})",
            S::DEBUG_STR,
            self.start_address()
        ))
    }
}

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl<S: PageSize> Sub<Self> for Page<S> {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

/// A range of pages with exclusive upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PageRange<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: Page<S>,
    /// The end of the range, exclusive.
    pub end: Page<S>,
}

impl<S: PageSize> PageRange<S> {
    /// Returns wether this range contains no pages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns the number of pages in the range.
    #[inline]
    pub fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start
        } else {
            0
        }
    }

    /// Returns the size in bytes of all pages within the range.
    #[inline]
    pub fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> Iterator for PageRange<S> {
    type Item = Page<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let page = self.start;
            self.start += 1;
            Some(page)
        } else {
            None
        }
    }
}

impl PageRange<Size2MiB> {
    /// Converts the range of 2MiB pages to a range of 4KiB pages.
    #[inline]
    pub fn as_4kib_page_range(self) -> PageRange<Size4KiB> {
        PageRange {
            start: Page::containing_address(self.start.start_address()),
            end: Page::containing_address(self.end.start_address()),
        }
    }
}

impl<S: PageSize> fmt::Debug for PageRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PageRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// A range of pages with inclusive upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PageRangeInclusive<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: Page<S>,
    /// The end of the range, inclusive.
    pub end: Page<S>,
}

impl<S: PageSize> PageRangeInclusive<S> {
    /// Returns whether this range contains no pages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// Returns the number of frames in the range.
    #[inline]
    pub fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start + 1
        } else {
            0
        }
    }

    /// Returns the size in bytes of all frames within the range.
    #[inline]
    pub fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> Iterator for PageRangeInclusive<S> {
    type Item = Page<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let page = self.start;

            // If the end of the inclusive range is the maximum page possible for size S,
            // incrementing start until it is greater than the end will cause an integer overflow.
            // So instead, in that case we decrement end rather than incrementing start.
            let max_page_addr = u64::MAX - (S::SIZE - 1);
            if self.start.start_address() < max_page_addr {
                self.start += 1;
            } else {
                self.end -= 1;
            }
            Some(page)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PageRangeInclusive<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PageRangeInclusive")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// The given address was not sufficiently aligned.
#[derive(Debug)]
pub struct AddressNotAligned;

impl fmt::Display for AddressNotAligned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the given address was not sufficiently aligned")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    // TODO: Make private when our minimum supported stable Rust version is 1.61
    pub(crate) start_address: u64,
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    /// Returns the frame that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid frame start).
    #[inline]
    pub fn from_start_address(address: u64) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned(S::SIZE) {
            return Err(AddressNotAligned);
        }

        // SAFETY: correct address alignment is checked above
        Ok(unsafe { PhysFrame::from_start_address_unchecked(address) })
    }

    /// Returns the frame that starts at the given virtual address.
    ///
    /// ## Safety
    ///
    /// The address must be correctly aligned.
    #[inline]
    pub unsafe fn from_start_address_unchecked(start_address: u64) -> Self {
        PhysFrame {
            start_address,
            size: PhantomData,
        }
    }

    /// Returns the frame that contains the given physical address.
    #[inline]
    pub fn containing_address(address: u64) -> Self {
        PhysFrame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the frame.
    #[inline]
    pub fn start_address(self) -> u64 {
        self.start_address
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    #[inline]
    pub fn size(self) -> u64 {
        S::SIZE
    }

    /// Returns a range of frames, exclusive `end`.
    #[inline]
    pub fn range(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRange<S> {
        PhysFrameRange { start, end }
    }

    /// Returns a range of frames, inclusive `end`.
    #[inline]
    pub fn range_inclusive(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRangeInclusive<S> {
        PhysFrameRangeInclusive { start, end }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "PhysFrame[{}]({:#x})",
            S::DEBUG_STR,
            self.start_address()
        ))
    }
}

impl<S: PageSize> Add<u64> for PhysFrame<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for PhysFrame<S> {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for PhysFrame<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for PhysFrame<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl<S: PageSize> Sub<PhysFrame<S>> for PhysFrame<S> {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: PhysFrame<S>) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

/// An range of physical memory frames, exclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PhysFrameRange<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The end of the range, exclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRange<S> {
    /// Returns whether the range contains no frames.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns the number of frames in the range.
    #[inline]
    pub fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start
        } else {
            0
        }
    }

    /// Returns the size in bytes of all frames within the range.
    #[inline]
    pub fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> Iterator for PhysFrameRange<S> {
    type Item = PhysFrame<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let frame = self.start;
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// An range of physical memory frames, inclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PhysFrameRangeInclusive<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The start of the range, inclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRangeInclusive<S> {
    /// Returns whether the range contains no frames.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// Returns the number of frames in the range.
    #[inline]
    pub fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start + 1
        } else {
            0
        }
    }

    /// Returns the size in bytes of all frames within the range.
    #[inline]
    pub fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> Iterator for PhysFrameRangeInclusive<S> {
    type Item = PhysFrame<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start;
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRangeInclusive<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRangeInclusive")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}
