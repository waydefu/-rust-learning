//! Site verification over normalized HTML and URL facts from the HTML5 parser.
//! The native collector and the historical adapter share one typed verifier.
use super::Result;
#[cfg(feature = "migration-compat")]
use super::{count, emit, field};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};
#[cfg(feature = "migration-compat")]
fn flag<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<bool> {
    match field(f)? {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("網站驗證旗標無效".into()),
    }
}
#[cfg(feature = "migration-compat")]
fn strings<'a>(f: &mut impl Iterator<Item = &'a str>, limit: usize) -> Result<Vec<&'a str>> {
    (0..count(f, limit)?).map(|_| field(f)).collect()
}
fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
fn safe_relative(s: &str) -> bool {
    !s.is_empty()
        && !s.contains('\\')
        && s.split('/')
            .all(|p| !p.is_empty() && !matches!(p, "." | ".."))
        && Path::new(s)
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
}
fn file_size(root: &Path, name: &str) -> Option<u64> {
    if !safe_relative(name) {
        return None;
    }
    let path = root.join(name);
    let m = fs::metadata(path).ok()?;
    if m.is_file() { Some(m.len()) } else { None }
}
pub(super) struct Note {
    pub kind: String,
    pub role: String,
    pub title: String,
}
pub(super) struct Link {
    pub address: String,
    pub path: String,
    pub hash: String,
}
pub(super) struct Page {
    pub route: String,
    pub h1: usize,
    pub main: usize,
    pub lang: String,
    pub missing_alt: usize,
    pub ids: Vec<String>,
    pub notes: Vec<Note>,
    pub links: Vec<Link>,
}
pub(super) struct Lesson<'a> {
    pub id: &'a str,
    pub slug: &'a str,
    pub path: &'a str,
    pub chapter: bool,
    pub resource: bool,
    pub stage_count: usize,
    pub source_id: &'a str,
    pub source_valid: bool,
    pub headings: Vec<(usize, &'a str)>,
    pub fences: Vec<(usize, &'a str, &'a str)>,
    pub notes: Vec<&'a str>,
}
pub(super) struct RenderReport<'a> {
    pub layout: &'a str,
    pub pages: usize,
    pub markdown: Vec<(&'a str, &'a str)>,
    pub renderer: &'a str,
    pub native: usize,
    pub total: usize,
    pub fallback: usize,
}
pub(super) struct Input<'a> {
    pub base: &'a str,
    pub lessons: Vec<Lesson<'a>>,
    pub pages: Vec<Page>,
    pub report: RenderReport<'a>,
    pub shape_valid: bool,
    pub expected_index: Vec<IndexEntry<'a>>,
    pub index: Vec<IndexEntry<'a>>,
}
pub(super) struct Summary {
    pub lessons: usize,
    pub pages: usize,
    pub links: usize,
    pub bytes: u64,
}
fn check_note(title: &str, role: &str) -> bool {
    role == "note"
        && !title.trim().is_empty()
        && !title.contains("callout:")
        && !matches!(
            title.trim(),
            "來龍去脈" | "生活比喻" | "程式拆解" | "常見陷阱" | "動手驗收"
        )
}

#[derive(PartialEq, Eq)]
pub(super) struct IndexEntry<'a> {
    pub title: &'a str,
    pub id: &'a str,
    pub description: &'a str,
    pub href: &'a str,
    pub text: &'a str,
    pub headings: Vec<(&'a str, &'a str)>,
}
#[cfg(feature = "migration-compat")]
fn read_index<'a>(f: &mut impl Iterator<Item = &'a str>) -> Result<Vec<IndexEntry<'a>>> {
    let mut entries = Vec::new();
    for _ in 0..count(f, 1000)? {
        let [title, id, description, href, text] =
            [field(f)?, field(f)?, field(f)?, field(f)?, field(f)?];
        let mut headings = Vec::new();
        for _ in 0..count(f, 5000)? {
            headings.push((field(f)?, field(f)?));
        }
        entries.push(IndexEntry {
            title,
            id,
            description,
            href,
            text,
            headings,
        });
    }
    Ok(entries)
}

