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
pub(super) struct OwnershipStep<'a> {
    pub code: &'a str,
    pub source: &'a str,
    pub target: &'a str,
    pub owner: Option<&'a str>,
    pub borrowed: Option<bool>,
    pub message: &'a str,
}

pub(super) struct PageData<'a> {
    pub chapter: Option<&'a str>,
    pub ownership: Option<Vec<OwnershipStep<'a>>>,
    pub answer: Option<usize>,
}

pub(super) fn render(base: &str, paths: &[(&str, &str)], data: &PageData<'_>) -> String {
    let mut page = String::from("{");
    property(&mut page, "base", base);
    page.push_str(",\"chapterPaths\":[");
    for (i, (id, path)) in paths.iter().enumerate() {
        if i != 0 {
            page.push(',')
        }
        page.push('[');
        string(&mut page, id);
        page.push(',');
        string(&mut page, path);
        page.push(']');
    }
    page.push(']');
    if let Some(chapter) = data.chapter {
        page.push(',');
        property(&mut page, "chapter", chapter);
    }
    if let Some(steps) = &data.ownership {
        page.push_str(",\"ownershipSteps\":[");
        for (i, step) in steps.iter().enumerate() {
            if i != 0 {
                page.push(',')
            }
            page.push('{');
            property(&mut page, "code", step.code);
            page.push(',');
            property(&mut page, "source", step.source);
            page.push(',');
            property(&mut page, "target", step.target);
            page.push_str(",\"owner\":");
            if let Some(owner) = step.owner {
                string(&mut page, owner);
            } else {
                page.push_str("null");
            }
            if let Some(borrowed) = step.borrowed {
                write!(&mut page, ",\"borrowed\":{borrowed}").unwrap();
            }
            page.push(',');
            property(&mut page, "message", step.message);
            page.push('}');
        }
        page.push(']');
    }
    if let Some(answer) = data.answer {
        write!(&mut page, ",\"answer\":{answer}").unwrap();
    }
    page.push('}');
    page
}

fn optional(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

pub(super) fn process_data<'a>(
    n: usize,
    f: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    let base = field(f)?;
    let paths = (0..count(f, 1000)?)
        .map(|_| Ok((field(f)?, field(f)?)))
        .collect::<Result<Vec<_>>>()?;
    for _ in 0..n {
        let chapter = optional(field(f)?);
        let ownership = if flag(f)? {
            let mut steps = Vec::new();
            for _ in 0..count(f, 100)? {
                steps.push(OwnershipStep {
                    code: field(f)?,
                    source: field(f)?,
                    target: field(f)?,
                    owner: optional(field(f)?),
                    borrowed: match field(f)? {
                        "" => None,
                        "true" => Some(true),
                        "false" => Some(false),
                        _ => return Err("借用旗標無效".into()),
                    },
                    message: field(f)?,
                });
            }
            Some(steps)
        } else {
            None
        };
        let answer = optional(field(f)?)
            .map(|v| v.parse::<usize>().map_err(|_| "測驗索引無效".to_owned()))
            .transpose()?;
        emit(
            out,
            &render(
                base,
                &paths,
                &PageData {
                    chapter,
                    ownership,
                    answer,
                },
            ),
        )?;
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
