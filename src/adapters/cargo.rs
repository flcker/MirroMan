use crate::adapters::PackageManagerAdapter;
use crate::config::Config;
use crate::mirror::Mirror;
use crate::utils::os::has_executable;
use crate::utils::paths::expand_tilde;
use anyhow::{Context, Result};
use std::fs;
use std::time::Instant;

pub struct CargoAdapter {
    mirrors: Vec<Mirror>,
}

impl CargoAdapter {
    pub fn new(config: &Config) -> Self {
        Self {
            mirrors: config.cargo.mirrors.clone(),
        }
    }

    /// Cargo 配置文件路径
    fn config_path() -> std::path::PathBuf {
        expand_tilde("~/.cargo/config.toml")
    }

    /// 拼写 cargo config 格式
    fn build_cargo_config(mirror_name: &str, mirror_url: &str) -> String {
        let index_url = if mirror_name.contains("官方") || mirror_name.contains("crates.io") {
            "https://github.com/rust-lang/crates.io-index".to_string()
        } else if mirror_url.contains(".git") {
            mirror_url.to_string()
        } else {
            // rsproxy 使用 sparse 协议
            if mirror_url.contains("sparse+") {
                mirror_url.to_string()
            } else if mirror_url.contains("rsproxy") {
                "sparse+https://rsproxy.cn/index/".to_string()
            } else {
                format!("{}/crates.io-index", mirror_url.trim_end_matches('/'))
            }
        };

        format!(
            r#"[source.crates-io]
replace-with = 'mirror'

[source.mirror]
registry = "{index_url}"

[registries.mirror]
index = "{index_url}"
"#
        )
    }
}

impl PackageManagerAdapter for CargoAdapter {
    fn name(&self) -> &'static str {
        "Cargo"
    }

    fn list_mirrors(&self) -> Vec<Mirror> {
        self.mirrors.clone()
    }

    fn switch_mirror(&self, mirror: &Mirror) -> Result<()> {
        let path = Self::config_path();

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建目录: {}", parent.display()))?;
        }

        let new_content = Self::build_cargo_config(&mirror.name, &mirror.url);

        // 如果已有配置文件，追加/替换 cargo source 配置
        let final_content = if path.exists() {
            let existing = fs::read_to_string(&path)
                .with_context(|| format!("无法读取: {}", path.display()))?;
            merge_cargo_config(&existing, &new_content)
        } else {
            new_content
        };

        fs::write(&path, &final_content)
            .with_context(|| format!("无法写入: {}", path.display()))?;

        Ok(())
    }

    fn test_mirror(&self, mirror: &Mirror) -> Result<u64> {
        let start = Instant::now();

        // 对镜像 URL 发送 HTTP HEAD 请求
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .context("无法创建 HTTP 客户端")?;

        let test_url = if mirror.url.contains("rsproxy") {
            "https://rsproxy.cn".to_string()
        } else if mirror.url.starts_with("sparse+") {
            mirror.url.trim_start_matches("sparse+").to_string()
        } else {
            mirror.url.clone()
        };

        client.head(&test_url).send().context("镜像不可达")?;
        Ok(start.elapsed().as_millis() as u64)
    }

    fn is_available(&self) -> bool {
        has_executable("cargo")
    }

    fn supported_platforms(&self) -> &'static str {
        "Win / Mac / Linux"
    }

    fn current_mirror_name(&self) -> Option<String> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path).ok()?;

        // 查找 replace-with = 'mirror' 对应的 registry URL
        let registry_url = content
            .lines()
            .find(|line| line.trim().starts_with("registry ="))
            .and_then(|line| {
                let v = line.split('"').nth(1).or_else(|| line.split('\'').nth(1));
                v.map(|s| s.to_string())
            })?;

        // 与 mirrors 中的 URL 比对
        self.mirrors.iter().find(|m| m.url == registry_url).map(|m| m.name.clone())
    }
}

/// 清理所有 [source.*] 和 [registries.*] 段，追加新的 source 配置
fn merge_cargo_config(existing: &str, new_source_section: &str) -> String {
    let mut result = String::new();
    let mut skip = false;

    for line in existing.lines() {
        let trimmed = line.trim();

        // 遇到 [source.*] 或 [registries.*] 开始跳过
        if trimmed.starts_with("[source.") || trimmed.starts_with("[registries.") {
            skip = true;
            continue;
        }

        // 遇到其他 [xxx] 段（非 source/registries），停止跳过
        if skip && trimmed.starts_with('[') {
            skip = false;
        }

        if !skip {
            result.push_str(line);
            result.push('\n');
        }
    }

    // 确保末尾有换行
    if !result.ends_with('\n') {
        result.push('\n');
    }

    result.push('\n');
    result.push_str(new_source_section);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_cargo_config_no_existing_sources() {
        let existing = "[build]\nrustflags = [\"-D\", \"warnings\"]\n";
        let new_source = "[source.crates-io]\nreplace-with = 'mirror'\n";
        let merged = merge_cargo_config(existing, new_source);
        assert!(merged.contains("[build]"));
        assert!(merged.contains("[source.crates-io]"));
    }

    #[test]
    fn test_merge_removes_old_source_sections() {
        let existing = "[build]\nrustflags = [\"-D\"]\n\n[source.rsproxy]\nregistry = \"old\"\n\n[registries.rsproxy]\nindex = \"old\"\n\n[other]\nkeep = true\n";
        let new_source = "[source.crates-io]\nreplace-with = 'mirror'\n";
        let merged = merge_cargo_config(existing, new_source);
        assert!(merged.contains("[build]"));
        assert!(merged.contains("[other]"));
        assert!(merged.contains("[source.crates-io]"));
        assert!(!merged.contains("[source.rsproxy]"));
        assert!(!merged.contains("[registries.rsproxy]"));
    }

    #[test]
    fn test_cargo_adapter_basics() {
        // 在 dev 环境下 cargo 应该存在
        let adapter = CargoAdapter {
            mirrors: vec![Mirror::new("test", "https://example.com")],
        };
        assert!(adapter.is_available());
        assert_eq!(adapter.name(), "Cargo");
        assert_eq!(adapter.list_mirrors().len(), 1);
    }
}
