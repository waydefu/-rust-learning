//! Native static rendering, highlighting, document templates and verification.
//! The production CLI reads course data natively; Node is a test oracle only.
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html,
};
use std::{
    collections::HashMap,
    io::{self, Read, Write},
    process::ExitCode,
};

mod examples;
mod fences;
mod highlight;
mod landing;
mod metadata;
mod native;
mod navigation;
mod page_data;
mod reader;
mod site_check;

type Result<T> = std::result::Result<T, String>;
const LIMIT: u64 = 64 * 1024 * 1024;
const KINDS: [&str; 5] = ["context", "analogy", "walkthrough", "warning", "practice"];
const ICONS: [&str; 7] = [
    "menu", "search19", "sun", "moon", "github", "close", "search",
];
const SITE: &str = "Rust 學習工坊";
const TEMPLATE: &str = include_str!("../templates/layout.html");

fn escape(value: &str) -> String {
    let mut result = String::new();
    for c in value.chars() {
        result.push_str(match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&#39;",
            _ => {
                result.push(c);
                continue;
            }
        });
    }
    result
}

fn field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Result<&'a str> {
    fields.next().ok_or_else(|| "渲染資料不完整".into())
}

fn count<'a>(fields: &mut impl Iterator<Item = &'a str>, max: usize) -> Result<usize> {
    field(fields)?
        .parse::<usize>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| "渲染資料數量無效".into())
}

struct Heading<'a> {
    level: u8,
    id: &'a str,
}
struct Fence<'a> {
    info: &'a str,
    code: &'a str,
    rendered: &'a str,
}

fn safe_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.chars().any(|c| c.is_control()) {
        return false;
    }
    let prefix = trimmed.split(['/', '?', '#']).next().unwrap_or("");
    match prefix.split_once(':') {
        Some((scheme, _)) => matches!(
            scheme.to_ascii_lowercase().as_str(),
            "http" | "https" | "mailto" | "ftp"
        ),
        None => true,
    }
}

fn raw<'a>(value: String) -> Event<'a> {
    Event::Html(CowStr::from(value))
}

fn callout<'a>(events: &[Event<'a>], start: usize) -> Option<(usize, usize, &'a str)> {
    if !matches!(events.get(start + 1), Some(Event::Start(Tag::Paragraph)))
        || !matches!(events.get(start + 2), Some(Event::Start(Tag::Strong)))
    {
        return None;
    }
    let end =
        (start + 3..events.len()).find(|&i| matches!(events[i], Event::End(TagEnd::Paragraph)))?;
    let marker = end.checked_sub(1)?;
    let Event::InlineHtml(comment) = events.get(marker)? else {
        return None;
    };
    let kind = comment
        .strip_prefix("<!-- callout:")?
        .strip_suffix(" -->")?;
    let kind = KINDS.into_iter().find(|value| *value == kind)?;
    let strong_end =
        (start + 3..marker).find(|&i| matches!(events[i], Event::End(TagEnd::Strong)))?;
    if !events[strong_end + 1..marker]
        .iter()
        .all(|e| matches!(e, Event::Text(t) if t.trim().is_empty()))
    {
        return None;
    }
    Some((strong_end, end, kind))
}

