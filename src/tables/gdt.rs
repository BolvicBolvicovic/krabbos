use lazy_static::lazy_static;
use crate::tables::DescriptorTablePointer;
use core::arch::asm;

const SEGMENT_LIMIT: u32 = 0xFFFFFFFF;
const SEGMENT_BASE: u32  = 0;

/***	 gdt descriptor access bit flags.	***/

// set access bit
const I86_GDT_DESC_ACCESS: u8 = 0x0001;			//00000001

// descriptor is readable and writable. default: read only
const I86_GDT_DESC_READWRITE: u8 = 0x0002;		//00000010

// set expansion direction bit
const I86_GDT_DESC_EXPANSION: u8 = 0x0004;		//00000100

// executable code segment. Default: data segment
const I86_GDT_DESC_EXEC_CODE: u8 = 0x0008;		//00001000

// set code or data descriptor. defult: system defined descriptor
const I86_GDT_DESC_CODEDATA	: u8 = 0x0010;		//00010000

// set dpl bits
const I86_GDT_DESC_DPL: u8 = 0x0060;			//01100000

// set "in memory" bit
const I86_GDT_DESC_MEMORY: u8 = 0x0080;			//10000000

/**	gdt descriptor grandularity bit flags	***/

// masks out limitHi (High 4 bits of limit)
const I86_GDT_GRAND_LIMITHI_MASK: u8 = 0x0f;	//00001111

// set os defined bit
const I86_GDT_GRAND_OS: u8 = 0x10;			    //00010000

// set if 32bit. default: 16 bit
const I86_GDT_GRAND_32BIT: u8 = 0x40;			//01000000

// set if 64bit. 32bit needs to be clear to use
const I86_GDT_GRAND_64BIT: u8 = 0x20;           //00100000

// 4k grandularity. default: none
const I86_GDT_GRAND_4K: u8 = 0x80;			    //10000000

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable([GDTEntry::null(); 8192]);
        // kernel Code Selector 32bits
        gdt.0[1].set_entry(SEGMENT_BASE, SEGMENT_LIMIT, 
	      I86_GDT_DESC_READWRITE | I86_GDT_DESC_EXEC_CODE | I86_GDT_DESC_CODEDATA | I86_GDT_DESC_MEMORY,
	      I86_GDT_GRAND_4K | I86_GDT_GRAND_32BIT | I86_GDT_GRAND_LIMITHI_MASK 
        );

        // kernel Code Selector 64bits
        gdt.0[2].set_entry(SEGMENT_BASE, SEGMENT_LIMIT, 
	      I86_GDT_DESC_READWRITE | I86_GDT_DESC_EXEC_CODE | I86_GDT_DESC_CODEDATA | I86_GDT_DESC_MEMORY,
	      I86_GDT_GRAND_4K | I86_GDT_GRAND_64BIT | I86_GDT_GRAND_LIMITHI_MASK 
        );

        // kernel Data Selector
        gdt.0[3].set_entry(SEGMENT_BASE, SEGMENT_LIMIT,
	      I86_GDT_DESC_READWRITE | I86_GDT_DESC_CODEDATA | I86_GDT_DESC_MEMORY,
	      I86_GDT_GRAND_4K | I86_GDT_GRAND_32BIT | I86_GDT_GRAND_LIMITHI_MASK 
        );

        // user Code Selector 32bits
        gdt.0[4].set_entry(SEGMENT_BASE, SEGMENT_LIMIT,
	      I86_GDT_DESC_DPL | I86_GDT_DESC_READWRITE | I86_GDT_DESC_EXEC_CODE | I86_GDT_DESC_CODEDATA | I86_GDT_DESC_MEMORY,
	      I86_GDT_DESC_DPL | I86_GDT_GRAND_4K | I86_GDT_GRAND_32BIT | I86_GDT_GRAND_LIMITHI_MASK 
        );

        // user Code Selector 64bits
        gdt.0[5].set_entry(SEGMENT_BASE, SEGMENT_LIMIT,
	      I86_GDT_DESC_DPL | I86_GDT_DESC_READWRITE | I86_GDT_DESC_EXEC_CODE | I86_GDT_DESC_CODEDATA | I86_GDT_DESC_MEMORY,
	      I86_GDT_DESC_DPL | I86_GDT_GRAND_4K | I86_GDT_GRAND_64BIT | I86_GDT_GRAND_LIMITHI_MASK 
        );

        // kernel Data Selector
        gdt.0[6].set_entry(SEGMENT_BASE, SEGMENT_LIMIT,
	      I86_GDT_DESC_DPL | I86_GDT_DESC_READWRITE | I86_GDT_DESC_CODEDATA | I86_GDT_DESC_MEMORY,
	      I86_GDT_DESC_DPL | I86_GDT_GRAND_4K | I86_GDT_GRAND_32BIT | I86_GDT_GRAND_LIMITHI_MASK 
        );

        gdt
    };
}


pub fn set_gdt() {
    GDT.load();
}

struct GlobalDescriptorTable(pub [GDTEntry; 8192]);

impl GlobalDescriptorTable {

    pub fn load(&self) {
        unsafe {
            let gdt = self.pointer();
            asm!("lgdt [{}]", in(reg) &gdt, options(readonly, nostack, preserves_flags));
        }
    }

    pub const fn limit(&self) -> u16 {
        use core::mem::size_of;
        // 0 < self.next_free <= MAX <= 2^13, so the limit calculation
        // will not underflow or overflow.
        (self.0.len() * size_of::<u64>() - 1) as u16
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base: self.0.as_ptr() as u64,
            limit: self.limit(),
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct GDTEntry {
    limit_low: u16,
    base_low:  u16,
    base_mid:   u8,
    access_byte:u8,

    // granularity contains flags on the 4 first bits and the limit_high on the 4 last
    granularity:u8,
    base_high:  u8,
}

impl GDTEntry {
    pub fn null() -> Self {
        GDTEntry {
            limit_low:  0,
            base_low:   0,
            base_mid:   0,
            access_byte:0,
            granularity:0,
            base_high:  0,
        }
    }

    pub fn set_entry(&mut self, base: u32, limit: u32, access_byte: u8, gran: u8) {
        // Set adresses
        self.base_low = (base & 0xFFFF) as u16;
        self.base_mid = ((base >> 16) & 0xFF) as u8;
        self.base_high = ((base >> 24) & 0xFF) as u8;
        self.limit_low = (limit & 0xFFFF) as u16;

        // Set flags
        self.granularity = ((limit >> 16) & 0x0F) as u8;
        self.granularity |= gran & 0xF0;
        self.access_byte = access_byte;
    }
}