#[cfg(feature = "migration-compat")]
pub(super) fn process_site_check<'a>(
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
        || base.split('/').any(|p| matches!(p, "." | ".."))
    {
        return Err("BASE_PATH 無效".into());
    }
    let mut lessons = Vec::new();
    for _ in 0..count(f, 1000)? {
        lessons.push(Lesson {
            id: field(f)?,
            slug: field(f)?,
            path: field(f)?,
            chapter: flag(f)?,
            resource: flag(f)?,
            stage_count: count(f, 100)?,
            source_id: field(f)?,
            source_valid: flag(f)?,
            headings: (0..count(f, 5000)?)
                .map(|_| Ok((count(f, 6)?, field(f)?)))
                .collect::<Result<Vec<_>>>()?,
            fences: (0..count(f, 5000)?)
                .map(|_| Ok((count(f, 1_000_000)?, field(f)?, field(f)?)))
                .collect::<Result<Vec<_>>>()?,
            notes: strings(f, 5000)?,
        });
    }
    let mut pages = Vec::new();
    for _ in 0..n {
        let route = field(f)?.to_owned();
        let h1 = count(f, 5000)?;
        let main = count(f, 5000)?;
        let lang = field(f)?.to_owned();
        let missing_alt = count(f, 5000)?;
        let ids = strings(f, 20_000)?.into_iter().map(str::to_owned).collect();
        let mut notes = Vec::new();
        for _ in 0..count(f, 5000)? {
            notes.push(Note {
                kind: field(f)?.to_owned(),
                role: field(f)?.to_owned(),
                title: field(f)?.to_owned(),
            });
        }
        let mut links = Vec::new();
        for _ in 0..count(f, 20_000)? {
            links.push(Link {
                address: field(f)?.to_owned(),
                path: field(f)?.to_owned(),
                hash: field(f)?.to_owned(),
            });
        }
        pages.push(Page {
            route,
            h1,
            main,
            lang,
            missing_alt,
            ids,
            notes,
            links,
        });
    }
    let report = RenderReport {
        layout: field(f)?,
        pages: count(f, 1000)?,
        markdown: (0..count(f, 1000)?)
            .map(|_| Ok((field(f)?, field(f)?)))
            .collect::<Result<Vec<_>>>()?,
        renderer: field(f)?,
        native: count(f, 100_000)?,
        total: count(f, 100_000)?,
        fallback: count(f, 100_000)?,
    };
    let input = Input {
        base,
        lessons,
        pages,
        report,
        shape_valid: flag(f)?,
        expected_index: read_index(f)?,
        index: read_index(f)?,
    };
    let summary = verify(&input)?;
    for value in [
        summary.lessons as u64,
        summary.pages as u64,
        summary.links as u64,
        summary.bytes,
    ] {
        emit(out, &value.to_string())?;
    }
    Ok(())
}

