//! Stable heading metadata: preserve the existing JavaScript anchor contract.
use super::Result;
#[cfg(feature = "migration-compat")]
use super::{count, emit, field};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use unicode_general_category::{GeneralCategory as G, get_general_category};
use unicode_normalization::UnicodeNormalization;

fn letter_or_number(c: char) -> bool {
    matches!(
        get_general_category(c),
        G::UppercaseLetter
            | G::LowercaseLetter
            | G::TitlecaseLetter
            | G::ModifierLetter
            | G::OtherLetter
            | G::DecimalNumber
            | G::LetterNumber
            | G::OtherNumber
    )
}
fn stem(label: &str) -> String {
    let lower = label.nfkc().collect::<String>().to_lowercase();
    let mut out = String::new();
    let mut gap = false;
    for c in lower.chars() {
        if letter_or_number(c) {
            if gap && !out.is_empty() {
                out.push('-');
            }
            out.push(c);
            gap = false;
        } else {
            gap = true;
        }
    }
    if out.is_empty() {
        "section".into()
    } else {
        out
    }
}
fn anchors<'a>(labels: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut seen = HashMap::new();
    labels
        .into_iter()
        .map(|label| {
            let stem = stem(label);
            let n = seen.entry(stem.clone()).or_insert(0usize);
            *n += 1;
            if *n == 1 {
                format!("s-{stem}")
            } else {
                format!("s-{stem}-{n}")
            }
        })
        .collect()
}
type HeadingMetadata = (String, Vec<(u8, String, String)>);

pub(super) fn headings(source: &str) -> Result<HeadingMetadata> {
    let mut found: Vec<(u8, String)> = Vec::new();
    let mut active: Option<(u8, String)> = None;
    let mut first = true;
    for event in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    ) {
        if first {
            first = false;
            if !matches!(
                &event,
                Event::Start(Tag::Heading {
                    level: pulldown_cmark::HeadingLevel::H1,
                    ..
                })
            ) {
                return Err("教材必須以 H1 開頭".into());
            }
        }
        match event {
            Event::Start(Tag::Heading { level, .. }) => active = Some((level as u8, String::new())),
            Event::End(TagEnd::Heading(_)) => found.push(active.take().ok_or("標題事件不完整")?),
            Event::Text(text) | Event::Code(text) | Event::InlineHtml(text) => {
                if let Some((_, label)) = &mut active {
                    label.push_str(&text);
                }
            }
            _ => {}
        }
    }
    if found.first().map(|h| h.0) != Some(1) || found.iter().filter(|h| h.0 == 1).count() != 1 {
        return Err("教材必須有單一 H1".into());
    }
    let (_, title) = found.remove(0);
    let ids = anchors(found.iter().map(|(_, label)| label.as_str()));
    Ok((
        title,
        found
            .into_iter()
            .zip(ids)
            .map(|((level, label), id)| (level, label, id))
            .collect(),
    ))
}
#[cfg(feature = "migration-compat")]
pub(super) fn process_metadata<'a>(
    n: usize,
    fields: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    for _ in 0..n {
        let (title, headings) = headings(field(fields)?)?;
        emit(out, &title)?;
        emit(out, &headings.len().to_string())?;
        for (level, label, id) in headings {
            emit(out, &level.to_string())?;
            emit(out, &label)?;
            emit(out, &id)?;
        }
    }
    Ok(())
}
#[cfg(feature = "migration-compat")]
pub(super) fn process_anchors<'a>(
    n: usize,
    fields: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    for _ in 0..n {
        let labels = (0..count(fields, 5000)?)
            .map(|_| field(fields))
            .collect::<Result<Vec<_>>>()?;
        let ids = anchors(labels);
        emit(out, &ids.len().to_string())?;
        for id in ids {
            emit(out, &id)?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_unicode_and_preserves_duplicate_rules() {
        assert_eq!(
            anchors([
                "ＡＢＣ ①",
                "abc 1",
                "繁體中文—Rust",
                "🦀",
                "",
                "É",
                "e\u{301}",
                "ΟΣ"
            ]),
            [
                "s-abc-1",
                "s-abc-1-2",
                "s-繁體中文-rust",
                "s-section",
                "s-section-2",
                "s-é",
                "s-é-2",
                "s-ος"
            ]
        );
        assert_eq!(stem("a\u{345}b"), "a-b");
    }
    #[test]
    fn extracts_title_and_duplicate_headings() {
        let (title, h) =
            headings("# Rust **學習**\n\n## \x60Vec<T>\x60\n\n## \x60Vec<T>\x60\n\n### 字串\n")
                .unwrap();
        assert_eq!(title, "Rust 學習");
        assert_eq!(
            h,
            vec![
                (2, "Vec<T>".into(), "s-vec-t".into()),
                (2, "Vec<T>".into(), "s-vec-t-2".into()),
                (3, "字串".into(), "s-字串".into())
            ]
        );
        assert!(headings("no title").is_err());
        assert!(headings("# One\n# Two\n").is_err());
    }
}
