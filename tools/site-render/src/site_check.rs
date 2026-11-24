//! Site verification over normalized HTML and URL facts from the HTML5 parser.
//! Rust owns the rules, filesystem checks and final counts; the bridge does not
//! substitute a partial HTML parser for parse5 or WHATWG URL semantics.
use super::{Result,count,emit,field};
use std::{collections::{BTreeSet,BTreeMap},fs,path::{Path,Component}};
fn flag<'a>(f:&mut impl Iterator<Item=&'a str>)->Result<bool>{
    match field(f)?{"true"=>Ok(true),"false"=>Ok(false),_=>Err("網站驗證旗標無效".into())}
}
fn strings<'a>(f:&mut impl Iterator<Item=&'a str>,limit:usize)->Result<Vec<&'a str>>{
    (0..count(f,limit)?).map(|_|field(f)).collect()
}
fn unique<T:Ord>(values:&[T])->bool{values.iter().collect::<BTreeSet<_>>().len()==values.len()}
fn safe_relative(s:&str)->bool{
    !s.is_empty()&&!s.contains('\\')&&s.split('/').all(|p|!p.is_empty()&&!matches!(p,"."|".."))
        &&Path::new(s).components().all(|p|matches!(p,Component::Normal(_)))
}
fn file_size(root:&Path,name:&str)->Option<u64>{
    if !safe_relative(name){return None}
    let path=root.join(name);
    let m=fs::metadata(path).ok()?;
    if m.is_file(){Some(m.len())}else{None}
}
struct Note<'a>{kind:&'a str,role:&'a str,title:&'a str}
struct Link<'a>{address:&'a str,path:&'a str,hash:&'a str}
struct Page<'a>{ids:Vec<&'a str>,notes:Vec<Note<'a>>,links:Vec<Link<'a>>}
struct Lesson<'a>{slug:&'a str,route:String,chapter:bool,notes:Vec<&'a str>}
fn check_note(title:&str,role:&str)->bool{
    role=="note"&&!title.trim().is_empty()&&!title.contains("callout:")
        &&!matches!(title.trim(),"來龍去脈"|"生活比喻"|"程式拆解"|"常見陷阱"|"動手驗收")
}

#[derive(PartialEq,Eq)]
struct IndexEntry<'a>{title:&'a str,id:&'a str,description:&'a str,href:&'a str,text:&'a str,headings:Vec<(&'a str,&'a str)>}
fn read_index<'a>(f:&mut impl Iterator<Item=&'a str>)->Result<Vec<IndexEntry<'a>>>{
    let mut entries=Vec::new();
    for _ in 0..count(f,1000)?{
        let [title,id,description,href,text]=[field(f)?,field(f)?,field(f)?,field(f)?,field(f)?];
        let mut headings=Vec::new();
        for _ in 0..count(f,5000)?{headings.push((field(f)?,field(f)?));}
        entries.push(IndexEntry{title,id,description,href,text,headings});
    }
    Ok(entries)
}

