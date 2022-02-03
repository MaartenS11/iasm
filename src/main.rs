use core::panic;
use std::collections::HashMap;
use std::io::{self, Write};
use std::{fs, process};
use std::time::Instant;
use std::env;

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

fn evaluate<'a, 'b>(instruction: &'a str, variables: &'a mut HashMap<&'b str, i32>, memory: &mut [u8; 64]) {
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
        "lw" | "ld" => *variables.get_mut(params[0]).unwrap() = load_from_memory(memory, parse_memory_location(variables, params[1].trim())),
        "li" => *variables.get_mut(params[0]).unwrap() = params[1].trim().parse().expect("Expected number!"),
        "sw" | "sd" => store_to_memory(memory, parse_memory_location(variables, params[1].trim()), variables[params[0]]),
        "inc" => *variables.get_mut(params[0]).unwrap() += 1,
        "dec" => *variables.get_mut(params[0]).unwrap() -= 1,
        "add" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "addi" => {
            let a = variables[params[1].trim()];
            let b = params[2].trim().parse::<i32>().unwrap();
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "sub" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = *variables.get_mut(params[1].trim()).unwrap();
            let result = a - b;
            *variables.get_mut(params[0]).unwrap() = result;
            *variables.get_mut("ZF").unwrap() = (result == 0) as i32;
        },
        "mulw" => {
            let a = variables[params[1].trim()];
            let b = variables[params[2].trim()];
            *variables.get_mut(params[0]).unwrap() = a * b;
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
        "ret" => {
            *variables.get_mut("eip").unwrap() = variables["ra"] - 1;
        }
        _ => panic!("Instruction \"{}\" does not exist!", instruction_name)
    }

    *variables.get_mut("eip").unwrap() += 1;
}

fn load_from_memory(memory: &[u8; 64], address: i32) -> i32 {
    let address = address as usize;
    let mut value: [u8; 4] = Default::default();
    value.copy_from_slice(&memory[address..address + 4]);
    i32::from_be_bytes(value)
}

fn store_to_memory(memory: &mut [u8; 64], address: i32, value: i32) {
    let address = address as usize;
    let bytes = value.to_be_bytes();
    for i in 0..4 {
        memory[address + i] = bytes[i];
    }
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
    split_string[0].parse::<i32>().unwrap() + variables[reg]
}

fn print_stack(variables: &HashMap<&str, i32>, memory: &[u8; 64]) {
    for i in ((variables["sp"]/4))..(memory.len()/4) as i32 {
        println!("{:#04x}│{}", i*4, load_from_memory(memory, i*4));
    }
}

fn random_data() -> i32 {
    let ptr = Box::into_raw(Box::new(123));
    return ptr as i32;
}

fn compile(content: &str) -> Vec<String> {
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
        else if line == "" || line.starts_with('#') {
            offset += 1;
        }
        else {
            program.push(line);
        }
    }

    for i in 0..program.len() {
        let line = &program[i];
        if line.starts_with("j") || line.starts_with("call") || line.starts_with('b') {
            let (instruction_name, params) = line.split_once(" ").unwrap();
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
            else {
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
    program
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
    let program = compile(content);
    let digit_count = (program.len() -1).to_string().len();

    let mut memory: [u8; 64] = [0; 64];
    let mut variables: HashMap<&str, i32> = HashMap::new();
    variables.insert("ra", random_data());
    variables.insert("sp", memory.len() as i32);
    variables.insert("ZF", random_data());
    variables.insert("SF", random_data());
    variables.insert("eip", 0);
    
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

    let start = Instant::now();
    let mut eip = variables["eip"]  as usize;
    while eip < program.len() {
        let ins = &program[eip][..];
        println!("{}", ins);
        evaluate(ins, &mut variables, &mut memory);
        eip = variables["eip"]  as usize;
        
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
    println!("Total time elapsed: {}ms, {}ns", duration.as_millis(), duration.as_nanos());

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
