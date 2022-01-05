use core::panic;
use std::collections::HashMap;
use std::io::{self, Write};
use std::fs;

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

fn evaluate<'a, 'b>(instruction: &'a str, variables: &'a mut HashMap<&'b str, i32>) {
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
        "load" => *variables.get_mut(params[0]).unwrap() = params[1].trim().parse().expect("Expected number!"),
        "inc" => *variables.get_mut(params[0]).unwrap() += 1,
        "dec" => *variables.get_mut(params[0]).unwrap() -= 1,
        "add" => {
            let a = *variables.get_mut(params[0].trim()).unwrap();
            let b = *variables.get_mut(params[1].trim()).unwrap();
            *variables.get_mut(params[0]).unwrap() = a + b;
        }
        "cmp" => {
            let a = *variables.get_mut(params[0]).unwrap();
            let b = *variables.get_mut(params[1]).unwrap();
            *variables.get_mut("ZF").unwrap() = (a == b) as i32;
            *variables.get_mut("SF").unwrap() = (a - b < 0) as i32;
        },
        "nop" => (),
        "jmp" => {
            let jump_pos: i32 = params[0].trim().parse().expect("Expected number!");
            *variables.get_mut("eip").unwrap() = jump_pos - 1;
        },
        "jnz" => {
            if *variables.get_mut("ZF").unwrap() == 0 {
                let jump_pos: i32 = params[0].trim().parse().expect("Expected number!");
                *variables.get_mut("eip").unwrap() = jump_pos - 1;
            }
        },
        _ => panic!("Instruction does not exist!")
    }

    *variables.get_mut("eip").unwrap() += 1;
}

fn random_data() -> i32 {
    let ptr = Box::into_raw(Box::new(123));
    return ptr as i32;
}

fn main() {
    let content = fs::read_to_string("test.asm")
        .expect("Could not read file!");

    let lines: Vec<&str> = content.split("\n").collect();
    println!("Total amount of lines: {}", lines.len());
    let digit_count = (lines.len() -1).to_string().len();
    for (i, line) in lines.iter().enumerate() {
        println!("{:width$}|{}", i, line, width=digit_count);
    }

    let mut variables: HashMap<&str, i32> = HashMap::new();
    variables.insert("eax", random_data());
    variables.insert("ebx", random_data());
    variables.insert("ecx", random_data());
    variables.insert("edx", random_data());
    variables.insert("ZF", random_data());
    variables.insert("SF", random_data());
    variables.insert("eip", 0);
    
    let mut eip = variables["eip"]  as usize;
    while eip < lines.len() {
        let ins = lines[eip];
        println!("{}", ins);
        evaluate(ins, &mut variables);
        eip = variables["eip"]  as usize;
    }

    loop {
        let ins = &promt("$ ")[..];
        if variables.contains_key(ins) {
            println!("{}", variables[ins]);
            continue;
        }
        match ins {
            "exit" => break,
            _ => evaluate(ins, &mut &mut variables)
        }
    }
}
