use std::slice;
use std::mem;

pub enum Instruction {
  HALT = 0x00,
  PUSH = 0x01,
  POP  = 0x02,
}

pub struct VirtualMachine {
  pub var_stack:     [u8; 262144],
  pub compute_stack: [u8; 262144],
  pub frames:        Vec<u32>,

  var_top:     u32,
  compute_top: u32,
}

impl VirtualMachine {
  pub fn execute(&mut self, bytecode: &[u8]) -> Result<(), ()> {
    use self::Instruction::*;

    let mut ip = 0;

    loop {
      let code = bytecode[ip];

      match unsafe { mem::transmute::<u8, Instruction>(bytecode[ip]) } {
        PUSH => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let value = unsafe {
            slice::from_raw_parts((code as *const u8).offset(ip as isize), size as usize)
          };

          self.compute_stack[self.compute_top as usize .. (self.compute_top + size as u32) as usize].copy_from_slice(&value);

          ip += 1;

          self.compute_top += size as u32
        },

        POP => {
          ip += 1;

          let size = bytecode[ip];

          ip += 1;

          let raw_bytes = &bytecode[ip .. ip + size as usize];

          let mut bytes: [u8; 4] = [0, 0, 0, 0];
          bytes.copy_from_slice(raw_bytes);

          let address = unsafe {
            mem::transmute::<[u8; mem::size_of::<u32>()], u32>(bytes)
          };

          ip += 1;

          let variable = read(&self.var_stack, address + *self.frames.last().unwrap(), size as u32);
        
          self.compute_top -= size as u32;

          self.compute_stack[self.compute_top as usize .. (self.compute_top + size as u32) as usize].copy_from_slice(&variable);

          if self.var_top < (address + size as u32) {
            self.var_top = address + size as u32
          }
        }

        HALT => break,
      }
    }

    Ok(())
  }
}



fn read (mem: &[u8], from: u32, size: u32) -> &[u8] {
  &mem[from as usize .. (from + size) as usize]
}