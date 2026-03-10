#![no_std]
#![no_main]
#![feature(offset_of)]
use core::arch::asm;
use core::mem::offset_of;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;

type EfiVoid = u8;
type EfiHandle = u64;
type Result<T> = core::result::Result<T, &'static str>;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EfiGuid {
    pub data0: u32,
    pub data1: u16,
    pub data2: u16,
    pub data3: [u8; 8],
}

// Defined here(https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html#efi-graphics-output-protocol)
const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data0: 0x9042a9de,
    data1: 0x23dc,
    data2: 0x4a38,
    data3: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[must_use]
#[repr(u64)]
enum EfiStatus {
    Success = 0,
}

#[repr(C)]
struct EfiBootServicesTable {
    _reserved0: [u64; 40],
    locate_protocol: extern "win64" fn(
        protocol: *const EfiGuid,
        registration: *const EfiVoid,
        interface: *mut *mut EfiVoid,
    ) -> EfiStatus,
}

const _: () = assert!(offset_of!(EfiBootServicesTable, locate_protocol) == 320);

#[repr(C)]
struct EfiSystemTable {
    _reserved0: [u64; 12],
    pub boot_services: &'static EfiBootServicesTable,
}

const _: () = assert!(offset_of!(EfiSystemTable, boot_services) == 96);

#[repr(C)]
struct EfiGraphicsOutputtProtcolPixelInfo {
    version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    _padding0: [u32; 5],
    pub pixels_per_scan_line: u32,
}

const _: () = assert!(size_of::<EfiGraphicsOutputtProtcolPixelInfo>() == 36);

#[repr(C)]
struct EfiGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    pub info: &'a EfiGraphicsOutputtProtcolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize,
    pub frame_buffer_size: usize,
}

#[repr(C)]
struct EfiGraphicsOutputProtocol<'a> {
    reserved: [u64; 3],
    pub mode: &'a EfiGraphicsOutputProtocolMode<'a>,
}

fn locate_graphics_protocol<'a>(
    efi_system_table: &EfiSystemTable,
) -> Result<&'a EfiGraphicsOutputProtocol> {
    let mut graphic_output_protocol = null_mut::<EfiGraphicsOutputProtocol>();
    let status = (efi_system_table.boot_services.locate_protocol)(
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        null_mut::<EfiVoid>(),
        &mut graphic_output_protocol as *mut *mut EfiGraphicsOutputProtocol as *mut *mut EfiVoid,
    );

    if status != EfiStatus::Success {
        return Err("Faild to locate graphic output protocol");
    }

    Ok(unsafe { &*graphic_output_protocol })
}

pub fn hlt() {
    unsafe {
        asm! {"hlt"}
    }
}

#[no_mangle]
fn efi_main(_image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let efi_graphics_output_protocol = locate_graphics_protocol(efi_system_table).unwrap();
    let vram_addr = efi_graphics_output_protocol.mode.frame_buffer_base;
    let vram_size = efi_graphics_output_protocol.mode.frame_buffer_size;
    let vram = unsafe {
        core::slice::from_raw_parts_mut(vram_addr as *mut u32, vram_size / size_of::<u32>())
    };
    for e in vram {
        *e = 0xffffff;
    }
    loop {
        hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