pub(super) fn verify(input: &Input<'_>) -> Result<Summary> {
    let base = input.base;
    let mut errors = Vec::new();
    let mut total_fences = 0usize;
    let (mut ids, mut slugs, mut paths) = (Vec::new(), Vec::new(), Vec::new());
    let (mut chapter_ids, mut resource_ids) = (Vec::new(), Vec::new());
    for l in &input.lessons {
        let (id, slug, path) = (l.id, l.slug, l.path);
        let (chapter, resource) = (l.chapter, l.resource);
        let stage_count = l.stage_count;
        let (source_id, source_valid) = (l.source_id, l.source_valid);
        ids.push(id);
        slugs.push(slug);
        paths.push(path);
        if chapter {
            chapter_ids.push(id);
            if stage_count != 1 {
                errors.push("每章必須屬於一個階段".into())
            }
        }
        if resource {
            resource_ids.push(id)
        }
        if !safe_relative(path) || !safe_relative(slug) || slug.contains('/') {
            return Err("catalog 路徑無效".into());
        }
        if source_id.len() < 20
            || !source_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            || !source_valid
        {
            errors.push(format!("{slug} 缺少有效的來源記錄"));
        }
        let heading_count = l.headings.len();
        if heading_count == 0 {
            errors.push(format!("{slug} 缺少內容標題"))
        }
        let mut level = 1;
        for &(next, label) in &l.headings {
            if next > level + 1 {
                errors.push(format!("{slug} 標題層級跳躍 {label}"))
            }
            level = next;
        }
        total_fences += l.fences.len();
        for &(line, markup, ending) in &l.fences {
            let ending = ending.trim();
            if markup.is_empty()
                || !ending.starts_with(markup)
                || !ending.chars().all(|c| markup.starts_with(c))
            {
                errors.push(format!("{slug}.md:{line} 缺少程式碼結尾"));
            }
        }
    }
    if chapter_ids.len() != 21
        || chapter_ids
            .iter()
            .enumerate()
            .any(|(i, id)| id.parse::<usize>().ok() != Some(i + 1))
    {
        errors.push("章節必須依序涵蓋 01–21".into());
    }
    if resource_ids.len() != 8
        || resource_ids.iter().enumerate().any(|(i, id)| {
            id.strip_prefix('A').and_then(|s| s.parse::<usize>().ok()) != Some(i + 1)
        })
    {
        errors.push("附錄必須依序涵蓋 A01–A08".into());
    }
    if !unique(&ids) || !unique(&slugs) || !unique(&paths) {
        errors.push("課程識別碼、slug 與路徑不可重複".into())
    }
    let source_files = fs::read_dir("content")
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    let source_files: BTreeSet<_> = source_files
        .into_iter()
        .filter(|s| s.ends_with(".md"))
        .collect();
    if source_files.len() != input.lessons.len()
        || input
            .lessons
            .iter()
            .any(|l| !source_files.contains(&format!("{}.md", l.slug)))
    {
        errors.push("Markdown 檔案與 catalog.json 不一致".into());
    }
    let mut pages = BTreeMap::new();
    for page in &input.pages {
        let route = page.route.as_str();
        let (h1, main, missing_alt) = (page.h1, page.main, page.missing_alt);
        let lang = page.lang.as_str();
        let ids = &page.ids;
        if !unique(ids) {
            errors.push(format!("{route} 有重複 id"))
        }
        if h1 != 1 {
            errors.push(format!("{route} 必須包含單一 H1"))
        }
        if main != 1 || !ids.iter().any(|id| id == "main") {
            errors.push(format!("{route} 缺少內容地標"))
        }
        if lang != "zh-Hant" {
            errors.push(format!("{route} 缺少 zh-Hant"))
        }
        if missing_alt > 0 {
            errors.push(format!("{route} 圖片缺少替代文字"))
        }
        if pages.insert(route, page).is_some() {
            return Err("重複頁面路徑".into());
        }
    }
    for route in [
        base.to_owned(),
        format!("{base}resources/"),
        format!("{base}404.html"),
    ]
    .into_iter()
    .chain(input.lessons.iter().map(|l| format!("{base}{}/", l.path)))
    {
        if !pages.contains_key(route.as_str()) {
            errors.push(format!("{route} 尚未生成"))
        }
    }
    for l in input.lessons.iter().filter(|l| l.chapter) {
        let route = format!("{base}{}/", l.path);
        if let Some(page) = pages.get(route.as_str()) {
            let kinds: Vec<_> = page.notes.iter().map(|n| n.kind.as_str()).collect();
            if kinds.is_empty()
                || kinds != l.notes
                || kinds.iter().any(|k| !super::KINDS.contains(k))
            {
                errors.push(format!("{} 教學方塊與教材來源不一致", route));
            }
            for note in &page.notes {
                if !check_note(&note.title, &note.role) {
                    errors.push(format!("{} 教學方塊缺少文字標題或語意", route))
                }
            }
        }
    }
    let dist = Path::new("dist");
    let mut assets = BTreeSet::new();
    let mut checked = 0usize;
    for (route, page) in &pages {
        for link in &page.links {
            checked += 1;
            let Some(name) = link.path.strip_prefix(base) else {
                errors.push(format!("{route}: {} 超出部署子路徑", link.address));
                continue;
            };
            if let Some(target) = pages.get(link.path.as_str()) {
                if !link.hash.is_empty() && !target.ids.contains(&link.hash) {
                    errors.push(format!("{route}: 錨點不存在 {}", link.address))
                }
            } else if file_size(dist, name).is_some() {
                assets.insert(name);
            } else {
                errors.push(format!("{route}: 資源或頁面不存在 {}", link.address))
            }
        }
    }
    let report = &input.report;
    let mut report_valid = report.layout == "rust"
        && report.pages == pages.len()
        && report.markdown.len() == input.lessons.len();
    let mut has_rust = false;
    for (i, &(slug, renderer)) in report.markdown.iter().enumerate() {
        report_valid &= input.lessons.get(i).is_some_and(|l| l.slug == slug) && renderer == "rust";
        has_rust |= renderer == "rust";
    }
    if !report_valid || !has_rust {
        errors.push("Rust 渲染報告不完整".into())
    }
    if report.renderer != "rust"
        || report.native != report.total
        || report.total != total_fences
        || report.fallback != 0
    {
        errors.push("Rust 高亮報告未達零回退".into())
    }
    let index = &input.index;
    let expected_index = &input.expected_index;
    let shape_valid = input.shape_valid;
    if !shape_valid || index != expected_index {
        errors.push("Rust 搜尋索引與原有資料契約不一致".into())
    }
    let index_valid = index.len() == input.lessons.len()
        && index.iter().all(|entry| {
            !entry.text.is_empty()
                && pages.get(entry.href).is_some_and(|p| {
                    entry
                        .headings
                        .iter()
                        .all(|(id, _)| p.ids.iter().any(|value| value == id))
                })
        });
    if !index_valid {
        errors.push("搜尋索引的頁面或章節定位不完整".into())
    }
    let mut initial = vec!["index.html"];
    initial.extend(assets.iter().copied().filter(|s| !s.ends_with(".json")));
    let mut bytes = 0u64;
    for path in initial {
        bytes += file_size(dist, path).ok_or_else(|| format!("資源不存在 {path}"))?;
    }
    if bytes > 160_000 {
        errors.push(format!("首頁與共用資源超過 160 kB（實際 {bytes} bytes）"))
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(Summary {
        lessons: input.lessons.len(),
        pages: pages.len(),
        links: checked,
        bytes,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn path_and_note_guards() {
        assert!(safe_relative("assets/site.css"));
        assert!(!safe_relative("../README.md"));
        assert!(!safe_relative("a\\b"));
        assert!(check_note("為何要借用？", "note"));
        assert!(!check_note("來龍去脈", "note"));
        assert!(!check_note("內容", ""));
    }
}
