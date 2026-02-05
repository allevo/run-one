use std::{collections::VecDeque, error::Error};

use run_one::{parse_args, run};

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: VecDeque<String> = std::env::args().collect();
    let cmd = parse_args(args, std::env::vars())?;

    loop {
        match run(&cmd) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    print!("\x07");

    Ok(())
}
