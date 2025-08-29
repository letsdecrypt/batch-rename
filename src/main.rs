use clap::{Parser, Subcommand};
use std::fs;
use std::io;
use std::path::{Path};

/// 批量重命名工具
#[derive(Parser)]
#[command(about, author, version)]
struct Cli {
    /// 目标目录路径（默认为当前目录）
    #[clap(short, long, default_value = ".")]
    directory: String,

    /// 显示详细信息
    #[clap(short, long)]
    verbose: bool,

    /// 执行操作的子命令
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 删除文件名中的指定字符串
    Remove {
        /// 要删除的字符串
        #[clap(help = "要从文件名中删除的字符串")]
        pattern: String,
    },

    /// 替换文件名中的字符串
    Replace {
        /// 要替换的原字符串
        #[clap(help = "要被替换的原字符串")]
        old: String,

        /// 替换后的新字符串
        #[clap(help = "替换后的新字符串")]
        new: String,
    },

    /// 为文件名添加前缀
    AddPrefix {
        /// 要添加的前缀
        #[clap(help = "要添加到文件名开头的前缀")]
        prefix: String,
    },

    /// 为文件名添加后缀
    AddSuffix {
        /// 要添加的后缀
        #[clap(help = "要添加到文件名末尾的后缀")]
        suffix: String,
    },

    /// 使用正则表达式替换
    RegexReplace {
        /// 正则表达式模式
        #[clap(help = "要匹配的正则表达式模式")]
        pattern: String,

        /// 替换字符串
        #[clap(help = "替换后的字符串")]
        replacement: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let target_dir = Path::new(&cli.directory);

    if !target_dir.exists() {
        return Err(format!("目录不存在: {:?}", target_dir).into());
    }

    if !target_dir.is_dir() {
        return Err(format!("指定路径不是目录: {:?}", target_dir).into());
    }

    if cli.verbose {
        println!("目标目录: {:?}", target_dir);
    }

    match &cli.command {
        Commands::Remove { pattern } => {
            if cli.verbose {
                println!("删除字符串: \"{}\"", pattern);
            }
            batch_rename(target_dir, |name| remove_string(name, pattern), cli.verbose)?;
        }
        Commands::Replace { old, new } => {
            if cli.verbose {
                println!("替换 \"{}\" 为 \"{}\"", old, new);
            }
            batch_rename(target_dir, |name| replace_string(name, old, new), cli.verbose)?;
        }
        Commands::AddPrefix { prefix } => {
            if cli.verbose {
                println!("添加前缀: \"{}\"", prefix);
            }
            batch_rename(target_dir, |name| add_prefix(name, prefix), cli.verbose)?;
        }
        Commands::AddSuffix { suffix } => {
            if cli.verbose {
                println!("添加后缀: \"{}\"", suffix);
            }
            batch_rename(target_dir, |name| add_suffix(name, suffix), cli.verbose)?;
        }
        Commands::RegexReplace { pattern, replacement } => {
            if cli.verbose {
                println!("正则替换: \"{}\" -> \"{}\"", pattern, replacement);
            }
            batch_rename(target_dir, |name| regex_replace(name, pattern, replacement), cli.verbose)?;
        }
    }

    Ok(())
}

fn batch_rename<F>(dir: &Path, rename_func: F, verbose: bool) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&str) -> String,
{
    let entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    if entries.is_empty() {
        println!("目录为空");
        return Ok(());
    }

    if verbose {
        println!("找到 {} 个文件/目录", entries.len());
    }

    let mut changes = Vec::new();

    // 预览所有更改
    for entry in &entries {
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                let new_name = rename_func(name_str);
                if new_name != name_str {
                    changes.push((name_str.to_string(), new_name, path.clone()));
                }
            }
        }
    }

    if changes.is_empty() {
        println!("没有需要更改的文件名");
        return Ok(());
    }

    println!("\n预览更改 (共 {} 个):", changes.len());
    for (old_name, new_name, _) in &changes {
        println!("  {} -> {}", old_name, new_name);
    }

    println!("\n确认执行更改吗? (y/N): ");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("操作已取消");
        return Ok(());
    }

    // 执行重命名
    let mut success_count = 0;
    let mut error_count = 0;

    for (old_name, new_name, old_path) in changes {
        let parent = old_path.parent().unwrap_or(dir);
        let new_path = parent.join(&new_name);

        match fs::rename(&old_path, &new_path) {
            Ok(_) => {
                if verbose {
                    println!("✓ {} -> {}", old_name, new_name);
                }
                success_count += 1;
            }
            Err(e) => {
                println!("✗ {} -> {} (错误: {})", old_name, new_name, e);
                error_count += 1;
            }
        }
    }

    println!("\n完成! 成功: {}, 失败: {}", success_count, error_count);
    Ok(())
}

fn remove_string(name: &str, pattern: &str) -> String {
    name.replace(pattern, "")
}

fn replace_string(name: &str, old: &str, new: &str) -> String {
    name.replace(old, new)
}

fn add_prefix(name: &str, prefix: &str) -> String {
    format!("{}{}", prefix, name)
}

fn add_suffix(name: &str, suffix: &str) -> String {
    if let Some(dot_index) = name.rfind('.') {
        let (base, ext) = name.split_at(dot_index);
        format!("{}{}{}", base, suffix, ext)
    } else {
        format!("{}{}", name, suffix)
    }
}

fn regex_replace(name: &str, pattern: &str, replacement: &str) -> String {
    match regex::Regex::new(pattern) {
        Ok(re) => re.replace_all(name, replacement).to_string(),
        Err(_) => name.to_string(), // 如果正则表达式无效，返回原名
    }
}