pub(super) fn process_site_check<'a>(n:usize,f:&mut impl Iterator<Item=&'a str>,out:&mut Vec<u8>)->Result<()>{
    let base=field(f)?;
    if !base.starts_with('/')||!base.ends_with('/')||base.starts_with("//")
        ||!base.bytes().all(|b|b.is_ascii_alphanumeric()||b"/_.-".contains(&b))
        ||base.split('/').any(|p|matches!(p,"."|"..")){return Err("BASE_PATH 無效".into())}
    let mut errors=Vec::new();
    let mut total_fences=0usize;
    let mut lessons=Vec::new();let(mut ids,mut slugs,mut paths)=(Vec::new(),Vec::new(),Vec::new());
    let(mut chapter_ids,mut resource_ids)=(Vec::new(),Vec::new());
    for _ in 0..count(f,1000)?{
        let id=field(f)?;let slug=field(f)?;let path=field(f)?;
        let chapter=flag(f)?;let resource=flag(f)?;let stage_count=count(f,100)?;
        let source_id=field(f)?;let source_valid=flag(f)?;
        ids.push(id);slugs.push(slug);paths.push(path);
        if chapter{chapter_ids.push(id);if stage_count!=1{errors.push("每章必須屬於一個階段".into())}}
        if resource{resource_ids.push(id)}
        if !safe_relative(path)||!safe_relative(slug)||slug.contains('/'){return Err("catalog 路徑無效".into())}
        if source_id.len()<20||!source_id.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))||!source_valid{
            errors.push(format!("{slug} 缺少有效的來源記錄"));
        }
        let heading_count=count(f,5000)?;
        if heading_count==0{errors.push(format!("{slug} 缺少內容標題"))}
        let mut level=1;
        for _ in 0..heading_count{
            let next=count(f,6)?;let label=field(f)?;
            if next>level+1{errors.push(format!("{slug} 標題層級跳躍 {label}"))}
            level=next;
        }
        let fence_count=count(f,5000)?;
        total_fences+=fence_count;
        for _ in 0..fence_count{
            let line=count(f,1_000_000)?;let markup=field(f)?;let ending=field(f)?.trim();
            if markup.is_empty()||!ending.starts_with(markup)||!ending.chars().all(|c|Some(c)==markup.chars().next()){
                errors.push(format!("{slug}.md:{line} 缺少程式碼結尾"));
            }
        }
        let notes=strings(f,5000)?;
        lessons.push(Lesson{slug,route:format!("{base}{path}/"),chapter,notes});
    }
    if chapter_ids.len()!=21||chapter_ids.iter().enumerate().any(|(i,id)|id.parse::<usize>().ok()!=Some(i+1)){
        errors.push("章節必須依序涵蓋 01–21".into());
    }
    if resource_ids.len()!=8||resource_ids.iter().enumerate().any(|(i,id)|id.strip_prefix('A').and_then(|s|s.parse::<usize>().ok())!=Some(i+1)){
        errors.push("附錄必須依序涵蓋 A01–A08".into());
    }
    if !unique(&ids)||!unique(&slugs)||!unique(&paths){errors.push("課程識別碼、slug 與路徑不可重複".into())}
    let source_files=fs::read_dir("content").map_err(|e|e.to_string())?
        .map(|e|e.map(|e|e.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>().map_err(|e|e.to_string())?;
    let source_files:BTreeSet<_>=source_files.into_iter().filter(|s|s.ends_with(".md")).collect();
    if source_files.len()!=lessons.len()||lessons.iter().any(|l|!source_files.contains(&format!("{}.md",l.slug))){
        errors.push("Markdown 檔案與 catalog.json 不一致".into());
    }
    let mut pages=BTreeMap::new();
    for _ in 0..n{
        let route=field(f)?;let h1=count(f,5000)?;let main=count(f,5000)?;let lang=field(f)?;
        let missing_alt=count(f,5000)?;
        let ids=strings(f,20_000)?;
        if !unique(&ids){errors.push(format!("{route} 有重複 id"))}
        if h1!=1{errors.push(format!("{route} 必須包含單一 H1"))}
        if main!=1||!ids.contains(&"main"){errors.push(format!("{route} 缺少內容地標"))}
        if lang!="zh-Hant"{errors.push(format!("{route} 缺少 zh-Hant"))}
        if missing_alt>0{errors.push(format!("{route} 圖片缺少替代文字"))}
        let mut notes=Vec::new();
        for _ in 0..count(f,5000)?{notes.push(Note{kind:field(f)?,role:field(f)?,title:field(f)?})}
        let mut links=Vec::new();
        for _ in 0..count(f,20_000)?{links.push(Link{address:field(f)?,path:field(f)?,hash:field(f)?})}
        if pages.insert(route,Page{ids,notes,links}).is_some(){return Err("重複頁面路徑".into())}
    }
    for route in [base.to_owned(),format!("{base}resources/"),format!("{base}404.html")].into_iter().chain(lessons.iter().map(|l|l.route.clone())){
        if !pages.contains_key(route.as_str()){errors.push(format!("{route} 尚未生成"))}
    }
    for l in lessons.iter().filter(|l|l.chapter){
        if let Some(page)=pages.get(l.route.as_str()){
            let kinds:Vec<_>=page.notes.iter().map(|n|n.kind).collect();
            if kinds.is_empty()||kinds!=l.notes||kinds.iter().any(|k|!super::KINDS.contains(k)){
                errors.push(format!("{} 教學方塊與教材來源不一致",l.route));
            }
            for note in &page.notes{if !check_note(note.title,note.role){errors.push(format!("{} 教學方塊缺少文字標題或語意",l.route))}}
        }
    }
    let dist=Path::new("dist");
    let mut assets=BTreeSet::new();
    let mut checked=0usize;
    for (route,page) in &pages{
        for link in &page.links{
            checked+=1;
            let Some(name)=link.path.strip_prefix(base)else{errors.push(format!("{route}: {} 超出部署子路徑",link.address));continue};
            if let Some(target)=pages.get(link.path){
                if !link.hash.is_empty()&&!target.ids.contains(&link.hash){errors.push(format!("{route}: 錨點不存在 {}",link.address))}
            }else if file_size(dist,name).is_some(){assets.insert(name);}
            else{errors.push(format!("{route}: 資源或頁面不存在 {}",link.address))}
        }
    }
    let layout=field(f)?;let report_pages=count(f,1000)?;
    let report_count=count(f,1000)?;let mut report_valid=layout=="rust"&&report_pages==pages.len()&&report_count==lessons.len();
    let mut has_rust=false;
    for i in 0..report_count{
        let slug=field(f)?;let renderer=field(f)?;
        report_valid &= lessons.get(i).is_some_and(|l|l.slug==slug)&&matches!(renderer,"rust"|"legacy");
        has_rust |= renderer=="rust";
    }
    if !report_valid||!has_rust{errors.push("Rust 渲染報告不完整".into())}
    let renderer=field(f)?;let native=count(f,100_000)?;let total=count(f,100_000)?;let fallback=count(f,100_000)?;
    if renderer!="rust"||native!=total||total!=total_fences||fallback!=0{errors.push("Rust 高亮報告未達零回退".into())}
    let shape_valid=flag(f)?;
    let expected_index=read_index(f)?;
    let index=read_index(f)?;
    if !shape_valid||index!=expected_index{errors.push("Rust 搜尋索引與原有資料契約不一致".into())}
    let index_valid=index.len()==lessons.len()&&index.iter().all(|entry|!entry.text.is_empty()&&
        pages.get(entry.href).is_some_and(|p|entry.headings.iter().all(|(id,_)|p.ids.contains(id))));
    if !index_valid{errors.push("搜尋索引的頁面或章節定位不完整".into())}
    let mut initial=vec!["index.html"];
    initial.extend(assets.iter().copied().filter(|s|!s.ends_with(".json")));
    let mut bytes=0u64;
    for path in initial{bytes+=file_size(dist,path).ok_or_else(||format!("資源不存在 {path}"))?;}
    if bytes>160_000{errors.push(format!("首頁與共用資源超過 160 kB（實際 {bytes} bytes）"))}
    if !errors.is_empty(){return Err(errors.join("\n"))}
    for value in [lessons.len() as u64,pages.len() as u64,checked as u64,bytes]{emit(out,&value.to_string())?;}
    Ok(())
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn path_and_note_guards(){
        assert!(safe_relative("assets/site.css"));assert!(!safe_relative("../README.md"));assert!(!safe_relative("a\\b"));
        assert!(check_note("為何要借用？","note"));assert!(!check_note("來龍去脈","note"));assert!(!check_note("內容",""));
    }
}
