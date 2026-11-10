//! Velapor promises to read the machine and never change it. This test reads every source
//! file of the core and fails if one of them reaches for an API that writes, deletes or
//! starts programs outside the one allowed place.

use std::fs;
use std::path::{Path, PathBuf};

/// Calls that would change files, settings or processes.
const FORBIDDEN: &[&str] = &[
    "fs::write",
    "File::create",
    "OpenOptions",
    "remove_file",
    "remove_dir",
    "create_dir",
    "fs::rename",
    "fs::copy",
    "set_permissions",
    "hard_link",
    "symlink",
    "kill_with",
    "set_var",
];

/// Starting other programs is allowed in this one file only, and only from its fixed list.
/// It may also stop a tool it started itself when that tool takes too long; nothing else.
const COMMAND_HOME: &str = "system_query.rs";

/// Every `.rs` file below a folder.
fn rust_files(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder).expect("source folder is readable") {
        let path = entry.expect("entry is readable").path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
    files
}

#[test]
/// No source file of the core uses an API that writes or deletes.
fn the_core_only_reads() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offences = Vec::new();
    for file in rust_files(&source) {
        let text = fs::read_to_string(&file).expect("source file is readable");
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for word in FORBIDDEN {
                if code.contains(word) {
                    offences.push(format!("{}:{} uses {}", file.display(), number + 1, word));
                }
            }
            if (code.contains("Command::new") || code.contains(".kill(")) && name != COMMAND_HOME {
                offences.push(format!(
                    "{}:{} starts or stops a program outside {}",
                    file.display(),
                    number + 1,
                    COMMAND_HOME
                ));
            }
        }
    }
    assert!(offences.is_empty(), "Velapor must only read:\n{}", offences.join("\n"));
}
