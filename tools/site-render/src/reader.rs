//! Native reader and interactive templates. Trusted Markdown fragments are supplied by the bridge.
use super::{Result,count,emit,escape,field,render_template};
use std::collections::HashMap;
fn fill(t:&str, v:&[(&str,&str)])->Result<String>{
    render_template(t,&v.iter().map(|(k,v)|(*k,(*v).to_owned())).collect::<HashMap<_,_>>())
}
fn flag<'a>(f:&mut impl Iterator<Item=&'a str>)->Result<bool>{
    match field(f)? {"true"=>Ok(true),"false"=>Ok(false),_=>Err("互動旗標無效".into())}
}
fn safe(s:&str)->bool{!s.is_empty()&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b))}
fn quiz<'a>(f:&mut impl Iterator<Item=&'a str>,slug:&str,icon:&str)->Result<String>{
    if !flag(f)?{return Ok(String::new())}
    let prompt=escape(field(f)?);let explanation=escape(field(f)?);
    let n=count(f,100)?;let answer=count(f,100)?;
    if answer>=n{return Err("測驗答案無效".into())}
    let mut choices=String::new();let mut correct=String::new();
    for i in 0..n{
        let choice=escape(field(f)?);
        if i==answer{correct=choice.clone()}
        choices.push_str(&format!(r#"<label><input type="radio" name="answer" value="{i}" required><span>{choice}</span></label>"#));
    }
    fill(include_str!("../templates/quiz.html"),&[("slug",slug),("icon",icon),("prompt",&prompt),("choices",&choices),("answer",&correct),("explanation",&explanation)])
}
fn flow<'a>(f:&mut impl Iterator<Item=&'a str>,icon:&str)->Result<String>{
    if !flag(f)?{return Ok(String::new())}
    let title=escape(field(f)?);let caption=escape(field(f)?);let mut nodes=String::new();
    for i in 0..count(f,100)?{
        let node=escape(field(f)?);
        nodes.push_str(&format!("<li>{}<span>{node}</span></li>",if i==0{""}else{icon}));
    }
    fill(include_str!("../templates/flow.html"),&[("title",&title),("caption",&caption),("nodes",&nodes)])
}
fn ownership<'a>(f:&mut impl Iterator<Item=&'a str>,icon:&str)->Result<String>{
    if !flag(f)?{return Ok(String::new())}
    let(mut buttons,mut steps,mut first_code,mut first_message)=(String::new(),String::new(),String::new(),String::new());
    for(i,label)in ["建立","移動","借用","釋放"].into_iter().enumerate(){
        let code=escape(field(f)?);let message=escape(field(f)?);
        if i==0{first_code=code.clone();first_message=message.clone()}
        buttons.push_str(&format!(r#"<button type="button" data-step="{i}" aria-controls="memory-state" aria-pressed="{}">{} {label}</button>"#,i==0,i+1));
        steps.push_str(&format!("<li><code>{code}</code> {message}</li>"));
    }
    fill(include_str!("../templates/ownership-lab.html"),&[("icon",icon),("buttons",&buttons),("code",&first_code),("message",&first_message),("steps",&steps)])
}
pub(super) fn process_reader<'a>(n:usize,f:&mut impl Iterator<Item=&'a str>,out:&mut Vec<u8>)->Result<()>{
    let base=field(f)?;
    if !base.starts_with('/')||!base.ends_with('/')||base.starts_with("//")||!base.bytes().all(|b|b.is_ascii_alphanumeric()||b"/_.-".contains(&b))||base.split('/').any(|s|matches!(s,"."|"..")){return Err("閱讀頁網址無效".into())}
    let [chevron,list,check,target,layers,flow_icon]=[field(f)?,field(f)?,field(f)?,field(f)?,field(f)?,field(f)?];
    let footer=fill(include_str!("../templates/footer.html"),&[("base",base)])?;
    for _ in 0..n{
        let slug=field(f)?;if !safe(slug){return Err("教材 slug 無效".into())}
        let chapter=flag(f)?;let resource=flag(f)?;
        let entry_title=escape(field(f)?);let title=escape(field(f)?);let description=escape(field(f)?);let label=escape(field(f)?);
        let stage_title=escape(field(f)?);let label_icon=field(f)?;let source_id=field(f)?;
        if !safe(source_id){return Err("來源 ID 無效".into())}
        let source=format!("https://docs.google.com/document/d/{source_id}/edit");
        let [intro,rest,outline,extra,outline_links,pagination]=[field(f)?,field(f)?,field(f)?,field(f)?,field(f)?,field(f)?];
        let lab=ownership(f,layers)?;let flow=flow(f,flow_icon)?;let quiz=quiz(f,slug,target)?;
        let stage=if stage_title.is_empty(){String::new()}else{format!(r#"<span aria-hidden="true"> · </span>{stage_title}"#)};
        let crumb=if resource{format!(r#"<a href="{base}resources/">延伸資源</a>{chevron}"#)}else{String::new()};
        let completion=if chapter{format!(r#"<section class="completion" aria-labelledby="complete-title"><div><h2 id="complete-title">完成本章</h2><p data-completion-status role="status">試著用自己的話說明本章重點，再記下閱讀進度。</p></div><button type="button" class="button outline" id="complete" aria-pressed="false" hidden>{check} <span>標記完成</span></button></section>"#)}else{String::new()};
        emit(out,&fill(include_str!("../templates/reader.html"),&[("base",base),("chevron",chevron),("resourceCrumb",&crumb),("entryTitle",&entry_title),("labelIcon",label_icon),("label",&label),("stage",&stage),("title",&title),("description",&description),("list",list),("outline",outline),("extra",extra),("outlineLinks",outline_links),("ownership",&lab),("intro",intro),("flow",&flow),("rest",rest),("quiz",&quiz),("completion",&completion),("source",&source),("pagination",pagination),("footer",&footer)])?)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn quiz_checks_answers_and_escapes(){
        let fields=["true","<問>","&答","2","1","A","B"];
        let html=quiz(&mut fields.into_iter(),"test","").unwrap();
        assert!(html.contains("&lt;問&gt;"));assert!(html.contains("<strong>B</strong>"));
        assert!(quiz(&mut ["true","Q","E","1","1","A"].into_iter(),"test","").is_err());
    }
}
