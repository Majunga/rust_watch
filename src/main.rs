use clap::Parser;
use glob::glob;
use std::thread::sleep;
use std::time::Duration;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    pattern: String,

    #[arg()]
    command: String,
}

fn main() {
    let args = Args::parse();
    let mut cache = glob_and_cache(args.pattern.clone());

    loop {
        let new_cache = glob_and_cache(args.pattern.clone());

        let mut has_changes = false;
        if new_cache != cache {
            command_handler(&args.command);
            has_changes = true;
        }

        if has_changes {
            cache = new_cache;
        }

        sleep(Duration::from_secs(1));
    }
}

fn command_handler(command_to_execute: &str) {
    let commands = (&command_to_execute).split("&&");

    for cmd in commands {
        let program = cmd.split(" ").next().expect("program to be present");
        let args = cmd.split(" ").skip(1);
        
        let output = std::process::Command::new(program).args(args).output();

        match output {
            Ok(output) => {
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

fn glob_and_cache(pattern: String) -> String {
    let mut cache = Vec::<String>::new();

    for entry in glob(&pattern).expect("Failed to readpattern") {
        match entry {
            Ok(path) => {
                cache.push(sha256::try_digest(path).expect("File to be hashed"));
            }
            Err(e) => println!("{:?}", e),
        }
    }

    cache.concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_and_cache() {
        let expected_lock_hash = "75b6ee4f82a89fe973adbbbf91941bc60c79305dad1cbbfd97ba42b23e9febac";
        let expected_toml_hash = "1c9b383260ce264fcfc24160fc5c49230e49b1fe1f342b0b7fd2e5491442c215";

        let pattern = "./Cargo.*".to_string();
        let cache = glob_and_cache(pattern);

        assert_eq!(format!("{expected_lock_hash}{expected_toml_hash}"), cache);
    }

    #[test]
    fn test_command_handler() {
        let command = "echo hello";
        command_handler(command);
    }
}