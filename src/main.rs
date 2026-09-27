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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Build a throwaway tree of source files and scan it.
    fn fixture(files: &[(&str, &str)]) -> (PathBuf, Vec<Item>) {
        let root = std::env::temp_dir().join(format!(
            "todo-scanner-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        for (name, content) in files {
            let path = root.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }
        let mut items = Vec::new();
        scan(&root, &mut items);
        let _ = fs::remove_dir_all(&root);
        (root, items)
    }

    #[test]
    fn finds_every_tag() {
        let (_, items) = fixture(&[
            ("a.rs", "// TODO: write the docs\n"),
            ("b.py", "# FIXME: this crashes\n"),
            ("c.js", "// HACK: temporary\n"),
            ("d.go", "// XXX: revisit\n"),
        ]);
        let mut tags: Vec<String> = items.iter().map(|item| item.2.clone()).collect();
        tags.sort();
        assert_eq!(tags, vec!["FIXME", "HACK", "TODO", "XXX"]);
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn reports_line_numbers_and_text() {
        let (_, items) = fixture(&[("a.rs", "fn main() {}\n// TODO: fix this\n")]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].1, 2, "the marker is on line 2");
        assert!(items[0].3.contains("fix this"), "text: {}", items[0].3);
    }

    #[test]
    fn one_tag_per_line() {
        // a line mentioning two tags is reported once
        let (_, items) = fixture(&[("a.rs", "// TODO: this is really a FIXME\n")]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].2, "TODO");
    }

    #[test]
    fn descends_into_subdirectories() {
        let (_, items) = fixture(&[("deep/nested/a.rs", "// TODO: below\n")]);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn ignores_noise_directories() {
        let (_, items) = fixture(&[
            ("src/a.rs", "// TODO: real\n"),
            ("target/b.rs", "// TODO: build artifact\n"),
            ("node_modules/c.rs", "// TODO: dependency\n"),
            (".git/d.rs", "// TODO: internal\n"),
        ]);
        assert_eq!(items.len(), 1, "only the source tree counts: {:?}", items);
    }

    #[test]
    fn ignores_unrelated_extensions() {
        let (_, items) = fixture(&[
            ("notes.md", "TODO: not source\n"),
            ("data.json", "{\"todo\": true}\n"),
        ]);
        assert!(items.is_empty(), "got {:?}", items);
    }

    #[test]
    fn clean_tree_yields_nothing() {
        let (_, items) = fixture(&[("a.rs", "fn main() { println!(\"hi\"); }\n")]);
        assert!(items.is_empty());
    }

    #[test]
    fn missing_directory_is_silent() {
        let mut items = Vec::new();
        scan(&PathBuf::from("/definitely/not/here"), &mut items);
        assert!(items.is_empty());
    }

    #[test]
    fn reads_nested_directories_recursively() {
        let (_, items) = fixture(&[("a.rs", "// TODO: one\n"), ("sub/b.c", "// TODO: two\n")]);
        assert_eq!(items.len(), 2);
    }
}
