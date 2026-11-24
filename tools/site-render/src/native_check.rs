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
fn send_index(fields: &mut Fields, v: &Value) -> Result<()> {
    let entries = v.as_array().ok_or("搜尋索引非清單")?;
    number(fields, entries.len());
    for e in entries {
        add(
            fields,
            &[
                s(e, "title")?,
                s(e, "id")?,
                s(e, "description")?,
                s(e, "href")?,
                s(e, "text")?,
            ],
        );
        let headings = list(e, "headings")?;
        number(fields, headings.len());
        for h in headings {
            add(fields, &[s(h, "id")?, s(h, "label")?]);
        }
    }
    Ok(())
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
    let mut fields = vec![base.to_owned(), course.lessons.len().to_string()];
    for l in &course.lessons {
        let e = &l.entry;
        add(&mut fields, &[&e.id, &e.slug, &e.path]);
        flag(&mut fields, e.chapter());
        flag(&mut fields, e.resource());
        number(
            &mut fields,
            course
                .stages()?
                .iter()
                .filter(|st| s(st, "id").ok() == Some(e.group.as_str()))
                .count(),
        );
        add(&mut fields, &[&e.source_id]);
        flag(&mut fields, e.source_valid);
        number(&mut fields, l.headings.len());
        for (level, label, _) in &l.headings {
            number(&mut fields, *level as usize);
            add(&mut fields, &[label]);
        }
        number(&mut fields, l.examples.len());
        for ex in &l.examples {
            number(&mut fields, ex.line);
            add(&mut fields, &[&ex.markup, &ex.ending]);
        }
        number(&mut fields, l.notes.len());
        for note in &l.notes {
            add(&mut fields, &[note]);
        }
    }
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
        let ids: Vec<_> = nodes
            .iter()
            .filter_map(|n| n.value().attr("id"))
            .filter(|s| !s.is_empty())
            .collect();
        add(&mut fields, &[&route]);
        number(
            &mut fields,
            nodes.iter().filter(|n| n.value().name() == "h1").count(),
        );
        number(
            &mut fields,
            nodes.iter().filter(|n| n.value().name() == "main").count(),
        );
        add(
            &mut fields,
            &[html.root_element().value().attr("lang").unwrap_or("")],
        );
        number(
            &mut fields,
            nodes
                .iter()
                .filter(|n| n.value().name() == "img" && n.value().attr("alt").is_none())
                .count(),
        );
        number(&mut fields, ids.len());
        add(&mut fields, &ids);
        let notes: Vec<_> = nodes
            .iter()
            .filter(|n| n.value().attr("data-callout").is_some())
            .collect();
        number(&mut fields, notes.len());
        for note in notes {
            let title = note
                .select(&note_title)
                .find(|t| t.value().attr("class") == Some("callout-title"))
                .map(|t| t.text().collect::<String>())
                .unwrap_or_default();
            add(
                &mut fields,
                &[
                    note.value().attr("data-callout").unwrap_or(""),
                    note.value().attr("role").unwrap_or(""),
                    title.trim(),
                ],
            );
        }
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
            links.push((
                address.to_owned(),
                target.path().to_owned(),
                decode_fragment(target.fragment().unwrap_or(""))?,
            ));
        }
        number(&mut fields, links.len());
        for (address, path, hash) in links {
            add(&mut fields, &[&address, &path, &hash]);
        }
    }
    let report = load(root.join("build-render-report.json"))?;
    add(&mut fields, &[s(&report, "layout")?]);
    number(&mut fields, num(&report, "pages")?);
    let md = list(&report, "markdown")?;
    number(&mut fields, md.len());
    for item in md {
        add(&mut fields, &[s(item, "slug")?, s(item, "renderer")?]);
    }
    let high = report.get("highlight").ok_or("缺少高亮報告")?;
    add(&mut fields, &[s(high, "renderer")?]);
    for k in ["native", "total", "fallback"] {
        number(&mut fields, num(high, k)?);
    }
    let index = load(root.join("assets/search-index.json"))?;
    let shape = index_shape(&index);
    flag(&mut fields, shape);
    send_index(&mut fields, &course.index(base))?;
    if shape {
        send_index(&mut fields, &index)?
    } else {
        number(&mut fields, 0);
    }
    let result = packet("site-check-v1", names.len(), fields)?;
    if result.len() != 4 {
        return Err("網站驗證回傳不完整".into());
    }
    let bytes = result[3].parse::<u64>().map_err(fail)?;
    println!(
        "{} 份 Markdown、{} 個 HTML、{} 個站內引用與搜尋索引通過，首頁與共用資源 {:.1} kB（未壓縮），BASE_PATH={base}",
        result[0],
        result[1],
        result[2],
        bytes as f64 / 1000.0
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
