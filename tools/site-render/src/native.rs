//! Node-independent course input, native rendering, and CLI.
use super::{Fence, Heading, Result, callout, fences, highlight, markdown, metadata, process};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
#[path = "native_check.rs"]
mod check;
type Fields = Vec<String>;
fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn read(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(path).map_err(fail)
}
fn load(path: impl AsRef<Path>) -> Result<Value> {
    serde_json::from_str(&read(path)?).map_err(fail)
}
fn s<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("缺少文字欄位 {key}"))
}
fn list<'a>(v: &'a Value, key: &str) -> Result<&'a [Value]> {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("缺少清單 {key}"))
}
fn num(v: &Value, key: &str) -> Result<usize> {
    v.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("缺少數量 {key}"))
}
fn add(f: &mut Fields, values: &[&str]) {
    f.extend(values.iter().map(|v| (*v).to_owned()));
}
fn number(f: &mut Fields, n: usize) {
    f.push(n.to_string());
}
fn flag(f: &mut Fields, b: bool) {
    f.push(b.to_string());
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
fn framed(mode: &str, n: usize, fields: &Fields) -> Result<String> {
    if fields.iter().any(|f| f.contains('\0')) {
        return Err("資料含 NUL".into());
    }
    Ok(format!("{mode}\0{n}\0{}\0", fields.join("\0")))
}
fn packet(mode: &str, n: usize, fields: Fields) -> Result<Vec<String>> {
    let raw = String::from_utf8(process(&framed(mode, n, &fields)?)?).map_err(fail)?;
    let mut it = raw
        .strip_suffix('\0')
        .ok_or("原生回傳缺少結尾")?
        .split('\0');
    if it.next() != Some(mode) || it.next() != Some(n.to_string().as_str()) {
        return Err("原生回傳版本不相容".into());
    }
    Ok(it.map(str::to_owned).collect())
}
fn child_tool(
    manifest: &str,
    bin: Option<&str>,
    args: &[&str],
    input: Option<&str>,
) -> Result<String> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.args(["run", "--quiet", "--locked", "--manifest-path", manifest]);
    if let Some(bin) = bin {
        command.args(["--bin", bin]);
    }
    if !args.is_empty() {
        command.arg("--").args(args);
    }
    command.stderr(Stdio::inherit());
    if let Some(input) = input {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(fail)?;
        child
            .stdin
            .take()
            .ok_or("無法傳遞資料")?
            .write_all(input.as_bytes())
            .map_err(fail)?;
        let result = child.wait_with_output().map_err(fail)?;
        if !result.status.success() {
            return Err(format!("原生工具失敗：{manifest} {bin:?}"));
        }
        String::from_utf8(result.stdout).map_err(fail)
    } else {
        let status = command.status().map_err(fail)?;
        if status.success() {
            Ok(String::new())
        } else {
            Err(format!("原生工具失敗：{manifest} {bin:?}"))
        }
    }
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
    data: Value,
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
        let data = load("content/site-data.json")?;
        let catalog = load("content/catalog.json")?;
        let entries = catalog.as_array().ok_or("catalog 必須是清單")?;
        if entries.is_empty() || entries.len() > 1000 {
            return Err("catalog 數量無效".into());
        }
        let mut lessons = Vec::new();
        let (mut ids, mut slugs, mut paths) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for v in entries {
            let source = v.get("source").ok_or("教材缺少來源")?;
            let e = Entry {
                id: s(v, "id")?.into(),
                slug: s(v, "slug")?.into(),
                path: s(v, "path")?.into(),
                title: s(v, "title")?.into(),
                description: s(v, "description")?.into(),
                group: v.get("group").and_then(Value::as_str).unwrap_or("").into(),
                source_id: s(source, "id")?.into(),
                source_valid: source_date(s(source, "modified")?)
                    && num(source, "paragraphs").is_ok(),
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
        let stages = result.stages()?;
        if chapters.iter().any(|l| {
            stages
                .iter()
                .filter(|st| s(st, "id").ok() == Some(l.entry.group.as_str()))
                .count()
                != 1
        }) {
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
    fn stages(&self) -> Result<&[Value]> {
        list(&self.data, "stages")
    }
    fn stage_chapters(&self, stage: &Value) -> Result<Vec<&Lesson>> {
        let id = s(stage, "id")?;
        Ok(self
            .chapters()
            .into_iter()
            .filter(|l| l.entry.group == id)
            .collect())
    }
    fn icon(&self, name: &str, size: usize) -> Result<String> {
        let path = self
            .data
            .get("icons")
            .and_then(|v| v.get(name))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("未知圖示 {name}"))?;
        Ok(format!(
            "<svg width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">{path}</svg>"
        ))
    }
    fn icons(&self, fields: &mut Fields, icons: &[(&str, usize)]) -> Result<()> {
        for (name, size) in icons {
            fields.push(self.icon(name, *size)?)
        }
        Ok(())
    }
    fn quiz(&self, l: &Lesson) -> Option<&Value> {
        self.data.get("quizzes")?.get(&l.entry.slug)
    }
    fn flow(&self, l: &Lesson) -> Option<&Value> {
        self.data.get("flows")?.get(&l.entry.slug)
    }
    fn ownership(&self) -> Result<&[Value]> {
        list(&self.data, "ownershipSteps")
    }
    fn index(&self, base: &str) -> Value {
        Value::Array(self.lessons.iter().map(|l|json!({"title":l.entry.title,"id":l.entry.id,"description":l.entry.description,"href":l.entry.href(base),
        "headings":l.headings.iter().filter(|(n,_,_)|*n<=3).map(|(_,label,id)|json!({"id":id,"label":label})).collect::<Vec<_>>(),"text":l.full_text})).collect())
    }
    fn preflight(&self) -> Result<()> {
        let names: Vec<_> = self
            .chapters()
            .iter()
            .map(|l| format!("content/{}.md", l.entry.slug))
            .collect();
        child_tool(
            "tools/lesson-check/Cargo.toml",
            None,
            &names.iter().map(String::as_str).collect::<Vec<_>>(),
            None,
        )?;
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
fn render(course: &Course, base: &str) -> Result<(Vec<(String, String)>, Value)> {
    let chapters = course.chapters();
    let resources = course.resources();
    let stages = course.stages()?;
    let pages = pages(course);
    let mut fields = vec![base.to_owned(), resources.len().to_string()];
    course.icons(
        &mut fields,
        &[
            ("home", 19),
            ("route", 19),
            ("layers", 19),
            ("down", 16),
            ("check", 14),
        ],
    )?;
    number(&mut fields, stages.len());
    for stage in stages {
        let items = course.stage_chapters(stage)?;
        add(&mut fields, &[s(stage, "id")?, s(stage, "title")?]);
        number(&mut fields, items.len());
        for l in items {
            add(&mut fields, &[&l.entry.id, &l.entry.path, &l.entry.title]);
        }
    }
    for p in &pages {
        add(&mut fields, &[&p.current, &p.id]);
    }
    let nav = packet("navigation-v1", pages.len(), fields)?;
    let mut fields = vec![base.to_owned()];
    course.icons(
        &mut fields,
        &[
            ("chevron", 14),
            ("route", 18),
            ("book", 16),
            ("layers", 16),
            ("target", 24),
            ("list", 17),
            ("check", 15),
            ("chevron", 15),
        ],
    )?;
    add(
        &mut fields,
        &[&chapters.first().ok_or("沒有章節")?.entry.path],
    );
    number(&mut fields, chapters.len());
    number(&mut fields, resources.len());
    for l in resources {
        let e = &l.entry;
        add(&mut fields, &[&e.id, &e.path, &e.title, &e.description]);
    }
    number(&mut fields, stages.len());
    for stage in stages {
        let items = course.stage_chapters(stage)?;
        add(
            &mut fields,
            &[
                s(stage, "id")?,
                s(stage, "title")?,
                s(stage, "description")?,
            ],
        );
        fields.push(course.icon(s(stage, "icon")?, 22)?);
        number(&mut fields, items.len());
        for l in items {
            let e = &l.entry;
            add(&mut fields, &[&e.id, &e.path, &e.title, &e.description]);
            flag(&mut fields, e.slug == "ownership");
        }
    }
    let mut bodies = packet("landing-v1", 3, fields)?;
    let mut fields = vec![base.to_owned()];
    course.icons(
        &mut fields,
        &[
            ("chevron", 14),
            ("list", 17),
            ("check", 17),
            ("target", 18),
            ("layers", 17),
            ("chevron", 16),
        ],
    )?;
    number(&mut fields, stages.len());
    for stage in stages {
        let items = course.stage_chapters(stage)?;
        add(&mut fields, &[s(stage, "title")?]);
        number(&mut fields, items.len());
        for l in items {
            add(&mut fields, &[&l.entry.id, &l.entry.path, &l.entry.title]);
        }
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut markdown_report = Vec::new();
    let copy = course.icon("copy", 16)?;
    for l in &course.lessons {
        let e = &l.entry;
        let stage = stages
            .iter()
            .find(|st| s(st, "id").ok() == Some(e.group.as_str()));
        let stage_title = stage.map(|st| s(st, "title")).transpose()?.unwrap_or("");
        let stage_icon = course.icon(
            stage.map(|st| s(st, "icon")).transpose()?.unwrap_or("book"),
            17,
        )?;
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
        add(&mut fields, &[&e.slug]);
        flag(&mut fields, e.chapter());
        flag(&mut fields, e.resource());
        add(
            &mut fields,
            &[
                &e.id,
                &e.title,
                &l.title,
                &e.description,
                stage_title,
                &stage_icon,
                &e.source_id,
                &intro,
                &rest,
            ],
        );
        let toc: Vec<_> = l
            .headings
            .iter()
            .filter(|(level, _, _)| *level == 2)
            .collect();
        number(&mut fields, toc.len());
        for (_, label, id) in toc {
            add(&mut fields, &[id, label]);
        }
        flag(&mut fields, e.slug == "ownership");
        if e.slug == "ownership" {
            for step in course.ownership()? {
                add(&mut fields, &[s(step, "code")?, s(step, "message")?]);
            }
        }
        let flow = course.flow(l);
        flag(&mut fields, flow.is_some());
        if let Some(f) = flow {
            add(&mut fields, &[s(f, "title")?, s(f, "caption")?]);
            let nodes = list(f, "nodes")?;
            number(&mut fields, nodes.len());
            for n in nodes {
                add(&mut fields, &[n.as_str().ok_or("流程圖節點無效")?]);
            }
        }
        let quiz = course.quiz(l);
        flag(&mut fields, quiz.is_some());
        if let Some(q) = quiz {
            add(&mut fields, &[s(q, "prompt")?, s(q, "explanation")?]);
            let choices = list(q, "choices")?;
            number(&mut fields, choices.len());
            number(&mut fields, num(q, "answer")?);
            for c in choices {
                add(&mut fields, &[c.as_str().ok_or("測驗選項無效")?]);
            }
        }
        markdown_report.push(json!({"slug":e.slug,"renderer":"rust","reason":""}));
    }
    bodies.extend(packet("reader-v1", course.lessons.len(), fields)?);
    let mut fields = vec![base.to_owned()];
    number(&mut fields, chapters.len());
    for l in chapters {
        add(&mut fields, &[&l.entry.id, &l.entry.path]);
    }
    for p in &pages {
        let l = course.lessons.iter().find(|l| l.entry.path == p.current);
        let chapter = l
            .filter(|l| l.entry.chapter())
            .map(|l| l.entry.id.as_str())
            .unwrap_or("");
        let ownership = l.is_some_and(|l| l.entry.slug == "ownership");
        add(&mut fields, &[chapter]);
        flag(&mut fields, ownership);
        if ownership {
            let steps = course.ownership()?;
            number(&mut fields, steps.len());
            for step in steps {
                add(
                    &mut fields,
                    &[
                        s(step, "code")?,
                        s(step, "source")?,
                        s(step, "target")?,
                        step.get("owner").and_then(Value::as_str).unwrap_or(""),
                    ],
                );
                fields.push(
                    step.get("borrowed")
                        .and_then(Value::as_bool)
                        .map(|b| b.to_string())
                        .unwrap_or_default(),
                );
                add(&mut fields, &[s(step, "message")?]);
            }
        }
        fields.push(
            l.and_then(|l| course.quiz(l))
                .map(|q| num(q, "answer").map(|v| v.to_string()))
                .transpose()?
                .unwrap_or_default(),
        );
    }
    let data = packet("page-data-v1", pages.len(), fields)?;
    if nav.len() != pages.len() || bodies.len() != pages.len() || data.len() != pages.len() {
        return Err("頁面數量不相容".into());
    }
    let mut fields = vec![base.to_owned()];
    course.icons(
        &mut fields,
        &[
            ("menu", 20),
            ("search", 19),
            ("sun", 20),
            ("moon", 20),
            ("github", 20),
            ("close", 20),
            ("search", 20),
        ],
    )?;
    for (i, p) in pages.iter().enumerate() {
        add(
            &mut fields,
            &[&p.title, &p.description, &nav[i], &bodies[i], &data[i]],
        );
    }
    let html = packet("layout-v1", pages.len(), fields)?;
    let total: usize = counts.values().sum();
    let by_language: BTreeMap<_, _> = counts
        .into_iter()
        .map(|(k, n)| (k, json!({"native":n,"total":n})))
        .collect();
    let report = json!({"layout":"rust","pages":pages.len(),"markdown":markdown_report,"highlight":{"renderer":"rust","native":total,"total":total,"fallback":0,"byLanguage":by_language}});
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
    let mut fields = Vec::new();
    for l in &course.lessons {
        let e = &l.entry;
        add(
            &mut fields,
            &[&e.title, &e.id, &e.description, &e.href(base), &l.full_text],
        );
        number(&mut fields, l.headings.len());
        for (level, label, id) in &l.headings {
            add(&mut fields, &[id, label, &level.to_string()]);
        }
    }
    let search = child_tool(
        "tools/site-tools/Cargo.toml",
        Some("search-index"),
        &[],
        Some(&framed("rust-search-v1", course.lessons.len(), &fields)?),
    )?;
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
    let fields = output
        .iter()
        .flat_map(|(p, s)| [p.clone(), s.clone()])
        .collect();
    child_tool(
        "tools/site-tools/Cargo.toml",
        Some("site-write"),
        &[],
        Some(&framed("site-write-v1", output.len(), &fields)?),
    )?;
    println!(
        "Built {} HTML pages at {base} (Rust CLI, no Node)",
        pages(course).len()
    );
    Ok(())
}
fn check_examples(course: &Course) -> Result<()> {
    let mut fields = Vec::new();
    for l in &course.lessons {
        add(&mut fields, &[&l.entry.slug]);
        number(&mut fields, l.examples.len());
        for e in &l.examples {
            number(&mut fields, e.line);
            add(&mut fields, &[&e.info, &e.code]);
        }
    }
    let output = packet("examples-v1", course.lessons.len(), fields)?;
    if output.len() < 6 {
        return Err("範例執行器結果不完整".into());
    }
    let n = |i: usize| output[i].parse::<usize>().map_err(fail);
    let (compiled, run, error, project, messages) = (n(1)?, n(2)?, n(3)?, n(4)?, n(5)?);
    if output.len() != 6 + messages {
        return Err("範例訊息不完整".into());
    }
    println!("{}", output[0]);
    for message in &output[6..] {
        println!("{message}");
    }
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
            child_tool(
                "tools/site-tools/Cargo.toml",
                Some("site-preview"),
                &["dist"],
                None,
            )
            .map(|_| ())
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
