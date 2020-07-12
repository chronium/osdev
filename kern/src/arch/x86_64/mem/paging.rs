use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        mapper::{MapToError, Mapper},
        page_table::FrameError,
        FrameAllocator, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub unsafe fn init(phys_mem_offs: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_l4_table(phys_mem_offs);
    println!("Offset page table loaded");
    OffsetPageTable::new(l4_table, phys_mem_offs)
}

pub unsafe fn active_l4_table(phys_mem_offs: VirtAddr) -> &'static mut PageTable {
    let (l4_tframe, _) = Cr3::read();

    let phys = l4_tframe.start_address();
    let virt = phys_mem_offs + phys.as_u64();
    let ptable_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *ptable_ptr
}

pub struct BootInfoFrameAllocator {
    mem_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(mem_map: &'static MemoryMap) -> Self {
        Self { mem_map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.mem_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));

        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub fn identity_map(
    from: PhysAddr,
    to: PhysAddr,
    flags: PageTableFlags,
    inclusive: bool,
) -> Result<(), MapToError<Size4KiB>> {
    use crate::{FRAME_ALLOC, MAPPER};

    let mut frame_allocator = FRAME_ALLOC.wait().lock();
    let mut mapper = MAPPER.wait().lock();

    if inclusive {
        for frame in PhysFrame::range_inclusive(
            PhysFrame::from_start_address(from).expect("Address not aligned"),
            PhysFrame::from_start_address(to).expect("Address not aligned"),
        ) {
            unsafe {
                mapper
                    .identity_map(frame, flags, &mut *frame_allocator)?
                    .flush()
            };
        }
    } else {
        for frame in PhysFrame::range(
            PhysFrame::from_start_address(from).expect("Address not aligned"),
            PhysFrame::from_start_address(to).expect("Address not aligned"),
        ) {
            unsafe {
                mapper
                    .identity_map(frame, flags, &mut *frame_allocator)?
                    .flush()
            };
        }
    }

    Ok(())
}
