use lazy_static::lazy_static;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kern_code = gdt.add_entry(Descriptor::kernel_code_segment());
        let user_code = gdt.add_entry(Descriptor::user_code_segment());
        let user_data = gdt.add_entry(Descriptor::user_data_segment());
        let tss_selec = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kern_code,
                user_code,
                user_data,
                tss_selec,
            },
        )
    };
}

#[allow(unused)]
struct Selectors {
    kern_code: SegmentSelector,
    user_code: SegmentSelector,
    user_data: SegmentSelector,
    tss_selec: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::{segmentation::set_cs, tables::load_tss};

    GDT.0.load();
    unsafe {
        set_cs(GDT.1.kern_code);
        load_tss(GDT.1.tss_selec);
    }
    print!("GDT loaded");
    ok!();
}
