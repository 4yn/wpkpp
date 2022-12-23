use anyhow::Result;
use colored::Colorize;
use miniserde::{json, Deserialize, Serialize};
use std::io;
use std::{cmp::max, io::Write};

use crate::{
    parse::parse_file,
    task::Task,
    util::ResetableTimer,
    vm::{Vm, WpkOpcount},
};

#[derive(Serialize, Deserialize, Debug)]
struct InstructionCount {
    inc: u64,
    cdec: u64,
    load: u64,
    inv: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimeTaken {
    parse: f64,
    vm: f64,
    grade: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct GradeResult {
    verdict: String,
    score: u64,
    total: u64,
    runtime: i64,
    memory: i64,
    instructions: InstructionCount,
    time_taken: TimeTaken,
}

pub fn do_grade(task: Task, wpk_path: &str, progress: bool, color: bool, json: bool) -> Result<()> {
    let mut timer = ResetableTimer::new();
    let mut parse_time: f64 = 0.0;
    let mut vm_time: f64 = 0.0;
    let mut grade_time: f64 = 0.0;

    let instructions = parse_file(wpk_path, true)?;
    let opcounts = instructions.opcount();

    parse_time += timer.seconds_since();

    let mut vm = Vm::new(instructions);

    vm_time += timer.seconds_since();

    let mut max_runtime: i64 = 0;
    let mut max_memory: i64 = 0;
    let mut total: u64 = 0;
    let mut correct: u64 = 0;

    for tc_id in 0..100 {
        let (input_mem, ans_mem) = task.load_tc(tc_id)?;
        vm.reset();
        vm.memory[0..input_mem.len()].copy_from_bitslice(&input_mem);
        vm_time += timer.seconds_since();

        let run_stats = vm.run();

        let output_mem = &vm.memory[input_mem.len()..(input_mem.len() + ans_mem.len())];

        let res = output_mem == ans_mem;

        max_runtime = max(max_runtime, run_stats.runtime);
        max_memory = max(max_memory, run_stats.memory);

        total += 1;
        if res {
            correct += 1;
        }

        if progress && !json {
            let mut res_text = match res {
                true => "O".green(),
                false => "X".red(),
            };
            if !color {
                res_text = res_text.clear();
            }

            print!("{}", res_text);
            io::stdout().flush().unwrap();
        }
        grade_time += timer.seconds_since();
    }

    if progress && !json {
        println!("");
    }

    if json {
        let gr = GradeResult {
            verdict: format!(
                "{}",
                match total == correct {
                    true => "OK",
                    false => "WA",
                }
            ),
            score: correct,
            total: total,
            runtime: max_runtime,
            memory: max_memory,
            instructions: InstructionCount {
                inc: opcounts.0,
                cdec: opcounts.1,
                load: opcounts.2,
                inv: opcounts.3,
            },
            time_taken: TimeTaken {
                parse: parse_time,
                vm: vm_time,
                grade: grade_time,
            },
        };

        println!("{}", json::to_string(&gr));
    } else {
        let mut res_text = match total == correct {
            true => "OK 🎉".green(),
            false => "WA ❌".red(),
        };
        if !color {
            res_text = res_text.clear();
        }

        println!("Verdict: {}", res_text);
        println!("Score: {}/{}", correct, total);
        println!("Instructions: {}", max_runtime);
        println!("Memory Usage: {}", max_memory);
        println!(
            "Instruction Counts: INC {} / CDEC {} / LOAD {} / INV {}",
            opcounts.0, opcounts.1, opcounts.2, opcounts.3
        );
        println!(
            "Time: Parse {:.3}s / VM Setup {:.3}s / Grading {:.3}s",
            parse_time, vm_time, grade_time
        );
    }

    Ok(())
}
