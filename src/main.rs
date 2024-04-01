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
    let mut cache = glob_and_cache(args.pattern.clone(), &mut |_, _| {});

    println!("Watching for changes in: {}", args.pattern.clone());

    loop {
        let mut has_changes = false;

        let mut check_for_change = |i: usize, hash: String| {
            if hash != cache[i] {
                has_changes = true;
            }
        };

        let new_cache = glob_and_cache(args.pattern.clone(), &mut check_for_change);

        if has_changes {
            command_handler(&args.command);
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
                print!("{}", String::from_utf8_lossy(&output.stdout));
                print!("{}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

fn glob_and_cache(pattern: String, change_check: &mut dyn FnMut (usize, String)) -> Vec<String> {
    let mut cache = Vec::<String>::new();

    for (i, entry) in glob(&pattern).expect("Failed to readpattern").enumerate() {
        match entry {
            Ok(path) => {
                let hash = sha256::try_digest(path).expect("File to be hashed");
                change_check(i, hash.clone());

                cache.push(hash);
            }
            Err(e) => println!("{:?}", e),
        }
    }

    cache
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_and_cache() {
        let expected_lock_hash = "75b6ee4f82a89fe973adbbbf91941bc60c79305dad1cbbfd97ba42b23e9febac".to_string();
        let expected_toml_hash = "1c9b383260ce264fcfc24160fc5c49230e49b1fe1f342b0b7fd2e5491442c215".to_string();
        let expected = vec![expected_lock_hash, expected_toml_hash];

        let pattern = "./Cargo.*".to_string();
        let actual = glob_and_cache(pattern, &mut |_, _| {});

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_command_handler() {
        let command = "echo hello";
        command_handler(command);
    }
}