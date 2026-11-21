use serde::Serialize;
use std::{fs, path::Path};

#[derive(Default, Serialize)]
pub struct HostMetrics {
    pub cpu_percent: Option<f64>,
    pub memory_used: Option<u64>,
    pub memory_total: Option<u64>,
    pub disk_free: Option<u64>,
    pub uptime: Option<f64>,
}

pub fn sample() -> HostMetrics {
    let memory = fs::read_to_string("/proc/meminfo").ok().map(|stat| {
        let value = |name: &str| {
            stat.lines()
                .find(|line| line.starts_with(name))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|number| number.parse::<u64>().ok())
                .map(|number| number * 1024)
        };
        let total = value("MemTotal:");
        let used = value("MemTotal:")
            .zip(value("MemAvailable:"))
            .map(|(total, available)| total.saturating_sub(available));
        (used, total)
    });
    let path = std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join(".bitcoin"))
        .unwrap_or_else(|| ".".into());
    HostMetrics {
        cpu_percent: cpu_sample(),
        memory_used: memory.and_then(|(used, _)| used),
        memory_total: memory.and_then(|(_, total)| total),
        disk_free: disk_free(&path),
        uptime: fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|value| value.split_whitespace().next()?.parse().ok()),
    }
}

fn cpu_sample() -> Option<f64> {
    fn counters() -> Option<(u64, u64)> {
        let line = fs::read_to_string("/proc/stat")
            .ok()?
            .lines()
            .next()?
            .to_owned();
        let values: Vec<u64> = line
            .split_whitespace()
            .skip(1)
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        let idle = *values.get(3)? + values.get(4).copied().unwrap_or(0);
        Some((values.iter().sum(), idle))
    }
    let first = counters()?;
    std::thread::sleep(std::time::Duration::from_millis(80));
    let second = counters()?;
    let total = second.0.saturating_sub(first.0);
    let idle = second.1.saturating_sub(first.1);
    (total > 0).then(|| (total.saturating_sub(idle) as f64 / total as f64) * 100.0)
}

#[cfg(target_os = "linux")]
fn disk_free(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    let path = CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // Add libc as a direct dependency rather than invoking a shell utility.
    let rc = unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let stats = unsafe { stats.assume_init() };
    Some(stats.f_bavail.saturating_mul(stats.f_frsize))
}
#[cfg(not(target_os = "linux"))]
fn disk_free(_: &Path) -> Option<u64> {
    None
}
