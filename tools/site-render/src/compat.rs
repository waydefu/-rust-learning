//! Historical NUL-delimited migration interface, retained for the pinned oracle.
//! The production CLI never calls this module.
use super::{
    Fence, Heading, ICONS, Result, examples, fences, highlight, landing, layout, markdown,
    metadata, navigation, page_data, reader, site_check,
};
use std::io::{self, Read, Write};

const LIMIT: u64 = 64 * 1024 * 1024;

pub(super) fn field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Result<&'a str> {
    fields.next().ok_or_else(|| "渲染資料不完整".into())
}

pub(super) fn count<'a>(fields: &mut impl Iterator<Item = &'a str>, max: usize) -> Result<usize> {
    field(fields)?
        .parse::<usize>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| "渲染資料數量無效".into())
}

pub(super) fn emit(out: &mut Vec<String>, value: &str) -> Result<()> {
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

// Historical protocol dispatch. Production callers use the typed renderer APIs.
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

pub(super) fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_malformed_envelopes() {
        assert!(process("anchors-v1").is_err());
        assert!(process("unknown\0\x30\0").is_err());
        assert!(process("anchors-v1\0\x30\0extra\0").is_err());
        assert_eq!(process("anchors-v1\0\x30\0").unwrap(), b"anchors-v1\0\x30\0");
    }
}
