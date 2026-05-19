//! Registry hardcoded mapeando linguagem -> LSP server -> manifests -> install command.
//!
//! Cada entry descreve um servidor LSP suportado oficialmente pela v0.6.0:
//! - quais manifests (go.mod, Cargo.toml, etc) indicam que o projeto usa essa stack
//! - qual binario invocar
//! - como instalar (por OS)
//! - extensoes de arquivo associadas

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerId {
    RustAnalyzer,
    Gopls,
    Pyright,
    TypeScriptLanguageServer,
    Intelephense,
    Clangd,
    RubyLsp,
    LuaLanguageServer,
}

impl ServerId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerId::RustAnalyzer => "rust-analyzer",
            ServerId::Gopls => "gopls",
            ServerId::Pyright => "pyright",
            ServerId::TypeScriptLanguageServer => "typescript-language-server",
            ServerId::Intelephense => "intelephense",
            ServerId::Clangd => "clangd",
            ServerId::RubyLsp => "ruby-lsp",
            ServerId::LuaLanguageServer => "lua-language-server",
        }
    }

    pub fn all() -> &'static [ServerId] {
        &[
            ServerId::RustAnalyzer,
            ServerId::Gopls,
            ServerId::Pyright,
            ServerId::TypeScriptLanguageServer,
            ServerId::Intelephense,
            ServerId::Clangd,
            ServerId::RubyLsp,
            ServerId::LuaLanguageServer,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSpec {
    pub id: ServerId,
    pub name: &'static str,
    pub language: &'static str,
    pub binary: &'static str,
    pub args: Vec<&'static str>,
    pub manifests: Vec<&'static str>,
    pub extensions: Vec<&'static str>,
    pub install_cmd_linux: &'static str,
    pub install_cmd_macos: &'static str,
    pub install_cmd_windows: &'static str,
}

pub fn spec(id: ServerId) -> ServerSpec {
    match id {
        ServerId::RustAnalyzer => ServerSpec {
            id,
            name: "rust-analyzer",
            language: "Rust",
            binary: "rust-analyzer",
            args: vec![],
            manifests: vec!["Cargo.toml"],
            extensions: vec!["rs"],
            install_cmd_linux: "rustup component add rust-analyzer",
            install_cmd_macos: "rustup component add rust-analyzer",
            install_cmd_windows: "rustup component add rust-analyzer",
        },
        ServerId::Gopls => ServerSpec {
            id,
            name: "gopls",
            language: "Go",
            binary: "gopls",
            args: vec![],
            manifests: vec!["go.mod"],
            extensions: vec!["go"],
            install_cmd_linux: "go install golang.org/x/tools/gopls@latest",
            install_cmd_macos: "go install golang.org/x/tools/gopls@latest",
            install_cmd_windows: "go install golang.org/x/tools/gopls@latest",
        },
        ServerId::Pyright => ServerSpec {
            id,
            name: "pyright",
            language: "Python",
            binary: "pyright-langserver",
            args: vec!["--stdio"],
            manifests: vec!["pyproject.toml", "setup.py", "requirements.txt", "Pipfile"],
            extensions: vec!["py", "pyi"],
            install_cmd_linux: "npm install -g pyright",
            install_cmd_macos: "npm install -g pyright",
            install_cmd_windows: "npm install -g pyright",
        },
        ServerId::TypeScriptLanguageServer => ServerSpec {
            id,
            name: "typescript-language-server",
            language: "TypeScript/JavaScript",
            binary: "typescript-language-server",
            args: vec!["--stdio"],
            manifests: vec!["package.json", "tsconfig.json", "jsconfig.json"],
            extensions: vec!["ts", "tsx", "js", "jsx", "mjs", "cjs"],
            install_cmd_linux: "npm install -g typescript-language-server typescript",
            install_cmd_macos: "npm install -g typescript-language-server typescript",
            install_cmd_windows: "npm install -g typescript-language-server typescript",
        },
        ServerId::Intelephense => ServerSpec {
            id,
            name: "intelephense",
            language: "PHP",
            binary: "intelephense",
            args: vec!["--stdio"],
            manifests: vec!["composer.json"],
            extensions: vec!["php"],
            install_cmd_linux: "npm install -g intelephense",
            install_cmd_macos: "npm install -g intelephense",
            install_cmd_windows: "npm install -g intelephense",
        },
        ServerId::Clangd => ServerSpec {
            id,
            name: "clangd",
            language: "C/C++",
            binary: "clangd",
            args: vec![],
            manifests: vec!["CMakeLists.txt", "compile_commands.json", "Makefile"],
            extensions: vec!["c", "cc", "cpp", "cxx", "h", "hh", "hpp"],
            install_cmd_linux: "apt install clangd  # ou: dnf install clang-tools-extra",
            install_cmd_macos: "brew install llvm",
            install_cmd_windows: "winget install LLVM.LLVM",
        },
        ServerId::RubyLsp => ServerSpec {
            id,
            name: "ruby-lsp",
            language: "Ruby",
            binary: "ruby-lsp",
            args: vec![],
            manifests: vec!["Gemfile", "*.gemspec"],
            extensions: vec!["rb", "rake"],
            install_cmd_linux: "gem install ruby-lsp",
            install_cmd_macos: "gem install ruby-lsp",
            install_cmd_windows: "gem install ruby-lsp",
        },
        ServerId::LuaLanguageServer => ServerSpec {
            id,
            name: "lua-language-server",
            language: "Lua",
            binary: "lua-language-server",
            args: vec![],
            manifests: vec![".luarc.json", ".luarc.jsonc"],
            extensions: vec!["lua"],
            install_cmd_linux:
                "apt install lua-language-server  # ou: snap install lua-language-server",
            install_cmd_macos: "brew install lua-language-server",
            install_cmd_windows: "winget install LuaLS.lua-language-server",
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub id: ServerId,
    pub name: String,
    pub language: String,
    pub binary: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub install_cmd: String,
}

pub fn detect_server(id: ServerId) -> ServerStatus {
    let s = spec(id);
    let install_cmd = if cfg!(target_os = "macos") {
        s.install_cmd_macos
    } else if cfg!(target_os = "windows") {
        s.install_cmd_windows
    } else {
        s.install_cmd_linux
    };

    let (installed, path, version) = match which::which(s.binary) {
        Ok(p) => {
            let v = probe_version(&p, s.binary);
            (true, Some(p.to_string_lossy().into_owned()), v)
        }
        Err(_) => (false, None, None),
    };

    ServerStatus {
        id,
        name: s.name.to_string(),
        language: s.language.to_string(),
        binary: s.binary.to_string(),
        installed,
        path,
        version,
        install_cmd: install_cmd.to_string(),
    }
}

pub fn detect_all() -> Vec<ServerStatus> {
    ServerId::all().iter().copied().map(detect_server).collect()
}

pub fn server_for_extension(ext: &str) -> Option<ServerId> {
    for id in ServerId::all() {
        if spec(*id).extensions.contains(&ext) {
            return Some(*id);
        }
    }
    None
}

pub fn server_for_path(path: &Path) -> Option<ServerId> {
    let ext = path.extension()?.to_str()?;
    server_for_extension(ext)
}

pub fn servers_for_project(root: &Path) -> Vec<ServerId> {
    let mut out = Vec::new();
    for id in ServerId::all() {
        let s = spec(*id);
        for manifest in &s.manifests {
            if let Some(suffix) = manifest.strip_prefix('*') {
                if has_file_with_suffix(root, suffix) {
                    out.push(*id);
                    break;
                }
            } else if root.join(manifest).exists() {
                out.push(*id);
                break;
            }
        }
    }
    out
}

fn has_file_with_suffix(root: &Path, suffix: &str) -> bool {
    let Ok(rd) = std::fs::read_dir(root) else {
        return false;
    };
    for entry in rd.flatten() {
        if let Some(n) = entry.file_name().to_str() {
            if n.ends_with(suffix) {
                return true;
            }
        }
    }
    false
}

fn probe_version(binary_path: &Path, binary_name: &str) -> Option<String> {
    let flag = match binary_name {
        "rust-analyzer" | "gopls" | "clangd" => "--version",
        "ruby-lsp" | "lua-language-server" => "--version",
        "typescript-language-server" | "pyright-langserver" | "intelephense" => "--version",
        _ => "--version",
    };
    let out = std::process::Command::new(binary_path)
        .arg(flag)
        .output()
        .ok()?;
    let raw = if !out.stdout.is_empty() {
        String::from_utf8_lossy(&out.stdout).into_owned()
    } else {
        String::from_utf8_lossy(&out.stderr).into_owned()
    };
    let line = raw.lines().next()?.trim().to_string();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_servers_have_specs() {
        for id in ServerId::all() {
            let s = spec(*id);
            assert!(!s.binary.is_empty());
            assert!(!s.manifests.is_empty());
            assert!(!s.extensions.is_empty());
            assert!(!s.install_cmd_linux.is_empty());
        }
    }

    #[test]
    fn extension_lookup() {
        assert_eq!(server_for_extension("rs"), Some(ServerId::RustAnalyzer));
        assert_eq!(server_for_extension("go"), Some(ServerId::Gopls));
        assert_eq!(
            server_for_extension("ts"),
            Some(ServerId::TypeScriptLanguageServer)
        );
        assert_eq!(server_for_extension("py"), Some(ServerId::Pyright));
        assert_eq!(server_for_extension("php"), Some(ServerId::Intelephense));
        assert_eq!(server_for_extension("rb"), Some(ServerId::RubyLsp));
        assert_eq!(
            server_for_extension("lua"),
            Some(ServerId::LuaLanguageServer)
        );
        assert_eq!(server_for_extension("cpp"), Some(ServerId::Clangd));
        assert_eq!(server_for_extension("xyz"), None);
    }

    #[test]
    fn project_detection_via_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(tmp.path().join("go.mod"), "module x").unwrap();

        let detected = servers_for_project(tmp.path());
        assert!(detected.contains(&ServerId::RustAnalyzer));
        assert!(detected.contains(&ServerId::Gopls));
        assert!(!detected.contains(&ServerId::Pyright));
    }

    #[test]
    fn detect_returns_install_cmd_for_missing() {
        let status = detect_server(ServerId::Gopls);
        assert!(!status.install_cmd.is_empty());
        assert_eq!(status.name, "gopls");
    }
}
