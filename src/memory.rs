use std::ops::{Index, IndexMut};
use std::ops::Range;
use std::cmp;

pub struct Memory {
    pub stack_memory: Vec<u8>,
    pub program_break: usize,
    pub heap_memory: Vec<u8>,
    pub virtual_memory_size: usize,
    verbose: bool
}

impl Memory {
    pub fn new(verbose: bool) -> Self {
        Memory {
            stack_memory: vec![0; 2048],
            program_break: 0,
            heap_memory:  Vec::new(),
            virtual_memory_size: 4096,
            verbose
        }
    }

    pub fn load_from(&self, address: i64) -> i64 {
        let address = address as usize;
        let mut value: [u8; 8] = Default::default();
        value.copy_from_slice(&self[address..address + 8]);
        i64::from_le_bytes(value)
    }

    pub fn store_to(&mut self, address: i64, value: i64, byte_count: usize) {
        let address = address as usize;
        let bytes = value.to_le_bytes();
        //println!("{} {} {} {}", bytes[0], bytes[1], bytes[2], bytes[3]);
        for i in 0..byte_count {
            let val = bytes[i];
            if self.verbose {
                print!("\x1b[33m");
                print!("memory[{:#04x}] = {} '{}'", address + i, val, val as char);
                println!("\x1b[0m");
            }
            self[address + i] = val;
        }
    }

    fn is_stack_address(&self, address: i32) -> bool {
        address as usize >= self.virtual_memory_size - self.stack_memory.len()
    }

    fn get_segment_for_address(&self, address: i32) -> &Vec<u8> {
        if self.is_stack_address(address) {
            return &self.stack_memory
        }
        &self.heap_memory
    }

    fn get_segment_for_address_mut(&mut self, address: i32) -> &mut Vec<u8> {
        if self.is_stack_address(address) {
            return &mut self.stack_memory
        }
        &mut self.heap_memory
    }

    fn address_to_index(&self, address: i32) -> usize {
        if self.is_stack_address(address) {
            return (address as usize) - (self.virtual_memory_size - self.stack_memory.len());
        }
        address as usize
    }

    fn is_address_valid(&self, address: usize) -> bool {
        if self.is_stack_address(address as i32) {
            return address - (self.virtual_memory_size - self.stack_memory.len()) < self.stack_memory.len()
        }
        address < self.heap_memory.len()
    }
}

impl Index<usize> for Memory {
    type Output = u8;
    fn index(&self, address: usize) -> &Self::Output {
        assert!(self.is_address_valid(address), "Address {:#x} is not in allocated memory space!", address);
        let index = self.address_to_index(address as i32);
        &self.get_segment_for_address(address as i32)[index]
    }
}

impl Index<Range<usize>> for Memory {
    type Output = [u8];
    fn index(&self, range: Range<usize>) -> &Self::Output {
        assert!(self.is_address_valid(range.start), "Address {:#x} (start of range) is not in allocated memory space!", range.start);
        assert!(self.is_address_valid(range.end-1), "Address {:#x} (end of range) is not in allocated memory space!", range.end);
        let segment = self.get_segment_for_address(range.start as i32);
        if segment != self.get_segment_for_address(range.end as i32) {
            panic!("Range has to be in the same segment!");
        }
        let range_len = cmp::max(range.end - range.start, 0);
        let range_start = self.address_to_index(range.start as i32);
        &segment[range_start..range_start + range_len]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, address: usize) -> &mut Self::Output {
        assert!(self.is_address_valid(address), "Address {:#x} is not in allocated memory space!", address);
        let index = self.address_to_index(address as i32);
        &mut self.get_segment_for_address_mut(address as i32)[index]
    }
}
