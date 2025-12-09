//! 极简 updater：无控制台窗口，负责覆盖 app.asar 并重启主程序。
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::{
    env,
    fs,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000; // CREATE_NO_WINDOW

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        log_line("参数错误：期待 updater.exe <updateAsar> <appAsar> <executable>");
        return 1;
    }

    let update_asar = Path::new(&args[1]);
    let app_asar = Path::new(&args[2]);
    let executable = Path::new(&args[3]);

    // 短暂等待，避免主进程尚未完全退出时文件被占用
    thread::sleep(Duration::from_millis(700));

    if let Err(err) = copy_with_retry(update_asar, app_asar, 3, Duration::from_millis(300)) {
        log_line(&format!(
            "复制 asar 失败: {} -> {} | {}",
            update_asar.display(),
            app_asar.display(),
            err
        ));
        return 2;
    }

    if let Err(err) = launch_executable(executable) {
        log_line(&format!(
            "重启主程序失败: {} | {}",
            executable.display(),
            err
        ));
        return 3;
    }

    0
}

fn copy_with_retry(
    src: &Path,
    dest: &Path,
    retries: usize,
    gap: Duration,
) -> io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    for attempt in 0..=retries {
        match fs::copy(src, dest) {
            Ok(_) => return Ok(()),
            Err(err) if attempt < retries => {
                log_line(&format!(
                    "复制失败，重试 {}/{}: {}",
                    attempt + 1,
                    retries,
                    err
                ));
                thread::sleep(gap);
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn launch_executable(exe: &Path) -> io::Result<()> {
    let mut cmd = Command::new(exe);
    // 使用 CREATE_NO_WINDOW 避免唤起控制台窗口
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map(|_| ())
}

fn log_line(msg: &str) {
    if let Some(mut file) = open_log_file() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(file, "[{}] {}", ts, msg);
    }
}

fn open_log_file() -> Option<fs::File> {
    let path = current_dir_log_path();
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
}

fn current_dir_log_path() -> PathBuf {
    let fallback = PathBuf::from("updater.log");
    let exe_path = env::current_exe().ok();
    exe_path
        .as_ref()
        .and_then(|p| p.parent().map(|dir| dir.join("updater.log")))
        .unwrap_or(fallback)
}
