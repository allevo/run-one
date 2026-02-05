use std::{collections::VecDeque, process::Command};

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Cmd {
    cmd: String,
    args: Vec<String>,
    wait: Option<u64>,
}

pub fn parse_args<I>(mut args: VecDeque<String>, mut vars: I) -> Result<Cmd>
where
    I: Iterator<Item = (String, String)>,
{
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(args: &[&str]) -> VecDeque<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    fn make_vars(vars: &[(&str, &str)]) -> impl Iterator<Item = (String, String)> {
        vars.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn test_parse_args_simple_command() {
        let args = make_args(&["run-one", "echo", "hello"]);
        let vars = make_vars(&[]);

        let cmd = parse_args(args, vars).unwrap();

        assert_eq!(cmd.cmd, "echo");
        assert_eq!(cmd.args, vec!["hello"]);
        assert_eq!(cmd.wait, None);
    }

    #[test]
    fn test_parse_args_command_without_arguments() {
        let args = make_args(&["run-one", "ls"]);
        let vars = make_vars(&[]);

        let cmd = parse_args(args, vars).unwrap();

        assert_eq!(cmd.cmd, "ls");
        assert!(cmd.args.is_empty());
        assert_eq!(cmd.wait, None);
    }

    #[test]
    fn test_parse_args_with_wait_env_var() {
        let args = make_args(&["run-one", "echo", "test"]);
        let vars = make_vars(&[("RUN_ONE_WAIT", "5")]);

        let cmd = parse_args(args, vars).unwrap();

        assert_eq!(cmd.cmd, "echo");
        assert_eq!(cmd.args, vec!["test"]);
        assert_eq!(cmd.wait, Some(5));
    }

    #[test]
    fn test_parse_args_with_invalid_wait_env_var() {
        let args = make_args(&["run-one", "echo"]);
        let vars = make_vars(&[("RUN_ONE_WAIT", "not_a_number")]);

        let cmd = parse_args(args, vars).unwrap();

        assert_eq!(cmd.cmd, "echo");
        assert_eq!(cmd.wait, None);
    }

    #[test]
    fn test_parse_args_missing_command() {
        let args = make_args(&["run-one"]);
        let vars = make_vars(&[]);

        let result = parse_args(args, vars);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unable to get the command"));
    }

    #[test]
    fn test_parse_args_empty_args() {
        let args: VecDeque<String> = VecDeque::new();
        let vars = make_vars(&[]);

        let result = parse_args(args, vars);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unable to get the name of the program"));
    }

    #[test]
    fn test_parse_args_multiple_arguments() {
        let args = make_args(&["run-one", "git", "commit", "-m", "test message"]);
        let vars = make_vars(&[]);

        let cmd = parse_args(args, vars).unwrap();

        assert_eq!(cmd.cmd, "git");
        assert_eq!(cmd.args, vec!["commit", "-m", "test message"]);
    }

    #[test]
    fn test_run_successful_command() {
        let cmd = Cmd {
            cmd: "true".to_string(),
            args: vec![],
            wait: None,
        };

        let result = run(&cmd);

        assert!(result.is_ok());
    }

    #[test]
    fn test_run_failing_command() {
        let cmd = Cmd {
            cmd: "false".to_string(),
            args: vec![],
            wait: None,
        };

        let result = run(&cmd);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Command failed"));
    }

    #[test]
    fn test_run_nonexistent_command() {
        let cmd = Cmd {
            cmd: "nonexistent_command_12345".to_string(),
            args: vec![],
            wait: None,
        };

        let result = run(&cmd);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to execute"));
    }

    #[test]
    fn test_run_command_with_arguments() {
        let cmd = Cmd {
            cmd: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            wait: None,
        };

        let result = run(&cmd);

        assert!(result.is_ok());
    }
}
