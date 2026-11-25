//! Build the search index from typed lesson data.
use std::fmt::Write as _;

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

/// Validate typed input before encoding the browser contract.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Lesson<'static> {
        Lesson {
            title: "所有權",
            id: "04",
            description: "借用",
            href: "/-rust-learning/chapters/ownership/",
            text: "Rust E0382 🦀",
            headings: vec![
                Heading {
                    id: "s-借用",
                    label: "借用",
                    level: 2,
                },
                Heading {
                    id: "s-細節",
                    label: "細節",
                    level: 4,
                },
            ],
        }
    }

    #[test]
    fn preserves_search_schema_and_filters_deep_headings() {
        assert_eq!(
            render(&[sample()]).unwrap(),
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
    fn rejects_invalid_typed_entries() {
        assert!(render(&[sample(), sample()]).is_err());
        let mut item = sample();
        item.href = "//invalid/";
        assert!(render(&[item]).is_err());
        let mut item = sample();
        item.headings[0].id = "";
        assert!(render(&[item]).is_err());
        let mut item = sample();
        item.headings[0].level = 1;
        assert!(render(&[item]).is_err());
    }
}