fn markdown(
    source: &str,
    headings: &[Heading<'_>],
    fences: &[Fence<'_>],
) -> Result<(String, String)> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<_> = Parser::new_ext(source, options).collect();
    // Validate the full stream before any special handling (including titles and callouts).
    if events.iter().any(|event| matches!(event,
        Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) if !safe_url(dest_url))) {
        return Err("拒絕不安全的 Markdown 網址".into());
    }
    if !matches!(
        events.first(),
        Some(Event::Start(Tag::Heading {
            level: HeadingLevel::H1,
            ..
        }))
    ) {
        return Err("教材必須以 H1 開頭".into());
    }
    let end_title = events
        .iter()
        .position(|e| matches!(e, Event::End(TagEnd::Heading(HeadingLevel::H1))))
        .ok_or("找不到教材標題結尾")?;
    let mut i = end_title + 1;
    let mut out = Vec::new();
    let mut intro = String::new();
    let mut heading_index = 0;
    let mut fence_index = 0;
    let mut quotes = Vec::new();
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading { level, .. }) => {
                if heading_index == 0 {
                    if !quotes.is_empty() {
                        return Err("第一節標題位於引用內".into());
                    }
                    html::push_html(&mut intro, out.drain(..));
                }
                let expected = headings.get(heading_index).ok_or("Rust 找到額外標題")?;
                let level = *level as u8;
                if level != expected.level || level == 1 {
                    return Err("Rust 標題層級不相容".into());
                }
                out.push(raw(format!(
                    "<h{level} id=\"{}\" tabindex=\"-1\">",
                    escape(expected.id)
                )));
                heading_index += 1;
            }
            Event::End(TagEnd::Heading(level)) => {
                out.push(raw(format!("</h{}>\n", *level as u8)));
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let CodeBlockKind::Fenced(info) = kind else {
                    return Err("目前原生渲染只接受標記語言的 fenced code".into());
                };
                let expected = fences.get(fence_index).ok_or("Rust 找到額外程式區塊")?;
                let mut code = String::new();
                i += 1;
                while i < events.len() && !matches!(events[i], Event::End(TagEnd::CodeBlock)) {
                    match &events[i] {
                        Event::Text(value) => code.push_str(value),
                        _ => return Err("Rust 程式區塊包含未知事件".into()),
                    }
                    i += 1;
                }
                if i == events.len() || info.trim() != expected.info || code != expected.code {
                    return Err("Rust 程式區塊與原有解析結果不同".into());
                }
                // This HTML comes from the trusted, escaping Rust highlighter.
                out.push(raw(expected.rendered.to_owned()));
                fence_index += 1;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                if let Some((strong_end, end, kind)) = callout(&events, i) {
                    quotes.push(true);
                    out.push(raw(format!("<div class=\"lesson-callout callout-{kind}\" data-callout=\"{kind}\" role=\"note\">\n<p class=\"callout-title\">")));
                    out.extend(events[i + 2..=strong_end].iter().cloned().map(|e| match e {
                        Event::Html(t) | Event::InlineHtml(t) => Event::Text(t),
                        other => other,
                    }));
                    out.push(raw("</p>\n".into()));
                    i = end;
                } else {
                    quotes.push(false);
                    out.push(events[i].clone());
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if quotes.pop().ok_or("引用區塊不完整")? {
                    out.push(raw("</div>\n".into()));
                } else {
                    out.push(events[i].clone());
                }
            }
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. })
                if !safe_url(dest_url) =>
            {
                return Err("拒絕不安全的 Markdown 網址".into());
            }
            // Never pass arbitrary HTML from the Markdown source through as markup.
            Event::Html(t) | Event::InlineHtml(t) => out.push(Event::Text(t.clone())),
            Event::Start(Tag::HtmlBlock) | Event::End(TagEnd::HtmlBlock) => {}
            other => out.push(other.clone()),
        }
        i += 1;
    }
    if heading_index != headings.len() || fence_index != fences.len() || !quotes.is_empty() {
        return Err("Rust 解析結果缺少標題、程式區塊或引用結尾".into());
    }
    let mut rest = String::new();
    html::push_html(&mut rest, out.into_iter());
    Ok((intro, rest))
}

// Substitute trusted HTML fragments and escaped plain text exactly once. Values
// containing literal {{...}} cannot trigger a second expansion.
fn render_template(template: &str, values: &HashMap<&str, String>) -> Result<String> {
    let mut rest = template;
    let mut result = String::new();
    while let Some((before, after)) = rest.split_once("{{") {
        result.push_str(before);
        let (key, remaining) = after.split_once("}}").ok_or("模板欄位未關閉")?;
        result.push_str(
            values
                .get(key)
                .ok_or_else(|| format!("未知模板欄位 {key}"))?,
        );
        rest = remaining;
    }
    result.push_str(rest);
    Ok(result)
}

