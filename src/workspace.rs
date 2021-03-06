/// Workspace on tmpfs
///
/// TODO: Use fuse rewrite in the future for
/// better performance and less memory usage
use std::fs::{create_dir, write};
use std::io;
use std::path::{Path, PathBuf};

use liboj::structures::*;
use nix::mount::{mount, umount, MsFlags};
use tempfile::{tempdir, TempDir};

pub struct Workspace {
    inner: TempDir,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace::default()
    }
}

impl Default for Workspace {
    fn default() -> Workspace {
        let workspace = Workspace {
            inner: tempdir().expect("Failed to create a temp dir"),
        };

        mount(
            Option::<&Path>::None,
            workspace.as_path(),
            Some("tmpfs"),
            MsFlags::empty(),
            Some("mode=0700"),
        )
        .expect("Failed to mount Tmpfs on workspace");

        create_dir(&workspace.runtime_dir()).expect("Failed to create runtime directory");
        mount(
            Option::<&Path>::None,
            &workspace.runtime_dir(),
            Some("tmpfs"),
            MsFlags::empty(),
            Some("size=64m,mode=0700"),
        )
        .expect("Failed to mount Tmpfs on runtime dir");

        workspace
    }
}

impl AsRef<Path> for Workspace {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl Workspace {
    fn as_path(&self) -> &Path {
        self.inner.path()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        umount(self.runtime_dir().as_path()).unwrap();
        umount(self.inner.path()).unwrap();
    }
}

pub trait WorkDir {
    fn source_file(&self) -> PathBuf;
    fn problem_dir(&self) -> PathBuf;
    fn runtime_dir(&self) -> PathBuf;

    fn prepare_task(&self, task: &Task) -> io::Result<()>;
}

pub trait RuntimeDir {
    fn executable_file(&self) -> PathBuf;
}

pub trait ProblemDir {
    fn test_case_dirs(&self) -> Vec<PathBuf>;

    fn prepare_problem(&self, problem: &Problem) -> io::Result<()>;
}

pub trait SpecialJudgeProblemDir: ProblemDir {
    fn spj_file(&self) -> PathBuf;
    fn spj_source(&self) -> PathBuf;

    fn prepare_special_judge_problem(&self, problem: &Problem) -> io::Result<()>;
}

pub trait TestCaseDir {
    fn input_file(&self) -> PathBuf;
    fn output_file(&self) -> PathBuf;
    fn answer_file(&self) -> PathBuf;

    fn prepare_test_case(&self, test_case: &TestCase) -> io::Result<()>;
}

impl WorkDir for Workspace {
    fn source_file(&self) -> PathBuf {
        self.inner.path().join("source")
    }

    fn problem_dir(&self) -> PathBuf {
        self.inner.path().join("problem")
    }

    fn runtime_dir(&self) -> PathBuf {
        self.inner.path().join("runtime")
    }

    fn prepare_task(&self, task: &Task) -> io::Result<()> {
        create_dir(self.problem_dir())?;
        write(self.source_file(), &task.source.code)?;
        self.problem_dir().prepare_problem(&task.problem)?;
        Ok(())
    }
}

impl ProblemDir for Path {
    fn test_case_dirs(&self) -> Vec<PathBuf> {
        let mut res = Vec::new();
        for i in 0.. {
            let test_case_dir = self.join(i.to_string());
            if test_case_dir.exists() {
                res.push(test_case_dir);
            } else {
                break;
            }
        }
        res
    }

    fn prepare_problem(&self, problem: &Problem) -> io::Result<()> {
        match problem {
            Problem::Normal { cases, .. } => {
                for (i, test_case) in cases.iter().enumerate() {
                    let test_case_dir = self.join(i.to_string());
                    create_dir(&test_case_dir)?;
                    test_case_dir.prepare_test_case(&test_case)?
                }
            }
            Problem::Special { cases, .. } => {
                self.prepare_special_judge_problem(problem)?;
                for (i, test_case) in cases.iter().enumerate() {
                    let test_case_dir = self.join(i.to_string());
                    create_dir(&test_case_dir)?;
                    test_case_dir.prepare_test_case(&test_case)?;
                }
            }
        }
        Ok(())
    }
}

impl SpecialJudgeProblemDir for Path {
    fn spj_file(&self) -> PathBuf {
        self.join("spj")
    }

    fn spj_source(&self) -> PathBuf {
        self.join("spj")
    }

    fn prepare_special_judge_problem(&self, problem: &Problem) -> io::Result<()> {
        if let Problem::Special { spj, .. } = problem {
            write(self.spj_source(), &spj.code)?;
        }
        Ok(())
    }
}

impl RuntimeDir for Path {
    fn executable_file(&self) -> PathBuf {
        self.join("main")
    }
}

impl TestCaseDir for Path {
    fn input_file(&self) -> PathBuf {
        self.join("input")
    }

    fn output_file(&self) -> PathBuf {
        self.join("output")
    }

    fn answer_file(&self) -> PathBuf {
        self.join("answer")
    }

    fn prepare_test_case(&self, test_case: &TestCase) -> io::Result<()> {
        write(self.input_file(), &test_case.input)?;
        write(self.answer_file(), &test_case.answer)?;
        Ok(())
    }
}
