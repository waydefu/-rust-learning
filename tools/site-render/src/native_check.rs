//! HTML5 parsing, WHATWG URL resolution and native verification.
use super::*;
use scraper::{Html, Selector};
const ORIGIN: &str = "https://rust-learning.test";
fn html_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for e in fs::read_dir(dir).map_err(fail)? {
        let e = e.map_err(fail)?;
        let ty = e.file_type().map_err(fail)?;
        let path = e.path();
        if ty.is_symlink() {
            return Err(format!("網站含符號連結：{}", path.display()));
        }
        if ty.is_dir() {
            html_files(root, &path, out)?
        } else if ty.is_file() && path.extension().is_some_and(|p| p == "html") {
            let relative = path
                .strip_prefix(root)
                .map_err(fail)?
                .components()
                .map(|c| c.as_os_str().to_str().ok_or("非 UTF-8 網站路徑"))
                .collect::<std::result::Result<Vec<_>, _>>()?
                .join("/");
            out.push(relative);
        }
    }
    Ok(())
}
fn decode_fragment(s: &str) -> Result<String> {
    let mut bytes = Vec::new();
    let mut i = 0;
    let input = s.as_bytes();
    while i < input.len() {
        if input[i] == b'%' {
            let code = input.get(i + 1..i + 3).ok_or("不完整的 URL 編碼")?;
            bytes.push(
                u8::from_str_radix(std::str::from_utf8(code).map_err(fail)?, 16).map_err(fail)?,
            );
            i += 3;
        } else {
            bytes.push(input[i]);
            i += 1;
        }
    }
    String::from_utf8(bytes).map_err(fail)
}
fn typed_index(v: &Value) -> Result<Vec<super::super::site_check::IndexEntry<'_>>> {
    use super::super::site_check::IndexEntry;
    v.as_array()
        .ok_or("搜尋索引非清單")?
        .iter()
        .map(|e| {
            Ok(IndexEntry {
                title: s(e, "title")?,
                id: s(e, "id")?,
                description: s(e, "description")?,
                href: s(e, "href")?,
                text: s(e, "text")?,
                headings: list(e, "headings")?
                    .iter()
                    .map(|h| Ok((s(h, "id")?, s(h, "label")?)))
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect()
}
fn index_shape(v: &Value) -> bool {
    v.as_array().is_some_and(|a| {
        a.iter().all(|e| {
            e.as_object().is_some_and(|o| o.len() == 6)
                && ["title", "id", "description", "href", "text"]
                    .iter()
                    .all(|k| s(e, k).is_ok())
                && list(e, "headings").is_ok_and(|h| {
                    h.iter().all(|h| {
                        h.as_object().is_some_and(|o| o.len() == 2)
                            && s(h, "id").is_ok()
                            && s(h, "label").is_ok()
                    })
                })
        })
    })
}
pub(super) fn check(course: &Course, base: &str) -> Result<()> {
    let root = Path::new("dist");
    use super::super::site_check;
    let stages = course.stages()?;
    let lessons = course
        .lessons
        .iter()
        .map(|l| {
            let e = &l.entry;
            site_check::Lesson {
                id: &e.id,
                slug: &e.slug,
                path: &e.path,
                chapter: e.chapter(),
                resource: e.resource(),
                stage_count: stages
                    .iter()
                    .filter(|st| s(st, "id").ok() == Some(e.group.as_str()))
                    .count(),
                source_id: &e.source_id,
                source_valid: e.source_valid,
                headings: l
                    .headings
                    .iter()
                    .map(|(level, label, _)| (*level as usize, label.as_str()))
                    .collect(),
                fences: l
                    .examples
                    .iter()
                    .map(|ex| (ex.line, ex.markup.as_str(), ex.ending.as_str()))
                    .collect(),
                notes: l.notes.iter().map(String::as_str).collect(),
            }
        })
        .collect();
    let mut pages = Vec::new();
    let mut names = Vec::new();
    html_files(root, root, &mut names)?;
    names.sort();
    let all = Selector::parse("*").map_err(|e| format!("{e:?}"))?;
    let note_title = Selector::parse(".callout-title").map_err(|e| format!("{e:?}"))?;
    let origin = url::Url::parse(ORIGIN).map_err(fail)?;
    for name in &names {
        let suffix = if name == "index.html" {
            String::new()
        } else if let Some(p) = name.strip_suffix("/index.html") {
            format!("{p}/")
        } else {
            name.clone()
        };
        let route = format!("{base}{suffix}");
        let html = Html::parse_document(&read(root.join(name))?);
        let nodes: Vec<_> = html.select(&all).collect();
        let ids = nodes
            .iter()
            .filter_map(|n| n.value().attr("id"))
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        let h1 = nodes.iter().filter(|n| n.value().name() == "h1").count();
        let main = nodes.iter().filter(|n| n.value().name() == "main").count();
        let lang = html
            .root_element()
            .value()
            .attr("lang")
            .unwrap_or("")
            .to_owned();
        let missing_alt = nodes
            .iter()
            .filter(|n| n.value().name() == "img" && n.value().attr("alt").is_none())
            .count();
        let notes = nodes
            .iter()
            .filter(|n| n.value().attr("data-callout").is_some())
            .map(|note| {
                let title = note
                    .select(&note_title)
                    .find(|t| t.value().attr("class") == Some("callout-title"))
                    .map(|t| t.text().collect::<String>())
                    .unwrap_or_default();
                site_check::Note {
                    kind: note.value().attr("data-callout").unwrap_or("").to_owned(),
                    role: note.value().attr("role").unwrap_or("").to_owned(),
                    title: title.trim().to_owned(),
                }
            })
            .collect();
        let mut links = Vec::new();
        let page = origin.join(&route).map_err(fail)?;
        for node in nodes {
            let v = node.value();
            let Some(address) = v
                .attr("href")
                .or_else(|| v.attr("src"))
                .filter(|s| !s.is_empty())
            else {
                continue;
            };
            let target = page.join(address).map_err(fail)?;
            if target.origin() != origin.origin() {
                continue;
            }
            links.push(site_check::Link {
                address: address.to_owned(),
                path: target.path().to_owned(),
                hash: decode_fragment(target.fragment().unwrap_or(""))?,
            });
        }
        pages.push(site_check::Page {
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
    let report = load(root.join("build-render-report.json"))?;
    let high = report.get("highlight").ok_or("缺少高亮報告")?;
    let render_report = site_check::RenderReport {
        layout: s(&report, "layout")?,
        pages: num(&report, "pages")?,
        markdown: list(&report, "markdown")?
            .iter()
            .map(|item| Ok((s(item, "slug")?, s(item, "renderer")?)))
            .collect::<Result<Vec<_>>>()?,
        renderer: s(high, "renderer")?,
        native: num(high, "native")?,
        total: num(high, "total")?,
        fallback: num(high, "fallback")?,
    };
    let index = load(root.join("assets/search-index.json"))?;
    let shape = index_shape(&index);
    let expected_index = course.index(base);
    let summary = site_check::verify(&site_check::Input {
        base,
        lessons,
        pages,
        report: render_report,
        shape_valid: shape,
        expected_index: typed_index(&expected_index)?,
        index: if shape {
            typed_index(&index)?
        } else {
            Vec::new()
        },
    })?;
    println!(
        "{} 份 Markdown、{} 個 HTML、{} 個站內引用與搜尋索引通過，首頁與共用資源 {:.1} kB（未壓縮），BASE_PATH={base}",
        summary.lessons,
        summary.pages,
        summary.links,
        summary.bytes as f64 / 1000.0
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoded_fragments() {
        assert_eq!(decode_fragment("%E6%95%99%E6%9D%90").unwrap(), "教材");
        assert!(decode_fragment("%GG").is_err());
    }
}
