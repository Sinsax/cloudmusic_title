use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::time::Duration;

// 引入 chrono 库用于时间戳 (需要在 Cargo.toml 中添加 chrono = "0.4")
// use chrono::Local; // 注释掉，直接在main中使用::

// --- 错误处理枚举 ---
#[derive(Debug)]
enum MonitoringError {
    /// 外部命令执行失败 (例如 xdotool/xprop 不存在或权限问题)
    CommandExecution(String),
    /// I/O 错误 (例如写入文件失败)
    IoError(io::Error),
}

// 转换 io::Error 到 MonitoringError::IoError
impl From<io::Error> for MonitoringError {
    fn from(err: io::Error) -> Self {
        MonitoringError::IoError(err)
    }
}

// --- 窗口监控器结构体 ---
struct WindowMonitor {
    class_name: String,
    output_file: String,
    last_content: Option<String>,
}

impl WindowMonitor {
    /// 构造函数
    fn new(class_name: &str, output_file: &str) -> Self {
        WindowMonitor {
            class_name: class_name.to_string(),
            output_file: output_file.to_string(),
            last_content: None,
        }
    }

    /// 执行外部命令并返回标准输出
    fn execute_command(&self, program: &str, args: &[&str]) -> Result<String, MonitoringError> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| MonitoringError::CommandExecution(format!("Failed to execute {}: {}", program, e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            // 命令执行失败，可能是窗口未找到，但 Command::new/output 成功，我们将其作为 CommandExecution 错误返回
            Err(MonitoringError::CommandExecution(format!(
                "Command failed: {} {:?}. Stderr: {}",
                program,
                args,
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    /// 尝试获取窗口标题
    /// 成功获取返回 Some(title)，未找到窗口返回 None，执行命令出错返回 MonitoringError
    fn get_window_name(&self) -> Result<Option<String>, MonitoringError> {
        // 1. 获取第一个窗口ID
        let id_result = self.execute_command(
            "xdotool",
            &["search", "--classname", "--limit", "1", &self.class_name],
        );

        // 如果 xdotool 执行成功但找不到窗口，通常会返回 CommandExecution 错误
        let window_id = match id_result {
            Ok(id) if !id.is_empty() => id,
            // 捕获 CommandExecution 错误或空输出，视作未找到 (Ok(None))
            Err(MonitoringError::CommandExecution(_)) | Ok(_) => return Ok(None),
            Err(e) => return Err(e), // 其他更严重的错误 (如文件未找到等)
        };

        // 2. 使用窗口ID获取 WM_NAME
        let wm_name_output = self.execute_command("xprop", &["-id", &window_id, "WM_NAME"])?;

        // 3. 截取等号后内容
        if let Some((_, value)) = wm_name_output.split_once('=') {
            // 去除引号和首尾空白
            let trimmed_value = value.trim().trim_matches('"').to_string();
            Ok(Some(trimmed_value))
        } else {
            Ok(None)
        }
    }

    /// 封装写入逻辑：如果当前内容与上一次写入的内容不同，则写入文件并更新状态
    /// 返回值：True 表示发生了写入，False 表示内容未改变未写入
    fn write_content_if_changed(&mut self, current_content: &str) -> Result<bool, MonitoringError> {
        let needs_write = self.last_content.as_ref().map_or(true, |last| last != current_content);

        if needs_write {
            // 写入文件
            let mut file = File::create(&self.output_file)?;
            // 写入内容
            file.write_all(current_content.as_bytes())?;

            // 更新上一次写入的内容
            self.last_content = Some(current_content.to_string());

            Ok(true) // 发生了写入
        } else {
            Ok(false) // 内容未变
        }
    }
}

fn main() -> Result<(), MonitoringError> {
    use std::thread;

    // --- 配置常量 ---
    const CLASS_NAME: &str = "cloudmusic.exe";
    const OUTPUT_FILE: &str = "title.txt"; // <--- 已更新为 title.txt
    const CHECK_INTERVAL: u64 = 1;

    println!("👀 正在监控窗口类名: {}", CLASS_NAME);
    println!("💾 数据将写入文件: {}", OUTPUT_FILE);
    println!("⏱️ 检查间隔: {} 秒", CHECK_INTERVAL);

    // 创建监控器实例
    let mut monitor = WindowMonitor::new(CLASS_NAME, OUTPUT_FILE);
    let interval_duration = Duration::from_secs(CHECK_INTERVAL);

    // 主循环
    loop {
        // 1. 获取当前窗口标题
        let current_content = match monitor.get_window_name() {
            Ok(Some(name)) => name, // 成功获取到标题
            Ok(None) => String::new(), // 未找到窗口，按要求输出空字符
            Err(e) => {
                // 如果是 I/O 错误或 Command execution 失败（如 xprop 失败），打印错误，并继续使用空字符串
                eprintln!("❌ 错误：{:?}", e);
                String::new()
            }
        };

        // 2. 写入文件并更新状态
        let now = chrono::Local::now().format("%H:%M:%S");
        match monitor.write_content_if_changed(&current_content) {
            Ok(true) => {
                // 发生了写入，输出到控制台
                if current_content.is_empty() {
                    println!("[{}] 写入: <空字符> (程序未开启/获取失败)", now);
                } else {
                    println!("[{}] 写入: {}", now, current_content);
                }
            }
            Ok(false) => {
                // 内容未变，不写入也不输出
            }
            Err(e) => {
                eprintln!("❌ 写入文件错误: {:?}", e);
            }
        }

        // 3. 等待
        thread::sleep(interval_duration);
    }
}

// 依赖项 (Cargo.toml):
// [dependencies]
// chrono = "0.4"