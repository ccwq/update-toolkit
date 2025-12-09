# update-toolkit

极简的 Windows GUI 子系统版 `updater.exe`，用于替换 `electron-asar-hot-updater` 自带的控制台版本，避免更新时出现黑色 CMD 弹窗。

## 功能
- 接收参数：`updater.exe <updateAsar> <appAsar> <executable>`
- 等待主进程退出后，将新 asar 覆盖旧 asar，并拉起主程序。
- 无控制台窗口（GUI 子系统），可选写入 `updater.log` 便于排查。

## 代码结构
- `src/main.rs`：核心逻辑（参数校验、复制重试、启动主程序）。
- `Cargo.toml`：仅依赖标准库，无三方依赖。

## 本地构建（Windows/MSVC）
```bash
cargo build --release
# 产物：target/release/update-toolkit.exe
```

## 通过容器跨编译（Linux -> Windows GNU）
需要 Docker/Podman：
```bash
cd update-toolkit
docker compose up --build
# 产物：target/x86_64-pc-windows-gnu/release/update-toolkit.exe
```

## 集成到桌面端
1) 将生成的 `update-toolkit.exe` 重命名为 `updater.exe`。
2) 覆盖到 `node-libs/electron-asar-hot-updater/updater.exe`（并同步到 `node_modules/...`，再重新打包 Electron）。
3) 保持 `electron/main/update2.ts` 使用本地补丁版 spawn（直接启动 `updater.exe`，无 cmd）。

## 建议的 Git 子仓库化流程
> 说明：当前环境无法直接推送到 GitHub，仅提供操作指引。
1) 在 `update-toolkit/` 目录初始化独立仓库：`git init`，添加 `.gitignore`（可选）。
2) 提交代码：`git add . && git commit -m "init: updater toolkit"`。
3) 在 GitHub 创建仓库（如 `update-toolkit`），添加远程并推送：  
   `git remote add origin <YOUR_REPO_URL>`  
   `git push -u origin main`
4) 在上层项目中以子模块方式引入：  
   `git submodule add <YOUR_REPO_URL> update-toolkit`

## 注意
- `RUSTFLAGS=-C link-args=-Wl,--subsystem,windows` 由 docker-compose 预设，保证无控制台窗口；如在本机 MSVC 构建，可在代码层面已有 `windows_subsystem` 属性，无需额外设置。
- 若需日志，请确保运行目录可写；否则日志静默失败但不影响更新流程。***
