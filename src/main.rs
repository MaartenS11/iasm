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

fn evaluate<'a, 'b>(instruction: &'a str, variables: &'a mut HashMap<&'b str, i32>, memory: &mut [i32; 16]) {
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
        "mov" => *variables.get_mut(params[0]).unwrap() = variables[params[1].trim()],
        "ld" => *variables.get_mut(params[0]).unwrap() = memory[parse_memory_location(variables, params[1].trim()) as usize],
        "li" => *variables.get_mut(params[0]).unwrap() = params[1].trim().parse().expect("Expected number!"),
        "st" => memory[parse_memory_location(variables, params[1].trim()) as usize] = variables[params[0]],
        "inc" => *variables.get_mut(params[0]).unwrap() += 1,
        "dec" => *variables.get_mut(params[0]).unwrap() -= 1,
        "add" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = *variables.get_mut(params[1].trim()).unwrap();
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "addi" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = params[1].trim().parse::<i32>().unwrap();
            *variables.get_mut(params[0]).unwrap() = a + b;
        },
        "sub" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = *variables.get_mut(params[1].trim()).unwrap();
            let result = a - b;
            *variables.get_mut(params[0]).unwrap() = result;
            *variables.get_mut("ZF").unwrap() = (result == 0) as i32;
        },
        "mul" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = *variables.get_mut(params[1].trim()).unwrap();
            *variables.get_mut(params[0]).unwrap() = a * b;
        },
        "cmp" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = *variables.get_mut(params[1].trim()).unwrap();
            *variables.get_mut("ZF").unwrap() = (a == b) as i32;
            *variables.get_mut("SF").unwrap() = (a - b < 0) as i32;
        },
        "nop" => (),
        "jmp" => {
            let jump_pos;
            if variables.contains_key(params[0]) {
                jump_pos = *variables.get_mut(params[0].trim()).unwrap();
            }
            else {
                jump_pos = params[0].trim().parse().expect("Expected number!");
            }
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        },
        "jne" | "jnz" => {
            if *variables.get_mut("ZF").unwrap() == 0 {
                let jump_pos: i32 = params[0].trim().parse().expect("Expected number!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        "push" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            stack_push(variables, memory, a);
        },
        "pop" => {
            *variables.get_mut(params[0].trim()).unwrap() = stack_pop(variables, memory);
        },
        "call" => {
            let jump_pos: i32 = params[0].trim().parse().expect("Expected number!");
            let eip = *variables.get_mut("eip").unwrap();
            stack_push(variables, memory, eip + 1);
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        },
        "ret" => {
            let jump_pos = stack_pop(variables, memory);
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        }
        _ => panic!("Instruction does not exist!")
    }

    *variables.get_mut("eip").unwrap() += 1;
}

fn stack_push(variables: &mut HashMap<&str, i32>, memory: &mut [i32; 16], value: i32) {
    *variables.get_mut("esp").unwrap() -= 4;
    let esp = *variables.get_mut("esp").unwrap() as usize;
    memory[esp/4] = value;
}

fn stack_pop(variables: &mut HashMap<&str, i32>, memory: &mut [i32; 16]) -> i32 {
    let esp = *variables.get_mut("esp").unwrap() as usize;
    let value = memory[esp/4];
    *variables.get_mut("esp").unwrap() += 4;
    value
}

// Take a String and parse it into a index for a block of memory.
fn parse_memory_location<'a>(variables: &HashMap<&'a str, i32>, str: &'a str) -> i32 {
    if !str.starts_with('[') || !str.ends_with(']') {
        panic!("Incorrectly formatted address mode");
    }
    let str = &str[1..str.len()-1].trim();
    if variables.contains_key(str) {
        return variables[str]/4;
    }
    let split = str.split_once("+").unwrap();
    let reg = split.0.trim();
    let reg_value = variables[reg];
    let offset = split.1.trim().parse::<i32>().unwrap();
    (reg_value + offset)/4
}

fn print_stack(variables: &HashMap<&str, i32>, memory: &[i32; 16]) {
    for i in ((variables["esp"]/4) as usize)..memory.len() {
        println!("{:#04x}│{}", i*4, memory[i]);
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
        
        let line = line.trim().to_string();
        let line = String::from(&line[..line.find(';').unwrap_or_else(|| line.len())]);

        if line.ends_with(":") {
            jump_tag_map.insert(line[..line.len()-1].to_string(), i - offset);
            offset += 1; // We are removing the line with the jump tag.
        }
        else if line == "" || line.starts_with(";") {
            offset += 1;
        }
        else {
            program.push(line);
        }
    }

    for i in 0..program.len() {
        let line = &program[i];
        if line.starts_with("j") || line.starts_with("call") {
            let (instruction_name, param) = line.split_once(" ").unwrap();
            if jump_tag_map.contains_key(param) {
                program[i] = format!("{} {}", instruction_name, jump_tag_map[param]).to_string();
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

    let mut memory: [i32; 16] = [0; 16];
    let mut variables: HashMap<&str, i32> = HashMap::new();
    variables.insert("eax", random_data());
    variables.insert("ebx", random_data());
    variables.insert("ecx", random_data());
    variables.insert("edx", random_data());
    variables.insert("esi", random_data());
    variables.insert("edi", random_data());
    variables.insert("esp", (memory.len() * 4) as i32);
    variables.insert("ebp", random_data());
    variables.insert("ZF", random_data());
    variables.insert("SF", random_data());
    variables.insert("eip", 0);

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
