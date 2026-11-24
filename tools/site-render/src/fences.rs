//! Extract and validate fenced examples from the original Markdown.
use super::Result;
#[cfg(feature = "migration-compat")]
use super::{count, emit, field};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

fn name(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}
fn diagnostic(s: &str) -> bool {
    s.len() == 5 && s.starts_with('E') && s[1..].bytes().all(|b| b.is_ascii_digit())
}
fn path(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("src/") else {
        return false;
    };
    let rest = rest.strip_prefix("bin/").unwrap_or(rest);
    rest.strip_suffix(".rs")
        .is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
}
fn valid(info: &str) -> bool {
    if matches!(info, "text" | "bash" | "toml") {
        return true;
    }
    let p: Vec<_> = info.split(' ').collect();
    match p.as_slice() {
        [
            "rust",
            "run" | "program" | "fragment" | "continuation" | "test",
        ] => true,
        ["rust", "error", codes] => codes.split(',').all(diagnostic),
        ["rust", "project", arg] => arg.split_once(':').is_some_and(|(n, p)| name(n) && path(p)),
        _ => false,
    }
}
pub(super) fn extract(source: &str) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    let mut active: Option<(String, String)> = None;
    for event in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    ) {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let info = info.trim().to_owned();
                if !valid(&info) {
                    return Err(format!("無效範例標注：{info}"));
                }
                active = Some((info, String::new()));
            }
            Event::Text(text) => {
                if let Some((_, code)) = &mut active {
                    code.push_str(&text)
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(value) = active.take() {
                    result.push(value)
                }
            }
            _ => {}
        }
    }
    if active.is_some() {
        return Err("程式區塊不完整".into());
    }
    Ok(result)
}
#[cfg(feature = "migration-compat")]
pub(super) fn process_fences<'a>(
    n: usize,
    f: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    for _ in 0..n {
        let source = field(f)?;
        let expected = count(f, 5000)?;
        let actual = extract(source)?;
        if actual.len() != expected {
            return Err("程式區塊數量不同".into());
        }
        emit(out, &actual.len().to_string())?;
        for (info, code) in actual {
            emit(out, &info)?;
            emit(out, &code)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_expected_modes() {
        assert!(valid("rust error E0382,E0502"));
        assert!(valid("rust project external:src/bin/timer.rs"));
        assert!(!valid("rust error E123"));
        assert!(!valid("rust project external:src/../main.rs"));
    }
    #[test]
    fn extracts_source() {
        let found = extract("# 教材\n\n```rust run\nlet x = 1;\n```\n").unwrap();
        assert_eq!(found, vec![("rust run".into(), "let x = 1;\n".into())]);
    }
}
