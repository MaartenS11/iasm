extern crate unescape;

use std::io::{self, Write};
use std::{fs, process};
use std::time::Instant;
use std::env;
use crate::fs::File;

use crate::memory::Memory;
use crate::compile::compile_files;
use crate::registers::Registers;
use crate::evaluator::Evaluator;

mod compile;
mod memory;
mod registers;
mod evaluator;

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).expect("Failed to read line!");
    if !name.contains('\n') {
        println!();
    }
    String::from(name.trim())
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

    let mut evaluator = Evaluator::new(verbose);
    let (program, entry_point, data_segment_size) = compile_files(files, &mut evaluator.memory, verbose);
    
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
        write!(output, "{:#04x} ", evaluator.memory[evaluator.memory.virtual_memory_size - data_segment_size + i]).unwrap();
    }
    
    println!("Write finished");
    
    let digit_count = (program.len() -1).to_string().len();

    evaluator.registers["sp"] = (evaluator.memory.virtual_memory_size - data_segment_size) as i64;
    evaluator.registers["eip"] = entry_point;

    let start = Instant::now();
    let mut ins_executed = 0;
    let mut eip = evaluator.registers["eip"]  as usize;
    while eip < program.len() {
        let ins = &program[eip][..];
        if verbose {
            println!("{}", ins);
        }
        evaluator.evaluate(ins).expect("Error");
        eip = evaluator.registers["eip"]  as usize;
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
            let mut input = prompt("$ ");
            while input != "stop" && input != "continue" && input != "" {
                if evaluator.registers.has_register(&input[..]) {
                    println!("{}", evaluator.registers[&input[..]]);
                }
                else if input == "stack" {
                    print_stack(&evaluator.registers, &evaluator.memory);
                }
                else {
                    println!("Invalid command");
                }
                input = prompt("$ ");
            }
            if input == "stop" {
                break;
            }
        }
    }
    let duration = start.elapsed();
    println!("Total time elapsed: {}ms, {}ns | Executed {} instructions", duration.as_millis(), duration.as_nanos(), ins_executed);

    loop {
        match &prompt("$ ")[..] {
            reg if evaluator.registers.has_register(reg) => {
                println!("{}", evaluator.registers[reg]);
            }
            "stack" => {
                print_stack(&evaluator.registers, &evaluator.memory);
            }
            "exit" => break,
            ins => {
                match evaluator.evaluate(ins) {
                    Ok(()) => {},
                    Err(err) => println!("\x1b[31mError: {err}\x1b[0m")
                }
            }
        }
    }
}
