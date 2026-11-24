//! Native reader and interactive templates. Trusted Markdown fragments are supplied by the bridge.
use super::{Result, count, emit, escape, field, render_template};
use std::collections::HashMap;
fn fill(t: &str, v: &[(&str, &str)]) -> Result<String> {
    render_template(
        t,
        &v.iter()
            .map(|(k, v)| (*k, (*v).to_owned()))
            .collect::<HashMap<_, _>>(),
    )
}
fn flag<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<bool> {
    match field(f)? {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("互動旗標無效".into()),
    }
}
fn safe(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
pub(super) struct Quiz<'a> {
    pub prompt: &'a str,
    pub explanation: &'a str,
    pub choices: Vec<&'a str>,
    pub answer: usize,
}
fn read_quiz<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<Option<Quiz<'a>>> {
    if !flag(f)? {
        return Ok(None);
    }
    let prompt = field(f)?;
    let explanation = field(f)?;
    let n = count(f, 100)?;
    let answer = count(f, 100)?;
    let choices = (0..n).map(|_| field(f)).collect::<Result<Vec<_>>>()?;
    Ok(Some(Quiz {
        prompt,
        explanation,
        choices,
        answer,
    }))
}
fn quiz(data: Option<&Quiz<'_>>, slug: &str, icon: &str) -> Result<String> {
    let Some(data) = data else {
        return Ok(String::new());
    };
    let prompt = escape(data.prompt);
    let explanation = escape(data.explanation);
    let n = data.choices.len();
    let answer = data.answer;
    if answer >= n {
        return Err("測驗答案無效".into());
    }
    let mut choices = String::new();
    let mut correct = String::new();
    for (i, choice) in data.choices.iter().enumerate() {
        let choice = escape(choice);
        if i == answer {
            correct = choice.clone()
        }
        choices.push_str(&format!(r#"<label><input type="radio" name="answer" value="{i}" required><span>{choice}</span></label>"#));
    }
    fill(
        include_str!("../templates/quiz.html"),
        &[
            ("slug", slug),
            ("icon", icon),
            ("prompt", &prompt),
            ("choices", &choices),
            ("answer", &correct),
            ("explanation", &explanation),
        ],
    )
}
pub(super) struct Flow<'a> {
    pub title: &'a str,
    pub caption: &'a str,
    pub nodes: Vec<&'a str>,
}
fn read_flow<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<Option<Flow<'a>>> {
    if !flag(f)? {
        return Ok(None);
    }
    Ok(Some(Flow {
        title: field(f)?,
        caption: field(f)?,
        nodes: (0..count(f, 100)?)
            .map(|_| field(f))
            .collect::<Result<Vec<_>>>()?,
    }))
}
fn flow(data: Option<&Flow<'_>>, icon: &str) -> Result<String> {
    let Some(data) = data else {
        return Ok(String::new());
    };
    let title = escape(data.title);
    let caption = escape(data.caption);
    let mut nodes = String::new();
    for (i, node) in data.nodes.iter().enumerate() {
        let node = escape(node);
        nodes.push_str(&format!(
            "<li>{}<span>{node}</span></li>",
            if i == 0 { "" } else { icon }
        ));
    }
    fill(
        include_str!("../templates/flow.html"),
        &[("title", &title), ("caption", &caption), ("nodes", &nodes)],
    )
}
type Ownership<'a> = [(&'a str, &'a str); 4];
fn read_ownership<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<Option<Ownership<'a>>> {
    if !flag(f)? {
        return Ok(None);
    }
    Ok(Some([
        (field(f)?, field(f)?),
        (field(f)?, field(f)?),
        (field(f)?, field(f)?),
        (field(f)?, field(f)?),
    ]))
}
fn ownership(data: Option<&Ownership<'_>>, icon: &str) -> Result<String> {
    let Some(data) = data else {
        return Ok(String::new());
    };
    let (mut buttons, mut steps, mut first_code, mut first_message) =
        (String::new(), String::new(), String::new(), String::new());
    for (i, label) in ["建立", "移動", "借用", "釋放"].into_iter().enumerate() {
        let code = escape(data[i].0);
        let message = escape(data[i].1);
        if i == 0 {
            first_code = code.clone();
            first_message = message.clone()
        }
        buttons.push_str(&format!(r#"<button type="button" data-step="{i}" aria-controls="memory-state" aria-pressed="{}">{} {label}</button>"#,i==0,i+1));
        steps.push_str(&format!("<li><code>{code}</code> {message}</li>"));
    }
    fill(
        include_str!("../templates/ownership-lab.html"),
        &[
            ("icon", icon),
            ("buttons", &buttons),
            ("code", &first_code),
            ("message", &first_message),
            ("steps", &steps),
        ],
    )
}

#[derive(Clone, Copy)]
pub(super) struct Chapter<'a> {
    pub id: &'a str,
    pub path: &'a str,
    pub title: &'a str,
}
pub(super) struct Stage<'a> {
    pub title: &'a str,
    pub chapters: Vec<Chapter<'a>>,
}
pub(super) enum LessonKind {
    Chapter,
    Resource,
    Guide,
}
pub(super) struct Lesson<'a> {
    pub slug: &'a str,
    pub kind: LessonKind,
    pub id: &'a str,
    pub entry_title: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub stage_title: &'a str,
    pub label_icon: &'a str,
    pub source_id: &'a str,
    pub intro: &'a str,
    pub rest: &'a str,
    pub toc: Vec<(&'a str, &'a str)>,
    pub ownership: Option<Ownership<'a>>,
    pub flow: Option<Flow<'a>>,
    pub quiz: Option<Quiz<'a>>,
}
fn chapter_number(id: &str) -> Result<usize> {
    id.parse::<usize>().map_err(|_| "章節編號無效".into())
}
fn chapter_path(s: &str) -> bool {
    !s.is_empty()
        && s.split('/').all(|p| {
            !p.is_empty()
                && !matches!(p, "." | "..")
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
}
fn chapter_link(base: &str, path: &str, label: &str, title: &str) -> String {
    format!(
        r#"<a href="{base}{path}/"><small>{label}</small><strong>{}</strong></a>"#,
        escape(title)
    )
}
fn pagination(base: &str, chapters: &[Chapter<'_>], id: &str) -> Result<String> {
    let index = chapters
        .iter()
        .position(|c| c.id == id)
        .ok_or("章節不在索引內")?;
    let previous = if index == 0 {
        chapter_link(base, "guide", "開始之前", "學習指南")
    } else {
        let c = &chapters[index - 1];
        chapter_link(base, c.path, "上一章", c.title)
    };
    let next = if let Some(c) = chapters.get(index + 1) {
        chapter_link(base, c.path, "下一章", c.title)
    } else {
        chapter_link(base, "resources", "繼續學習", "延伸資源")
    };
    Ok(format!(
        "<nav class=\"chapter-pagination\" aria-label=\"相鄰章節\">\n        {previous}\n        {next}</nav>"
    ))
}
fn outline(toc: &[(&str, &str)], chapter: bool, ownership: bool) -> String {
    let mut out = String::from("<ul>");
    if ownership {
        out.push_str("<li><a href=\"#ownership-lab\">互動記憶體圖</a></li>");
    }
    for (id, label) in toc {
        let id = escape(id);
        let label = escape(label);
        out.push_str(&format!(r##"<li><a href="#{id}">{label}</a></li>"##));
    }
    if chapter {
        out.push_str("<li><a href=\"#quick-check\">自我測驗</a></li>");
    }
    out.push_str("</ul>");
    out
}

pub(super) fn process_reader<'a>(
    n: usize,
    f: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    let base = field(f)?;
    if !base.starts_with('/')
        || !base.ends_with('/')
        || base.starts_with("//")
        || !base
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_.-".contains(&b))
        || base.split('/').any(|s| matches!(s, "." | ".."))
    {
        return Err("閱讀頁網址無效".into());
    }
    let [chevron, list, check, target, layers, flow_icon] = [
        field(f)?,
        field(f)?,
        field(f)?,
        field(f)?,
        field(f)?,
        field(f)?,
    ];
    let mut stages = Vec::new();
    for _ in 0..count(f, 100)? {
        let title = field(f)?;
        let mut chapters = Vec::new();
        for _ in 0..count(f, 1000)? {
            chapters.push(Chapter {
                id: field(f)?,
                path: field(f)?,
                title: field(f)?,
            });
        }
        stages.push(Stage { title, chapters });
    }
    let mut lessons = Vec::new();
    for _ in 0..n {
        let slug = field(f)?;
        let kind = match (flag(f)?, flag(f)?) {
            (true, false) => LessonKind::Chapter,
            (false, true) => LessonKind::Resource,
            (false, false) => LessonKind::Guide,
            _ => return Err("教材分類無效".into()),
        };
        lessons.push(Lesson {
            slug,
            kind,
            id: field(f)?,
            entry_title: field(f)?,
            title: field(f)?,
            description: field(f)?,
            stage_title: field(f)?,
            label_icon: field(f)?,
            source_id: field(f)?,
            intro: field(f)?,
            rest: field(f)?,
            toc: (0..count(f, 5000)?)
                .map(|_| Ok((field(f)?, field(f)?)))
                .collect::<Result<Vec<_>>>()?,
            ownership: read_ownership(f)?,
            flow: read_flow(f)?,
            quiz: read_quiz(f)?,
        });
    }
    for html in render(
        base,
        [chevron, list, check, target, layers, flow_icon],
        &stages,
        &lessons,
    )? {
        emit(out, &html)?;
    }
    Ok(())
}

pub(super) fn render(
    base: &str,
    icons: [&str; 6],
    stages: &[Stage<'_>],
    lessons: &[Lesson<'_>],
) -> Result<Vec<String>> {
    let [chevron, list, check, target, layers, flow_icon] = icons;
    let mut out = Vec::new();
    let footer = fill(include_str!("../templates/footer.html"), &[("base", base)])?;
    let mut chapters = Vec::new();
    let mut index = String::from("<section class=\"outline-links\"><h2>網站章節索引</h2>");
    for stage in stages {
        let title = escape(stage.title);
        index.push_str(&format!("<h3>{title}</h3><ol>"));
        for c in &stage.chapters {
            let Chapter { id, path, title } = *c;
            let number = chapter_number(id)?;
            if !chapter_path(path) || chapters.iter().any(|c: &Chapter<'_>| c.id == id) {
                return Err("章節索引無效".into());
            }
            index.push_str(&format!(
                r#"<li><a href="{base}{path}/">第 {number} 章　{}</a></li>"#,
                escape(title)
            ));
            chapters.push(Chapter { id, path, title });
        }
        index.push_str("</ol>");
    }
    index.push_str(&format!(
        r#"<p><a href="{base}resources/">瀏覽延伸教材</a></p></section>"#
    ));
    for l in lessons {
        let slug = l.slug;
        if !safe(slug) {
            return Err("教材 slug 無效".into());
        }
        let chapter = matches!(l.kind, LessonKind::Chapter);
        let resource = matches!(l.kind, LessonKind::Resource);
        let id = l.id;
        let entry_title = escape(l.entry_title);
        let title = escape(l.title);
        let description = escape(l.description);
        let label = if chapter {
            format!("第 {} 章", chapter_number(id)?)
        } else if resource {
            format!("延伸教材 {}", escape(id))
        } else {
            "課程導覽".into()
        };
        let stage_title = escape(l.stage_title);
        let label_icon = l.label_icon;
        let source_id = l.source_id;
        if !safe(source_id) {
            return Err("來源 ID 無效".into());
        }
        let source = format!("https://docs.google.com/document/d/{source_id}/edit");
        let (intro, rest) = (l.intro, l.rest);
        let outline = outline(&l.toc, chapter, slug == "ownership");
        let extra = if slug == "guide" {
            format!(
                r#"<p class="guide-links"><a href="{base}guide/outline/">課程總目錄</a><a href="{base}resources/">延伸資源</a></p>"#
            )
        } else {
            String::new()
        };
        let outline_links = if slug == "outline" {
            index.as_str()
        } else {
            ""
        };
        let pagination = if chapter {
            pagination(base, &chapters, id)?
        } else {
            String::new()
        };
        let lab = ownership(l.ownership.as_ref(), layers)?;
        let flow = flow(l.flow.as_ref(), flow_icon)?;
        let quiz = quiz(l.quiz.as_ref(), slug, target)?;
        let stage = if stage_title.is_empty() {
            String::new()
        } else {
            format!(r#"<span aria-hidden="true"> · </span>{stage_title}"#)
        };
        let crumb = if resource {
            format!(r#"<a href="{base}resources/">延伸資源</a>{chevron}"#)
        } else {
            String::new()
        };
        let completion = if chapter {
            format!(
                r#"<section class="completion" aria-labelledby="complete-title"><div><h2 id="complete-title">完成本章</h2><p data-completion-status role="status">試著用自己的話說明本章重點，再記下閱讀進度。</p></div><button type="button" class="button outline" id="complete" aria-pressed="false" hidden>{check} <span>標記完成</span></button></section>"#
            )
        } else {
            String::new()
        };
        out.push(fill(
            include_str!("../templates/reader.html"),
            &[
                ("base", base),
                ("chevron", chevron),
                ("resourceCrumb", &crumb),
                ("entryTitle", &entry_title),
                ("labelIcon", label_icon),
                ("label", &label),
                ("stage", &stage),
                ("title", &title),
                ("description", &description),
                ("list", list),
                ("outline", &outline),
                ("extra", &extra),
                ("outlineLinks", outline_links),
                ("ownership", &lab),
                ("intro", intro),
                ("flow", &flow),
                ("rest", rest),
                ("quiz", &quiz),
                ("completion", &completion),
                ("source", &source),
                ("pagination", &pagination),
                ("footer", &footer),
            ],
        )?);
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quiz_checks_answers_and_escapes() {
        let fields = ["true", "<問>", "&答", "2", "1", "A", "B"];
        let data = read_quiz(&mut fields.into_iter()).unwrap();
        let html = quiz(data.as_ref(), "test", "").unwrap();
        assert!(html.contains("&lt;問&gt;"));
        assert!(html.contains("<strong>B</strong>"));
        assert!(
            quiz(
                read_quiz(&mut ["true", "Q", "E", "1", "1", "A"].into_iter())
                    .unwrap()
                    .as_ref(),
                "test",
                ""
            )
            .is_err()
        );
    }
}
