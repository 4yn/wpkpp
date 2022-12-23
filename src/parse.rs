use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter};
use utf8_chars::BufReadCharsExt;

use crate::vm::{Instruction, Instructions, VmUsize, WpkOpcount, MEM_SIZE};

const INC_STR: &str = "INC";
const CDEC_STR: &str = "CDEC";
const LOAD_STR: &str = "LOAD";
const INV_STR: &str = "INV";

const INC_M_STR: char = '>';
const CDEC_M_STR: char = '<';
const LOAD_M_STR: char = '?';
const LOAD_M_STR_ALT: char = 'v';
const INV_M_STR: char = '!';
const INV_M_STR_ALT: char = '^';

const MEGABYTE: u64 = 1_000_000;
const MAX_FILE_SIZE: u64 = 10_000_000;
const MAX_M_FILE_SIZE: u64 = 5_000_000;

pub fn check_valid_extension(path: &str) -> bool {
    path.ends_with(".wpk") || path.ends_with(".wpkm")
}

fn push_and_compress_instruction(instructions: &mut Instructions, new_instruction: Instruction) {
    let n_instructions = instructions.len();
    let tail = instructions
        .get(n_instructions.wrapping_sub(1))
        .map(|x| x.clone());
    let tail_two = instructions
        .get(n_instructions.wrapping_sub(2))
        .map(|x| x.clone());

    match (new_instruction, tail, tail_two) {
        (Instruction::Null, _, _) => {}
        (Instruction::Inc(x), Some(Instruction::Inc(y)), _) => {
            instructions[n_instructions - 1] = Instruction::Inc(y.wrapping_add(x));
        }
        (Instruction::Cdec(x), Some(Instruction::Cdec(y)), _) => {
            instructions[n_instructions - 1] = Instruction::Cdec(y.wrapping_add(x));
        }
        (Instruction::Inc(x), Some(Instruction::Cdec(_)), Some(Instruction::Inc(y))) => {
            instructions[n_instructions - 2] = Instruction::Inc(y.wrapping_add(x));
        }
        (Instruction::Cdec(x), Some(Instruction::Inc(_)), Some(Instruction::Cdec(y))) => {
            instructions[n_instructions - 2] = Instruction::Cdec(y.wrapping_add(x));
        }
        _ => {
            instructions.push(new_instruction);
        }
    }
}

fn parse_wpk_line(raw_instruction: &[&str], line_trace: usize) -> Result<Instruction> {
    let instruction = match raw_instruction {
        [] => Instruction::Null,
        [INC_STR] => Instruction::Inc(1),
        [INC_STR, nstr] => {
            let x: u64 = nstr.parse().map_err(|e| {
                anyhow!(
                    "{}: {} @ line {}",
                    e,
                    raw_instruction.join(" "),
                    line_trace + 1
                )
            })?;
            if (x as usize) >= MEM_SIZE {
                Err(anyhow!(
                    "INC repetition of {} too large @ line {}",
                    x,
                    line_trace
                ))?;
            }

            Instruction::Inc(x as VmUsize)
        }
        [CDEC_STR] => Instruction::Cdec(1),
        [CDEC_STR, nstr] => {
            let x: u64 = nstr.parse().map_err(|e| {
                anyhow!(
                    "{}: {} @ line {}",
                    e,
                    raw_instruction.join(" "),
                    line_trace + 1
                )
            })?;
            if (x as usize) >= MEM_SIZE {
                Err(anyhow!(
                    "CDEC repetition of {} too large @ line {}",
                    x,
                    line_trace
                ))?;
            }

            Instruction::Cdec(x as VmUsize)
        }
        [LOAD_STR] => Instruction::Load,
        [INV_STR] => Instruction::Inv,
        _ => {
            return Err(anyhow!(
                "Unknown instruction '{:?}' @ line {}",
                raw_instruction,
                line_trace
            ))
        }
    };

    Ok(instruction)
}

fn parse_wpk(path: &str, check_size: bool) -> Result<Instructions> {
    let file = File::options().read(true).open(path)?;

    if check_size {
        let filesize = file.metadata()?.len();
        if filesize >= MAX_FILE_SIZE {
            return Err(anyhow!(
                "File size {:.2}/{:.2} MB is too large; try compressing your instructions",
                (filesize as f64) / (MEGABYTE as f64),
                (MAX_FILE_SIZE as f64) / (MEGABYTE as f64)
            ));
        }
    }

    let reader = BufReader::new(file);

    let mut instructions: Instructions = vec![];

    for (line_idx, line) in reader.lines().enumerate() {
        let raw_line = line?;
        let raw_instruction = raw_line.split_whitespace().collect::<Vec<_>>();
        let new_instruction: Instruction = parse_wpk_line(raw_instruction.as_slice(), line_idx)?;

        push_and_compress_instruction(&mut instructions, new_instruction);
    }

    Ok(instructions)
}

