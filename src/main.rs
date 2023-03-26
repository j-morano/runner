use std::env;
use std::collections::HashMap;
use std::process::{Command, Stdio};


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: runner <command> <args>");
        return;
    }
    let command = &args[1];
    let command_args = &args[2..];

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
            for arg in value {
                combinations.push(vec![(*key, *arg)]);
            }
        }
        else {
            let mut new_combinations = Vec::new();
            for arg in value {
                for combination in &combinations {
                    let mut new_combination = combination.clone();
                    new_combination.push((*key, *arg));
                    new_combinations.push(new_combination);
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
        println!("Running with args: {:?}", combination);
        let mut command = Command::new(&command);
        for (key, value) in combination {
            command.arg(key).arg(value);
        }
        let mut child = command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to execute process");
        child.wait().expect("failed to wait on child");
    }
}
