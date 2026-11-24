//! Native sidebar renderer. Icons are trusted static SVG from the icon library.
use super::{Result, count, emit, escape, field};

struct Chapter<'a> {
    id: &'a str,
    path: &'a str,
    title: &'a str,
}
struct Stage<'a> {
    id: &'a str,
    title: &'a str,
    chapters: Vec<Chapter<'a>>,
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
fn current_attr(active: bool) -> &'static str {
    if active { " aria-current=\"page\"" } else { "" }
}
fn row(base: &str, path: &str, label: &str, icon: &str, current: &str, meta: &str) -> String {
    let href = if path.is_empty() {
        base.to_owned()
    } else {
        format!("{base}{path}/")
    };
    let meta = if meta.is_empty() {
        String::new()
    } else {
        format!("<small>{meta}</small>")
    };
    format!(
        "<a href=\"{href}\" class=\"nav-row\"{}>{icon}<span>{label}</span>{meta}</a>",
        current_attr(current == path)
    )
}
fn navigation(
    base: &str,
    stages: &[Stage<'_>],
    resources: usize,
    icons: &[&str],
    current: &str,
    id: &str,
) -> String {
    let mut groups = String::new();
    for (i, stage) in stages.iter().enumerate() {
        let open = if (i == 0 && current.is_empty()) || stage.chapters.iter().any(|e| e.id == id) {
            " open"
        } else {
            ""
        };
        let entries = stage.chapters.iter().map(|e| format!(
            "<a href=\"{base}{}/\"{} data-nav-chapter=\"{}\"><span class=\"nav-number\">{}</span><span>{}</span><span class=\"nav-done\" aria-label=\"已完成\" hidden>{}</span></a>",
            e.path, current_attr(e.id == id), escape(e.id), escape(e.id), escape(e.title), icons[4]
        )).collect::<Vec<_>>().join("");
        groups.push_str(&format!("<details class=\"nav-group\"{open}>\n        <summary><span class=\"stage-dot stage-{}\"></span><span>{}</span>{}</summary>\n        <div class=\"nav-chapters\">{entries}</div>\n      </details>", stage.id, escape(stage.title), icons[3]));
    }
    format!(
        "<nav class=\"sidebar-inner\" aria-label=\"課程導覽\">\n      <p class=\"nav-caption\">學習</p>\n      {}\n      {}\n      {}\n      <p class=\"nav-caption grouped-caption\">章節目錄</p>\n      {groups}\n      <div class=\"sidebar-foot\"><span class=\"stage-dot\"></span> Rust 2024 Edition</div>\n    </nav>",
        row(base, "", "課程總覽", icons[0], current, ""),
        row(base, "guide", "學習指南", icons[1], current, ""),
        row(
            base,
            "resources",
            "延伸資源",
            icons[2],
            current,
            &if resources == 0 {
                String::new()
            } else {
                resources.to_string()
            }
        )
    )
}
pub(super) fn process_navigation<'a>(
    n: usize,
    fields: &mut impl Iterator<Item = &'a str>,
    out: &mut Vec<String>,
) -> Result<()> {
    let base = field(fields)?;
    if base != "/"
        && (!base.starts_with('/') || !base.ends_with('/') || !safe_path(&base[1..base.len() - 1]))
    {
        return Err("導覽基礎網址無效".into());
    }
    let resources = count(fields, 1000)?;
    let icons = (0..5).map(|_| field(fields)).collect::<Result<Vec<_>>>()?;
    let mut stages = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for _ in 0..count(fields, 100)? {
        let id = field(fields)?;
        let title = field(fields)?;
        if !safe_path(id) || id.contains('/') {
            return Err("課程群組無效".into());
        }
        let mut chapters = Vec::new();
        for _ in 0..count(fields, 1000)? {
            let id = field(fields)?;
            let path = field(fields)?;
            let title = field(fields)?;
            if id.len() != 2
                || !id.bytes().all(|b| b.is_ascii_digit())
                || !seen.insert(id)
                || !safe_path(path)
            {
                return Err("章節導覽資料無效".into());
            }
            chapters.push(Chapter { id, path, title });
        }
        stages.push(Stage {
            id,
            title,
            chapters,
        });
    }
    for _ in 0..n {
        let current = field(fields)?;
        let id = field(fields)?;
        if !current.is_empty() && !safe_path(current) {
            return Err("目前網址無效".into());
        }
        emit(
            out,
            &navigation(base, &stages, resources, &icons, current, id),
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marks_active_chapter_and_escapes_catalog_text() {
        let stages = [Stage {
            id: "fundamentals",
            title: "入門",
            chapters: vec![Chapter {
                id: "01",
                path: "chapters/start",
                title: "Rust <入門>",
            }],
        }];
        let icons = [""; 5];
        let page = navigation("/course/", &stages, 8, &icons, "chapters/start", "01");
        assert!(page.contains("class=\"nav-group\" open"));
        assert!(page.contains(
            "href=\"/course/chapters/start/\" aria-current=\"page\" data-nav-chapter=\"01\""
        ));
        assert!(page.contains("Rust &lt;入門&gt;"));
        assert!(page.contains("<small>8</small>"));
        assert!(
            !navigation("/", &stages, 8, &icons, "404", "").contains("class=\"nav-group\" open")
        );
    }
}
