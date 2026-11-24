//! Build the search index from typed lesson data. The CLI adapter retains the migration protocol.
use std::{
    fmt::Write as _,
    io::{self, Read, Write},
    process::ExitCode,
};

const LIMIT: u64 = 16 * 1024 * 1024;

pub struct Heading<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub level: u8,
}

pub struct Lesson<'a> {
    pub title: &'a str,
    pub id: &'a str,
    pub description: &'a str,
    pub href: &'a str,
    pub text: &'a str,
    pub headings: Vec<Heading<'a>>,
}

fn field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, String> {
    fields.next().ok_or_else(|| "搜尋資料不完整".into())
}

fn count<'a>(fields: &mut impl Iterator<Item = &'a str>, limit: usize) -> Result<usize, String> {
    field(fields)?
        .parse::<usize>()
        .ok()
        .filter(|&n| n <= limit)
        .ok_or_else(|| "搜尋資料數量無效".into())
}

fn decode(input: &str) -> Result<Vec<Lesson<'_>>, String> {
    let mut fields = input
        .strip_suffix('\0')
        .ok_or("搜尋資料缺少結尾")?
        .split('\0');
    if field(&mut fields)? != "rust-search-v1" {
        return Err("搜尋資料版本不相容".into());
    }
    let n = count(&mut fields, 1000)?;
    let mut lessons = Vec::with_capacity(n);
    let mut ids = std::collections::HashSet::new();
    for _ in 0..n {
        let title = field(&mut fields)?;
        let id = field(&mut fields)?;
        let description = field(&mut fields)?;
        let href = field(&mut fields)?;
        let text = field(&mut fields)?;
        if title.is_empty()
            || id.is_empty()
            || text.is_empty()
            || !href.starts_with('/')
            || href.starts_with("//")
            || !href.ends_with('/')
            || !ids.insert(id)
        {
            return Err("搜尋條目識別碼、路徑或內容無效".into());
        }
        let n = count(&mut fields, 5000)?;
        let mut headings = Vec::with_capacity(n);
        for _ in 0..n {
            let id = field(&mut fields)?;
            let label = field(&mut fields)?;
            let level = count(&mut fields, 6)? as u8;
            if level < 2 || id.is_empty() {
                return Err("標題層級或錨點無效".into());
            }
            headings.push(Heading { id, label, level });
        }
        lessons.push(Lesson {
            title,
            id,
            description,
            href,
            text,
            headings,
        });
    }
    if fields.next().is_some() {
        return Err("搜尋資料包含多餘欄位".into());
    }
    Ok(lessons)
}

// JSON strings preserve Unicode and escape every control character.
fn string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if c <= '\u{1f}' => {
                write!(out, "\\u{:04x}", c as u32).expect("String write");
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn property(out: &mut String, name: &str, value: &str) {
    string(out, name);
    out.push(':');
    string(out, value);
}

fn encode(lessons: &[Lesson<'_>]) -> String {
    let mut out = String::from("[");
    for (i, lesson) in lessons.iter().enumerate() {
        if i != 0 {
            out.push(',');
        }
        out.push('{');
        for (i, (name, value)) in [
            ("title", lesson.title),
            ("id", lesson.id),
            ("description", lesson.description),
            ("href", lesson.href),
        ]
        .into_iter()
        .enumerate()
        {
            if i != 0 {
                out.push(',');
            }
            property(&mut out, name, value);
        }
        out.push_str(",\"headings\":[");
        for (i, heading) in lesson.headings.iter().filter(|h| h.level <= 3).enumerate() {
            if i != 0 {
                out.push(',');
            }
            out.push('{');
            property(&mut out, "id", heading.id);
            out.push(',');
            property(&mut out, "label", heading.label);
            out.push('}');
        }
        out.push_str("],");
        property(&mut out, "text", lesson.text);
        out.push('}');
    }
    out.push(']');
    out
}

/// Validate typed input as well as the compatibility protocol.
pub fn render(lessons: &[Lesson<'_>]) -> Result<String, String> {
    let mut ids = std::collections::HashSet::new();
    if lessons.len() > 1000 {
        return Err("搜尋資料數量無效".into());
    }
    for l in lessons {
        if l.title.is_empty()
            || l.id.is_empty()
            || l.text.is_empty()
            || !l.href.starts_with('/')
            || l.href.starts_with("//")
            || !l.href.ends_with('/')
            || !ids.insert(l.id)
        {
            return Err("搜尋條目識別碼、路徑或內容無效".into());
        }
        if l.headings.len() > 5000
            || l.headings
                .iter()
                .any(|h| !(2..=6).contains(&h.level) || h.id.is_empty())
        {
            return Err("標題層級或錨點無效".into());
        }
    }
    Ok(encode(lessons))
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin()
        .lock()
        .take(LIMIT + 1)
        .read_to_string(&mut input)?;
    if input.len() as u64 > LIMIT {
        return Err("搜尋資料超過大小限制".into());
    }
    let lessons = decode(&input)?;
    io::stdout()
        .lock()
        .write_all(render(&lessons)?.as_bytes())?;
    Ok(())
}

pub fn cli() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("search-index: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        [
            "rust-search-v1",
            "1",
            "所有權",
            "04",
            "借用",
            "/-rust-learning/chapters/ownership/",
            "Rust E0382 🦀",
            "2",
            "s-借用",
            "借用",
            "2",
            "s-細節",
            "細節",
            "4",
            "",
        ]
        .join("\0")
    }

    #[test]
    fn preserves_search_schema_and_filters_deep_headings() {
        let input = sample();
        assert_eq!(
            encode(&decode(&input).unwrap()),
            r#"[{"title":"所有權","id":"04","description":"借用","href":"/-rust-learning/chapters/ownership/","headings":[{"id":"s-借用","label":"借用"}],"text":"Rust E0382 🦀"}]"#
        );
    }

    #[test]
    fn escapes_json_strings() {
        let mut out = String::new();
        string(&mut out, "引號\"\\\n\r\t\u{08}\u{0c}\u{01}\u{0}");
        assert_eq!(out, r#""引號\"\\\n\r\t\b\f\u0001\u0000""#);
    }

    #[test]
    fn rejects_truncated_or_extra_data() {
        let input = sample();
        assert!(decode(input.trim_end_matches('\0')).is_err());
        assert!(decode(&(input.clone() + "extra\0")).is_err());
        assert!(decode(&input.replace("rust-search-v1", "old")).is_err());
        assert!(decode(&input.replace("\0s-細節\0", "\0\0")).is_err());
    }
}
