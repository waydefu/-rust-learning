//! Conservative native syntax highlighting. The bridge retains the reference
//! fragment whenever it differs, so migration cannot change visual output.
use super::{Result,emit,field,escape};

fn span(out:&mut String,class:&str,text:&str){
    out.push_str("<span class=\"hljs-");out.push_str(class);out.push_str("\">");
    out.push_str(&escape(text));out.push_str("</span>");
}
fn classify(lang:&str,word:&str)->Option<&'static str>{
    match lang {
        "rust" => match word {
            "true"|"false"=>Some("literal"),
            "fn"|"let"|"mut"|"const"|"static"|"pub"|"use"|"mod"|"struct"|"enum"|"impl"|"trait"|"type"|"where"|"match"|"if"|"else"|"while"|"loop"|"for"|"in"|"break"|"continue"|"return"|"as"|"move"|"async"|"await"|"unsafe"|"extern"|"ref"|"dyn"|"crate"|"super"|"self"|"Self"=>Some("keyword"),
            "i8"|"i16"|"i32"|"i64"|"i128"|"isize"|"u8"|"u16"|"u32"|"u64"|"u128"|"usize"|"f32"|"f64"|"bool"|"char"|"str"=>Some("type"),
            _=>None,
        },
        "bash"=>match word{"if"|"then"|"else"|"fi"|"for"|"while"|"do"|"done"|"case"|"esac"|"function"|"in"=>Some("keyword"),_=>None},
        "toml"=>match word{"true"|"false"=>Some("literal"),_=>None},
        _=>None,
    }
}
fn highlight(lang:&str,source:&str)->String{
    if !matches!(lang,"rust"|"bash"|"toml"){return escape(source)}
    let mut out=String::new();let mut i=0;
    while i<source.len(){
        let rest=&source[i..];
        let ch=rest.chars().next().unwrap();
        let (len,kind)=if (lang=="rust"&&rest.starts_with("//"))||(lang!="rust"&&ch=='#'){
            (rest.find('\n').unwrap_or(rest.len()),Some("comment"))
        }else if ch=='"'||(lang!="rust"&&ch=='\''){
            let mut len=ch.len_utf8();let mut escaped=false;
            for c in rest[len..].chars(){len+=c.len_utf8();if escaped{escaped=false}else if c=='\\'{escaped=true}else if c==ch{break}}
            (len,Some("string"))
        }else if ch.is_ascii_digit(){
            let len=rest.find(|c:char|!c.is_ascii_digit()&&c!='_').unwrap_or(rest.len());
            (len,Some("number"))
        }else if ch.is_alphabetic()||ch=='_'{
            let len=rest.find(|c:char|!c.is_alphanumeric()&&c!='_').unwrap_or(rest.len());
            (len,classify(lang,&rest[..len]))
        }else{(ch.len_utf8(),None)};
        let text=&rest[..len];
        if let Some(class)=kind{span(&mut out,class,text)}else{out.push_str(&escape(text))}
        i+=len;
    }
    out
}
fn block(info:&str,highlighted:&str,icon:&str)->String{
    let mut parts=info.split_whitespace();
    let language=parts.next().unwrap_or("");
    let mode=parts.next().unwrap_or("");
    let name=match language{"rust"=>"Rust","bash"=>"Shell","toml"=>"TOML","text"=>"文字",_=>"程式碼"};
    let note=match mode{"fragment"=>"片段","continuation"=>"接續範例","test"=>"測試片段","project"=>"專案範例","error"=>"編譯反例","exercise"=>"練習反例",_=>""};
    let note=if note.is_empty(){String::new()}else{format!(" <small>{note}</small>")};
    format!("<div class=\"code-block\"><div class=\"code-header\"><span>{name}{note}</span><button type=\"button\" class=\"copy-button\" data-copy hidden>{icon}<span>複製</span></button></div><pre tabindex=\"0\" aria-label=\"{name}程式碼\"><code class=\"language-{}\">{highlighted}</code></pre></div>\n",escape(language))
}
pub(super) fn process_highlight<'a>(n:usize,f:&mut impl Iterator<Item=&'a str>,out:&mut Vec<u8>)->Result<()>{
    let icon=field(f)?;
    for _ in 0..n{
        let info=field(f)?;let code=field(f)?;let reference=field(f)?;
        let language=info.split_whitespace().next().unwrap_or("");
        emit(out,&block(info,&highlight(language,code),icon))?;
        emit(out,&block(info,reference,icon))?;
    }
    Ok(())
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn native_candidate_escapes_and_colors(){
        let h=highlight("rust","let x = 42; // <&\n");
        assert!(h.contains("hljs-keyword"));assert!(h.contains("hljs-number"));
        assert!(h.contains("&lt;&amp;"));assert!(!h.contains("<&"));
        assert_eq!(highlight("text","<b>"),"&lt;b&gt;");
    }
    #[test]fn wrapper_preserves_contract(){
        let h=block("rust error E0382","&lt;","ICON");
        assert!(h.contains("Rust <small>編譯反例</small>"));
        assert!(h.contains("data-copy hidden>ICON"));
        assert!(h.ends_with("</code></pre></div>\n"));
    }
}
