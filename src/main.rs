//! 编写一个cli工具，用于对文件进行重命名为uuid

use clap::ArgAction::SetTrue;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use uuid::Uuid;

mod build_info;

fn main() -> Result<(), RenameError> {
    let config = AppConfig::from_args();
    FileRenamer::process(&config)
}

#[derive(Debug)]
enum RenameError {
    #[allow(unused)]
    Io(std::io::Error),
    InvalidPath,
    NoParentDirectory,
}

impl From<std::io::Error> for RenameError {
    fn from(error: std::io::Error) -> Self {
        RenameError::Io(error)
    }
}

///命令行参数解析结果
struct AppConfig {
    ///目标路径
    target_path: PathBuf,
    ///是否递归
    recursive: bool,
}

impl AppConfig {
    /// 从命令行参数初始化配置
    fn from_args() -> Self {
        let matches = clap::Command::new("uuid-renamer")
            .version("1.0")
            .author("Your Name")
            .about("批量重命名文件为UUID格式，只对文件进行重命名，不会修改目录名")
            .arg(
                clap::Arg::new("path")
                    .help("目标路径")
                    .required(true)
                    .index(1),
            )
            // 获得参数后将值设置为true
            .arg(
                clap::Arg::new("recursive")
                    .short('r')
                    .long("recursive")
                    .help("递归处理子目录")
                    .action(SetTrue),
            )
            .get_matches();

        AppConfig {
            target_path: PathBuf::from(matches.get_one::<String>("path").unwrap()),
            recursive: matches.get_flag("recursive"),
        }
    }
}

/// 文件重命名器
struct FileRenamer;

impl FileRenamer {
    /// 处理文件
    pub fn process(config: &AppConfig) -> Result<(), RenameError> {
        let path = &config.target_path;
        if !path.exists() {
            return Err(RenameError::InvalidPath);
        }

        if path.is_file() {
            Self::rename_file(path)?
        } else {
            Self::process_directory(path, config.recursive)?
        }
        Ok(())
    }
    /// 对文件进行重命名
    fn rename_file(path: &Path) -> Result<(), RenameError> {
        let new_name = Self::generate_new_name(path)?;
        let parent = path.parent().ok_or(RenameError::NoParentDirectory)?;
        let new_path = parent.join(new_name);

        println!("正在重命名:{:?} -> {:?}", path, new_path);
        std::fs::rename(path, new_path)?;
        Ok(())
    }
    /// 对目录进行处理
    fn process_directory(dir: &Path, recursive: bool) -> Result<(), RenameError> {
        // 检查目录是否存在
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                //如果是一个目录，看看是否需要递归处理内容
                if recursive {
                    Self::process_directory(&path, recursive)?;
                }
            } else if path.is_file() {
                // 是一个文件就直接处理
                Self::rename_file(&path)?;
            }
        }
        Ok(())
    }
    /// 生成新的文件名
    fn generate_new_name(path: &Path) -> Result<String, RenameError> {
        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .map(|s| format!("{}", s))
            .unwrap_or_default();

        Ok(format!("{}.{}", Self::format_uuid(), extension))
    }
    /// 生成uuid
    fn format_uuid() -> String {
        Uuid::new_v4().simple().to_string().to_uppercase()
    }
}
