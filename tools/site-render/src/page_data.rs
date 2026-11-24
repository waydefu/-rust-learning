//! Serialize the browser page-data contract without accepting pre-rendered JSON.
use super::{Result, count, emit, field};
use std::fmt::Write as _;
fn string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '<' => out.push_str("\\u003c"),
            c if c <= '\u{1f}' => {
                write!(out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
fn property(out: &mut String, key: &str, value: &str) {
    string(out, key);
    out.push(':');
    string(out, value)
}
fn flag<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<bool> {
    match field(f)? {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("頁面資料旗標無效".into()),
    }
}
pub(super) fn process_data<'a>(
    n: usize,
    f: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    let mut shared = String::from("{");
    property(&mut shared, "base", field(f)?);
    shared.push_str(",\"chapterPaths\":[");
    for i in 0..count(f, 1000)? {
        if i != 0 {
            shared.push(',')
        }
        shared.push('[');
        string(&mut shared, field(f)?);
        shared.push(',');
        string(&mut shared, field(f)?);
        shared.push(']');
    }
    shared.push(']');
    for _ in 0..n {
        let mut page = shared.clone();
        let chapter = field(f)?;
        if !chapter.is_empty() {
            page.push(',');
            property(&mut page, "chapter", chapter);
        }
        if flag(f)? {
            page.push_str(",\"ownershipSteps\":[");
            for i in 0..count(f, 100)? {
                if i != 0 {
                    page.push(',')
                }
                page.push('{');
                property(&mut page, "code", field(f)?);
                page.push(',');
                property(&mut page, "source", field(f)?);
                page.push(',');
                property(&mut page, "target", field(f)?);
                page.push_str(",\"owner\":");
                let owner = field(f)?;
                if owner.is_empty() {
                    page.push_str("null")
                } else {
                    string(&mut page, owner)
                }
                match field(f)? {
                    "" => {}
                    "true" => page.push_str(",\"borrowed\":true"),
                    "false" => page.push_str(",\"borrowed\":false"),
                    _ => return Err("借用旗標無效".into()),
                }
                page.push(',');
                property(&mut page, "message", field(f)?);
                page.push('}');
            }
            page.push(']');
        }
        let answer = field(f)?;
        if !answer.is_empty() {
            let answer = answer.parse::<usize>().map_err(|_| "測驗索引無效")?;
            write!(&mut page, ",\"answer\":{answer}").unwrap();
        }
        page.push('}');
        emit(out, &page)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_script_and_controls() {
        let mut s = String::new();
        string(&mut s, "</script>\n\"");
        assert_eq!(s, "\"\\u003c/script>\\n\\\"\"");
    }
}
