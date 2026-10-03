use core::ptr::{read_volatile, write_volatile};

/// Reads the address and sets the preffered bits
pub fn set_bits(addr: usize, mask: u32) {
    let ptr = addr as *mut u32;
    let ptr_value = unsafe { read_volatile(ptr) };
    let ptr_rez = ptr_value | mask;
    unsafe { write_volatile(ptr, ptr_rez) };
}

/// Write-only the entire register
pub fn write_reg(addr: usize, value: u32) {
    let ptr = addr as *mut u32;
    unsafe { write_volatile(ptr, value) };
}
