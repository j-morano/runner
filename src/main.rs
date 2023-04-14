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


fn wait_for_child(child: &mut Child) -> bool {
    // If the return value of the prgram is different than 0, then return false,
    //  otherwise return true.
    match child.try_wait() {
        Ok(Some(status)) => {
            println!("exited with: {status}");
            if status.success() {
                true
            } else {
                false
            }
        },
        Ok(None) => {
            println!("status not ready yet, let's really wait");
            let res = child.wait();
            println!("result: {res:?}");
            if res.unwrap().success() {
                true
            } else {
                false
            }
        }
        Err(e) => {
            println!("error attempting to wait: {e}");
            false
        }
    }
}


fn print_command(command_obj: &Command) -> String {
    let mut command_string = String::new();
    println!("{}", "-".repeat(80));
    let command_name = command_obj.get_program().to_str().unwrap();
    print!("$ {} ", command_name);
    command_string.push_str(command_name);
    command_string.push_str(" ");
    for arg in command_obj.get_args() {
        let arg_str = arg.to_str().unwrap();
        print!("{} ", arg_str);
        command_string.push_str(arg_str);
        command_string.push_str(" ");
    }
    println!();
    command_string
}


/// Given a vector containing a partial Cartesian product, and a list of items,
/// return a vector adding the list of items to the partial Cartesian product.
///
/// # Example
///
/// ```
/// let partial_product = vec![vec![1, 4], vec![1, 5], vec![2, 4], vec![2, 5]];
/// let items = &[6, 7];
/// let next_product = partial_cartesian(partial_product, items);
/// assert_eq!(next_product, vec![vec![1, 4, 6],
///                               vec![1, 4, 7],
///                               vec![1, 5, 6],
///                               vec![1, 5, 7],
///                               vec![2, 4, 6],
///                               vec![2, 4, 7],
///                               vec![2, 5, 6],
///                               vec![2, 5, 7]]);
/// ```
pub fn partial_cartesian<T: Clone>(a: Vec<Vec<T>>, b: &[T]) -> Vec<Vec<T>> {
    a.into_iter().flat_map(|xs| {
        b.iter().cloned().map(|y| {
            let mut vec = xs.clone();
            vec.push(y);
            vec
        }).collect::<Vec<_>>()
    }).collect()
}


/// Creates a Cartesian product of the given lists.
pub fn cartesian_product<T: Clone>(lists: &[Vec<T>]) -> Vec<Vec<T>> {
    match lists.split_first() {
        Some((first, rest)) => {
            let init: Vec<Vec<T>> = first.iter().cloned().map(|n| vec![n.clone()]).collect();

            rest.iter().cloned().fold(init, |vec, list| {
                partial_cartesian(vec, &list)
            })
        },
        None => {
            vec![]
        }
    }
}


/// Combines the elements that are in the same relative position.
pub fn ordered_combinations<T: Clone>(lists: &[Vec<T>],) -> Vec<Vec<T>> {
    let mut combs: Vec<Vec<T>> = Vec::new();
    // Get length of the first list.
    let num_combinations = lists[0].len();
    for i in 0..num_combinations {
        let mut comb: Vec<T> = Vec::new();
        for list in lists {
            if i < list.len() {
                comb.push(list[i].clone());
            } else {
                comb.push(list[0].clone());
            }
        }
        combs.push(comb);
    }
    combs
}


