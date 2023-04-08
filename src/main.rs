use std::env;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::process::{Command, Stdio, Child, exit};


const HELP: &str = "\
Usage: runner [option] <command> [--] <args>
Options:
    --bg-runner         Run the commands in the background.
    --filter-runs <combs>
                        Filter certain combinations of arguments.
    -h, --help          Print this help message.
    --dry-runner        Print the commands that would be executed without
                          actually executing them.
    --ordered-runner    Combine only the arguments that are in the same
                          relative position.
    --runners           Number of commands to run in parallel.
    -v, --version       Print the version of runner.\
";


fn print_version() {
    // Get the version from Cargo.toml.
    let version = env!("CARGO_PKG_VERSION");
    println!("runner {}", version);
}


fn wait_for_child(child: &mut Child) {
    match child.try_wait() {
        Ok(Some(status)) => println!("exited with: {status}"),
        Ok(None) => {
            println!("status not ready yet, let's really wait");
            let res = child.wait();
            println!("result: {res:?}");
        }
        Err(e) => println!("error attempting to wait: {e}"),
    }
}


fn print_command(command_obj: &Command) {
    println!("{}", "-".repeat(80));
    print!("$ {} ", command_obj.get_program().to_str().unwrap());
    for arg in command_obj.get_args() {
        print!("{} ", arg.to_str().unwrap());
    }
    println!();
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut runners = 1;
    let mut command_start = 1;
    if args.len() < 2 {
        println!("{}", HELP);
        return;
    }
    if args[1] == "-v" || args[1] == "--version" {
        print_version();
        return;
    }
    else if args[1] == "-h" || args[1] == "--help" {
        println!("{}", HELP);
        return;
    }
    else if args[1] == "--runners" {
        if args.len() < 3 {
            println!("Error: --runners requires an argument.");
            return;
        }
        runners = args[2].parse().expect("Error: --runners requires an integer argument.");
        if runners < 1 {
            println!("Error: --runners requires an integer argument greater than 0.");
            return;
        }
        println!("Parallel runners: {}.", runners);
        command_start = 3;
    }

    let command_args = &args[command_start..];

    let mut dry_run = false;
    let mut bg_run = false;
    let mut new_command_args = Vec::new();
    let mut filter_combs = Vec::new();
    let mut filter = false;
    let mut ordered_runner = false;
    for arg in command_args {
        if filter {
            if arg.starts_with("-") {
                filter = false;
            } else {
                // If string contains a '+' character, then join the first part
                //   until the comma with all the other parts separated by the
                //   '+' character.
                if arg.contains("+") {
                    let mut parts = arg.split(",");
                    let first_part = parts.next().unwrap();
                    let second_part = parts.next().unwrap().to_string();
                    let second_parts = second_part.split("+");
                    for part in second_parts {
                        filter_combs.push(format!("{},{}", first_part, part));
                    }
                } else {
                    filter_combs.push(arg.to_string());
                }
                continue;
            }
        }
        if arg == "--dry-runner" {
            dry_run = true;
        } else if arg == "--bg-runner" {
            bg_run = true;
        } else if arg == "--filter-runs" {
            filter = true; 
        } else if arg == "--ordered-runner" {
            ordered_runner = true;
        } else {
            new_command_args.push(arg);
        }
    }
    let command_args = new_command_args;
    let mut command = Vec::new();
    // The command is the string before the first argument that starts with a
    // dash.
    let mut i = 0;
    while i<command_args.len() {
        // Check if command argument is equal to "--".
        if command_args[i] == "--" {
            i += 1;
            break;
        }
        command.push(&command_args[i]);
        i += 1;
    }
    // The remaining arguments are the arguments for the command.
    let command_args = &command_args[i..];

    let mut multi_args = BTreeMap::new();
    let mut i = 0;
    let empty_string = "".to_string();
    loop {
        if i >= command_args.len() {
            break;
        }
        if command_args[i].starts_with("-") {
            multi_args.insert(i, (command_args[i], Vec::new()));
            for j in i+1..command_args.len() {
                if !command_args[j].starts_with("-") {
                    multi_args.get_mut(&i).unwrap().1.push(command_args[j]);
                }
                else {
                    i = j-1;
                    break;
                }
            }
        } else {
            if i == 0 {
                println!("First argument does not start with a dash.");
                println!("=> Using all arguments as a single main argument.");
                multi_args.insert(0, (&empty_string, command_args.to_vec()));
            }
        }
        i += 1;
    }

    // Print the command that will be executed.
    print!("$ ");
    for arg in &command {
        print!("{} ", arg);
    }
    println!();
    // Pretty print multi_args.
    for (_, value) in &multi_args {
        println!("  {}: {:?}", value.0, value.1);
    }
    println!();

    //// Compute all the different combinations of arguments possible.
    let mut combinations = Vec::<Vec<(&str, &str)>>::new();
    if ordered_runner {
        if multi_args.len() == 0 {
            println!("Warning: --ordered-runner requires at least one argument, so it is ignored.");
            exit(1);
        } else {
            let mut num_args = 0;
            for (_, value) in &multi_args {
                if value.1.len() != num_args {
                    if num_args != 0 {
                        println!(
                            "Error: --ordered-runner requires all arguments to have the same number of\
                            \n       values, so it is ignored.");
                        exit(1);
                    } else {
                        num_args = value.1.len();
                    }
                }
            }
            for i in 0..num_args {
                let mut combination = Vec::new();
                for (_, value) in &multi_args {
                    combination.push((value.0.as_str(), value.1[i].as_str()));
                }
                combinations.push(combination);
            }
        }
    } else {
        for (_, value) in &multi_args {
            if combinations.len() == 0 {
                if value.1.len() == 0 {
                    combinations.push(vec![(value.0, "")]);
                } else {
                    for arg in &value.1 {
                        combinations.push(vec![(value.0, &*arg)]);
                    }
                }
            } else {
                let mut new_combinations = Vec::new();
                if value.1.len() == 0 {
                    for combination in &combinations {
                        let mut new_combination = combination.clone();
                        new_combination.push((value.0, ""));
                        new_combinations.push(new_combination);
                    }
                }
                else {
                    for arg in &value.1 {
                        for combination in &combinations {
                            let mut new_combination = combination.clone();
                            new_combination.push((value.0, &*arg));
                            new_combinations.push(new_combination);
                        }
                    }
                }
                combinations = new_combinations;
            }
        }
    }

    //// Filter combinations.
    let mut new_combinations = Vec::new();
    if filter_combs.len() > 0 && combinations.len() > 0 {
        println!("Filtered combinations:");
        for filter_comb in &filter_combs {
            println!("  {}", filter_comb);
        }
        for combination in &combinations {
            // Convert arguments to a string joined by commas
            let mut comb_str = String::new();
            for (_key, value) in combination {
                comb_str.push_str(value);
                comb_str.push_str(",");
            }
            // Remove the last comma.
            comb_str.pop();
            // Check if the combination is in the filter list.
            // If it is, remove it from the combinations list.
            if !filter_combs.contains(&&comb_str) {
                new_combinations.push(combination.clone());
            }
        }
        combinations = new_combinations;
        println!();
    }

    if combinations.len() > 0 {
        println!("Combinations ({}):", combinations.len());
        for combination in &combinations {
            println!("  {:?}", combination);
        }
    } else {
        // Just run the command without any arguments.
        println!("Running command with no arguments.");
        combinations.push(vec![]);
    }

    let mut commands_run = 0;
    // Array of commands that are currently running.
    let mut running_commands = Vec::new();
    for combination in &combinations {
        let mut command_obj = Command::new(&command[0]);
        for arg in &command[1..] {
            command_obj.arg(arg);
        }
        for (key, value) in combination {
            // Add the key if it is not an empty string.
            if !key.is_empty() {
                command_obj.arg(key);
            }
            if !value.is_empty() {
                command_obj.arg(value);
            }
        }
        println!();
        if dry_run {
            print_command(&command_obj);
        } else {
            if running_commands.len() >= runners {
                // Wait for a command to finish.
                let mut child: Child = running_commands.remove(0);
                //https://doc.rust-lang.org/std/process/struct.Child.html
                wait_for_child(&mut child);
            }
            // Print the command that will be executed without the quotes.
            print_command(&command_obj);
            let child = match command_obj
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    // If the error is because the command was not found,
                    // exit the program.
                    if e.kind() == ErrorKind::NotFound {
                        println!(
                            "Command not found: {}",
                            command_obj.get_program().to_str().unwrap()
                        );
                        println!("Exiting...");
                        exit(1);
                    } else {
                        continue;
                    }
                }
            };
            // Run command detached
            running_commands.push(child);
        }
        commands_run += 1;
    }
    if dry_run || !bg_run {
        for mut child in running_commands {
            wait_for_child(&mut child);
        }
        print!("\n{} commands run.", commands_run);
        if dry_run {
            println!(" (dry run)");
        } else {
            println!();
        }
    }
}
