use bitvec::prelude::*;
use std::cmp::{min, max};

pub type VmUsize = u32;
pub const MEM_SIZE: usize = 1 << 32;

// pub type VmUsize = u16;
// pub const MEM_SIZE: usize = 1 << 16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Null,
    Inc(VmUsize),
    Cdec(VmUsize),
    Load,
    Inv,
}
pub type Instructions = Vec<Instruction>;

impl Instruction {
    pub fn to_wpk_string(&self) -> String {
        match self {
            Self::Null => unreachable!(),
            Self::Inc(1) => "INC\n".to_string(),
            Self::Inc(x) => format!("INC {}\n", x),
            Self::Cdec(1) => "CDEC\n".to_string(),
            Self::Cdec(x) => format!("CDEC {}\n", x),
            Self::Load => "LOAD\n".to_string(),
            Self::Inv => "INV\n".to_string()
        }
    }

    pub fn to_wpkm_string(&self) -> String {
        match self {
            Self::Null => unreachable!(),
            Self::Inc(1) => ">".to_string(),
            Self::Inc(x) => format!("{}>", x),
            Self::Cdec(1) => "<".to_string(),
            Self::Cdec(x) => format!("{}<", x),
            Self::Load => "?".to_string(),
            Self::Inv => "!".to_string()
        }
    }
}

pub struct MemoryPointer {
    pub ptr: VmUsize,
    pub ptr_i: i64,
    pub ptr_lb: i64,
    pub ptr_ub: i64,
}

impl MemoryPointer {
    pub fn new() -> Self {
        Self {
            ptr: 0,
            ptr_i: 0,
            ptr_lb: 0,
            ptr_ub: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ptr = 0;
        self.ptr_i = 0;
        self.ptr_lb = 0;
        self.ptr_ub = 0;
    }

    pub fn inc(&mut self, x: VmUsize) {
        self.ptr = self.ptr.wrapping_add(x);
        self.ptr_i += x as i64;
        self.ptr_ub = max(self.ptr_ub, self.ptr_i);
    }

    pub fn dec(&mut self, x: VmUsize) {
        self.ptr = self.ptr.wrapping_sub(x);
        self.ptr_i -= x as i64;
        self.ptr_lb = min(self.ptr_lb, self.ptr_i);
    }

    pub fn span(&self) -> i64 {
        min(self.ptr_ub - self.ptr_lb + 1, MEM_SIZE as i64)
    }
}

pub struct Vm {
    pub memory: BitVec<u8>,
    pub memory_pointer: MemoryPointer,

    pub program: Instructions,
    pub intsruction_pointer: usize,
    pub runtime: i64,
    pub halted: bool,

    pub register: bool
}

pub struct RunResult {
    pub runtime: i64,
    pub memory: i64,
}

impl Vm {
    pub fn new(program: Instructions) -> Self {
        let proglen = program.len();
        Self {
            memory: bitvec![u8, Lsb0; 0; MEM_SIZE],
            memory_pointer: MemoryPointer::new(),

            program,
            intsruction_pointer: 0,
            halted: proglen == 0,
            runtime: 0,

            register: false,
        }
    }

    pub fn reset(&mut self) {
        self.memory.fill(false);
        self.memory_pointer.reset();
        self.intsruction_pointer = 0;
        self.halted = self.program.len() == 0;
        self.runtime = 0;
        self.register = false;
    }

    pub fn run(&mut self) -> RunResult {
        while !self.halted {
            let current_memory = self.memory[self.memory_pointer.ptr as usize];

            match self.program[self.intsruction_pointer] {
                Instruction::Inc(x) => {
                    self.memory_pointer.inc(x);
                    self.runtime += x as i64;
                }
                Instruction::Cdec(x) => {
                    if self.register {
                        self.memory_pointer.dec(x);
                    }
                    self.runtime += x as i64;
                }
                Instruction::Load => {
                    self.register = current_memory;
                    self.runtime += 1;
                }
                Instruction::Inv => {
                    self.memory.set(self.memory_pointer.ptr as usize, !current_memory);
                    self.runtime += 1;
                },
                Instruction::Null => {
                    unreachable!();
                }
            }

            self.intsruction_pointer += 1;
            if self.intsruction_pointer == self.program.len() {
                self.halted = true;
            }
        }

        return RunResult {
            runtime: self.runtime,
            memory: self.memory_pointer.span()
        }
    }

    pub fn opcount(&self) -> (u64, u64, u64, u64) {
        let mut inc_count: u64 = 0;
        let mut cdec_count: u64 = 0;
        let mut load_count: u64 = 0;
        let mut inv_count: u64 = 0;

        for instruction in self.program.iter() {
            match instruction {
                Instruction::Inc(x) => {
                    inc_count += *x as u64;
                }
                Instruction::Cdec(x) => {
                    cdec_count += *x as u64;
                }
                Instruction::Load => {
                    load_count += 1;
                }
                Instruction::Inv => {
                    inv_count += 1;
                },
                Instruction::Null => {
                    unreachable!();
                }
            }
        }

        return (inc_count, cdec_count, load_count, inv_count)
    }
}