use std::{collections::VecDeque, env::Vars, process::Command};

use anyhow::{anyhow, Result};

pub struct Cmd {
    cmd: String,
    args: Vec<String>,
    wait: Option<u64>,
}

pub fn parse_args(mut args: VecDeque<String>, mut vars: Vars) -> Result<Cmd> {
    // The first argument is the name of the program itself.
    match args.pop_front() {
        Some(_) => (),
        None => return Err(anyhow!("Unable to get the name of the program.")),
    };

    let cmd = match args.pop_front() {
        Some(cmd) => cmd,
        None => return Err(anyhow!("Unable to get the command.")),
    };

    let wait = vars
        .find(|(key, _)| key == "RUN_ONE_WAIT")
        .map(|(_, val)| val);
    let wait = wait.and_then(|val| match val.parse::<u64>() {
        Ok(val) => Some(val),
        Err(e) => {
            eprintln!("Invalid value for RUN_ONE_WAIT: {e}");
            None
        }
    });

    Ok(Cmd {
        cmd,
        args: args.into_iter().collect(),
        wait,
    })
}

pub fn run(cmd: &Cmd) -> Result<()> {
    let cmd_res = Command::new(&cmd.cmd).args(&cmd.args).spawn();

    let r = match cmd_res {
        Ok(mut child) => {
            let status = child.wait().unwrap();
            if !status.success() {
                Err(anyhow!("Command failed with exit code: {}", status))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(anyhow!("Failed to execute command: {}", e)),
    };

    if let Some(wait) = cmd.wait {
        std::thread::sleep(std::time::Duration::from_secs(wait));
    }

    r
}
