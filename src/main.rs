use std::process::{self, Command, Stdio, Child};
use std::io::{self, Write};
use std::env;
use std::path::Path;

type SplittedCommands<'a> = Vec<Vec<&'a str>>;

enum Program {
    Builtin(Builtin),
    Program,
}

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

fn cd(args: &[&str]) -> String {
    let new_dir = args.get(0).map_or("/", |x| *x);
    let root = Path::new(new_dir);
    match env::set_current_dir(&root) {
        Ok(_) => format!(""),
        Err(_) => format!("cd: no such file or directory: {}\n", new_dir),
    }
    
}

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

fn split_commands(commands: &str) -> SplittedCommands {
    commands.trim()
        .split("|")
        .map(|x| x.trim().split(' ').collect())
        .collect()
}

fn prompt() {
    print!("> ");
    io::stdout().flush().unwrap();
}

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

fn stdin_input() -> String {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf
}
