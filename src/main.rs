extern crate unescape;

use core::panic;
use std::collections::HashMap;
use std::io::{self, Write};
use std::{fs, process};
use std::time::Instant;
use std::env;
use std::ops::{Index, IndexMut};
use std::ops::Range;
use std::cmp;
use std::cmp::min;
use crate::fs::File;

fn promt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read line!");
    if !name.contains('\n') {
        println!();
    }
    String::from(name.trim())
}

fn evaluate<'a, 'b>(instruction: &'a str, variables: &'a mut HashMap<&'b str, i64>, memory: &mut Memory, verbose: bool) {
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
        "mv" => *variables.get_mut(params[0]).unwrap() = variables[params[1].trim()],
        "ld" => *variables.get_mut(params[0]).unwrap() = memory.load_from(parse_memory_location(variables, params[1].trim())),
        //"lw" => *variables.get_mut(params[0]).unwrap() = (memory.load_from(parse_memory_location(variables, params[1].trim())) as i32) as i64,
        "lw" => {
            let address = parse_memory_location(variables, params[1].trim()) as usize;
            let mut value: [u8; 4] = Default::default();
            value.copy_from_slice(&memory[address..address + 4]);
            *variables.get_mut(params[0]).unwrap() = i32::from_le_bytes(value) as i64
        },
        "lwu" => *variables.get_mut(params[0]).unwrap() = (memory.load_from(parse_memory_location(variables, params[1].trim())) as u32) as i64,
        "lbu" => {
            let address = parse_memory_location(variables, params[1].trim()) as usize;
            *variables.get_mut(params[0]).unwrap() = (memory[address] as i64) & 0xff; //Remove sign extension
        },
        "lb" => {
            let address = parse_memory_location(variables, params[1].trim()) as usize;
            *variables.get_mut(params[0]).unwrap() = memory[address] as i64; //Sign extend to i64
        },
        "lhu" => {
            let address = parse_memory_location(variables, params[1].trim()) as usize;
            *variables.get_mut(params[0]).unwrap() = ((memory[address] as u64 | (memory[address+1] as u64) << 8) as i64) & 0xffff; //Remove sign extension
        },
        "li" | "lla" => *variables.get_mut(params[0]).unwrap() = parse_immediate(params[1].trim()),
        "sd" => memory.store_to(parse_memory_location(variables, params[1].trim()), variables[params[0]], 8),
        "sw" => memory.store_to(parse_memory_location(variables, params[1].trim()), variables[params[0]], 4),
        "sh" => memory.store_to(parse_memory_location(variables, params[1].trim()), variables[params[0]], 2),
        "sb" => memory.store_to(parse_memory_location(variables, params[1].trim()), variables[params[0]], 1),
        "inc" => *variables.get_mut(params[0]).unwrap() += 1,
        "dec" => *variables.get_mut(params[0]).unwrap() -= 1,
        "add" | "addw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "addi" | "addiw" => {
            let a = variables[params[1].trim()];
            let b = params[2].trim().parse::<i64>().unwrap();
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "sub" | "subw" => {
            let a = *variables.get_mut(params[1].trim()).unwrap();
            let b = *variables.get_mut(params[2].trim()).unwrap();
            let result = a - b;
            *variables.get_mut(params[0]).unwrap() = result;
        },
        "mul" | "mulw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a * b;
        },
        "divw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a / b;
        },
        "negw" => {
            let a = variables[params[1].trim()];
            *variables.get_mut(params[0]).unwrap() = -a; //subw rd, x0, rs
        },
        "nop" => (),
        "j" => {
            let jump_pos;
            if variables.contains_key(params[0]) {
                jump_pos = *variables.get_mut(params[0].trim()).unwrap();
            }
            else {
                jump_pos = params[0].trim().parse().expect("Expected number!");
            }
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        },
        "jr" => {
            let jump_pos = *variables.get_mut(params[0].trim()).unwrap();
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        }
        "jal" => {
            let jump_pos: i64 = params[params.len()-1].trim().parse().expect("Expected address!");
            *variables.get_mut(params[0]).unwrap() = variables["eip"] + 1;
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        },
        "bne" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a != b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "beq" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a == b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "ble" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a <= b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "blt" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a < b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bnez" => {
            let a = variables[params[0].trim()];
            if a != 0 {
                let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bge" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a >= b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bgtz" => {
            let a = variables[params[0].trim()];
            if a > 0 {
                let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bgez" => {
            let a = variables[params[0].trim()];
            if a >= 0 {
                let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bgtu" => {
            let a = variables[params[0].trim()] as u64;
            let b = variables[params[1].trim()] as u64;
            if a > b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bltu" => {
            let a = variables[params[0].trim()] as u64;
            let b = variables[params[1].trim()] as u64;
            if a < b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "ret" => {
            *variables.get_mut("eip").unwrap() = variables["ra"] - 1;
        },
        "slli" | "slliw" => {
            let a = variables[params[1].trim()] as u64;
            let b = parse_immediate(params[2].trim()) & 0b11111;
            *variables.get_mut(params[0]).unwrap() = (a << b) as i64;
        },
        "srli" | "slriw" => {
            let a = variables[params[1].trim()] as u64;
            let b = parse_immediate(params[2].trim()) & 0b11111;
            *variables.get_mut(params[0]).unwrap() = a.checked_shr(b as u32).unwrap_or(0) as i64;
        },
        "sraiw" => {
            let a = variables[params[1].trim()];
            let b = parse_immediate(params[2].trim()) & 0b11111;
            *variables.get_mut(params[0]).unwrap() = a.checked_shr(b as u32).unwrap_or(0);
        },
        "or" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a | b;
        },
        "ori" => {
            let a = variables[params[1].trim()];
            let b = parse_immediate(params[2].trim());
            *variables.get_mut(params[0]).unwrap() = a | b;
        },
        "and" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a & b;
        },
        "andi" => {
            let a = variables[params[1].trim()];
            let b = parse_immediate(params[2].trim());
            *variables.get_mut(params[0]).unwrap() = a & b;
        },
        "remw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a % b;
        },
        "sext.w" => {
            let a = variables[params[1].trim()];
            *variables.get_mut(params[0]).unwrap() = a;
        },
        "ecall" => {
            let syscall_nr = variables["a7"];
            match syscall_nr {
                3 => {
                    let fd = variables["a0"];
                    let buf = variables["a1"];
                    let count = variables["a2"];
                    if verbose {
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
                        memory[(buf as usize) + i] = c as u8;
                    }
                    *variables.get_mut("a0").unwrap() = max as i64;
                }
                4 => {
                    let fd = variables["a0"];
                    let buf = variables["a1"];
                    let count = variables["a2"];
                    if verbose {
                        print!("\x1b[34m");
                        print!("syscall: write(fd = {}, *buf = {:#x}, count = {})", fd, buf, count);
                        println!("\x1b[0m");
                    }

                    for i in buf..buf+count {
                        let c = memory[i as usize] as char;
                        print!("{}", c);
                    }
                    io::stdout().flush().unwrap();
                },
                45 => {
                    let addr = variables["a0"] as usize;
                    if verbose {
                        print!("\x1b[34m");
                        print!("syscall: brk(*addr = {:#x})", addr);
                        println!("\x1b[0m");
                    }
                    memory.program_break = addr as usize;
                    memory.heap_memory.resize(addr, 0);
                },
                _ => panic!("Syscall {} is not supported", syscall_nr)
            }
        }
        _ => panic!("Instruction \"{}\" does not exist!", instruction_name)
    }

    *variables.get_mut("eip").unwrap() += 1;
}

