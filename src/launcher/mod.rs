use std::fs;
use std::io;
use std::path;
use std::process;
use std::thread;
use std::time;

mod cgroup;

#[derive(Debug)]
pub enum LaunchResult {
    Pass,
    TLE,
    MLE,
    OLE,
    RE,
}

pub struct Report {
    pub status: LaunchResult,
    pub time: u64,   // us
    pub memory: u64, // bytes
}

pub fn launch(
    executable_file: &path::Path,
    input_file: &path::Path,
    output_file: &path::Path,
    time_limit: u64,   // us
    memory_limit: u64, // bytes
) -> io::Result<Report> {
    let mut limit = cgroup::Cgroup::new(time_limit, memory_limit)?;
    let mut child = process::Command::new(&executable_file)
        .stdin(fs::File::open(&input_file)?)
        .stdout(fs::File::create(&output_file)?)
        .spawn()
        .unwrap();
    let child_pid = child.id();
    limit.set_task(child_pid)?;

    thread::spawn(move || {
        thread::sleep(time::Duration::from_micros(time_limit + 1000));
        unsafe {
            libc::kill(child_pid as i32, libc::SIGKILL);
        }
    });

    let status = child.wait()?;
    let (time, memory) = limit.report()?;

    let status = {
        if time / 1000 > time_limit {
            LaunchResult::TLE
        } else if memory > memory_limit {
            LaunchResult::MLE
        } else if status.success() {
            LaunchResult::Pass
        } else {
            LaunchResult::RE
        }
    };

    Ok(Report {
        status: status,
        time: time,
        memory: memory,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io;
    use std::io::prelude::*;

    #[test]
    fn test_launcher() -> io::Result<()> {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        env::set_var("ANA_JUDGE_ID", "test_launcher");

        let input_file =
            path::Path::new(&env::var("ANA_WORK_DIR").unwrap()).join("test_launcher.in");
        let output_file =
            path::Path::new(&env::var("ANA_WORK_DIR").unwrap()).join("test_launcher.out");

        fs::File::create(&input_file)?.write_all(b"echo hello world")?;
        match launch(
            &path::Path::new("bash"),
            &input_file,
            &output_file,
            1000000,          // 1 Sec
            64 * 1024 * 1024, // 64 Mb
        )?
        .status
        {
            LaunchResult::Pass => {
                let mut output = String::new();
                fs::File::open(&output_file)?.read_to_string(&mut output)?;
                assert_eq!(output, "hello world\n");
            }
            _ => panic!("Failed to execute program"),
        }
        fs::remove_file(&input_file)?;
        fs::remove_file(&output_file)?;
        Ok(())
    }

    #[test]
    fn test_memory_limit() {
        unimplemented!("TODO: How to test memory")
    }

    #[test]
    fn test_time_limit() -> io::Result<()> {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        env::set_var("ANA_JUDGE_ID", "test_time_limit");

        let input_file =
            path::Path::new(&env::var("ANA_WORK_DIR").unwrap()).join("test_time_limit.in");
        let output_file =
            path::Path::new(&env::var("ANA_WORK_DIR").unwrap()).join("test_time_limit.out");
        fs::File::create(&input_file)?.write_all(b"while true; do echo -n; done")?;
        match launch(
            &path::Path::new("bash"),
            &input_file,
            &output_file,
            1000000,          // 1 Sec
            64 * 1024 * 1024, // 64 Mb
        )?
        .status
        {
            LaunchResult::TLE => {}
            _ => panic!("Failed when test time limit"),
        }
        fs::remove_file(&input_file)?;
        fs::remove_file(&output_file)?;
        Ok(())
    }

    #[test]
    fn test_runtime_error() -> io::Result<()> {
        env::set_var("ANA_WORK_DIR", env::temp_dir());
        env::set_var("ANA_JUDGE_ID", "test_runtime_error");

        let input_file =
            path::Path::new(&env::var("ANA_WORK_DIR").unwrap()).join("test_runtime_error.in");
        let output_file =
            path::Path::new(&env::var("ANA_WORK_DIR").unwrap()).join("test_runtime_error.out");
        fs::File::create(&input_file)?.write_all(b"exit 1")?;
        match launch(
            &path::Path::new("bash"),
            &input_file,
            &output_file,
            1000000,          // 1 Sec
            64 * 1024 * 1024, // 64 Mb
        )?
        .status
        {
            LaunchResult::RE => {}
            _ => panic!("Failed when test runtime error"),
        }
        fs::remove_file(&input_file)?;
        fs::remove_file(&output_file)?;
        Ok(())
    }
}