fn parse_wpkm(path: &str, check_size: bool) -> Result<Instructions> {
    let file = File::options().read(true).open(path)?;

    if check_size {
        let filesize = file.metadata()?.len();
        if filesize >= MAX_M_FILE_SIZE {
            return Err(anyhow!(
                "File size {:.2}/{:.2} MB is too large; try compressing your instructions",
                (filesize as f64) / (MEGABYTE as f64),
                (MAX_FILE_SIZE as f64) / (MEGABYTE as f64)
            ));
        }
    }

    let mut reader = BufReader::new(file);
    let mut instructions: Instructions = vec![];
    let mut ctr: Option<u64> = None;

    for (c_trace, c) in reader.chars().enumerate() {
        let c = c.unwrap();
        let new_instruction: Instruction = match c {
            INC_M_STR => {
                let x = ctr.unwrap_or(1);
                if (x as usize) >= MEM_SIZE {
                    Err(anyhow!(
                        "INC repetition of {} too large @ char {}",
                        x,
                        c_trace
                    ))?;
                }
                let i = Instruction::Inc(x as VmUsize);
                ctr = None;
                i
            }
            CDEC_M_STR => {
                let x = ctr.unwrap_or(1);
                if (x as usize) >= MEM_SIZE {
                    Err(anyhow!(
                        "CDEC repetition of {} too large @ char {}",
                        x,
                        c_trace
                    ))?;
                }
                let i = Instruction::Cdec(x as VmUsize);
                ctr = None;
                i
            }
            LOAD_M_STR | LOAD_M_STR_ALT => {
                if let Some(x) = ctr {
                    Err(anyhow!(
                        "Cannot repeat LOAD instruction {} times @ char {}",
                        x,
                        c_trace
                    ))?;
                }
                Instruction::Load
            }
            INV_M_STR | INV_M_STR_ALT => {
                if let Some(x) = ctr {
                    Err(anyhow!(
                        "Cannot repeat INV instruction {} times @ char {}",
                        x,
                        c_trace
                    ))?;
                }
                Instruction::Inv
            }
            '0'..='9' => {
                ctr = match ctr {
                    None => Some(c.to_digit(10).unwrap() as u64),
                    Some(ctr_i) => {
                        let ctr_new = ctr_i * 10 + c.to_digit(10).unwrap() as u64;
                        if ctr_new > MEM_SIZE as u64 {
                            Err(anyhow!(
                                "Repeat of {} times too large @ char {}",
                                ctr_new,
                                c_trace
                            ))?;
                        }
                        Some(ctr_new)
                    }
                };
                Instruction::Null
            }
            ' ' | '\n' | '\t' => Instruction::Null,
            _ => return Err(anyhow!("Invalid instruction {} @ char {}", &c, c_trace)),
        };

        push_and_compress_instruction(&mut instructions, new_instruction);
    }

    if let Some(c) = ctr {
        return Err(anyhow!("Dangling repeat {} at end of script", &c));
    }

    Ok(instructions)
}

pub fn parse_file(path: &str, check_size: bool) -> Result<Instructions> {
    if !check_valid_extension(path) {
        Err(anyhow!(
            "Invalid input woodpecker script name {}, should end in \".wpk\" or \".wpkm\"",
            path
        ))?;
    }

    if path.ends_with(".wpk") {
        parse_wpk(path, check_size)
    } else if path.ends_with(".wpkm") {
        parse_wpkm(path, check_size)
    } else {
        Err(anyhow!("Unknown file type {}", path))
    }
}

pub fn do_compress(input_path: &str, output_path: &str) -> Result<()> {
    if !check_valid_extension(input_path) {
        Err(anyhow!(
            "Invalid input woodpecker script name {}, should end in \".wpk\" or \".wpkm\"",
            input_path
        ))?;
    }
    if !check_valid_extension(output_path) {
        Err(anyhow!(
            "Invalid output woodpecker script name {}, should end in \".wpk\" or \".wpkm\"",
            output_path
        ))?;
    }
    if input_path == output_path {
        Err(anyhow!("Input and output paths the same; aborting"))?;
    }

    println!("Compressing {} => {}", input_path, output_path);
    println!("Parsing...");
    let instructions = parse_file(input_path, false)?;
    let opcounts = instructions.opcount();

    println!(
        "Instruction Counts: INC {} / CDEC {} / LOAD {} / INV {}",
        opcounts.0, opcounts.1, opcounts.2, opcounts.3
    );
    println!(
        "Total {} step(s) compressed into {} instructions",
        opcounts.0 + opcounts.1 + opcounts.2 + opcounts.3,
        instructions.len()
    );

    println!("Writing...");
    let output_file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(output_path)?;
    let mut writer = BufWriter::new(output_file);
    if output_path.ends_with(".wpk") {
        for instruction in instructions.iter() {
            writer.write(instruction.to_wpk_string().as_bytes())?;
        }
    } else if output_path.ends_with(".wpkm") {
        for instruction in instructions.iter() {
            writer.write(instruction.to_wpkm_string().as_bytes())?;
        }
    } else {
        unreachable!();
    }
    println!("Done!");

    Ok(())
}
