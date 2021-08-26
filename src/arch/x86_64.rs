use core::fmt;
use core::ops;
use gimli::{Register, X86_64};

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub registers: [usize; 16],
    pub ra: usize,
    pub mcxsr: usize,
    pub fcw: usize,
}

pub struct Arch;

impl Arch {
    pub const SP: Register = X86_64::RSP;
    pub const RA: Register = X86_64::RA;
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=15 {
            fmt.field(
                X86_64::register_name(Register(i as _)).unwrap(),
                &self.registers[i],
            );
        }
        fmt.field("ra", &self.ra)
            .field("mcxsr", &self.mcxsr)
            .field("fcw", &self.fcw)
            .finish()
    }
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=15) => &self.registers[reg.0 as usize],
            X86_64::RA => &self.ra,
            X86_64::MXCSR => &self.mcxsr,
            X86_64::FCW => &self.fcw,
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=15) => &mut self.registers[reg.0 as usize],
            X86_64::RA => &mut self.ra,
            X86_64::MXCSR => &mut self.mcxsr,
            X86_64::FCW => &mut self.fcw,
            _ => unimplemented!(),
        }
    }
}

#[naked]
pub extern "C-unwind" fn save_context() -> Context {
    // No need to save caller-saved registers here.
    unsafe {
        asm!(
            "
            mov rax, rdi
            mov [rax + 0x18], rbx
            mov [rax + 0x30], rbp

            /* Adjust the stack to account for the return address */
            lea rdi, [rsp + 8]
            mov [rax + 0x38], rdi

            mov [rax + 0x60], r12
            mov [rax + 0x68], r13
            mov [rax + 0x70], r14
            mov [rax + 0x78], r15
            mov rdx, [rsp]
            mov [rax + 0x80], rdx
            stmxcsr [rax + 0x88]
            fnstcw [rax + 0x90]
            ret
            ",
            options(noreturn)
        );
    }
}

#[naked]
pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    unsafe {
        asm!(
            "
            /* Restore stack */
            mov rsp, [rdi + 0x38]

            /* Restore callee-saved control registers */
            ldmxcsr [rdi + 0x88]
            fldcw [rdi + 0x90]

            /* Restore return address */
            mov rax, [rdi + 0x80]
            push rax

            /*
            * Restore general-purpose registers. Non-callee-saved registers are
            * also restored because sometimes it's used to pass unwind arguments.
            */
            mov rax, [rdi + 0x00]
            mov rdx, [rdi + 0x08]
            mov rcx, [rdi + 0x10]
            mov rbx, [rdi + 0x18]
            mov rsi, [rdi + 0x20]
            mov rbp, [rdi + 0x30]
            mov r8 , [rdi + 0x40]
            mov r9 , [rdi + 0x48]
            mov r10, [rdi + 0x50]
            mov r11, [rdi + 0x58]
            mov r12, [rdi + 0x60]
            mov r13, [rdi + 0x68]
            mov r14, [rdi + 0x70]
            mov r15, [rdi + 0x78]

            /* RDI resotred last */
            mov rdi, [rdi + 0x28]

            ret
            ",
            options(noreturn)
        );
    }
}
