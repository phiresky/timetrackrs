use crate::prelude::*;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

pub trait ProgressReporter: Debug + Send + Sync {
    fn report(&self, p: Vec<ProgressState>);
}

#[derive(Debug)]
enum Mode {
    Root(Arc<dyn ProgressReporter>),
    Child(Progress),
}
#[derive(Debug)]
struct ProgressInner {
    parent: Mode,
    state: ProgressState,
}
#[derive(Clone, Debug)]
pub struct Progress {
    inner: Arc<RwLock<ProgressInner>>,
}

#[derive(Clone, Debug, Serialize, TypeScriptify)]
pub struct ProgressState {
    desc: String,
    current: i64,
    total: Option<i64>,
}
impl Drop for ProgressInner {
    fn drop(&mut self) {
        // println!("dropping progress {:?}", self);
        match &self.parent {
            Mode::Root(r) => {
                println!("reporting root done");
                r.report(vec![]);
            }
            Mode::Child(parent) => {
                parent.report();
            }
        }
    }
}

impl Progress {
    pub fn root(reporter: Arc<dyn ProgressReporter>) -> Progress {
        Progress::new(Mode::Root(reporter))
    }
    fn new(parent: Mode) -> Progress {
        Progress {
            inner: Arc::new(RwLock::new(ProgressInner {
                parent,
                state: ProgressState {
                    current: 0,
                    total: None,
                    desc: "".to_string(),
                },
            })),
        }
    }
    pub fn child(
        &self,
        current: i64,
        total: impl Into<Option<i64>>,
        desc: impl Into<String>,
    ) -> Progress {
        self.update(current, total, desc);
        Progress::new(Mode::Child(self.clone()))
    }
    /*fn get_parent(&self) -> Option<&Progress> {
        self.inner.read().unwrap().parent.as_ref()
    }*/
    pub fn update(&self, current: i64, total: impl Into<Option<i64>>, desc: impl Into<String>) {
        {
            let mut p = self.inner.write().unwrap();
            p.state = ProgressState {
                current,
                total: total.into(),
                desc: desc.into(),
            };
        }
        self.report();
    }
    pub fn inc(&self, desc: impl Into<String>) {
        {
            let mut p = self.inner.write().unwrap();
            p.state.current += 1;
            p.state.desc = desc.into();
        }
        self.report();
    }
    pub fn child_inc(&self, desc: impl Into<String>) -> Progress {
        self.inc(desc);
        Progress::new(Mode::Child(self.clone()))
    }
    fn report(&self) {
        let (reporter, state) = self.get_full_state();
        reporter.report(state);
    }
    fn get_full_state(&self) -> (Arc<dyn ProgressReporter>, Vec<ProgressState>) {
        let mut state = match &self.inner.read().unwrap().parent {
            Mode::Root(r) => (r.clone(), vec![]),
            Mode::Child(parent) => parent.get_full_state(),
        };
        state.1.push(self.inner.read().unwrap().state.clone());
        state
    }
}

#[derive(Debug)]
#[allow(unused)]
struct TerminalReporter {}
impl ProgressReporter for TerminalReporter {
    fn report(&self, state: Vec<ProgressState>) {
        // print!("{}", ansi_escapes::CursorUp((state.len()) as u16));

        for (i, state) in state.iter().enumerate() {
            let prog_str = if let Some(total) = state.total {
                format!(
                    "{:.0}% ({}/{})",
                    state.current as f64 * 100.0 / (total as f64),
                    state.current,
                    total
                )
            } else {
                format!("{}", state.current)
            };
            println!(
                "{}{}: {}         ",
                (0..=2 * i).map(|_| " ").collect::<String>(), // indent
                state.desc,
                prog_str
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rand::Rng;
    use tokio::time::sleep;

    use super::*;

    async fn scan_files(dir: &str, progress: Progress) -> Vec<String> {
        let mut files = vec![];
        let max = 142;

        for i in 1..max {
            let filename = format!("{}/files/{}/{}", dir, i / 35, i % 35);
            progress.update(i, Some(max), format!("Scanning file {filename}"));
            sleep(Duration::from_millis(20)).await;
            files.push(filename);
        }
        files
    }

    #[tokio::test]
    async fn test_progress() {
        println!("Test progress");
        let directory = "/foo";
        let root = Progress::root(Arc::new(TerminalReporter {}));
        let progress = Progress::child(&root, 0, Some(1), "Doing Stuff");

        let files = scan_files(directory, progress.child(0, Some(2), "Scanning directory")).await;

        let copy_prog = progress.child(1, Some(2), "Copying");
        let mut rng = rand::thread_rng();

        for (i, file) in files.iter().enumerate() {
            // ...

            copy_prog.update(
                i as i64,
                Some(files.len() as i64),
                format!("Copying {file}"),
            );
            sleep(Duration::from_millis(rng.gen_range(400..1000))).await;
        }
    }
}

/* fn main() {

}


fn scan_files(directory: &Path, progress: Progress) {
    for (i, file) in directory.enumerate() {
        ...
        progress.report(i, None, format!("found {}", file));
    }
    return ...
}*/
