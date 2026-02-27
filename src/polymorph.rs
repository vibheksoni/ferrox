#![feature(optimize_attribute)]

use rand::Rng;
use std::arch::asm;

pub static mut OPAQUE_VALUE_1: u64 = 0x1337DEADBEEFC0DE;
pub static mut OPAQUE_VALUE_2: u64 = 0xCAFEBABE13371337;
pub static mut OPAQUE_VALUE_3: u64 = 0xFEEDFACEDEADC0DE;

#[inline(never)]
#[cold]
#[optimize(speed)]
pub fn random_nop_sled() {
    unsafe {
        match rand::random::<u8>() % 10 {
            0 => asm!("nop"),
            1 => asm!("nop", "nop"),
            2 => asm!("xor eax, eax", "test eax, eax"),
            3 => asm!("lea rax, [rax]"),
            4 => asm!("push rax", "pop rax"),
            5 => asm!("mov eax, eax"),
            6 => asm!("nop", "nop", "nop"),
            7 => asm!("xchg ax, ax"),
            8 => asm!("lea rax, [rax + 0]"),
            _ => asm!("pause"),
        }
    }
}

#[inline(never)]
#[cold]
pub fn extended_nop_sled() {
    let choice = rand::random::<u8>() % 8;
    unsafe {
        match choice {
            0 => {
                asm!("nop", "nop", "nop", "nop");
            }
            1 => {
                asm!("mov eax, eax", "mov ebx, ebx");
            }
            2 => {
                asm!("xor eax, eax", "add eax, 0", "test eax, eax");
            }
            3 => {
                asm!("push rax", "push rbx", "pop rbx", "pop rax");
            }
            4 => {
                asm!("lea rax, [rax]", "lea rbx, [rbx]");
            }
            5 => {
                asm!("xchg ax, ax", "xchg bx, bx");
            }
            6 => {
                asm!("mov eax, eax", "xor eax, eax", "test eax, eax");
            }
            _ => {
                asm!("pause", "pause");
            }
        }
    }
}

#[inline(never)]
#[optimize(none)]
pub fn opaque_predicate_1() -> bool {
    unsafe {
        let val = std::ptr::read_volatile(std::ptr::addr_of!(OPAQUE_VALUE_1));
        std::hint::black_box(val.wrapping_mul(2) > val)
    }
}

#[inline(never)]
#[optimize(none)]
pub fn opaque_predicate_2() -> bool {
    unsafe {
        let val = std::ptr::read_volatile(std::ptr::addr_of!(OPAQUE_VALUE_2));
        std::hint::black_box(val | 1 != 0)
    }
}

#[inline(never)]
#[optimize(none)]
pub fn opaque_predicate_3() -> bool {
    unsafe {
        let val = std::ptr::read_volatile(std::ptr::addr_of!(OPAQUE_VALUE_3));
        std::hint::black_box(val ^ val == 0)
    }
}

#[inline(never)]
#[optimize(none)]
pub fn opaque_false_predicate() -> bool {
    unsafe {
        let val = std::ptr::read_volatile(std::ptr::addr_of!(OPAQUE_VALUE_1));
        std::hint::black_box(val == 0)
    }
}

#[inline(never)]
#[optimize(none)]
pub fn polymorphic_add(a: u32, b: u32) -> u32 {
    std::hint::black_box(match rand::random::<u8>() % 3 {
        0 => a.wrapping_add(b),
        1 => a.wrapping_sub(0u32.wrapping_sub(b)),
        _ => {
            let xor = a ^ b;
            let and = (a & b) << 1;
            xor.wrapping_add(and)
        }
    })
}

#[inline(never)]
#[optimize(none)]
pub fn polymorphic_compare(a: u32, b: u32) -> bool {
    std::hint::black_box(match rand::random::<u8>() % 3 {
        0 => a > b,
        1 => b < a,
        _ => a.wrapping_sub(b) != 0 && a != b && a > b
    })
}

#[inline(never)]
#[optimize(none)]
pub fn stack_junk_small() {
    let _junk1 = rand::random::<u64>();
    let _junk2 = rand::random::<u64>();
    let _junk3 = rand::random::<u64>();
    std::hint::black_box(_junk1);
    std::hint::black_box(_junk2);
    std::hint::black_box(_junk3);
}

#[inline(never)]
#[optimize(none)]
pub fn stack_junk_large() {
    let _junk1 = rand::random::<u64>();
    let _junk2 = rand::random::<u64>();
    let _junk3 = rand::random::<u64>();
    let _junk4 = rand::random::<u64>();
    let _junk5 = rand::random::<u64>();
    let _junk6 = rand::random::<u64>();
    let _junk7 = rand::random::<u64>();
    let _junk8 = rand::random::<u64>();
    std::hint::black_box(_junk1);
    std::hint::black_box(_junk2);
    std::hint::black_box(_junk3);
    std::hint::black_box(_junk4);
    std::hint::black_box(_junk5);
    std::hint::black_box(_junk6);
    std::hint::black_box(_junk7);
    std::hint::black_box(_junk8);
}

#[inline(never)]
#[optimize(none)]
pub fn polymorphic_sleep_ms(ms: u64) {
    match rand::random::<u8>() % 4 {
        0 => std::thread::sleep(std::time::Duration::from_millis(ms)),
        1 => {
            let start = std::time::Instant::now();
            while start.elapsed().as_millis() < ms as u128 {
                std::hint::spin_loop();
            }
        }
        2 => {
            for _ in 0..ms {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        _ => {
            let intervals = (ms / 10).max(1);
            for _ in 0..intervals {
                std::thread::sleep(std::time::Duration::from_millis(ms / intervals));
            }
        }
    }
}

#[inline(never)]
#[cold]
#[optimize(none)]
pub fn confuse_control_flow() {
    let choice = rand::random::<u8>() % 5;
    
    if opaque_predicate_1() {
        match choice {
            0 => random_nop_sled(),
            1 => stack_junk_small(),
            2 => extended_nop_sled(),
            3 => { let _ = polymorphic_add(choice as u32, 42); }
            _ => {}
        }
    }
    
    if opaque_predicate_2() {
        let _temp = polymorphic_compare(choice as u32, 100);
        std::hint::black_box(_temp);
    }
}

#[macro_export]
macro_rules! polymorph_guard {
    ($code:block) => {{
        $crate::polymorph::random_nop_sled();
        if $crate::polymorph::opaque_predicate_1() {
            let result = $code;
            $crate::polymorph::stack_junk_small();
            result
        } else {
            unreachable!()
        }
    }};
}

#[macro_export]
macro_rules! polymorph_call {
    ($func:expr) => {{
        $crate::polymorph::confuse_control_flow();
        let result = $func;
        $crate::polymorph::random_nop_sled();
        result
    }};
}

#[macro_export]
macro_rules! polymorph_heavy {
    ($code:block) => {{
        $crate::polymorph::extended_nop_sled();
        $crate::polymorph::stack_junk_large();
        if $crate::polymorph::opaque_predicate_1() && $crate::polymorph::opaque_predicate_2() {
            let result = $code;
            $crate::polymorph::confuse_control_flow();
            result
        } else {
            unreachable!()
        }
    }};
}
