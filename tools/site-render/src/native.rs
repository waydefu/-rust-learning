//! Node-independent course input, native rendering, and CLI.
use super::schema;
use super::{
    Fence, Heading, Result, callout, fences, highlight, landing, layout, markdown, metadata,
    navigation, page_data, reader,
};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::de::DeserializeOwned;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::Path,
};
#[path = "native_check.rs"]
mod check;
fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn read(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(path).map_err(fail)
}
fn load<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    serde_json::from_str(&read(path)?).map_err(|e| format!("{}: {e}", path.display()))
}
fn safe_path(s: &str) -> bool {
    !s.is_empty()
        && s.split('/').all(|p| {
            !p.is_empty()
                && !matches!(p, "." | "..")
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
}
fn base_path() -> Result<String> {
    let mut b = env::var("BASE_PATH").unwrap_or_else(|_| "/".into());
    if !b.ends_with('/') {
        b.push('/')
    }
    if b != "/" && (!b.starts_with('/') || !safe_path(&b[1..b.len() - 1])) {
        return Err("BASE_PATH 必須是 / 或安全部署子路徑".into());
    }
    Ok(b)
}
#[derive(Debug)]
struct Entry {
    id: String,
    slug: String,
    path: String,
    title: String,
    description: String,
    group: String,
    source_id: String,
    source_valid: bool,
}
impl Entry {
    fn chapter(&self) -> bool {
        self.id.len() == 2 && self.id.bytes().all(|b| b.is_ascii_digit())
    }
    fn resource(&self) -> bool {
        self.id.len() == 3
            && self.id.starts_with('A')
            && self.id.as_bytes()[1..].iter().all(u8::is_ascii_digit)
    }
    fn href(&self, base: &str) -> String {
        format!("{base}{}/", self.path)
    }
}
struct Example {
    info: String,
    code: String,
    line: usize,
    markup: String,
    ending: String,
}
struct Lesson {
    entry: Entry,
    source: String,
    title: String,
    headings: Vec<(u8, String, String)>,
    examples: Vec<Example>,
    notes: Vec<String>,
    full_text: String,
}
struct Course {
    data: schema::SiteData,
    lessons: Vec<Lesson>,
}
fn source_date(s: &str) -> bool {
    let Some((date, time)) = s.split_once('T') else {
        return false;
    };
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let Ok(year) = parts[0].parse::<u32>() else {
        return false;
    };
    let Ok(month) = parts[1].parse::<usize>() else {
        return false;
    };
    let Ok(day) = parts[2].parse::<u32>() else {
        return false;
    };
    let days = [
        31,
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month) || day == 0 || day > days[month - 1] {
        return false;
    }
    let Some(time) = time.strip_suffix('Z') else {
        return false;
    };
    let (clock, fraction) = time.split_once('.').unwrap_or((time, ""));
    if !fraction.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let v: Vec<_> = clock.split(':').collect();
    v.len() == 3
        && v.iter()
            .all(|p| p.len() == 2 && p.bytes().all(|b| b.is_ascii_digit()))
        && v[0].parse::<u8>().is_ok_and(|v| v < 24)
        && v[1].parse::<u8>().is_ok_and(|v| v < 60)
        && v[2].parse::<u8>().is_ok_and(|v| v < 60)
}
fn source_examples(source: &str) -> Result<Vec<Example>> {
    let extracted = fences::extract(source)?;
    let mut spans = Vec::new();
    for (event, range) in Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .into_offset_iter()
    {
        if matches!(
            event,
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
        ) {
            let block = &source[range.clone()];
            let start = block.lines().next().unwrap_or("").trim();
            let mark = start.chars().next().ok_or("程式碼起始缺失")?;
            let markup: String = start.chars().take_while(|c| *c == mark).collect();
            let ending = block.lines().last().unwrap_or("").trim().to_owned();
            let line = source[..range.start]
                .bytes()
                .filter(|b| *b == b'\n')
                .count()
                + 1;
            if !matches!(mark, '~' | '\x60')
                || markup.len() < 3
                || !ending.starts_with(&markup)
                || !ending.chars().all(|c| c == mark)
                || block.lines().count() < 2
            {
                return Err(format!("第 {line} 行缺少程式碼結尾"));
            }
            spans.push((line, markup, ending));
        }
    }
    if spans.len() != extracted.len() {
        return Err("程式碼範圍與內容數量不一致".into());
    }
    Ok(extracted
        .into_iter()
        .zip(spans)
        .map(|((info, code), (line, markup, ending))| Example {
            info,
            code,
            line,
            markup,
            ending,
        })
        .collect())
}
fn text_and_notes(source: &str) -> (String, Vec<String>) {
    let events: Vec<_> = Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    )
    .collect();
    let mut groups = Vec::new();
    let mut current = String::new();
    let mut notes = Vec::new();
    let mut skip = BTreeMap::new();
    for (i, e) in events.iter().enumerate() {
        if matches!(e, Event::Start(Tag::BlockQuote(_))) {
            if let Some((strong, end, kind)) = callout(&events, i) {
                skip.insert(strong + 1, end);
                notes.push(kind.to_owned());
            }
        }
    }
    fn flush(g: &mut Vec<String>, s: &mut String) {
        if !s.is_empty() {
            g.push(std::mem::take(s));
        }
    }
    let mut i = 0;
    while i < events.len() {
        if let Some(end) = skip.get(&i) {
            i = *end;
            continue;
        }
        match &events[i] {
            Event::Start(Tag::CodeBlock(_)) => {
                flush(&mut groups, &mut current);
                let mut code = String::new();
                i += 1;
                while i < events.len() && !matches!(events[i], Event::End(TagEnd::CodeBlock)) {
                    if let Event::Text(t) = &events[i] {
                        code.push_str(t)
                    }
                    i += 1;
                }
                groups.push(code);
            }
            Event::Start(
                Tag::Paragraph
                | Tag::Heading { .. }
                | Tag::Item
                | Tag::List(_)
                | Tag::BlockQuote(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::HtmlBlock,
            )
            | Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::List(_)
                | TagEnd::BlockQuote(_)
                | TagEnd::Table
                | TagEnd::TableHead
                | TagEnd::TableRow
                | TagEnd::TableCell
                | TagEnd::HtmlBlock,
            )
            | Event::Rule => flush(&mut groups, &mut current),
            Event::Text(t) | Event::Code(t) | Event::InlineHtml(t) | Event::Html(t) => {
                current.push_str(t)
            }
            _ => {}
        }
        i += 1;
    }
    flush(&mut groups, &mut current);
    let joined = groups.join(" ");
    let mut result = String::new();
    let mut gap = false;
    for c in joined.chars() {
        if c.is_whitespace() || c == '\u{feff}' {
            if !gap {
                result.push(' ')
            }
            gap = true
        } else {
            result.push(c);
            gap = false
        }
    }
    (result, notes)
}
impl Course {
    fn load() -> Result<Self> {
        let data: schema::SiteData = load("content/site-data.json")?;
        let entries: Vec<schema::CatalogEntry> = load("content/catalog.json")?;
        if entries.is_empty() || entries.len() > 1000 {
            return Err("catalog 數量無效".into());
        }
        let mut lessons = Vec::new();
        let (mut ids, mut slugs, mut paths) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for v in entries {
            let e = Entry {
                id: v.id,
                slug: v.slug,
                path: v.path,
                title: v.title,
                description: v.description,
                group: v.group,
                source_id: v.source.id,
                source_valid: source_date(&v.source.modified),
            };
            if !safe_path(&e.slug)
                || e.slug.contains('/')
                || !safe_path(&e.path)
                || e.path == "resources"
                || e.path == "404"
                || !ids.insert(e.id.clone())
                || !slugs.insert(e.slug.clone())
                || !paths.insert(e.path.clone())
                || !e.source_valid
                || e.source_id.len() < 20
                || !e
                    .source_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            {
                return Err(format!("無效或重複的教材資料：{}", e.slug));
            }
            let source = read(format!("content/{}.md", e.slug))?;
            let (title, headings) = metadata::headings(&source)?;
            let examples =
                source_examples(&source).map_err(|err| format!("{}.md: {err}", e.slug))?;
            let (full_text, notes) = text_and_notes(&source);
            lessons.push(Lesson {
                entry: e,
                source,
                title,
                headings,
                examples,
                notes,
                full_text,
            });
        }
        let result = Self { data, lessons };
        let chapters = result.chapters();
        let resources = result.resources();
        if chapters.len() != 21
            || resources.len() != 8
            || chapters
                .iter()
                .enumerate()
                .any(|(i, l)| l.entry.id != format!("{:02}", i + 1))
            || resources
                .iter()
                .enumerate()
                .any(|(i, l)| l.entry.id != format!("A{:02}", i + 1))
        {
            return Err("章節及延伸教材編號不完整".into());
        }
        let stages = result.stages();
        if chapters
            .iter()
            .any(|l| stages.iter().filter(|st| st.id == l.entry.group).count() != 1)
        {
            return Err("教材群組不完整".into());
        }
        let files = fs::read_dir("content")
            .map_err(fail)?
            .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(fail)?;
        let found: BTreeSet<_> = files.into_iter().filter(|p| p.ends_with(".md")).collect();
        if found != slugs.iter().map(|s| format!("{s}.md")).collect() {
            return Err("Markdown 檔案與 catalog 不一致".into());
        }
        Ok(result)
    }
    fn chapters(&self) -> Vec<&Lesson> {
        self.lessons.iter().filter(|l| l.entry.chapter()).collect()
    }
    fn resources(&self) -> Vec<&Lesson> {
        self.lessons.iter().filter(|l| l.entry.resource()).collect()
    }
    fn stages(&self) -> &[schema::Stage] {
        &self.data.stages
    }
    fn stage_chapters(&self, stage: &schema::Stage) -> Vec<&Lesson> {
        self.chapters()
            .into_iter()
            .filter(|l| l.entry.group == stage.id)
            .collect()
    }
    fn icon(&self, name: &str, size: usize) -> Result<String> {
        let path = self
            .data
            .icons
            .get(name)
            .ok_or_else(|| format!("未知圖示 {name}"))?;
        Ok(format!(
            "<svg width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">{path}</svg>"
        ))
    }
    fn quiz(&self, l: &Lesson) -> Option<&schema::Quiz> {
        self.data.quizzes.get(&l.entry.slug)
    }
    fn flow(&self, l: &Lesson) -> Option<&schema::Flow> {
        self.data.flows.get(&l.entry.slug)
    }
    fn ownership(&self) -> &[schema::OwnershipStep; 4] {
        &self.data.ownership_steps
    }
    fn index(&self, base: &str) -> Vec<schema::SearchEntry> {
        self.lessons
            .iter()
            .map(|l| schema::SearchEntry {
                title: l.entry.title.clone(),
                id: l.entry.id.clone(),
                description: l.entry.description.clone(),
                href: l.entry.href(base),
                text: l.full_text.clone(),
                headings: l
                    .headings
                    .iter()
                    .filter(|(level, _, _)| *level <= 3)
                    .map(|(_, label, id)| schema::SearchHeading {
                        id: id.clone(),
                        label: label.clone(),
                    })
                    .collect(),
            })
            .collect()
    }
    fn preflight(&self) -> Result<()> {
        let names: Vec<_> = self
            .chapters()
            .iter()
            .map(|l| format!("content/{}.md", l.entry.slug))
            .collect();
        let mut total = 0;
        let mut errors = Vec::new();
        for name in &names {
            match lesson_check::check_file(Path::new(name)) {
                Ok(count) => total += count,
                Err(messages) => {
                    errors.extend(
                        messages
                            .into_iter()
                            .map(|message| format!("{name}: {message}")),
                    );
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        println!("Rust 教材檢查：{} 章、{total} 個教學方塊通過", names.len());
        Ok(())
    }
}
struct Page {
    current: String,
    id: String,
    title: String,
    description: String,
    output: String,
}
fn pages(course: &Course) -> Vec<Page> {
    let mut result = vec![
        Page {
            current: "".into(),
            id: "".into(),
            title: "Rust 學習工坊".into(),
            description:
                "繁體中文 Rust 2024 學習平台。從編譯、所有權與借用一路到非同步和網頁伺服器。".into(),
            output: "index.html".into(),
        },
        Page {
            current: "resources".into(),
            id: "".into(),
            title: "延伸資源".into(),
            description: "八份 Rust 延伸教材與推理題庫。".into(),
            output: "resources/index.html".into(),
        },
        Page {
            current: "404".into(),
            id: "".into(),
            title: "找不到頁面".into(),
            description: "頁面可能已移動。".into(),
            output: "404.html".into(),
        },
    ];
    result.extend(course.lessons.iter().map(|l| Page {
        current: l.entry.path.clone(),
        id: l.entry.id.clone(),
        title: l.entry.title.clone(),
        description: l.entry.description.clone(),
        output: format!("{}/index.html", l.entry.path),
    }));
    result
}
fn render(course: &Course, base: &str) -> Result<(Vec<(String, String)>, schema::RenderReport)> {
    let chapters = course.chapters();
    let resources = course.resources();
    let stages = course.stages();
    let pages = pages(course);
    let nav_icons = [
        course.icon("home", 19)?,
        course.icon("route", 19)?,
        course.icon("layers", 19)?,
        course.icon("down", 16)?,
        course.icon("check", 14)?,
    ];
    let nav_icons = nav_icons.each_ref().map(String::as_str);
    let nav_stages = stages
        .iter()
        .map(|stage| {
            Ok(navigation::Stage {
                id: &stage.id,
                title: &stage.title,
                chapters: course
                    .stage_chapters(stage)
                    .into_iter()
                    .map(|l| navigation::Chapter {
                        id: &l.entry.id,
                        path: &l.entry.path,
                        title: &l.entry.title,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let nav: Vec<_> = pages
        .iter()
        .map(|p| {
            navigation::navigation(
                base,
                &nav_stages,
                resources.len(),
                &nav_icons,
                &p.current,
                &p.id,
            )
        })
        .collect();
    let landing_icons = [
        course.icon("chevron", 14)?,
        course.icon("route", 18)?,
        course.icon("book", 16)?,
        course.icon("layers", 16)?,
        course.icon("target", 24)?,
        course.icon("list", 17)?,
        course.icon("check", 15)?,
        course.icon("chevron", 15)?,
    ];
    fn entry(l: &Lesson) -> landing::Entry<'_> {
        landing::Entry {
            id: &l.entry.id,
            path: &l.entry.path,
            title: &l.entry.title,
            description: &l.entry.description,
        }
    }
    let landing_stages = stages
        .iter()
        .map(|stage| {
            Ok(landing::Stage {
                id: &stage.id,
                title: &stage.title,
                description: &stage.description,
                icon: course.icon(&stage.icon, 22)?,
                chapters: course
                    .stage_chapters(stage)
                    .into_iter()
                    .map(|l| (entry(l), l.entry.slug == "ownership"))
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut bodies = landing::render(&landing::Landing {
        base,
        icons: landing_icons.each_ref().map(String::as_str),
        first: &chapters.first().ok_or("沒有章節")?.entry.path,
        chapter_count: chapters.len(),
        resources: resources.into_iter().map(entry).collect(),
        stages: landing_stages,
    })?;
    let reader_icons = [
        course.icon("chevron", 14)?,
        course.icon("list", 17)?,
        course.icon("check", 17)?,
        course.icon("target", 18)?,
        course.icon("layers", 17)?,
        course.icon("chevron", 16)?,
    ];
    let reader_stages = stages
        .iter()
        .map(|stage| {
            Ok(reader::Stage {
                title: &stage.title,
                chapters: course
                    .stage_chapters(stage)
                    .into_iter()
                    .map(|l| reader::Chapter {
                        id: &l.entry.id,
                        path: &l.entry.path,
                        title: &l.entry.title,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut markdown_report = Vec::new();
    let copy = course.icon("copy", 16)?;
    let mut rendered_lessons = Vec::new();
    for l in &course.lessons {
        let e = &l.entry;
        let stage = stages.iter().find(|st| st.id == e.group);
        let stage_title = stage.map(|st| st.title.as_str()).unwrap_or("");
        let stage_icon = course.icon(stage.map(|st| st.icon.as_str()).unwrap_or("book"), 17)?;
        let mut rendered = Vec::new();
        for ex in &l.examples {
            let lang = ex.info.split_whitespace().next().unwrap_or("");
            *counts.entry(lang.to_owned()).or_default() += 1;
            rendered.push(highlight::block(
                &ex.info,
                &highlight::highlight(lang, &ex.code),
                &copy,
            ));
        }
        let headings: Vec<_> = l
            .headings
            .iter()
            .map(|(level, _, id)| Heading { level: *level, id })
            .collect();
        let fences: Vec<_> = l
            .examples
            .iter()
            .zip(&rendered)
            .map(|(ex, rendered)| Fence {
                info: &ex.info,
                code: &ex.code,
                rendered,
            })
            .collect();
        let (intro, rest) = markdown(&l.source, &headings, &fences)?;
        let (intro, rest) = if course.flow(l).is_some() {
            (intro, rest)
        } else {
            (String::new(), intro + &rest)
        };
        rendered_lessons.push((stage_title, stage_icon, intro, rest));
        markdown_report.push(schema::MarkdownReport {
            slug: e.slug.clone(),
            renderer: "rust".into(),
            reason: String::new(),
        });
    }
    let reader_lessons = course
        .lessons
        .iter()
        .zip(&rendered_lessons)
        .map(|(l, (stage_title, stage_icon, intro, rest))| {
            let e = &l.entry;
            let kind = if e.chapter() {
                reader::LessonKind::Chapter
            } else if e.resource() {
                reader::LessonKind::Resource
            } else {
                reader::LessonKind::Guide
            };
            let ownership = if e.slug == "ownership" {
                Some(
                    course
                        .ownership()
                        .each_ref()
                        .map(|step| (step.code.as_str(), step.message.as_str())),
                )
            } else {
                None
            };
            let flow = course.flow(l).map(|f| reader::Flow {
                title: &f.title,
                caption: &f.caption,
                nodes: f.nodes.iter().map(String::as_str).collect(),
            });
            let quiz = course.quiz(l).map(|q| reader::Quiz {
                prompt: &q.prompt,
                explanation: &q.explanation,
                choices: q.choices.iter().map(String::as_str).collect(),
                answer: q.answer,
            });
            Ok(reader::Lesson {
                slug: &e.slug,
                kind,
                id: &e.id,
                entry_title: &e.title,
                title: &l.title,
                description: &e.description,
                stage_title,
                label_icon: stage_icon,
                source_id: &e.source_id,
                intro,
                rest,
                toc: l
                    .headings
                    .iter()
                    .filter(|(level, _, _)| *level == 2)
                    .map(|(_, label, id)| (id.as_str(), label.as_str()))
                    .collect(),
                ownership,
                flow,
                quiz,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    bodies.extend(reader::render(
        base,
        reader_icons.each_ref().map(String::as_str),
        &reader_stages,
        &reader_lessons,
    )?);
    let paths: Vec<_> = chapters
        .iter()
        .map(|l| (l.entry.id.as_str(), l.entry.path.as_str()))
        .collect();
    let data = pages
        .iter()
        .map(|p| {
            let l = course.lessons.iter().find(|l| l.entry.path == p.current);
            let chapter = l.filter(|l| l.entry.chapter()).map(|l| l.entry.id.as_str());
            let ownership = if l.is_some_and(|l| l.entry.slug == "ownership") {
                Some(
                    course
                        .ownership()
                        .iter()
                        .map(|step| page_data::OwnershipStep {
                            code: &step.code,
                            source: &step.source,
                            target: &step.target,
                            owner: step.owner.as_deref().filter(|v| !v.is_empty()),
                            borrowed: step.borrowed,
                            message: &step.message,
                        })
                        .collect(),
                )
            } else {
                None
            };
            let answer = l.and_then(|l| course.quiz(l)).map(|q| q.answer);
            Ok(page_data::render(
                base,
                &paths,
                &page_data::PageData {
                    chapter,
                    ownership,
                    answer,
                },
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    if nav.len() != pages.len() || bodies.len() != pages.len() || data.len() != pages.len() {
        return Err("頁面數量不相容".into());
    }
    let icons = [
        course.icon("menu", 20)?,
        course.icon("search", 19)?,
        course.icon("sun", 20)?,
        course.icon("moon", 20)?,
        course.icon("github", 20)?,
        course.icon("close", 20)?,
        course.icon("search", 20)?,
    ];
    let icons = icons.each_ref().map(String::as_str);
    let html = pages
        .iter()
        .enumerate()
        .map(|(i, p)| {
            layout(
                base,
                &p.title,
                &p.description,
                &nav[i],
                &bodies[i],
                &data[i],
                &icons,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let total: usize = counts.values().sum();
    let by_language: BTreeMap<_, _> = counts
        .into_iter()
        .map(|(k, n)| {
            (
                k,
                schema::LanguageCount {
                    native: n,
                    total: n,
                },
            )
        })
        .collect();
    let report = schema::RenderReport {
        layout: "rust".into(),
        pages: pages.len(),
        markdown: markdown_report,
        highlight: schema::HighlightReport {
            renderer: "rust".into(),
            native: total,
            total,
            fallback: 0,
            by_language,
        },
    };
    Ok((
        pages
            .into_iter()
            .zip(html)
            .map(|(p, html)| (p.output, html))
            .collect(),
        report,
    ))
}
fn build(course: &Course, base: &str) -> Result<()> {
    course.preflight()?;
    let (mut output, report) = render(course, base)?;
    let hrefs: Vec<_> = course.lessons.iter().map(|l| l.entry.href(base)).collect();
    let lessons: Vec<_> = course
        .lessons
        .iter()
        .zip(&hrefs)
        .map(|(l, href)| site_tools::search_index::Lesson {
            title: &l.entry.title,
            id: &l.entry.id,
            description: &l.entry.description,
            href,
            text: &l.full_text,
            headings: l
                .headings
                .iter()
                .map(|(level, label, id)| site_tools::search_index::Heading {
                    id,
                    label,
                    level: *level,
                })
                .collect(),
        })
        .collect();
    let search = site_tools::search_index::render(&lessons)?;
    output.extend([
        (".nojekyll".into(), String::new()),
        ("assets/search-index.json".into(), search),
        (
            "third-party-licenses.txt".into(),
            include_str!("../third-party-licenses.txt").into(),
        ),
        (
            "build-render-report.json".into(),
            serde_json::to_string_pretty(&report).map_err(fail)?,
        ),
    ]);
    let files: Vec<_> = output
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    site_tools::site_write::write_site(Path::new("."), &files).map_err(fail)?;
    println!(
        "Built {} HTML pages at {base} (Rust CLI, no Node)",
        pages(course).len()
    );
    Ok(())
}
fn check_examples(course: &Course) -> Result<()> {
    let lessons: Vec<_> = course
        .lessons
        .iter()
        .map(|lesson| super::examples::Lesson {
            slug: &lesson.entry.slug,
            examples: lesson
                .examples
                .iter()
                .map(|example| super::examples::Example {
                    line: example.line,
                    info: &example.info,
                    source: &example.code,
                })
                .collect(),
        })
        .collect();
    let report = super::examples::check(&lessons)?;
    println!("{}", report.version);
    for message in &report.messages {
        println!("{message}");
    }
    let (compiled, run, error, project) =
        (report.compiled, report.run, report.error, report.project);
    println!(
        "Rust 區塊通過 {} 段（獨立編譯 {compiled} 段，其中 {run} 段執行或執行測試，預期診斷 {error} 段，Cargo 專案 {project} 段）",
        compiled + error + project
    );
    Ok(())
}
pub(super) fn run(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        return Err("使用方式：site-render build | check | check-rust | dev".into());
    }
    if matches!(args[0].as_str(), "--help" | "-h") {
        println!(
            "site-render build | check | check-rust | dev\nBASE_PATH、HOST、PORT 由環境變數設定。"
        );
        return Ok(());
    }
    let base = base_path()?;
    let course = Course::load()?;
    match args[0].as_str() {
        "build" => build(&course, &base),
        "check" => check::check(&course, &base),
        "check-rust" => check_examples(&course),
        "dev" => {
            build(&course, &base)?;
            check::check(&course, &base)?;
            site_tools::site_preview::serve(Path::new("dist")).map_err(fail)
        }
        _ => Err("未知指令；請使用 build、check、check-rust 或 dev".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates() {
        assert!(source_date("2026-11-19T16:06:53.585Z"));
        assert!(!source_date("2026-02-30T16:06:53Z"));
    }
    #[test]
    fn text_ignores_callout_marker() {
        let (text, notes) = text_and_notes(
            "# 標題\n\n> **為何？** <!-- callout:context -->\n>\n> 內容\n\n## 節\n\n文字 **粗體**\n",
        );
        assert_eq!(text, "標題 為何？ 內容 節 文字 粗體");
        assert_eq!(notes, ["context"]);
    }
}
