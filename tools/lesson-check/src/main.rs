//! Validate the constrained teaching-note convention before the static build.
//! This is not a general Markdown parser; markdown-it still renders CommonMark.
use std::{collections::HashSet, env, fs, path::Path, process::ExitCode};

const KINDS: [&str; 5] = ["context", "analogy", "walkthrough", "warning", "practice"];
const GENERIC_TITLES: [&str; 5] = ["來龍去脈", "生活比喻", "程式拆解", "常見陷阱", "動手驗收"];

fn parse_title(line: &str) -> Option<(&str, &str)> {
    let (title, kind) = line.strip_prefix("> **")?.split_once("** <!-- callout:")?;
    let kind = kind.strip_suffix(" -->")?;
    if title.trim().is_empty() || title.contains('*') || title.trim() != title {
        return None;
    }
    Some((title, kind))
}

// Recognize top-level CommonMark backtick/tilde fences (up to three leading spaces).
fn fence(line: &str) -> Option<(u8, usize, &str)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let marker = *trimmed.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let count = trimmed.bytes().take_while(|&b| b == marker).count();
    (count >= 3).then(|| (marker, count, &trimmed[count..]))
}

fn inspect(source: &str) -> Result<usize, Vec<String>> {
    let lines: Vec<_> = source.lines().collect();
    let mut active_fence: Option<(u8, usize)> = None;
    let mut count = 0;
    let mut titles = HashSet::new();
    let mut errors = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some((marker, size, rest)) = fence(line) {
            if let Some((opened, minimum)) = active_fence {
                if marker == opened && size >= minimum && rest.trim().is_empty() {
                    active_fence = None;
                }
            } else if marker != b'`' || !rest.contains('`') {
                active_fence = Some((marker, size));
            }
            continue;
        }
        if active_fence.is_some() {
            continue;
        }
        if !line.contains("<!-- callout:") {
            if GENERIC_TITLES.iter().any(|title| {
                line.strip_prefix("> **").and_then(|s| s.strip_suffix("**")) == Some(*title)
            }) {
                errors.push(format!("第 {} 行：使用具體內容作為標題，不使用內部分類", i + 1));
            }
            continue;
        }
        let Some((title, kind)) = parse_title(line) else {
            errors.push(format!("第 {} 行：教學方塊格式不正確", i + 1));
            continue;
        };
        if GENERIC_TITLES.contains(&title) {
            errors.push(format!("第 {} 行：標題不能只寫內部分類", i + 1));
        }
        if !titles.insert(title) {
            errors.push(format!("第 {} 行：重複的教學方塊標題 {title}", i + 1));
        }
        if !KINDS.contains(&kind) {
            errors.push(format!("第 {} 行：未知分類 {kind}", i + 1));
        } else {
            count += 1;
        }
        if lines.get(i + 1).copied() != Some(">")
            || !lines.get(i + 2).and_then(|line| line.strip_prefix("> "))
                .is_some_and(|body| !body.trim().is_empty())
        {
            errors.push(format!("第 {} 行：教學方塊缺少正文", i + 1));
        }
    }
    if count == 0 {
        errors.push("至少需要一個教學方塊".into());
    }
    if errors.is_empty() { Ok(count) } else { Err(errors) }
}

fn check_file(path: &Path) -> Result<usize, Vec<String>> {
    let source = fs::read_to_string(path).map_err(|err| vec![err.to_string()])?;
    inspect(&source)
}

fn main() -> ExitCode {
    let paths: Vec<_> = env::args_os().skip(1).collect();
    if paths.is_empty() {
        eprintln!("Usage: lesson-check <chapter.md>...");
        return ExitCode::FAILURE;
    }
    let mut total = 0;
    let mut failed = false;
    for path in &paths {
        let path = Path::new(path);
        match check_file(path) {
            Ok(count) => total += count,
            Err(errors) => {
                failed = true;
                for error in errors {
                    eprintln!("{}: {error}", path.display());
                }
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        println!("Rust 教材檢查：{} 章、{total} 個教學方塊通過", paths.len());
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> String {
        KINDS.iter().enumerate().map(|(i, kind)| {
            format!("> **問題 {i}？** <!-- callout:{kind} -->\n>\n> 具體內容。\n\n")
        }).collect::<Vec<_>>().join("")
    }

    #[test]
    fn accepts_five_topic_specific_notes() {
        assert_eq!(inspect(&valid()), Ok(5));
    }

    #[test]
    fn rejects_editorial_titles_and_duplicates() {
        assert!(inspect(&valid().replace("問題 0？", "來龍去脈")).is_err());
        assert!(inspect(&valid().replace("callout:analogy", "callout:unknown")).is_err());
        assert!(inspect(&valid().replace("問題 1？", "問題 0？")).is_err());
    }

    #[test]
    fn permits_flexible_note_counts_and_repeated_kinds() {
        assert_eq!(inspect(&valid().replace("callout:analogy", "callout:context")), Ok(5));
        assert_eq!(inspect("> **一個具體問題？** <!-- callout:context -->\n>\n> 解釋。\n"), Ok(1));
    }

    #[test]
    fn rejects_empty_or_missing_notes() {
        assert!(inspect(&valid().replace("> 具體內容。", "> ")).is_err());
        assert!(inspect("").is_err());
    }

    #[test]
    fn ignores_examples_inside_fenced_code() {
        let source = format!("````text\n```\n> **忽略** <!-- callout:context -->\n````\n{}", valid());
        assert_eq!(inspect(&source), Ok(5));
    }

    #[test]
    fn handles_unicode_titles_and_crlf() {
        assert_eq!(inspect(&valid().replace('\n', "\r\n")), Ok(5));
    }
}