struct Memory {
    stack_memory: Vec<u8>,
    program_break: usize,
    heap_memory: Vec<u8>,
    virtual_memory_size: usize,
    verbose: bool
}

impl Memory {
    fn new(verbose: bool) -> Self {
        Memory {
            stack_memory: vec![0; 2048],
            program_break: 0,
            heap_memory:  Vec::new(),
            virtual_memory_size: 4096,
            verbose
        }
    }

    fn load_from(&self, address: i64) -> i64 {
        let address = address as usize;
        let mut value: [u8; 8] = Default::default();
        value.copy_from_slice(&self[address..address + 8]);
        i64::from_le_bytes(value)
    }

    fn store_to(&mut self, address: i64, value: i64, byte_count: usize) {
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

fn parse_immediate(str: &str) -> i64 {
    if str.starts_with("0x") {
        return parse_hex_string(str);
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
fn parse_memory_location<'a>(variables: &HashMap<&'a str, i64>, str: &'a str) -> i64 {
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
    assert!(variables[reg] >= 0, "{} < 0", reg);
    split_string[0].parse::<i64>().unwrap() + variables[reg]
}

fn print_stack(variables: &HashMap<&str, i64>, memory: &Memory) {
    let stack_offset = (memory.virtual_memory_size - memory.stack_memory.len()) as i64;
    for i in (((variables["sp"] - stack_offset)/8))..(memory.stack_memory.len()/8) as i64 {
        let address = (i*8 + stack_offset) as usize;
        print!("{:#04x} {:020}", i*8 + stack_offset, memory.load_from(i*8 + stack_offset));
        print!(" ");
        for j in 0..8 {
            print!("{:02x} ", memory[(i*8 + stack_offset + j) as usize]);
        }

        let char_ar = [memory[address] as char, memory[address + 1] as char, memory[address + 2] as char, memory[address + 3] as char, memory[address + 4] as char, memory[address + 5] as char, memory[address + 6] as char, memory[address + 7] as char];
        print!(" ");
        for char in char_ar {
            if char == '\n' {
                print!("\\n");
            } else {
                print!("{}", char);
            }
        }
        println!();
    }
}

fn random_data() -> i64 {
    let ptr = Box::into_raw(Box::new(123));
    return ptr as i64;
}

fn compile(content: &str, memory: &mut Memory, verbose: bool, program: &mut Vec<String>, mut data_segment_size: usize) -> (i64, usize) {
    let start_program_length = program.len();
    let mut jump_tag_map: HashMap<String, usize> = HashMap::new();
    let mut last_jump_label: String = Default::default();
    let lines: Vec<&str> = content.split("\n").collect();
    if verbose {
        println!("Total amount of lines: {}", lines.len());
    }
    let digit_count = (lines.len() -1).to_string().len();
    let mut offset = 0;
    for (i, line) in lines.iter().enumerate() {
        if verbose {
            println!("{:width$}│{}", i, line, width=digit_count);
        }
        
        let line = line.trim().replace('\t', " ");
        let line = String::from(line[..line.find('#').unwrap_or_else(|| line.len())].trim());

        if line.ends_with(":") {
            last_jump_label = line[..line.len()-1].to_string();
            jump_tag_map.insert(last_jump_label[..].to_string(), i - offset + start_program_length);
            offset += 1; // We are removing the line with the jump tag.
        }
        else if line.starts_with('.') {
            offset += 1;
            if line.starts_with(".string") {
                let str = line.split_once(' ').unwrap().1.trim();
                let str = unescape::unescape(&str[1..str.len()-1]).unwrap();
                
                data_segment_size += str.len() + 1;
                let size = memory.stack_memory.len();
                for (i, c) in str.char_indices() {
                    memory.stack_memory[size - data_segment_size + i] = c as u8;
                }
                memory.stack_memory[size - data_segment_size + str.len()] = '\0' as u8;

                jump_tag_map.insert(last_jump_label[..].to_string(), memory.virtual_memory_size - data_segment_size);
            } else if line.starts_with(".zero") {
                let size = line.split_once(' ').unwrap().1.trim().parse::<usize>().unwrap();

                data_segment_size += size;
                jump_tag_map.insert(last_jump_label[..].to_string(), memory.virtual_memory_size - data_segment_size);
            }
        }
        else if line == "" || line.starts_with('#') {
            offset += 1;
        }
        else {
            program.push(line);
        }
    }

    for i in 0..program.len() {
        let line = &program[i];
        let (instruction_name, params) = line.split_once(" ").unwrap_or_else(|| (line, ""));
        let (mut instruction_name, mut params) = (instruction_name, String::from(params));
        if line.starts_with("j") || line.starts_with("call") || line.starts_with('b') || line.starts_with("lla") {
            if instruction_name == "call" {
                instruction_name = "jal";
                params = format!("ra, {}", params);
            }
            let params: Vec<&str> = params.split(',').collect();
            let label = params[params.len()-1].trim();
            let label = label.strip_suffix("@plt").unwrap_or(label);
            if jump_tag_map.contains_key(label) {
                if label == "brk" {
                    println!("Mapping label \"{}\" to {}", label, jump_tag_map[label]);
                }
            
                let mut ins = instruction_name.to_owned() + " ";
                for i in 0..params.len()-1 {
                    ins.push_str(params[i]);
                    ins.push_str(",");
                }
                ins.push_str(&jump_tag_map[label].to_string()[..]);
                program[i] = ins;
            }
            else if instruction_name != "jr" {
                //panic!("Jump label \"{}\" not found!", label);
                //println!("WARNING: Jump label \"{}\" not found!", label);
            }
            /*else if variables.contains_key(param) {
                program[i] = format!("{} {}", instruction_name, jump_tag_map[param]).to_string();
            }
            else {
                let jump_location = param.parse::<i32>();
                match jump_location {
                    Ok(val) => {
                        if val < 0 || val >= program.len() as i32 {
                            panic!("Parameter \"{}\" is not a valid jump destination!", param)
                        }
                    },
                    Err(_) => panic!("Label \"{}\" is not a valid jump destination!", param)
                }
            }*/
        }
    }
    if jump_tag_map.contains_key("main") {
        return (jump_tag_map["main"] as i64, data_segment_size)
    }
    (0, data_segment_size)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut debug = false;
    let mut verbose = false;
    if args.len() < 2 {
        eprintln!("Expected at least one argument!");
        process::exit(1);
    }
    
    if args.len() >= 3 {
        match &args[2][..] {
            "debug" => {
                debug = true;
                verbose = true;
            },
            "verbose" => verbose = true,
            _ => {
                eprintln!("Argument \"{}\" is not a valid option", args[2]);
                process::exit(1);
            }
        }
    }

    //let content = &fs::read_to_string(&args[1][..])
    //    .expect("Could not read file!")[..];
    let content = &fs::read_to_string("ctest/test2.s")
        .expect("Could not read file!")[..];
    let content2 = &fs::read_to_string("ctest/syscalls.s")
        .expect("Could not read file!")[..];
    let mut memory = Memory::new(verbose);
    let mut program = Vec::new();
    let (entry_point, data_segment_size) = compile(content, &mut memory, verbose, &mut program, 0);
    let (_, data_segment_size) = compile(content2, &mut memory, verbose, &mut program, data_segment_size);
    let digit_count = (program.len() -1).to_string().len();
    
    println!("Compilation finished");
    
    // Write compiled program to file
    let mut output = File::create("output.s").unwrap();
    writeln!(output, "header:").unwrap();
    writeln!(output, "entry_point = {}", entry_point).unwrap();
    writeln!(output, "data_segment_size = {}", data_segment_size).unwrap();
    writeln!(output, "code:").unwrap();
    for line in &program {
        writeln!(output, "{}", line).unwrap();
    }
    writeln!(output, "stack:").unwrap();
    for i in 0..data_segment_size {
        //write!(output, "{} ", memory[memory.virtual_memory_size - data_segment_size + i] as char).unwrap();
        write!(output, "{:#04x} ", memory[memory.virtual_memory_size - data_segment_size + i]).unwrap();
    }
    
    println!("Write finished");

    let mut variables: HashMap<&str, i64> = HashMap::new();
    variables.insert("zero", 0);
    variables.insert("ra", random_data());
    variables.insert("sp", (memory.virtual_memory_size - data_segment_size) as i64);
    variables.insert("eip", entry_point);
    
    variables.insert("t0", random_data());
    variables.insert("t1", random_data());
    variables.insert("t2", random_data());

    variables.insert("s0", random_data());
    variables.insert("s1", random_data());
    
    variables.insert("a0", random_data());
    variables.insert("a1", random_data());
    variables.insert("a2", random_data());
    variables.insert("a3", random_data());
    variables.insert("a4", random_data());
    variables.insert("a5", random_data());
    variables.insert("a6", random_data());
    variables.insert("a7", random_data());

    variables.insert("s2", random_data());
    variables.insert("s3", random_data());
    variables.insert("s4", random_data());
    variables.insert("s5", random_data());
    variables.insert("s6", random_data());
    variables.insert("s7", random_data());
    variables.insert("s8", random_data());
    variables.insert("s9", random_data());
    variables.insert("s10", random_data());
    variables.insert("s11", random_data());

    variables.insert("t3", random_data());
    variables.insert("t4", random_data());
    variables.insert("t5", random_data());
    variables.insert("t6", random_data());

    let start = Instant::now();
    let mut ins_executed = 0;
    let mut eip = variables["eip"]  as usize;
    while eip < program.len() {
        let ins = &program[eip][..];
        if verbose {
            println!("{}", ins);
        }
        evaluate(ins, &mut variables, &mut memory, verbose);
        eip = variables["eip"]  as usize;
        ins_executed += 1;

        if debug {
            for i in 0..program.len() {
                if i == eip {
                    print!("-> ") 
                }
                else {
                    print!("   ");
                }   
                //│ != |
                println!("{:width$}│{}", i, program[i], width=digit_count);
            }
            let mut input = promt("$ ");
            while input != "stop" && input != "continue" && input != "" {
                if variables.contains_key(&input[..]) {
                    println!("{}", variables[&input[..]]);
                }
                else if input == "stack" {
                    print_stack(&variables, &memory);
                }
                else {
                    println!("Invalid command");
                }
                input = promt("$ ");
            }
            if input == "stop" {
                break;
            }
        }
    }
    let duration = start.elapsed();
    println!("Total time elapsed: {}ms, {}ns | Executed {} instructions", duration.as_millis(), duration.as_nanos(), ins_executed);

    loop {
        let ins = &promt("$ ")[..];
        if variables.contains_key(ins) {
            println!("{}", variables[ins]);
            continue;
        }
        match ins {
            "stack" => {
                print_stack(&variables, &memory);
            }
            "exit" => break,
            _ => evaluate(ins, &mut variables, &mut memory, verbose)
        }
    }
}
