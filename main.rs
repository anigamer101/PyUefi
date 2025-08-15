#![no_std]
#![no_main]

extern crate alloc;

use core::ffi::c_void;
use core::ptr::null_mut;
use alloc::vec::Vec;
use r_efi::efi::{Handle, Status, SystemTable};
use r_efi::protocols::loaded_image::{Protocol as LoadedImage, PROTOCOL_GUID as LOADED_IMAGE_GUID};
use r_efi::protocols::simple_file_system::{Protocol as FSProtocol, PROTOCOL_GUID as FS_GUID};
use r_efi::protocols::file::Protocol as FileProtocol;
use r_efi::protocols::simple_text_input::InputKey;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

const SCAN_ESC: u16 = 0x0017;

#[no_mangle]
pub extern "efiapi" fn efi_main(image_handle: Handle, system_table: *mut SystemTable) -> Status {
    unsafe {
        ALLOCATOR.lock().init(0x100000 as usize, 1024 * 1024);
    }

    let st = unsafe { &mut *system_table };

    let mut loaded_image: *mut c_void = null_mut();
    let status = unsafe {
        ((*st.boot_services).handle_protocol)(
            image_handle,
            &LOADED_IMAGE_GUID as *const _ as *mut _,
            &mut loaded_image,
        )
    };
    if status != Status::SUCCESS {
        return status;
    }

    let loaded_image = unsafe { &*(loaded_image as *mut LoadedImage) };
    let device_handle = loaded_image.device_handle;

    let mut fs_ptr: *mut c_void = null_mut();
    let status = unsafe {
        ((*st.boot_services).handle_protocol)(
            device_handle,
            &FS_GUID as *const _ as *mut _,
            &mut fs_ptr,
        )
    };
    if status != Status::SUCCESS {
        return status;
    }

    let fs = unsafe { &mut *(fs_ptr as *mut FSProtocol) };
    let mut root: *mut FileProtocol = null_mut();
    let status = unsafe { (fs.open_volume)(fs, &mut root) };
    if status != Status::SUCCESS {
        return status;
    }

    let mut file: *mut FileProtocol = null_mut();
    let mut filename: Vec<u16> = "main.arc".encode_utf16().chain(core::iter::once(0)).collect();
    let status = unsafe {
        ((*root).open)(
            root,
            &mut file,
            filename.as_mut_ptr(),
            1,
            0,
        )
    };
    if status != Status::SUCCESS {
        return status;
    }

    let mut buffer = [0u8; 4096];
    let mut buffer_size = buffer.len();
    let status = unsafe {
        ((*file).read)(
            file,
            &mut buffer_size,
            buffer.as_mut_ptr() as *mut c_void,
        )
    };
    if status != Status::SUCCESS {
        return status;
    }

    let _ = unsafe { ((*file).close)(file) };

    let bytecode = &buffer[..buffer_size];
    let mut pc = 0;
    let mut stack: Vec<u64> = Vec::new();
    let mut vars: [u64; 64] = [0; 64];
    while pc < bytecode.len() {
        match bytecode[pc] {
            0x00 => pc += 1,

            0x01 => {
                if pc + 1 < bytecode.len() {
                    let len = bytecode[pc + 1] as usize;
                    if pc + 2 + len <= bytecode.len() {
                        let slice = &bytecode[pc + 2..pc + 2 + len];
                        if let Ok(text) = core::str::from_utf8(slice) {
                            let utf16: Vec<u16> = text.encode_utf16().chain(core::iter::once(0)).collect();
                            unsafe {
                                ((*st.con_out).output_string)(st.con_out, utf16.as_ptr() as *mut u16);
                            }
                        }
                        pc += 2 + len;
                    } else {
                        pc += 1;
                    }
                } else {
                    pc += 1;
                }
            }

            0x02 => { stack.push(2); pc += 1; }
            0x03 => { stack.push(3); pc += 1; }
            0x04 => { stack.push(42); pc += 1; }
            0x05 => { stack.push(0x53545249); pc += 1; }
            0x06 => { stack.push(99); pc += 1; }
            0x07 => { stack.push(0x57414954); pc += 1; }
            0x08 => { stack.push(0x53544400); pc += 1; }

            0x09 => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a + b);
                }
                pc += 1;
            }

            0x10 => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a - b);
                }
                pc += 1;
            }

            0x11 => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();
                    stack.push(a * b);
                }
                pc += 1;
            }

            0x12 => {
                if stack.len() >= 2 {
                    let b = stack.pop().unwrap_or(1);
                    let a = stack.pop().unwrap();
                    stack.push(a / b);
                }
                pc += 1;
            }

            0x13 => { stack.push(0xDEADBEEF); pc += 1; }
            0x14 => { stack.push(0xCAFEBABE); pc += 1; }
            0x15 => { stack.push(0xFEEDFACE); pc += 1; }

            0x16 => unsafe {
                ((*(*st).runtime_services).reset_system)(2, Status::SUCCESS, 0, null_mut());
            }

            0x17 => unsafe {
                ((*(*st).runtime_services).reset_system)(0, Status::SUCCESS, 0, null_mut());
            }

            0x18 => { stack.push(0xB105F00D); pc += 1; }
            0x19 => { stack.push(0xB007B007); pc += 1; }
            0x20 => { stack.push(0x10AD10AD); pc += 1; }
            0x21 => { stack.push(0xC0DECAFE); pc += 1; }
            0x22 => { stack.push(0x7E57C0DE); pc += 1; }
            0x23 => { stack.push(0x0BADC0DE); pc += 1; }
            0x24 => { stack.push(0xFACEFEED); pc += 1; }
            0x25 => { stack.push(0xC001D00D); pc += 1; }
            0x26 => { stack.push(0xBEEFBEEF); pc += 1; }
            0x27 => { stack.push(0xF00DF00D); pc += 1; }
            0x28 => { stack.push(0xB16B00B5); pc += 1; }
            0x29 => { stack.push(12); stack.push(34); stack.push(56); pc += 1; }

            0x30 => { stack.clear(); pc += 1; }
            0x31 => { break; }
            0x32 => {
                let val = stack.pop().unwrap_or(0);
                vars[0] = val;
                pc += 1;
            }

            0xFF => {
                if pc + 2 < bytecode.len() {
                    let jump_offset = u16::from_le_bytes([bytecode[pc + 1], bytecode[pc + 2]]) as usize;

                    let msg = "HALT: Press any key to continue, or ESC to jump...\r\n";
                    let utf16: Vec<u16> = msg.encode_utf16().chain(core::iter::once(0)).collect();

                    unsafe {
                        ((*st.con_out).output_string)(st.con_out, utf16.as_ptr() as *mut u16);
                        let _ = ((*st.con_in).reset)(st.con_in, false.into());

                        let mut key: InputKey = InputKey {
                            scan_code: 0,
                            unicode_char: 0,
                        };

                        loop {
                            let status = ((*st.con_in).read_key_stroke)(st.con_in, &mut key);
                            if status == Status::SUCCESS {
                                break;
                            }
                        }

                        if key.scan_code == SCAN_ESC && jump_offset < bytecode.len() {
                            pc = jump_offset;
                        } else {
                            pc += 3;
                        }
                    }
                } else {
                    pc += 1;
                }
            }

            _ => pc += 1, // Unknown opcode
        }
    }

    Status::SUCCESS
}