pub fn parse_rules<'a>(arg: &'a str, rules_combs: &mut Vec<Vec<&'a str>>) {
    // If string contains a '+' character, then join the first part
    //   until the comma with all the other parts separated by the
    //   '+' character.
    let options: Vec<&str> = arg.split(",").collect();
    if arg.contains("+") {
        let mut option_parts = Vec::new();
        for option in options {
            let parts: Vec<_> = option.split("+").collect();
            // Convert the iterator to a vector.
            option_parts.push(parts);
        }
        // Cartesian product of the options.
        let combs = cartesian_product(&option_parts);
        // Append the combinations to the filter combinations.
        rules_combs.append(&mut combs.clone());
    } else {
        rules_combs.push(options);
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    //// Basic standard command line options.
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
    // Exclude the first argument, which is the name of the program.
    let command_args = &args[1..];

    //// Parse command line options of runner.
    let mut runners = 1;
    let mut dry_run = false;
    let mut bg_run = false;
    let mut new_command_args = Vec::new();
    let mut filter_combs = Vec::new();
    let mut filter = false;
    let mut allow_combs = Vec::new();
    let mut allow = false;
    let mut ordered_runner = false;
    let mut parse_runners = false;
    for arg in command_args {
        if filter {
            if arg.starts_with("-") {
                filter = false;
            } else {
                parse_rules(arg, &mut filter_combs);
                continue;
            }
        } else if allow {
            if arg.starts_with("-") {
                allow = false;
            } else {
                parse_rules(arg, &mut allow_combs);
                continue;
            }
        } else if parse_runners {
            runners = match arg.parse() {
                Ok(n) => n,
                Err(_) => {
                    println!("Error: --runners requires an integer argument.");
                    exit(1);
                }
            };
            if runners < 1 {
                println!("Error: --runners requires an integer argument greater than 0.");
                exit(1);
            }
            parse_runners = false;
            continue;
        }
        if arg == "--dry-runner" {
            dry_run = true;
        } else if arg == "--runners" {
            parse_runners = true;
        } else if arg == "--bg-runner" {
            bg_run = true;
        } else if arg == "--filter-runs" {
            filter = true; 
        } else if arg == "--allow-runs" {
            allow = true;
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
    let mut positional_args = BTreeMap::new();
    let mut positional_size = 0;
    loop {
        if i >= command_args.len() {
            break;
        }
        if command_args[i].starts_with("-") {
            multi_args.insert(i, (command_args[i], Vec::new()));
            for j in i+1..command_args.len() {
                if !command_args[j].starts_with("-") {
                    // if contains , then split and add to multi_args.
                    if command_args[j].contains(",") {
                        positional_args.insert(i, (command_args[i].to_string(), Vec::new()));
                        let options: Vec<_> = command_args[j].split(",").collect();
                        if positional_size == 0 {
                            positional_size = options.len();
                        } else if positional_size != options.len() {
                            println!("Error: Positional arguments must have the same number of options.");
                            exit(1);
                        }
                        for option in options {
                            // Add option as &String
                            positional_args.get_mut(&i).unwrap().1.push(option.to_string());
                        }
                        // Remove the added to multi_args.
                        multi_args.remove(&i);
                    } else {
                        multi_args.get_mut(&i).unwrap().1.push(command_args[j].to_string());
                    }
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
                multi_args.insert(0, (&empty_string, Vec::new()));
                for j in 0..command_args.len() {
                    multi_args.get_mut(&0).unwrap().1.push(command_args[j].to_string());
                }
            }
        }
        i += 1;
    }

    // Print the command that will be executed.
    print!("$ ");
    for c in &command {
        print!("{} ", c);
    }
    println!();
    // Pretty print multi_args.
    for (_, value) in &multi_args {
        println!("  {}: {:?}", value.0, value.1);
    }
    for (_, value) in &positional_args {
        println!("  {}: {:?} (positional)", value.0, value.1);
    }
    println!();

    println!("Number of runners: {}", runners);
    println!();

    if filter_combs.len() > 0 {
        println!("Filter runs:");
        for comb in &filter_combs {
            println!("  \"{}\"", comb.join(","));
        }
        println!();
    }
    if allow_combs.len() > 0 {
        println!("Allow runs:");
        for comb in &allow_combs {
            println!("  \"{}\"", comb.join(","));
        }
        println!();
    }

    let mut options = Vec::new();
    let mut flags = Vec::new();
    let mut multi_args_values = Vec::new();
    let mut positional_args_values = Vec::new();
    for (_, value) in &multi_args {
        let values = &value.1.clone();
        // copy value.1 to values
        let values = values.clone();
        if values.len() > 0 {
            multi_args_values.push(values);
            options.push(value.0.clone());
        } else {
            flags.push(value.0.clone());
        }
    }
    for (_, value) in &positional_args {
        let values = &value.1.clone();
        // copy value.1 to values
        let values = values.clone();
        if values.len() > 0 {
            positional_args_values.push(values);
            options.push(value.0.clone());
        }
    }
    let mut combs;
    if ordered_runner {
        // Check that all the options have the same number of values.
        let length = multi_args_values[0].len();
        if !multi_args_values.iter().all(|x| x.len() == length || x.len() == 1) {
            println!(
                "Error: --ordered-runner requires all options with more than one value to have\
                \n       the same number of values."
            );
            exit(1);
        }

        combs = ordered_combinations(&multi_args_values);
    } else {
        combs = cartesian_product(&multi_args_values);
        if positional_args_values.len() > 0 {
            let positional_combs = ordered_combinations(&positional_args_values);
            let mut new_combs = Vec::new();
            for comb in &combs {
                for positional_comb in &positional_combs {
                    let mut new_comb = comb.clone();
                    for value in positional_comb {
                        new_comb.push(value.clone());
                    }
                    new_combs.push(new_comb);
                }
            }
            combs = new_combs;
        }
    } 
    let mut combinations = Vec::<Vec<(&str, &str)>>::new();
    for comb in &combs {
        let mut i = 0;
        let mut option_values = Vec::new();
        let mut this_comb = Vec::new();
        for option in &options {
            this_comb.push((option.as_str(), comb[i].as_str()));
            // Create string with all the option values separated by a comma.
            option_values.push(comb[i].as_str());
            i += 1;
        }
        for flag in &flags {
            this_comb.push((flag.as_str(), ""));
        }
        let mut match_found = false;
        for filter_comb in &filter_combs {
            // Check if all option values are in the filter combination.
            match_found = filter_comb.iter()
                .all(
                    |x| option_values.contains(&&x)
                );
            if match_found {
                break;
            }
        }
        for allow_comb in &allow_combs {
            // Check if any option values are in the allow combination.
            // If so, then check if all option values are in the allow
            //   combination.
            // Check if the first allow value is in the option values.
            //   If so, then check if all option values are in the allow
            //   combination.
            let first_present = option_values.contains(&&allow_comb[0]);
            if first_present {
                // Check if all option values are in the allow combination.
                match_found = !allow_comb.iter()
                    .all(
                        |x| option_values.contains(&&x)
                    );
                if match_found {
                    break;
                }
            }
        }
        if !match_found {
            combinations.push(this_comb);
        }
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
    let mut failed_commands = Vec::new();
    for combination in &combinations {
        // Get c as string
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
                let mut child: (Child, String) = running_commands.remove(0);
                //https://doc.rust-lang.org/std/process/struct.Child.html
                if !wait_for_child(&mut child.0) {
                    failed_commands.push(child.1);
                }
            }
            // Print the command that will be executed without the quotes.
            let c_str = print_command(&command_obj);
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
            running_commands.push((child, c_str));
        }
        commands_run += 1;
    }
    if dry_run || !bg_run {
        for mut child in running_commands {
            if !wait_for_child(&mut child.0) {
                failed_commands.push(child.1);
            }
        }
        print!("\n{} commands run.", commands_run);
        if dry_run {
            println!(" (dry run)");
        } else {
            println!();
        }
    }
    if failed_commands.len() > 0 {
        println!("Failed commands ({}):", failed_commands.len());
        for command in &failed_commands {
            println!("  $ {}", command);
        }
        exit(1);
    }
}
