use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let dir = args.first().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let mut items = Vec::new();
    scan(&dir, &mut items);
    items.sort();
    for (file, line_no, tag, text) in &items {
        println!("{file}:{line_no}: [{tag}] {text}");
    }
    println!("found {} TODO-type comment(s)", items.len());
    if items.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

type Item = (String, usize, String, String);

fn scan(dir: &PathBuf, items: &mut Vec<Item>) {
    let entries = match fs::read_dir(dir) { Ok(e) => e, Err(_) => return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !matches!(name, ".git" | "node_modules" | "target") { scan(&path, items); }
        } else if path.extension().map(|e| e == "rs" || e == "py" || e == "js" || e == "go" || e == "c").unwrap_or(false) {
            let content = match fs::read_to_string(&path) { Ok(c) => c, Err(_) => continue };
            for (i, line) in content.lines().enumerate() {
                for tag in ["TODO", "FIXME", "HACK", "XXX"] {
                    if let Some(pos) = line.find(tag) {
                        let text = line[pos..].trim().to_string();
                        items.push((path.display().to_string(), i + 1, tag.to_string(), text));
                        break;
                    }
                }
            }
        }
    }
}

use std::process::ExitCode;
