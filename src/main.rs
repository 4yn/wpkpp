use clap::{Parser, Args, Subcommand};

use wpkpp::{do_compress, do_grade, check_valid_extension};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Grade(Grade),
    Compress(Compress)
}

#[derive(Args)]
#[command(verbatim_doc_comment)]
/// Grade a woodpecker task
/// Current challenges:
///   0: 1 bit XOR
///   1: 1 bit half adder
///   2: 16 bit add
///   3: 16 bit multiplication
///   4: 16 bit add modulo 2**16 - 17
///   5: 16 bit multiplication modulo 2**16 - 17
struct Grade {
    /// Task number [0..5]
    #[arg(value_name = "task")]
    task_id: u8,
    /// Solution path
    #[arg(value_name = "script.(wpk|wpkm)", value_parser = valid_script_name)]
    wpk_path: String,
    /// Hide progress bar
    #[arg(long)]
    noprogress: bool,
    /// Disable color
    #[arg(long)]
    nocolor: bool,
    /// JSON ouptut
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
#[command(verbatim_doc_comment)]
/// Compress your woodpecker scripts to use repeating INC / CDEC instructions
/// *.wpk format uses "INC [?n]" / "CDEC [?n]" / "LOAD" / "INV"  
/// *.wpkm format uses "[?n]>" / "[?n]<" / "?" / "!"
struct Compress {
    /// Input file path
    #[arg(value_name = "infile.(wpk|wpkm)", value_parser = valid_script_name)]
    input_path: String,

    /// Output file path
    #[arg(value_name = "outfile.(wpk|wpkm)", value_parser = valid_script_name)]
    output_path: String,
}

fn valid_script_name(path: &str) -> Result<String, String> {
    match check_valid_extension(path) {
        true => Ok(path.to_string()),
        false => Err(format!("Invalid input woodpecker script name {}, should end in \".wpk\" or \".wpkm\"", path))
    }
}

fn main() {
    let args = Cli::parse();
    let res = match args.command {
        Commands::Grade(grade_args) => {
            do_grade(grade_args.task_id, &grade_args.wpk_path, !grade_args.noprogress, !grade_args.nocolor, grade_args.json)
        },
        Commands::Compress(compress) => {
            do_compress(compress.input_path.as_str(), compress.output_path.as_str())
        }
    };
    if let Some(e) = res.err() {
        println!("Error: {}", e);
    }
}