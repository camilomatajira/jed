use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use colored_json::to_colored_json_auto;
use serde_json::Value;
use std::io::Read;
#[derive(ClapParser)]
pub struct Cli {
    #[clap(short, long, action)]
    expression: String,
    input_file: Option<String>,
}

mod grammar;
use grammar::{parse_grammar, JedCommand};

mod commands;
use commands::{
    delete_on_specified_ranges, print_on_specified_ranges, substitute_keys,
    substitute_keys_on_specified_ranges, substitute_values, substitute_values_on_specified_ranges,
};

fn main() -> Result<()> {
    // Restore default SIGPIPE handling
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    let mut file_contents = String::from("");

    match cli.input_file {
        Some(ref input_file) => {
            file_contents = std::fs::read_to_string(input_file)
                .with_context(|| "Could not read file".to_string())?;
            file_contents = file_contents.parse::<String>()?;
        }
        None => {
            std::io::stdin().read_to_string(&mut file_contents)?;
        }
    }

    let input = &cli.expression;
    let mut v: Value = serde_json::from_str(&file_contents)
        .with_context(|| "Could not parse file into JSON".to_string())?;
    let (stack, command) = parse_grammar(input)?;

    match command {
        JedCommand::Substitute(params) => {
            let pattern = params.pattern;
            let replacement = params.replacement;
            if !stack.is_empty() {
                v = substitute_values_on_specified_ranges(v, stack, &pattern, &replacement);
            } else {
                v = substitute_values(v, &pattern, &replacement);
            }
        }
        JedCommand::SubstituteKeys(params) => {
            let pattern = params.pattern;
            let replacement = params.replacement;
            if !stack.is_empty() {
                v = substitute_keys_on_specified_ranges(v, stack, &pattern, &replacement);
            } else {
                v = substitute_keys(v, &pattern, &replacement);
            }
        }
        JedCommand::Print => {
            v = print_on_specified_ranges(v, stack);
        }
        JedCommand::Delete => {
            v = delete_on_specified_ranges(v, stack);
        }
        JedCommand::Other(_) => {
            println!("Only substitute command is supported for now");
            std::process::exit(1);
        }
    }

    println!(
        "{}",
        to_colored_json_auto(&v).context("Failed to colorize JSON output")?
    );
    Ok(())
}

#[cfg(test)]
mod tests;
