#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(once_cell)]

#[macro_use]
extern crate log;

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

pub struct IntTable {
    pub timer: fn(),
}

extern "Rust" {
    fn interrupt_table() -> IntTable;
}
