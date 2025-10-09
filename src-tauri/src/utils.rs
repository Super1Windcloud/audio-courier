use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

pub fn is_dev() -> bool {
    cfg!(debug_assertions)
}

pub fn write_some_log(msg: &str) {
    #[cfg(target_os = "macos")]
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        writeln!(file, "{}", msg).unwrap();
    }

    #[cfg(target_os = "windows")]
    {
        let mut file = OpenOptions::new()
            .create(true) // 文件不存在则创建
            .append(true) // 追加写入
            .open("app.log") // 日志文件名
            .unwrap();

        writeln!(file, "{}", msg).unwrap(); // 写入一行
    }
}

pub fn load_env_variables() {
    const ENV_CONTENT: &str = include_str!("../.env");

    let mut vars: HashMap<String, String> = HashMap::new();

    for line in ENV_CONTENT.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = parse_line(line) {
            vars.insert(key, value);
        }
    }

    for (key, value) in vars {
        env::set_var(key, value);
    }
}

fn parse_line(line: &str) -> Option<(String, String)> {
    if let Some(eq_pos) = line.find('=') {
        let key = line[0..eq_pos].trim().to_string();
        let value = line[eq_pos + 1..].trim().to_string();
        Some((key, value))
    } else {
        None
    }
}
