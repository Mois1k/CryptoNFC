#![no_std]
#![no_main]

use core::{
    arch::asm,
    panic::PanicInfo,
    ptr::{read_volatile, write_volatile},
};

const CPACR_ADDR: usize = 0xE000ED88;

#[panic_handler]
fn panic_handler(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

unsafe extern "C" {
    static _estack: u32;
    static _sidata: u32;
    static _sdata: u32;
    static _edata: u32;
    static _sbss: u32;
    static _ebss: u32;
}

#[repr(C)]
struct VectorTable {
    stack_pointer: *const u32,
    reset: unsafe extern "C" fn() -> !,
    nmi: unsafe extern "C" fn() -> !,
    hard_fault: unsafe extern "C" fn() -> !,
}

unsafe impl Sync for VectorTable {}

#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR: VectorTable = VectorTable {
    stack_pointer: unsafe { &_estack as *const u32 },
    reset: Reset,
    nmi: DefaultHandler,
    hard_fault: DefaultHandler,
};

#[used]
static mut DATA_CONTOR_TEST: u32 = 5;
#[used]
static mut BSS_ZERO_TEST: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn Reset() -> ! {
    let cpacr = CPACR_ADDR as *mut u32;
    let mask: u32 = 0x00f0_0000;
    let value = unsafe { read_volatile(cpacr) };
    let result = value | mask;

    unsafe { write_volatile(cpacr, result) };
    unsafe {
        asm!("dsb");
        asm!("isb");
    }

    let mut sursa_data = unsafe { &_sidata as *const u32 }; // FLASH ADDRESS SOURCE
    let mut destinatie_data = unsafe { &_sdata as *const u32 as *mut u32 }; // 
    let sfarsit_data = unsafe { &_edata as *const u32 as *mut u32 };

    unsafe {
        while destinatie_data < sfarsit_data {
            let val_data = sursa_data.read_volatile();
            destinatie_data.write_volatile(val_data);
            sursa_data = sursa_data.add(1);
            destinatie_data = destinatie_data.add(1);
        }
    }

    let mut destinatie_bss = unsafe { &_sbss as *const u32 as *mut u32 };
    let sfarsit_bss = unsafe { &_ebss as *const u32 as *mut u32 };

    unsafe {
        while destinatie_bss < sfarsit_bss {
            destinatie_bss.write_volatile(0);
            destinatie_bss = destinatie_bss.add(1);
        }
    }
    main()
}

fn main() -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() -> ! {
    loop {}
}
