use std::env;
use std::collections::HashMap;
use std::process::{Command, Stdio};


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: runner <command> <args>");
        return;
    }
    let command_args = &args[1..];

    let mut dry_run = false;
    let mut new_command_args = Vec::new();
    for arg in command_args {
        if arg == "--dry-run" {
            dry_run = true;
        }
        else {
            new_command_args.push(arg);
        }
    }
    let command_args = new_command_args;
    let command = &command_args[0];

    let mut multi_args = HashMap::new();
    let mut i = 0;
    loop {
        if i >= command_args.len() {
            break;
        }
        if command_args[i].starts_with("-") {
            multi_args.insert(&command_args[i], Vec::new());
            for j in i+1..command_args.len() {
                if !command_args[j].starts_with("-") {
                    multi_args.get_mut(&command_args[i]).unwrap().push(&command_args[j]);
                }
                else {
                    i = j-1;
                    break;
                }
            }
        }
        i += 1;
    }

    println!("Command: {}", command);
    println!("Args:");
    // Pretty print multi_args HashMap.
    for (key, value) in &multi_args {
        println!("{}: {:?}", key, value);
    }

    // Compute all the different combinations of arguments possible.
    let mut combinations = Vec::new();
    for (key, value) in &multi_args {
        if combinations.len() == 0 {
            if value.len() == 0 {
                combinations.push(vec![(*key, "")]);
            }
            else {
                for arg in value {
                    combinations.push(vec![(*key, *arg)]);
                }
            }
        }
        else {
            let mut new_combinations = Vec::new();
            if value.len() == 0 {
                for combination in &combinations {
                    let mut new_combination = combination.clone();
                    new_combination.push((*key, ""));
                    new_combinations.push(new_combination);
                }
            }
            else {
                for arg in value {
                    for combination in &combinations {
                        let mut new_combination = combination.clone();
                        new_combination.push((*key, *arg));
                        new_combinations.push(new_combination);
                    }
                }
            }
            combinations = new_combinations;
        }
    }
    println!("Combinations:");
    for combination in &combinations {
        println!("{:?}", combination);
    }
    for combination in &combinations {
        println!("{}", "-".repeat(80));
        let mut command = Command::new(&command);
        for (key, value) in combination {
            command.arg(key);
            if !value.is_empty() {
                command.arg(value);
            }
        }
        // Print the command that will be executed without the quotes.
        print!("$ {} ", command.get_program().to_str().unwrap());
        for arg in command.get_args() {
            print!("{} ", arg.to_str().unwrap());
        }
        println!();
        if dry_run {
            continue;
        }
        else {
            let mut child = command
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to execute process");
            child.wait().expect("failed to wait on child");
        }
    }
}
