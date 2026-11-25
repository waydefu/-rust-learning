//! Native sidebar renderer. Icons are trusted static SVG from the icon library.
use super::escape;

pub(super) struct Chapter<'a> {
    pub(super) id: &'a str,
    pub(super) path: &'a str,
    pub(super) title: &'a str,
}
pub(super) struct Stage<'a> {
    pub(super) id: &'a str,
    pub(super) title: &'a str,
    pub(super) chapters: Vec<Chapter<'a>>,
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
pub(super) fn navigation(
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