fn layout(
    base: &str,
    title: &str,
    description: &str,
    navigation: &str,
    body: &str,
    data: &str,
    icons: &[&str],
) -> Result<String> {
    if !base.starts_with('/')
        || !base.ends_with('/')
        || base.starts_with("//")
        || !base
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"/_.-".contains(&c))
        || base.split('/').any(|s| matches!(s, "." | ".."))
        || data.contains('<')
        || icons.len() != ICONS.len()
    {
        return Err("頁面模板輸入無效".into());
    }
    let title = if title == SITE {
        title.to_owned()
    } else {
        format!("{title}｜{SITE}")
    };
    let mut values = HashMap::from([
        ("title", escape(&title)),
        ("description", escape(description)),
        ("base", escape(base)),
        ("favicon", format!("{base}assets/favicon.svg")),
        ("theme", format!("{base}assets/theme.js")),
        ("stylesheet", format!("{base}assets/site.css")),
        ("app", format!("{base}assets/app.js")),
        ("siteName", SITE.to_owned()),
        (
            "repository",
            "https://github.com/waydefu/-rust-learning".to_owned(),
        ),
        ("navigation", navigation.to_owned()),
        ("body", body.to_owned()),
        ("pageData", data.to_owned()),
    ]);
    for (key, value) in ICONS.into_iter().zip(icons) {
        values.insert(key, (*value).to_owned());
    }
    render_template(TEMPLATE, &values)
}

fn emit(out: &mut Vec<String>, value: &str) -> Result<()> {
    if value.contains('\0') {
        return Err("輸出欄位包含 NUL".into());
    }
    out.push(value.to_owned());
    Ok(())
}

fn process(input: &str) -> Result<Vec<u8>> {
    let mut fields = input
        .strip_suffix('\0')
        .ok_or("渲染資料缺少結尾")?
        .split('\0');
    let mode = field(&mut fields)?;
    let n = count(&mut fields, 1000)?;
    let result = process_fields(mode, n, fields)?;
    let mut bytes = Vec::new();
    for value in std::iter::once(mode.to_owned())
        .chain(std::iter::once(n.to_string()))
        .chain(result)
    {
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
    Ok(bytes)
}

// The migration protocol is only a boundary adapter. Native callers pass
// fields directly, without framing, UTF-8 decoding or parsing a response.
fn process_fields<'a>(
    mode: &str,
    n: usize,
    mut fields: impl Iterator<Item = &'a str>,
) -> Result<Vec<String>> {
    if n > 1000 {
        return Err("渲染資料數量超過上限".into());
    }
    let mut result = Vec::new();
    match mode {
        "metadata-v1" => metadata::process_metadata(n, &mut fields, &mut result)?,
        "anchors-v1" => metadata::process_anchors(n, &mut fields, &mut result)?,
        "navigation-v1" => navigation::process_navigation(n, &mut fields, &mut result)?,
        "landing-v1" => landing::process_landing(n, &mut fields, &mut result)?,
        "reader-v1" => reader::process_reader(n, &mut fields, &mut result)?,
        "page-data-v1" => page_data::process_data(n, &mut fields, &mut result)?,
        "highlight-v2" => highlight::process_highlight(n, &mut fields, &mut result)?,
        "fences-v1" => fences::process_fences(n, &mut fields, &mut result)?,
        "examples-v1" => examples::process_examples(n, &mut fields, &mut result)?,
        "site-check-v1" => site_check::process_site_check(n, &mut fields, &mut result)?,
        "markdown-v1" => {
            for _ in 0..n {
                let source = field(&mut fields)?;
                let mut headings = Vec::new();
                for _ in 0..count(&mut fields, 5000)? {
                    headings.push(Heading {
                        level: count(&mut fields, 6)? as u8,
                        id: field(&mut fields)?,
                    });
                }
                let mut fences = Vec::new();
                for _ in 0..count(&mut fields, 5000)? {
                    fences.push(Fence {
                        info: field(&mut fields)?,
                        code: field(&mut fields)?,
                        rendered: field(&mut fields)?,
                    });
                }
                match markdown(source, &headings, &fences) {
                    Ok((intro, rest)) => {
                        emit(&mut result, "ok")?;
                        emit(&mut result, &intro)?;
                        emit(&mut result, &rest)?;
                    }
                    Err(error) => {
                        emit(&mut result, "error")?;
                        emit(&mut result, &error)?;
                        emit(&mut result, "")?;
                    }
                }
            }
        }
        "layout-v1" => {
            let base = field(&mut fields)?;
            let icons = (0..ICONS.len())
                .map(|_| field(&mut fields))
                .collect::<Result<Vec<_>>>()?;
            for _ in 0..n {
                let title = field(&mut fields)?;
                let description = field(&mut fields)?;
                let navigation = field(&mut fields)?;
                let body = field(&mut fields)?;
                let data = field(&mut fields)?;
                emit(
                    &mut result,
                    &layout(base, title, description, navigation, body, data, &icons)?,
                )?;
            }
        }
        _ => return Err("未知渲染介面版本".into()),
    }
    if fields.next().is_some() {
        return Err("渲染資料包含多餘欄位".into());
    }
    Ok(result)
}

fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin()
        .lock()
        .take(LIMIT + 1)
        .read_to_string(&mut input)?;
    if input.len() as u64 > LIMIT {
        return Err("渲染資料超過 64 MiB".into());
    }
    io::stdout().lock().write_all(&process(&input)?)?;
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.is_empty() {
        run().map_err(|e| e.to_string())
    } else {
        native::run(&args)
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("site-render: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_chinese_callouts_headings_and_fenced_code() {
        let source = "# 教材\n\n> **為什麼？** <!-- callout:context -->\n>\n> 說明。\n\n## 原理\n\n~~~rust run\nlet a = 1;\n~~~\n";
        let (intro, rest) = markdown(
            source,
            &[Heading {
                level: 2,
                id: "s-原理",
            }],
            &[Fence {
                info: "rust run",
                code: "let a = 1;\n",
                rendered: "<pre><code>let a = 1;\n</code></pre>",
            }],
        )
        .unwrap();
        assert!(intro.contains("data-callout=\"context\""));
        assert!(intro.contains("<strong>為什麼？</strong>"));
        assert!(!intro.contains("<!-- callout:"));
        assert!(rest.contains("<h2 id=\"s-原理\" tabindex=\"-1\">原理</h2>"));
        assert!(rest.contains("<pre><code>let a = 1;\n</code></pre>"));
    }

    #[test]
    fn escapes_source_html_and_rejects_unsafe_links() {
        let (_, rest) = markdown("# 教材\n\n文字 <script>alert(1)</script>\n", &[], &[]).unwrap();
        assert!(rest.contains("&lt;script&gt;"));
        assert!(!rest.contains("<script>"));
        assert!(markdown("# 教材\n\n[x](javascript:alert)\n", &[], &[]).is_err());
        assert!(
            markdown(
                "# 教材\n\n> **[x](javascript:alert)** <!-- callout:context -->\n>\n> 說明。\n",
                &[],
                &[]
            )
            .is_err()
        );
        assert!(markdown("# [x](javascript:alert)\n\n說明。\n", &[], &[]).is_err());
        assert!(
            markdown(
                "# 教材\n\n> **[x](https://example.com)** <!-- callout:context -->\n>\n> 說明。\n",
                &[],
                &[]
            )
            .is_ok()
        );
    }

    #[test]
    fn preserves_template_fields_without_recursive_substitution() {
        let values = HashMap::from([("x", "{{y}}".into()), ("y", "ok".into())]);
        assert_eq!(render_template("{{x}}/{{y}}", &values).unwrap(), "{{y}}/ok");
        assert!(render_template("{{missing}}", &values).is_err());
        assert_eq!(escape("<&\"'"), "&lt;&amp;&quot;&#39;");
    }

    #[test]
    fn layout_preserves_title_and_rejects_script_data() {
        let icons = [""; 7];
        let result = layout(
            "/course/",
            "<標題>",
            "描述",
            "<nav></nav>",
            "<h1>教材</h1>",
            "{}",
            &icons,
        )
        .unwrap();
        assert!(result.contains("<title>&lt;標題&gt;｜Rust 學習工坊</title>"));
        assert!(result.contains("/course/assets/site.css"));
        assert!(layout("/", SITE, "", "", "", "</script>", &icons).is_err());
    }
}
