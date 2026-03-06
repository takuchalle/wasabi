#![no_std]
#![no_main]

#[no_mangle]
fn efi_main() {
    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}
