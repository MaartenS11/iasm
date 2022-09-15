use std::{collections::HashMap, cmp::min};
use std::io::{self, Write};

use crate::{memory::Memory, registers::Registers};

pub struct Evaluator {
    pub registers: Registers, 
    pub memory: Memory, 
    verbose: bool
}

impl Evaluator {
    pub fn new(verbose: bool) -> Self {
        Evaluator {
            memory: Memory::new(verbose),
            registers: Registers::new(),
            verbose
        }
    }

    pub fn evaluate(&mut self, instruction: &str) {
        let mut instruction_name = instruction;
        let mut params_string = "";
        let s = instruction.split_once(" ");
        if s != None {
            let split = s.unwrap();
            instruction_name = split.0;
            params_string = split.1;
        }
        
        let params: Vec<&str> = params_string.trim().split(",").collect();
    
        match instruction_name {
            "mv" => self.registers[params[0]] = self.registers[params[1].trim()],
            "ld" =>self.registers[params[0]] = self.memory.load_from(Self::parse_memory_location(&self.registers, params[1].trim())),
            //"lw" => variables[params[0]] = (memory.load_from(parse_memory_location(variables, params[1].trim())) as i32) as i64,
            "lw" => {
                let address = Self::parse_memory_location(&self.registers, params[1].trim()) as usize;
                let mut value: [u8; 4] = Default::default();
                value.copy_from_slice(&self.memory[address..address + 4]);
               self.registers[params[0]] = i32::from_le_bytes(value) as i64
            },
            "lwu" =>self.registers[params[0]] = (self.memory.load_from(Self::parse_memory_location(&self.registers, params[1].trim())) as u32) as i64,
            "lbu" => {
                let address = Self::parse_memory_location(&self.registers, params[1].trim()) as usize;
               self.registers[params[0]] = (self.memory[address] as i64) & 0xff; //Remove sign extension
            },
            "lb" => {
                let address = Self::parse_memory_location(&self.registers, params[1].trim()) as usize;
               self.registers[params[0]] = self.memory[address] as i64; //Sign extend to i64
            },
            "lhu" => {
                let address = Self::parse_memory_location(&self.registers, params[1].trim()) as usize;
               self.registers[params[0]] = ((self.memory[address] as u64 | (self.memory[address+1] as u64) << 8) as i64) & 0xffff; //Remove sign extension
            },
            "li" | "lla" =>self.registers[params[0]] = Self::parse_immediate(params[1].trim()),
            "sd" => self.memory.store_to(Self::parse_memory_location(&self.registers, params[1].trim()),self.registers[params[0]], 8),
            "sw" => self.memory.store_to(Self::parse_memory_location(&self.registers, params[1].trim()),self.registers[params[0]], 4),
            "sh" => self.memory.store_to(Self::parse_memory_location(&self.registers, params[1].trim()),self.registers[params[0]], 2),
            "sb" => self.memory.store_to(Self::parse_memory_location(&self.registers, params[1].trim()),self.registers[params[0]], 1),
            "inc" =>self.registers[params[0]] += 1,
            "dec" =>self.registers[params[0]] -= 1,
            "add" | "addw" => {
                let a =self.registers[params[1].trim()];
                let b =self.registers[params[2].trim()];
               self.registers[params[0]] = a + b;
            },
            "addi" | "addiw" => {
                let a =self.registers[params[1].trim()];
                let b = params[2].trim().parse::<i64>().unwrap();
               self.registers[params[0]] = a + b;
            },
            "sub" | "subw" => {
                let a =self.registers[params[1].trim()];
                let b =self.registers[params[2].trim()];
                let result = a - b;
               self.registers[params[0]] = result;
            },
            "mul" | "mulw" => {
                let a =self.registers[params[1].trim()];
                let b =self.registers[params[2].trim()];
               self.registers[params[0]] = a * b;
            },
            "divw" => {
                let a =self.registers[params[1].trim()];
                let b =self.registers[params[2].trim()];
               self.registers[params[0]] = a / b;
            },
            "negw" => {
                let a =self.registers[params[1].trim()];
               self.registers[params[0]] = -a; //subw rd, x0, rs
            },
            "nop" => (),
            "j" => {
                let jump_pos;
                if self.registers.has_register(params[0]) {
                    jump_pos =self.registers[params[0].trim()];
                }
                else {
                    jump_pos = params[0].trim().parse().expect("Expected number!");
                }
               self.registers["eip"] = jump_pos - 1;
            },
            "jr" => {
                let jump_pos =self.registers[params[0].trim()];
               self.registers["eip"] = jump_pos - 1;
            }
            "jal" => {
                let jump_pos: i64 = params[params.len()-1].trim().parse().expect("Expected address!");
               self.registers[params[0]] =self.registers["eip"] + 1;
               self.registers["eip"] = jump_pos - 1;
            },
            "bne" => {
                let a =self.registers[params[0].trim()];
                let b =self.registers[params[1].trim()];
                if a != b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "beq" => {
                let a =self.registers[params[0].trim()];
                let b =self.registers[params[1].trim()];
                if a == b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "ble" => {
                let a =self.registers[params[0].trim()];
                let b =self.registers[params[1].trim()];
                if a <= b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "blt" => {
                let a =self.registers[params[0].trim()];
                let b =self.registers[params[1].trim()];
                if a < b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "bnez" => {
                let a =self.registers[params[0].trim()];
                if a != 0 {
                    let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "bge" => {
                let a =self.registers[params[0].trim()];
                let b =self.registers[params[1].trim()];
                if a >= b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "bgtz" => {
                let a =self.registers[params[0].trim()];
                if a > 0 {
                    let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "bgez" => {
                let a =self.registers[params[0].trim()];
                if a >= 0 {
                    let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "bgtu" => {
                let a =self.registers[params[0].trim()] as u64;
                let b =self.registers[params[1].trim()] as u64;
                if a > b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "bltu" => {
                let a =self.registers[params[0].trim()] as u64;
                let b =self.registers[params[1].trim()] as u64;
                if a < b {
                    let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                   self.registers["eip"] = jump_pos - 1;
                }
            },
            "ret" => {
               self.registers["eip"] =self.registers["ra"] - 1;
            },
            "slli" | "slliw" => {
                let a =self.registers[params[1].trim()] as u64;
                let b = Self::parse_immediate(params[2].trim()) & 0b11111;
               self.registers[params[0]] = (a << b) as i64;
            },
            "srli" | "slriw" => {
                let a =self.registers[params[1].trim()] as u64;
                let b = Self::parse_immediate(params[2].trim()) & 0b11111;
               self.registers[params[0]] = a.checked_shr(b as u32).unwrap_or(0) as i64;
            },
            "sraiw" => {
                let a =self.registers[params[1].trim()];
                let b = Self::parse_immediate(params[2].trim()) & 0b11111;
               self.registers[params[0]] = a.checked_shr(b as u32).unwrap_or(0);
            },
            "or" => {
                let a =self.registers[params[1].trim()];
                let b =self.registers[params[2].trim()];
               self.registers[params[0]] = a | b;
            },
            "ori" => {
                let a =self.registers[params[1].trim()];
                let b = Self::parse_immediate(params[2].trim());
               self.registers[params[0]] = a | b;
            },
            "and" => {
                let a = self.registers[params[1].trim()];
                let b = self.registers[params[2].trim()];
                self.registers[params[0]] = a & b;
            },
            "andi" => {
                let a = self.registers[params[1].trim()];
                let b = Self::parse_immediate(params[2].trim());
                self.registers[params[0]] = a & b;
            },
            "remw" => {
                let a = self.registers[params[1].trim()];
                let b = self.registers[params[2].trim()];
                self.registers[params[0]] = a % b;
            },
            "sext.w" => {
                let a = self.registers[params[1].trim()];
                self.registers[params[0]] = a;
            },
            "ecall" => {
                let syscall_nr = self.registers["a7"];
                match syscall_nr {
                    3 => {
                        let fd = self.registers["a0"];
                        let buf = self.registers["a1"];
                        let count = self.registers["a2"];
                        if self.verbose {
                            print!("\x1b[34m");
                            print!("syscall: read(fd = {}, *buf = {:#x}, count={})", fd, buf, count);
                            println!("\x1b[0m");
                        }
                        
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        let max = min(count as usize, input.len());
                        for (i, c) in input.chars().enumerate() {
                            if i >= max {
                                break;
                            }
                            self.memory[(buf as usize) + i] = c as u8;
                        }
                        self.registers["a0"] = max as i64;
                    }
                    4 => {
                        let fd = self.registers["a0"];
                        let buf = self.registers["a1"];
                        let count = self.registers["a2"];
                        if self.verbose {
                            print!("\x1b[34m");
                            print!("syscall: write(fd = {}, *buf = {:#x}, count = {})", fd, buf, count);
                            println!("\x1b[0m");
                        }
    
                        for i in buf..buf+count {
                            let c = self.memory[i as usize] as char;
                            print!("{}", c);
                        }
                        io::stdout().flush().unwrap();
                    },
                    45 => {
                        let addr = self.registers["a0"] as usize;
                        if self.verbose {
                            print!("\x1b[34m");
                            print!("syscall: brk(*addr = {:#x})", addr);
                            println!("\x1b[0m");
                        }
                        self.memory.program_break = addr as usize;
                        self.memory.heap_memory.resize(addr, 0);
                    },
                    _ => panic!("Syscall {} is not supported", syscall_nr)
                }
            }
            _ => panic!("Instruction \"{}\" does not exist!", instruction_name)
        }
    
        self.registers["eip"] += 1;
    }
    
    fn parse_immediate(str: &str) -> i64 {
        if str.starts_with("0x") {
            return Self::parse_hex_string(str);
        }
        str.parse::<i64>().expect("Expected a numeric value!")
    }
    
    fn parse_hex_string(str: &str) -> i64 {
        let str = &str[2..];
        let mut total = 0;
        let mut map: HashMap<char, i64> = HashMap::new();
        map.insert('0', 0);
        map.insert('1', 1);
        map.insert('2', 2);
        map.insert('3', 3);
        map.insert('4', 4);
        map.insert('5', 5);
        map.insert('6', 6);
        map.insert('7', 7);
        map.insert('8', 8);
        map.insert('9', 9);
        map.insert('a', 10);
        map.insert('b', 11);
        map.insert('c', 12);
        map.insert('d', 13);
        map.insert('e', 14);
        map.insert('f', 15);
        for (i, c) in str.char_indices() {
            total |= map[&c] << (str.len()-i-1)*4;
        }
        total
    }
    
    // Take a String and parse it into a index for a block of memory.
    fn parse_memory_location(registers: &Registers, str: &str) -> i64 {
        /*if !str.starts_with('[') || !str.ends_with(']') {
            panic!("Incorrectly formatted address mode");
        }
        let str = &str[1..str.len()-1].trim();
        if variables.contains_key(str) {
            return variables[str];
        }
        let split = str.split_once("+").unwrap();
        let reg = split.0.trim();
        let reg_value = variables[reg];
        let offset = split.1.trim().parse::<i32>().unwrap();
        reg_value + offset*/
        if !str.ends_with(')') {
            panic!("Incorrectly formatted address mode");
        }
        let split_string:Vec<&str> = str.split('(').collect();
        let reg = &split_string[1][..split_string[1].len()-1];
        assert!(registers[reg] >= 0, "{} < 0", reg);
        split_string[0].parse::<i64>().unwrap() + registers[reg]
    }
}
