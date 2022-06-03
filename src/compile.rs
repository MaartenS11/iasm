use std::collections::HashMap;
use std::fs;

use crate::memory::Memory;

fn compile(content: &str, memory: &mut Memory, verbose: bool, program: &mut Vec<String>, data_segment_size: &mut usize, jump_tag_map: &mut HashMap<String, usize>) -> i64 {
    let start_program_length = program.len();
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
                
                *data_segment_size += str.len() + 1;
                let size = memory.stack_memory.len();
                for (i, c) in str.char_indices() {
                    memory.stack_memory[size - *data_segment_size + i] = c as u8;
                }
                memory.stack_memory[size - *data_segment_size + str.len()] = '\0' as u8;

                jump_tag_map.insert(last_jump_label[..].to_string(), memory.virtual_memory_size - *data_segment_size);
            } else if line.starts_with(".zero") {
                let size = line.split_once(' ').unwrap().1.trim().parse::<usize>().unwrap();

                *data_segment_size += size;
                jump_tag_map.insert(last_jump_label[..].to_string(), memory.virtual_memory_size - *data_segment_size);
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
                if !label.starts_with('.') {
                    print!("\x1b[32m");
                    print!("Mapping label \"{}\" to {}", label, jump_tag_map[label]);
                    println!("\x1b[0m");
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
        return jump_tag_map["main"] as i64
    }
    0
}

pub fn compile_files(files: &[String], memory: &mut Memory, verbose: bool) -> (Vec<String>, i64, usize) {
    let mut program = Vec::new();
    let mut data_segment_size = 0;
    let mut entry_point = 0;
    
    let mut jump_tag_map: HashMap<String, usize> = HashMap::new();
    for file in files {
        print!("\x1b[92m");
        print!("\x1b[1m");
        print!("Compiling \"{}\"", file);
        println!("\x1b[0m");
        let content = &fs::read_to_string(file)
            .expect("Could not read file!")[..];
        let ep = compile(content, memory, verbose, &mut program, &mut data_segment_size, &mut jump_tag_map);
        if ep != 0 {
            entry_point = ep;
        }
    }
    print!("\x1b[92m");
    print!("\x1b[1m");
    print!("Compilation finished, entry_point = {}, data_segment_size = {} bytes", entry_point, data_segment_size);
    println!("\x1b[0m");
    return (program, entry_point, data_segment_size);
}
