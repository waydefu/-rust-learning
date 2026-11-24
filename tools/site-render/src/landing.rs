//! Native landing pages; the old JavaScript templates are the exact-output oracle.
use super::{Result, escape, render_template};
use std::collections::{HashMap, HashSet};

const HOME: &str = include_str!("../templates/home.html");
const STAGE: &str = include_str!("../templates/stage.html");
const CARD: &str = include_str!("../templates/chapter-card.html");
const RESOURCES: &str = include_str!("../templates/resources.html");
const RESOURCE_CARD: &str = include_str!("../templates/resource-card.html");
const FOOTER: &str = include_str!("../templates/footer.html");

fn fill(template: &str, values: &[(&str, &str)]) -> Result<String> {
    render_template(
        template,
        &values
            .iter()
            .map(|(k, v)| (*k, (*v).to_owned()))
            .collect::<HashMap<_, _>>(),
    )
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

pub(super) struct Entry<'a> {
    pub id: &'a str,
    pub path: &'a str,
    pub title: &'a str,
    pub description: &'a str,
}

pub(super) struct Stage<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub icon: String,
    pub chapters: Vec<(Entry<'a>, bool)>,
}

pub(super) struct Landing<'a> {
    pub base: &'a str,
    pub icons: [&'a str; 8],
    pub first: &'a str,
    pub chapter_count: usize,
    pub resources: Vec<Entry<'a>>,
    pub stages: Vec<Stage<'a>>,
}

fn validate_entry(entry: &Entry<'_>, resource: bool) -> Result<()> {
    let digits = if resource {
        entry.id.strip_prefix('A').unwrap_or("")
    } else {
        entry.id
    };
    if digits.len() != 2 || !digits.bytes().all(|b| b.is_ascii_digit()) || !safe_path(entry.path) {
        return Err("章節卡片資料無效".into());
    }
    Ok(())
}

fn card(
    base: &str,
    entry: &Entry<'_>,
    stage: &str,
    check: &str,
    chevron: &str,
    featured: bool,
) -> Result<String> {
    fill(
        CARD,
        &[
            ("base", base),
            ("path", entry.path),
            ("id", entry.id),
            ("stage", stage),
            ("title", &escape(entry.title)),
            ("description", &escape(entry.description)),
            ("check", check),
            ("chevron", chevron),
            ("featuredClass", if featured { " feature-card" } else { "" }),
            (
                "featuredTag",
                if featured {
                    r#"<span class="interactive-tag">互動圖解</span>"#
                } else {
                    ""
                },
            ),
        ],
    )
}

pub(super) fn render(input: &Landing<'_>) -> Result<Vec<String>> {
    let base = input.base;
    let first = input.first;
    let chapter_count = input.chapter_count;
    let resource_count = input.resources.len();
    let [chevron14, route, book, layers, target, list, check, chevron] = input.icons;
    let mut resources = String::new();
    let mut seen = HashSet::new();
    for e in &input.resources {
        validate_entry(e, true)?;
        if !seen.insert(e.id) {
            return Err("重複教材 ID".into());
        }
        resources.push_str(&fill(
            RESOURCE_CARD,
            &[
                ("base", base),
                ("path", e.path),
                ("id", e.id),
                ("title", &escape(e.title)),
                ("description", &escape(e.description)),
                ("chevron", chevron),
            ],
        )?);
    }
    let mut stages = String::new();
    let mut total = 0;
    let mut found_first = false;
    let mut stage_ids = HashSet::new();
    for stage in &input.stages {
        let Stage {
            id,
            title,
            description,
            icon,
            chapters,
        } = stage;
        let (id, title, description) = (*id, *title, *description);
        if !safe_path(id) || id.contains('/') || !stage_ids.insert(id) {
            return Err("課程群組無效".into());
        }
        if chapters.is_empty() {
            return Err("課程群組不可為空".into());
        }
        let mut cards = String::new();
        let mut start = String::new();
        let mut end = String::new();
        for (index, (e, featured)) in chapters.iter().enumerate() {
            validate_entry(e, false)?;
            if !seen.insert(e.id) {
                return Err("重複教材 ID".into());
            }
            let number = e.id.parse::<u8>().map_err(|_| "章節編號無效")?.to_string();
            if index == 0 {
                start = number.clone();
            }
            end = number;
            found_first |= e.path == first;
            cards.push_str(&card(base, e, id, check, chevron, *featured)?);
            total += 1;
        }
        stages.push_str(&fill(
            STAGE,
            &[
                ("id", id),
                ("title", &escape(title)),
                ("description", &escape(description)),
                ("icon", icon),
                ("first", &start),
                ("last", &end),
                ("cards", &cards),
            ],
        )?);
    }
    if total != chapter_count || !found_first {
        return Err("首頁章節清單不完整".into());
    }
    let footer = fill(FOOTER, &[("base", base)])?;
    let home = fill(
        HOME,
        &[
            ("base", base),
            ("first", first),
            ("chapterCount", &chapter_count.to_string()),
            ("resourceCount", &resource_count.to_string()),
            ("chevron14", chevron14),
            ("route", route),
            ("book", book),
            ("layers", layers),
            ("target", target),
            ("list", list),
            ("stages", &stages),
            ("footer", &footer),
        ],
    )?;
    let resource = fill(
        RESOURCES,
        &[
            ("base", base),
            ("chevron14", chevron14),
            ("cards", &resources),
            ("footer", &footer),
        ],
    )?;
    let missing = format!(
        r#"<div class="home-page secondary-page"><h1>找不到頁面</h1><p>請從課程總覽選擇章節，或使用搜尋查找教材。</p><a class="button primary" href="{base}" >返回課程總覽</a></div>"#
    );
    Ok(vec![home, resource, missing])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn card_preserves_feature_and_escapes_text() {
        let e = Entry {
            id: "04",
            path: "chapters/ownership",
            title: "<Rust>",
            description: "A & B",
        };
        let html = card("/course/", &e, "fundamentals", "CHECK", "NEXT", true).unwrap();
        assert!(html.contains("chapter-card feature-card"));
        assert!(html.contains("/course/chapters/ownership/"));
        assert!(html.contains("&lt;Rust&gt;"));
        assert!(html.contains("A &amp; B"));
        assert!(
            !card("/", &e, "fundamentals", "", "", false)
                .unwrap()
                .contains("interactive-tag")
        );
    }

    #[test]
    fn renders_landing_and_checks_counts() {
        let mut input = Landing {
            base: "/course/",
            icons: [""; 8],
            first: "chapters/start",
            chapter_count: 1,
            resources: vec![Entry {
                id: "A01",
                path: "resources/extra",
                title: "延伸",
                description: "說明",
            }],
            stages: vec![Stage {
                id: "fundamentals",
                title: "入門",
                description: "介紹",
                icon: String::new(),
                chapters: vec![(
                    Entry {
                        id: "01",
                        path: "chapters/start",
                        title: "開始",
                        description: "內容",
                    },
                    false,
                )],
            }],
        };
        let html = render(&input).unwrap().join("\n");
        assert!(html.contains("data-start"));
        assert!(html.contains("第 1–1 章"));
        assert!(html.contains("/course/resources/extra/"));
        input.chapter_count = 2;
        assert!(render(&input).is_err());
    }
}
