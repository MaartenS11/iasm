extern crate unescape;

use core::panic;
use std::collections::HashMap;
use std::io::{self, Write};
use std::{fs, process};
use std::time::Instant;
use std::env;
use std::cmp::min;
use crate::fs::File;

use crate::memory::Memory;
use crate::compile::compile_files;
use crate::registers::Registers;

mod compile;
mod memory;
mod registers;

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

fn evaluate<'a, 'b>(instruction: &'a str, registers: &'a mut Registers, memory: &mut Memory, verbose: bool) {
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
        "mv" => registers[params[0]] = registers[params[1].trim()],
        "ld" => registers[params[0]] = memory.load_from(parse_memory_location(registers, params[1].trim())),
        //"lw" => variables[params[0]] = (memory.load_from(parse_memory_location(variables, params[1].trim())) as i32) as i64,
        "lw" => {
            let address = parse_memory_location(registers, params[1].trim()) as usize;
            let mut value: [u8; 4] = Default::default();
            value.copy_from_slice(&memory[address..address + 4]);
            registers[params[0]] = i32::from_le_bytes(value) as i64
        },
        "lwu" => registers[params[0]] = (memory.load_from(parse_memory_location(registers, params[1].trim())) as u32) as i64,
        "lbu" => {
            let address = parse_memory_location(registers, params[1].trim()) as usize;
            registers[params[0]] = (memory[address] as i64) & 0xff; //Remove sign extension
        },
        "lb" => {
            let address = parse_memory_location(registers, params[1].trim()) as usize;
            registers[params[0]] = memory[address] as i64; //Sign extend to i64
        },
        "lhu" => {
            let address = parse_memory_location(registers, params[1].trim()) as usize;
            registers[params[0]] = ((memory[address] as u64 | (memory[address+1] as u64) << 8) as i64) & 0xffff; //Remove sign extension
        },
        "li" | "lla" => registers[params[0]] = parse_immediate(params[1].trim()),
        "sd" => memory.store_to(parse_memory_location(registers, params[1].trim()), registers[params[0]], 8),
        "sw" => memory.store_to(parse_memory_location(registers, params[1].trim()), registers[params[0]], 4),
        "sh" => memory.store_to(parse_memory_location(registers, params[1].trim()), registers[params[0]], 2),
        "sb" => memory.store_to(parse_memory_location(registers, params[1].trim()), registers[params[0]], 1),
        "inc" => registers[params[0]] += 1,
        "dec" => registers[params[0]] -= 1,
        "add" | "addw" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            registers[params[0]] = a + b;
        },
        "addi" | "addiw" => {
            let a = registers[params[1].trim()];
            let b = params[2].trim().parse::<i64>().unwrap();
            registers[params[0]] = a + b;
        },
        "sub" | "subw" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            let result = a - b;
            registers[params[0]] = result;
        },
        "mul" | "mulw" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            registers[params[0]] = a * b;
        },
        "divw" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            registers[params[0]] = a / b;
        },
        "negw" => {
            let a = registers[params[1].trim()];
            registers[params[0]] = -a; //subw rd, x0, rs
        },
        "nop" => (),
        "j" => {
            let jump_pos;
            if registers.has_register(params[0]) {
                jump_pos = registers[params[0].trim()];
            }
            else {
                jump_pos = params[0].trim().parse().expect("Expected number!");
            }
            registers["eip"] = jump_pos - 1;
        },
        "jr" => {
            let jump_pos = registers[params[0].trim()];
            registers["eip"] = jump_pos - 1;
        }
        "jal" => {
            let jump_pos: i64 = params[params.len()-1].trim().parse().expect("Expected address!");
            registers[params[0]] = registers["eip"] + 1;
            registers["eip"] = jump_pos - 1;
        },
        "bne" => {
            let a = registers[params[0].trim()];
            let b = registers[params[1].trim()];
            if a != b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "beq" => {
            let a = registers[params[0].trim()];
            let b = registers[params[1].trim()];
            if a == b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "ble" => {
            let a = registers[params[0].trim()];
            let b = registers[params[1].trim()];
            if a <= b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "blt" => {
            let a = registers[params[0].trim()];
            let b = registers[params[1].trim()];
            if a < b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "bnez" => {
            let a = registers[params[0].trim()];
            if a != 0 {
                let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "bge" => {
            let a = registers[params[0].trim()];
            let b = registers[params[1].trim()];
            if a >= b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "bgtz" => {
            let a = registers[params[0].trim()];
            if a > 0 {
                let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "bgez" => {
            let a = registers[params[0].trim()];
            if a >= 0 {
                let jump_pos: i64 = params[1].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "bgtu" => {
            let a = registers[params[0].trim()] as u64;
            let b = registers[params[1].trim()] as u64;
            if a > b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "bltu" => {
            let a = registers[params[0].trim()] as u64;
            let b = registers[params[1].trim()] as u64;
            if a < b {
                let jump_pos: i64 = params[2].trim().parse().expect("Expected address!");
                registers["eip"] = jump_pos - 1;
            }
        },
        "ret" => {
            registers["eip"] = registers["ra"] - 1;
        },
        "slli" | "slliw" => {
            let a = registers[params[1].trim()] as u64;
            let b = parse_immediate(params[2].trim()) & 0b11111;
            registers[params[0]] = (a << b) as i64;
        },
        "srli" | "slriw" => {
            let a = registers[params[1].trim()] as u64;
            let b = parse_immediate(params[2].trim()) & 0b11111;
            registers[params[0]] = a.checked_shr(b as u32).unwrap_or(0) as i64;
        },
        "sraiw" => {
            let a = registers[params[1].trim()];
            let b = parse_immediate(params[2].trim()) & 0b11111;
            registers[params[0]] = a.checked_shr(b as u32).unwrap_or(0);
        },
        "or" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            registers[params[0]] = a | b;
        },
        "ori" => {
            let a = registers[params[1].trim()];
            let b = parse_immediate(params[2].trim());
            registers[params[0]] = a | b;
        },
        "and" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            registers[params[0]] = a & b;
        },
        "andi" => {
            let a = registers[params[1].trim()];
            let b = parse_immediate(params[2].trim());
            registers[params[0]] = a & b;
        },
        "remw" => {
            let a = registers[params[1].trim()];
            let b = registers[params[2].trim()];
            registers[params[0]] = a % b;
        },
        "sext.w" => {
            let a = registers[params[1].trim()];
            registers[params[0]] = a;
        },
        "ecall" => {
            let syscall_nr = registers["a7"];
            match syscall_nr {
                3 => {
                    let fd = registers["a0"];
                    let buf = registers["a1"];
                    let count = registers["a2"];
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
                    registers["a0"] = max as i64;
                }
                4 => {
                    let fd = registers["a0"];
                    let buf = registers["a1"];
                    let count = registers["a2"];
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
                    let addr = registers["a0"] as usize;
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

    registers["eip"] += 1;
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
fn parse_memory_location<'a>(registers: &Registers, str: &'a str) -> i64 {
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

fn print_stack(registers: &Registers, memory: &Memory) {
    let stack_offset = (memory.virtual_memory_size - memory.stack_memory.len()) as i64;
    for i in (((registers["sp"] - stack_offset)/8))..(memory.stack_memory.len()/8) as i64 {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut debug = false;
    let mut verbose = false;
    let mut files = &args[1..];
    if args.len() < 2 {
        eprintln!("Expected at least one argument!");
        process::exit(1);
    }
    
    if args.len() >= 3 {
        match &args[args.len()-1][..] {
            "debug" => {
                debug = true;
                verbose = true;
                files = &args[1..args.len()-1];
            },
            "verbose" => {
                verbose = true;
                files = &args[1..args.len()-1];
            },
            _ => {}
        }
    }

    let mut memory = Memory::new(verbose);
    let (program, entry_point, data_segment_size) = compile_files(files, &mut memory, verbose);
    
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
    
    let digit_count = (program.len() -1).to_string().len();

    
    let mut registers = Registers::new();
    registers["sp"] = (memory.virtual_memory_size - data_segment_size) as i64;
    registers["eip"] = entry_point;

    let start = Instant::now();
    let mut ins_executed = 0;
    let mut eip = registers["eip"]  as usize;
    while eip < program.len() {
        let ins = &program[eip][..];
        if verbose {
            println!("{}", ins);
        }
        evaluate(ins, &mut registers, &mut memory, verbose);
        eip = registers["eip"]  as usize;
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
                if registers.has_register(&input[..]) {
                    println!("{}", registers[&input[..]]);
                }
                else if input == "stack" {
                    print_stack(&registers, &memory);
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
        if registers.has_register(ins) {
            println!("{}", registers[ins]);
            continue;
        }
        match ins {
            "stack" => {
                print_stack(&registers, &memory);
            }
            "exit" => break,
            _ => evaluate(ins, &mut registers, &mut memory, verbose)
        }
    }
}
