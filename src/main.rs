use core::panic;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Iter;
use std::{fs, process};
use std::time::Instant;
use std::env;
use std::ops::{Index, IndexMut};
use std::ops::Range;

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

fn evaluate<'a, 'b>(instruction: &'a str, variables: &'a mut HashMap<&'b str, i32>, memory: &mut Memory) {
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
        "lw" | "ld" => *variables.get_mut(params[0]).unwrap() = memory.load_from_memory(parse_memory_location(variables, params[1].trim())),
        "lbu" => {
            let address = parse_memory_location(variables, params[1].trim());
            *variables.get_mut(params[0]).unwrap() = memory.load_from_memory(address) & 0xff;
        },
        "li" => *variables.get_mut(params[0]).unwrap() = parse_immediate(params[1].trim()),
        "sw" | "sd" => memory.store_to_memory(parse_memory_location(variables, params[1].trim()), variables[params[0]], 4),
        "sb" => memory.store_to_memory(parse_memory_location(variables, params[1].trim()), variables[params[0]], 1),
        "inc" => *variables.get_mut(params[0]).unwrap() += 1,
        "dec" => *variables.get_mut(params[0]).unwrap() -= 1,
        "add" | "addw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "addi" | "addiw" => {
            let a = variables[params[1].trim()];
            let b = params[2].trim().parse::<i32>().unwrap();
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "sub" | "subw" => {
            let a = *variables.get_mut(params[1].trim()).unwrap();
            let b = *variables.get_mut(params[2].trim()).unwrap();
            let result = a - b;
            *variables.get_mut(params[0]).unwrap() = result;
        },
        "mulw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a * b;
        },
        "divw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a / b;
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
            let jump_pos: i32 = params[params.len()-1].trim().parse().expect("Expected address!");
            *variables.get_mut(params[0]).unwrap() = variables["eip"] + 1;
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        },
        "bne" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a != b {
                let jump_pos: i32 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "beq" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a == b {
                let jump_pos: i32 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "ble" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a <= b {
                let jump_pos: i32 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "blt" => {
            let a = variables[params[0].trim()];
            let b = variables[params[1].trim()];
            if a < b {
                let jump_pos: i32 = params[2].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "bnez" => {
            let a = variables[params[0].trim()];
            if a != 0 {
                let jump_pos: i32 = params[1].trim().parse().expect("Expected address!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "ret" => {
            *variables.get_mut("eip").unwrap() = variables["ra"] - 1;
        },
        "slli" | "slliw" => {
            let a = variables[params[1].trim()];
            let b = parse_immediate(params[2].trim());
            *variables.get_mut(params[0]).unwrap() = a << b;
        },
        "srli" | "slriw" => {
            let a = variables[params[1].trim()];
            let b = parse_immediate(params[2].trim()) & 0b11111;
            //*variables.get_mut(params[0]).unwrap() = a >> b;
            *variables.get_mut(params[0]).unwrap() = a.checked_shr(b as u32).unwrap_or(0);
        },
        "or" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a | b;
        },
        "andi" => {
            let a = variables[params[1].trim()];
            let b = parse_immediate(params[2].trim());
            *variables.get_mut(params[0]).unwrap() = a & b;
        },
        "sext.w" => {
            let a = variables[params[1].trim()];
            *variables.get_mut(params[0]).unwrap() = a;
        },
        "ecall" => {
            let syscall_nr = variables["a7"];
            match syscall_nr {
                4 => {
                    let fd = variables["a0"];
                    let buf = variables["a1"];
                    let count = variables["a2"];
                    print!("\x1b[34m");
                    print!("syscall: write(fd = {}, *buf = {:#x}, count = {})", fd, buf, count);
                    println!("\x1b[0m");
                    for i in buf..buf+count {
                        let c = memory[i as usize] as char;
                        print!("{}", c);
                    }
                },
                45 => {
                    let addr = variables["a0"];
                    print!("\x1b[34m");
                    print!("syscall: brk(*addr = {:#x})", addr);
                    println!("\x1b[0m");
                    memory.program_break = addr as usize;
                },
                _ => panic!("Syscall {} is not supported", syscall_nr)
            }
        }
        _ => panic!("Instruction \"{}\" does not exist!", instruction_name)
    }

    *variables.get_mut("eip").unwrap() += 1;
}

struct Memory {
    stack_memory: [u8; 1024],
    program_break: usize
}

impl Memory {
    fn new() -> Self {
        Memory {
            stack_memory: [0; 1024],
            program_break: 0
        }
    }

    fn load_from_memory(&self, address: i32) -> i32 {
        let address = address as usize;
        let mut value: [u8; 4] = Default::default();
        value.copy_from_slice(&self[address..address + 4]);
        i32::from_le_bytes(value)
    }

    fn store_to_memory(&mut self, address: i32, value: i32, byte_count: usize) {
        let address = address as usize;
        let bytes = value.to_le_bytes();
        //println!("{} {} {} {}", bytes[0], bytes[1], bytes[2], bytes[3]);
        for i in 0..byte_count {
            let val = bytes[i];
            print!("\x1b[33m");
            print!("memory[{:#04x}] = {} '{}'", address + i, val, val as char);
            println!("\x1b[0m");
            self[address + i] = val;
        }
    }
}

impl Index<usize> for Memory {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.stack_memory[index]
    }
}

impl Index<Range<usize>> for Memory {
    type Output = [u8];
    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.stack_memory[range]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.stack_memory[index]
    }
}

fn parse_immediate(str: &str) -> i32 {
    if str.starts_with("0x") {
        return parse_hex_string(str);
    }
    str.parse::<i32>().expect("Expected a numeric value!")
}

fn parse_hex_string(str: &str) -> i32 {
    let str = &str[2..];
    let mut total = 0;
    let mut map: HashMap<char, i32> = HashMap::new();
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
fn parse_memory_location<'a>(variables: &HashMap<&'a str, i32>, str: &'a str) -> i32 {
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
    split_string[0].parse::<i32>().unwrap() + variables[reg]
}

fn print_stack(variables: &HashMap<&str, i32>, memory: &Memory) {
    for i in ((variables["sp"]/4))..(memory.stack_memory.len()/4) as i32 {
        println!("{:#04x}│{}", i*4, memory.load_from_memory(i*4));
    }
}

fn random_data() -> i32 {
    let ptr = Box::into_raw(Box::new(123));
    return ptr as i32;
}

fn compile(content: &str) -> (Vec<String>, i32) {
    let mut program: Vec<String> = Vec::new();
    let mut jump_tag_map: HashMap<String, usize> = HashMap::new();
    let lines: Vec<&str> = content.split("\n").collect();
    println!("Total amount of lines: {}", lines.len());
    let digit_count = (lines.len() -1).to_string().len();
    let mut offset = 0;
    for (i, line) in lines.iter().enumerate() {
        println!("{:width$}│{}", i, line, width=digit_count);
        
        let line = line.trim().replace('\t', " ");
        let line = String::from(&line[..line.find('#').unwrap_or_else(|| line.len())]);

        if line.ends_with(":") {
            jump_tag_map.insert(line[..line.len()-1].to_string(), i - offset);
            offset += 1; // We are removing the line with the jump tag.
        }
        else if line == "" || line.starts_with('#') || line.starts_with('.') {
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
        if line.starts_with("j") || line.starts_with("call") || line.starts_with('b') {
            if instruction_name == "call" {
                instruction_name = "jal";
                params = format!("ra, {}", params);
            }
            let params: Vec<&str> = params.split(',').collect();
            let label = params[params.len()-1].trim();
            if jump_tag_map.contains_key(label) {
                let mut ins = instruction_name.to_owned() + " ";
                for i in 0..params.len()-1 {
                    ins.push_str(params[i]);
                    ins.push_str(",");
                }
                ins.push_str(&jump_tag_map[label].to_string()[..]);
                program[i] = ins;
            }
            else if instruction_name != "jr" {
                panic!("Jump label \"{}\" not found!", label);
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
        return (program, jump_tag_map["main"] as i32)
    }
    (program, 0)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut debug = false;
    if args.len() < 2 {
        eprintln!("Expected at least one argument!");
        process::exit(1);
    }
    
    if args.len() >= 3 {
        match &args[2][..] {
            "debug" => debug = true,
            _ => {
                eprintln!("Argument \"{}\" is not a valid option", args[2]);
                process::exit(1);
            }
        }
    }

    let content = &fs::read_to_string(&args[1][..])
        .expect("Could not read file!")[..];
    let (program, entry_point) = compile(content);
    let digit_count = (program.len() -1).to_string().len();

    let mut memory = Memory::new();
    let mut variables: HashMap<&str, i32> = HashMap::new();
    variables.insert("zero", 0);
    variables.insert("ra", random_data());
    variables.insert("sp", memory.stack_memory.len() as i32);
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
        println!("{}", ins);
        evaluate(ins, &mut variables, &mut memory);
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
            _ => evaluate(ins, &mut variables, &mut memory)
        }
    }
}
