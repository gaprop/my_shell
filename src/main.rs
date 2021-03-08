use std::process::{self, Command, Stdio, Child};
use std::io::{self, Write};
use std::env;
use std::path::Path;

type SplittedCommands<'a> = Vec<Vec<&'a str>>;

/// A program can either be a built in program (such as `cd`) or a generic program.
enum Program {
    Builtin(Builtin),
    Program,
}

/// Lists the different kinds of built in programs
enum Builtin {
    Cd,
    Exit,
}

fn main() {
    loop {
        prompt();
        let args = stdin_input();
        let args = split_commands(&args);
        let command = command_type(&args);

        match command {
            Err(e) => {
                print!("{}", e);
                io::stdout().flush().unwrap();
            },
            Ok(Program::Program) => {
                let child = create_processes(args);
                match child {
                    Err(e) => println!("{}", e),
                    Ok(child) => {
                        let output = run_commands(child);
                        print!("{}", output);
                    },
                }
            },
            Ok(Program::Builtin(builtin)) => {
                let output = run_builtin(builtin, args);
                print!("{}", output);
            },
        }
    }
}

/// Runs a given built in program
/// # Arguments
/// * builtin, a `Builtin` that is the program to be run
/// * args the arguments given to the program
/// # Returns
/// The output of the builtin program that has been run
fn run_builtin(builtin: Builtin, 
               args: SplittedCommands) -> String {
    match builtin {
        Builtin::Cd => {
            let args = args.get(0);
            if let None = args { return format!("") }
            cd(&args.unwrap()[1..])
        },
        Builtin::Exit => {
            process::exit(1);
        }
    }
}

/// Runs the builtin program `cd` on a given set of arguments
/// # Arguments
/// * args, a slice of slices of the path to go to
/// # Returns
/// A string as the output of the program
fn cd(args: &[&str]) -> String {
    let new_dir = args.get(0).map_or("/", |x| *x);
    let root = Path::new(new_dir);
    match env::set_current_dir(&root) {
        Ok(_) => format!(""),
        Err(_) => format!("cd: no such file or directory: {}\n", new_dir),
    }
    
}

/// Given a command, returns whether the command is a builtin or a different program.
/// # Arguments
/// * `commands`: The command to check (and its arguments)
/// # Returns
/// `Result<Program, String>`, where `Program` is the kind of program.
fn command_type(commands: &SplittedCommands) -> Result<Program, String> {
    let command = commands.get(0);
    if let None = command { return Err(format!("")); }
    let program = command.unwrap().get(0);

    match program {
        None => Err(format!("")),
        Some(&"cd") => Ok(Program::Builtin(Builtin::Cd)),
        Some(&"exit") => Ok(Program::Builtin(Builtin::Exit)),
        Some(_) => Ok(Program::Program),
    }
}

/// Given a string of commands, splices them into commands and arguments. Also splices a list of
/// piped programs into a list of lits of programs and their arguments.
/// # Arguments
/// * `commands`: A `&str` that is a string of the format "<command> [args] | [command] [args]".
/// # Returns
/// A slice where each list is a program and its arguments.
fn split_commands(commands: &str) -> SplittedCommands {
    commands.trim()
        .split("|")
        .map(|x| x.trim().split(' ').collect())
        .collect()
}

/// Creates a prompt at stdout.
fn prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}

/// Creates a process from a list of commands that can be piped together. Each command and its
/// arguments are listed in a `Vec<Vec<&str>>`.
/// # Arguments
/// * `args`: The list of commands to run. Can either be a single command or a list of lists of
/// commands to run piped.
/// # Returns
/// `Result<Child, String>`, where child is the process.
fn create_processes(mut args: SplittedCommands) -> Result<Child, String> {
    let mut commands = args.iter_mut().peekable();
    let mut prev_command = Err(String::from("my_shell: command not found"));
    while let Some(args) = commands.next() {
        let command = args.remove(0);
        let stdin = prev_command
            .map_or(Stdio::inherit(), 
                    |output: Child| Stdio::from(output.stdout.unwrap()));

        let stdout = if commands.peek().is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        prev_command = Command::new(command)
            .args(args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
            .map_err(|_| String::from("my_shell: command not found"));
    }
    prev_command
}

/// Runs the command of a given child process.
/// # Arguments
/// * `command`: The child to run
/// # Returns
/// The output of the child.
fn run_commands(command: Child) -> String {
    match command.wait_with_output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            String::from(stdout + stderr)
        },
        Err(_) => String::from("my_shell: commmand not found")
    }
}

/// Get input from stdin.
/// # Returns
/// A `String` from stdin.
fn stdin_input() -> String {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf
}
