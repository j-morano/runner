use std::env;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::process::{Command, Stdio, Child};


const HELP: &str = "\
Usage: runner [option] <command> [--] <args>
Options:
    --bg-runner         Run the commands in the background.
    --filter <combs>@   Filter certain combinations of arguments.
    -h, --help          Print this help message.
    --dry-runner        Print the commands that would be executed without
                        actually executing them.
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
    for arg in command_args {
        if filter {
            if arg.starts_with("-") || arg == "@" {
                filter = false;
            } else {
                if arg.ends_with("@") {
                    filter_combs.push(arg.trim_end_matches("@"));
                    filter = false;
                } else {
                    filter_combs.push(arg);
                }
            }
        } else if arg == "--dry-runner" {
            dry_run = true;
        } else if arg == "--bg-runner" {
            bg_run = true;
        } else if arg == "--filter" {
            filter = true; 
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
            multi_args.insert(command_args[i], Vec::new());
            for j in i+1..command_args.len() {
                if !command_args[j].starts_with("-") {
                    multi_args.get_mut(command_args[i]).unwrap().push(command_args[j]);
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
                multi_args.insert(&empty_string, command_args.to_vec());
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
    for (key, value) in multi_args.iter().rev() {
        println!("  {}: {:?}", key, value);
    }
    println!();

    // Compute all the different combinations of arguments possible.
    let mut combinations = Vec::new();
    for (key, value) in multi_args.iter().rev() {
        if combinations.len() == 0 {
            if value.len() == 0 {
                combinations.push(vec![(*key, "")]);
            } else {
                for arg in value {
                    combinations.push(vec![(*key, *arg)]);
                }
            }
        } else {
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
            if !filter_combs.contains(&comb_str.as_str()) {
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
            continue;
        }
        else {
            if running_commands.len() >= runners {
                // Wait for a command to finish.
                let mut child: Child = running_commands.remove(0);
                //https://doc.rust-lang.org/std/process/struct.Child.html
                wait_for_child(&mut child);
            }
            // Print the command that will be executed without the quotes.
            println!("{}", "-".repeat(80));
            print!("$ {} ", command_obj.get_program().to_str().unwrap());
            for arg in command_obj.get_args() {
                print!("{} ", arg.to_str().unwrap());
            }
            println!();
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
                        std::process::exit(1);
                    } else {
                        continue;
                    }
                }
            };
                // .expect("failed to execute process");
            // Run command detached
            running_commands.push(child);
            commands_run += 1;
        }
    }
    if dry_run || !bg_run {
        for mut child in running_commands {
            wait_for_child(&mut child);
        }
        println!("{} commands run.", commands_run);
    }
}
