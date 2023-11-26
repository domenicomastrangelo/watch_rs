use clap::{arg, command};
use crossterm::{cursor, terminal, ExecutableCommand};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

enum ByteOrString<'a> {
    Bytes(&'a Vec<u8>),
    String(Cow<'a, str>),
}

impl<'a> ByteOrString<'a> {
    fn as_bytes(&self) -> &[u8] {
        match self {
            ByteOrString::Bytes(bytes) => bytes,
            ByteOrString::String(string) => string.as_bytes(),
        }
    }

    fn as_string(&self) -> Cow<'_, str> {
        match self {
            ByteOrString::Bytes(bytes) => match std::str::from_utf8(bytes) {
                Ok(string) => Cow::Borrowed(string),
                Err(e) => {
                    println!("Failed to convert bytes to string: {}", e);
                    Cow::Borrowed("")
                }
            },
            ByteOrString::String(string) => Cow::Borrowed(string),
        }
    }
}

fn main() {
    let matches = command!()
        .arg(
            arg!(-n --interval <NUMBER> "Interval at which the executable is run")
                .value_parser(clap::value_parser!(f64)),
        )
        .arg(arg!(<COMMAND> "Command to run").required(true))
        .arg(arg!(-d --difference "Highlight the difference between refreshes"))
        .get_matches();

    const DEFAULT_INTERVAL: f64 = 2.0;

    let interval: &f64 = match matches.try_get_one::<f64>("interval") {
        Ok(interval) => match interval {
            Some(interval) => interval,
            None => &DEFAULT_INTERVAL,
        },
        Err(_) => &DEFAULT_INTERVAL,
    };

    let millis_to_sleep: u64 = (*interval * 1000.0) as u64;

    let cmd: &String = match matches.try_get_one::<String>("COMMAND") {
        Ok(cmd) => match cmd {
            Some(cmd) => cmd,
            None => {
                println!("Command is required");
                return;
            }
        },
        Err(e) => {
            println!("Command is required");
            println!("{}", e);
            return;
        }
    };

    let mut output: String = "".to_string();
    let mut diffed_output: String = "".to_string();

    loop {
        match std::process::Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
        {
            Ok(out) => {
                let current_output_hash = get_sha256(&ByteOrString::Bytes(&out.stdout));
                let old_output_hash = get_sha256(&ByteOrString::String(Cow::Borrowed(&output)));

                if current_output_hash != old_output_hash {
                    diffed_output = get_diffed_output(
                        &output,
                        &ByteOrString::Bytes(&out.stdout),
                        &ByteOrString::String(Cow::Borrowed(&output)),
                    );
                }

                output = ByteOrString::Bytes(&out.stdout).as_string().to_string();
            }
            Err(e) => {
                println!("Failed to run executable");
                println!("{}", e);
                return;
            }
        }

        clear_terminal();

        println!("Every {}s: {}\n", interval, cmd);
        println!("{}", diffed_output);

        std::thread::sleep(std::time::Duration::from_millis(millis_to_sleep));
    }
}

fn get_diffed_output(
    output: &str,
    current_output: &ByteOrString,
    old_output: &ByteOrString,
) -> String {
    if output.len() == 0 {
        return current_output.as_string().to_string();
    }

    let mut diffed_output: Vec<u8> = Vec::new();

    for i in 0..current_output.as_bytes().len() {
        let current_char = current_output.as_bytes()[i];

        if i < old_output.as_bytes().len() {
            let old_char = old_output.as_bytes()[i];

            if current_char != old_char {
                // if the current character is different from the old character, then we want to highlight it
                // we do this by substituting the current character with a red background and white foreground
                insert_modified_character(&mut diffed_output, current_char);
            } else {
                diffed_output.push(current_char);
            }
        } else {
            insert_modified_character(&mut diffed_output, current_char);
        }
    }

    ByteOrString::Bytes(&diffed_output).as_string().to_string()
}

fn insert_modified_character(output: &mut Vec<u8>, character: u8) {
    let character = match String::from_utf8(vec![character]) {
        Ok(string) => string,
        Err(_) => return,
    };

    let formatted_string = format!("\x1b[41m\x1b[30m{}\x1b[0m", character);

    formatted_string
        .as_bytes()
        .iter()
        .for_each(|byte| output.push(*byte));
}

fn clear_terminal() {
    let _ = std::io::stdout().execute(cursor::MoveTo(0, 0));
    let _ = std::io::stdout().execute(terminal::Clear(terminal::ClearType::All));
}

fn get_sha256(input: &ByteOrString) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
