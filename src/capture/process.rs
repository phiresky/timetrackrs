use crate::prelude::*;
use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;

#[derive(Debug, Serialize, Deserialize, TypeScriptify, Clone)]
#[allow(non_snake_case)]
pub struct ProcessData {
    pub pid: i32,
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: Option<String>,
    pub cwd: Option<String>,
    pub memory_kB: i64,
    pub parent: Option<i32>,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub cpu_usage: Option<f32>, // can be NaN -> null
}

pub fn get_process_data(system: &mut sysinfo::System, pid: usize) -> Option<ProcessData> {
    system.refresh_process(sysinfo::Pid::from(pid));
    if let Some(procinfo) = system.process(sysinfo::Pid::from(pid)) {
        Some(ProcessData {
            pid: procinfo.pid().as_u32() as i32,
            name: procinfo.name().to_string(),
            cmd: procinfo.cmd().to_vec(),
            exe: procinfo
                .exe()
                .map(|path| path.to_string_lossy().to_string()), // tbh i don't care if your executables have filenames that are not unicode
            cwd: procinfo
                .cwd()
                .map(|path| path.to_string_lossy().to_string()),
            memory_kB: (procinfo.memory() / 1024) as i64,
            parent: procinfo.parent().map(|p| p.as_u32() as i32),
            status: procinfo.status().to_string().to_string(),
            start_time: util::unix_epoch_millis_to_date((procinfo.start_time() as i64) * 1000),
            cpu_usage: Some(procinfo.cpu_usage()),
        })
    } else {
        None
    }
}
