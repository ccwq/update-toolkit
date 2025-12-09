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
    log_line("启动 updater.exe");

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        log_line("参数错误：期待 updater.exe <updateAsar> <appAsar> <executable>");
        return 1;
    }

    let update_asar = Path::new(&args[1]);
    let app_asar = Path::new(&args[2]);
    let executable = Path::new(&args[3]);

    log_line(&format!(
        "接收到参数: update_asar={:?}, app_asar={:?}, executable={:?}",
        update_asar, app_asar, executable
    ));

    // 短暂等待，避免主进程尚未完全退出时文件被占用
    thread::sleep(Duration::from_millis(700));

    if !update_asar.exists() {
        log_line("update_asar 不存在，退出");
        return 2;
    }

    if let Err(err) = copy_with_retry(update_asar, app_asar, 3, Duration::from_millis(300)) {
        log_line(&format!(
            "复制 asar 失败: {} -> {} | {}",
            update_asar.display(),
            app_asar.display(),
            err
        ));
        return 2;
    }

    log_line("复制成功，准备重启主程序");

    if let Err(err) = launch_executable(executable) {
        log_line(&format!(
            "重启主程序失败: {} | {}",
            executable.display(),
            err
        ));
        return 3;
    }

    log_line("updater 执行完成");
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

    // 将工作目录切到主程序所在目录，避免相对路径依赖（dll/资源）找不到
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
        log_line(&format!("设置工作目录: {:?}", dir));
    } else {
        log_line("未能解析主程序目录，沿用当前目录启动");
    }

    // 使用 CREATE_NO_WINDOW 避免唤起控制台窗口
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    log_line(&format!("启动主程序: {:?}", exe));

    cmd.spawn().map(|_| ()).map_err(|err| {
        // 追加 OS 错误码方便排查
        if let Some(code) = err.raw_os_error() {
            log_line(&format!("启动失败，os_error={}", code));
        }
        err
    })
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
