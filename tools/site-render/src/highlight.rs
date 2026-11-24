//! Native candidate highlighting. The bridge only selects exact reference matches.
//! Rust grammar behavior is compared against highlight.js 11.12.0 (BSD-3-Clause).
use super::{Result,emit,field,escape};

fn escaped(text:&str)->String{escape(text).replace("&#39;","&#x27;")}
fn span(out:&mut String,class:&str,text:&str){
    out.push_str("<span class=\"hljs-");out.push_str(class);out.push_str("\">");
    out.push_str(&escaped(text));out.push_str("</span>");
}
fn classify(lang:&str,word:&str)->Option<&'static str>{
    match lang {
        "rust"=>match word{
            "true"|"false"|"Some"|"None"|"Ok"|"Err"=>Some("literal"),
            "fn"|"let"|"mut"|"const"|"static"|"pub"|"use"|"mod"|"struct"|"enum"|"impl"|"trait"|"type"|"where"|"match"|"if"|"else"|"while"|"loop"|"for"|"in"|"break"|"continue"|"return"|"as"|"move"|"async"|"await"|"unsafe"|"extern"|"ref"|"dyn"|"crate"|"super"|"self"|"Self"|"abstract"|"become"|"box"|"do"|"final"|"macro"|"override"|"priv"|"raw"|"try"|"typeof"|"union"|"unsized"|"virtual"|"yield"=>Some("keyword"),
            "i8"|"i16"|"i32"|"i64"|"i128"|"isize"|"u8"|"u16"|"u32"|"u64"|"u128"|"usize"|"f16"|"f32"|"f64"|"f128"|"bool"|"char"|"str"|"Box"|"Option"|"Result"|"String"|"Vec"=>Some("type"),
            "drop"|"Copy"|"Send"|"Sized"|"Sync"|"Drop"|"Fn"|"FnMut"|"FnOnce"|"ToOwned"|"Clone"|"Debug"|"PartialEq"|"PartialOrd"|"Eq"|"Ord"|"AsRef"|"AsMut"|"Into"|"From"|"Default"|"Iterator"|"Extend"|"IntoIterator"|"DoubleEndedIterator"|"ExactSizeIterator"|"SliceConcatExt"|"ToString"
            |"assert!"|"assert_eq!"|"assert_ne!"|"bitflags!"|"bytes!"|"cfg!"|"col!"|"concat!"|"concat_idents!"|"debug_assert!"|"debug_assert_eq!"|"debug_assert_ne!"|"env!"|"eprintln!"|"panic!"|"file!"|"format!"|"format_args!"|"include_bytes!"|"include_str!"|"line!"|"local_data_key!"|"module_path!"|"option_env!"|"print!"|"println!"|"select!"|"stringify!"|"try!"|"unimplemented!"|"unreachable!"|"vec!"|"write!"|"writeln!"|"macro_rules!"=>Some("built_in"),
            _=>None,
        },
        "bash"=>match word{"if"|"then"|"else"|"fi"|"for"|"while"|"do"|"done"|"case"|"esac"|"function"|"in"=>Some("keyword"),_=>None},
        "toml"=>match word{"true"|"false"=>Some("literal"),_=>None},
        _=>None,
    }
}
fn ident(s:&str)->usize{
    let prefix=if s.starts_with("r#"){2}else{0};
    let tail=&s[prefix..];
    if !tail.as_bytes().first().is_some_and(|b|b.is_ascii_alphabetic()||*b==b'_'){return 0}
    prefix+tail.bytes().take_while(|b|b.is_ascii_alphanumeric()||*b==b'_').count()
}
fn whitespace(s:&str)->usize{s.len()-s.trim_start().len()}
fn quote_len(s:&str,offset:usize,quote:char)->usize{
    let mut len=offset;let mut escaped=false;
    for ch in s[offset..].chars(){
        len+=ch.len_utf8();
        if escaped{escaped=false}else if ch=='\\'{escaped=true}else if ch==quote{break}
    }
    len
}
fn raw_len(s:&str)->Option<usize>{
    let head=if s.starts_with("br"){2}else if s.starts_with('r'){1}else{return None};
    let hashes=s[head..].bytes().take_while(|b|*b==b'#').count();
    let start=head+hashes;
    if !s[start..].starts_with('"'){return None}
    let end=format!("\"{}","#".repeat(hashes));
    s[start+1..].find(&end).map(|i|start+1+i+end.len())
}
fn comment_len(s:&str)->usize{
    let mut depth=1;let mut i=2;
    while i<s.len(){
        let tail=&s[i..];
        if tail.starts_with("/*"){depth+=1;i+=2}
        else if tail.starts_with("*/"){depth-=1;i+=2;if depth==0{break}}
        else{i+=tail.chars().next().unwrap().len_utf8()}
    }
    i
}
fn number_len(s:&str)->usize{
    let (prefix,radix)=if s.starts_with("0x"){(2,16)}else if s.starts_with("0b"){(2,2)}else if s.starts_with("0o"){(2,8)}else{(0,10)};
    let digits=|tail:&str|tail.chars().take_while(|c|*c=='_'||c.is_digit(radix)).map(char::len_utf8).sum::<usize>();
    let mut i=prefix+digits(&s[prefix..]);
    if i==prefix{i=1}
    if radix==10{
        let tail=&s[i..];
        if tail.starts_with('.')&&tail.as_bytes().get(1).is_some_and(u8::is_ascii_digit){i+=1+digits(&tail[1..])}
        if s[i..].starts_with(['e','E']){
            let start=i+1;
            let start=if s[start..].starts_with(['+','-']){start+1}else{start};
            let len=digits(&s[start..]);if len>0{i=start+len}
        }
    }
    for suffix in ["usize","isize","u128","i128","f128","u64","i64","f64","u32","i32","f32","u16","i16","f16","u8","i8"]{
        if s[i..].starts_with(suffix){i+=suffix.len();break}
    }
    i
}
fn declaration(out:&mut String,s:&str,word:&str,end:usize)->Option<usize>{
    let gap=whitespace(&s[end..]);
    if gap==0{return None}
    let pos=end+gap;
    let mut start=pos;
    if word=="let"&&s[start..].starts_with("mut"){
        let n=whitespace(&s[start+3..]);
        if n>0{start+=3+n;}
    }
    let len=ident(&s[start..]);
    if len==0{return None}
    let class=match word{
        "fn"=>"title function_","let"=>"variable","type"|"trait"|"enum"|"struct"|"union"|"impl"=>"title class_",
        "for"=>{let tail=&s[start+len..];let gap=whitespace(tail);
            if gap>0&&tail[gap..].starts_with("in")&&ident(&tail[gap..])==2{"variable"}else{"title class_"}},
        _=>return None
    };
    span(out,"keyword",word);
    out.push_str(&escaped(&s[end..pos]));
    if start>pos{span(out,"keyword",&s[pos..start]);}
    span(out,class,&s[start..start+len]);
    Some(start+len)
}

fn shell_word(s:&str)->usize{
    s.bytes().take_while(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b"._-".contains(b)).count()
}
fn bash(source:&str)->String{
    let mut out=String::new();let mut i=0;
    while i<source.len(){
        let s=&source[i..];let ch=s.chars().next().unwrap();
        let before=source[..i].chars().next_back();
        if ch=='#'&&before.is_none_or(char::is_whitespace){
            let n=s.find('\n').unwrap_or(s.len());span(&mut out,"comment",&s[..n]);i+=n;continue
        }
        if ch=='/'{
            let n=s.bytes().take_while(|b|b.is_ascii_lowercase()||b"/._-".contains(b)).count();
            if n>1{out.push_str(&escaped(&s[..n]));i+=n;continue}
        }
        if ch=='"'||ch=='\''{
            let n=quote_len(s,1,ch);span(&mut out,"string",&s[..n]);i+=n;continue
        }
        if ch=='$'{
            let n=if s.as_bytes().get(1)==Some(&b'{'){s.find('}').map(|p|p+1).unwrap_or(1)}else{1+s[1..].bytes().take_while(|b|b.is_ascii_alphanumeric()||b"_#@".contains(b)).count()};
            if n>1{span(&mut out,"variable",&s[..n]);i+=n;continue}
        }
        if ch.is_ascii_lowercase()&&before.is_none_or(|b|!b.is_ascii_alphanumeric()&&b!='_'){
            let n=shell_word(s);let w=&s[..n];
            let scope=match w{
                "true"|"false"=>Some("literal"),
                "if"|"then"|"else"|"elif"|"fi"|"time"|"for"|"while"|"until"|"in"|"do"|"done"|"case"|"esac"|"coproc"|"function"|"select"=>Some("keyword"),
                "break"|"cd"|"continue"|"eval"|"exec"|"exit"|"export"|"getopts"|"hash"|"pwd"|"readonly"|"return"|"shift"|"test"|"times"|"trap"|"umask"|"unset"|"alias"|"bind"|"builtin"|"caller"|"command"|"declare"|"echo"|"enable"|"help"|"let"|"local"|"logout"|"mapfile"|"printf"|"read"|"readarray"|"source"|"sudo"|"type"|"typeset"|"ulimit"|"unalias"|"set"|"shopt"|"chcon"|"chgrp"|"chown"|"chmod"|"cp"|"dd"|"df"|"dir"|"ln"|"ls"|"mkdir"|"mktemp"|"mv"|"realpath"|"rm"|"rmdir"|"touch"|"cat"|"cut"|"head"|"tail"|"sort"|"uniq"|"wc"|"date"|"dirname"|"du"|"env"|"id"|"sleep"|"tee"|"timeout"|"uname"|"whoami"|"yes"=>Some("built_in"),
                _=>None
            };
            if let Some(class)=scope{span(&mut out,class,w)}else{out.push_str(&escaped(w))}
            i+=n;continue
        }
        out.push_str(&escaped(&s[..ch.len_utf8()]));i+=ch.len_utf8();
    }
    out
}
fn toml(source:&str)->String{
    let mut out=String::new();
    for line in source.split_inclusive('\n'){
        let gap=whitespace(line);out.push_str(&escaped(&line[..gap]));
        let s=&line[gap..];
        if s.starts_with('['){
            if let Some(end)=s.rfind(']'){span(&mut out,"section",&s[..end+1]);out.push_str(&escaped(&s[end+1..]));continue}
        }
        if s.starts_with('#')||s.starts_with(';'){
            let end=s.find('\n').unwrap_or(s.len());span(&mut out,"comment",&s[..end]);out.push_str(&escaped(&s[end..]));continue
        }
        let Some(eq)=s.find('=')else{out.push_str(&escaped(s));continue};
        let key=s[..eq].trim_end();
        span(&mut out,"attr",key);out.push_str(&escaped(&s[key.len()..eq+1]));
        let mut i=eq+1;
        while i<s.len(){
            let tail=&s[i..];let ch=tail.chars().next().unwrap();
            if ch=='"'||ch=='\''{
                let n=quote_len(tail,1,ch);span(&mut out,"string",&tail[..n]);i+=n;continue
            }
            if ch=='#'||ch==';'{
                let n=tail.find('\n').unwrap_or(tail.len());span(&mut out,"comment",&tail[..n]);i+=n;continue
            }
            if ch.is_ascii_digit(){let n=number_len(tail);span(&mut out,"number",&tail[..n]);i+=n;continue}
            let n=ident(tail);
            if n>0{
                let word=&tail[..n];if matches!(word,"true"|"false"|"yes"|"no"|"on"|"off"){span(&mut out,"literal",word)}else{out.push_str(&escaped(word))}
                i+=n;continue
            }
            out.push_str(&escaped(&tail[..ch.len_utf8()]));i+=ch.len_utf8();
        }
    }
    out
}

pub(super) fn highlight(lang:&str,source:&str)->String{
    if lang=="bash"{return bash(source)}
    if lang=="toml"{return toml(source)}
    if !matches!(lang,"rust"|"bash"|"toml"){return escape(source)}
    let mut out=String::new();let mut i=0;
    while i<source.len(){
        let s=&source[i..];let ch=s.chars().next().unwrap();
        if (lang=="rust"&&s.starts_with("//"))||(lang!="rust"&&ch=='#'){
            let len=s.find('\n').unwrap_or(s.len());span(&mut out,"comment",&s[..len]);i+=len;continue
        }
        if lang=="rust"{
            if s.starts_with("/*"){let len=comment_len(s);span(&mut out,"comment",&s[..len]);i+=len;continue}
            if let Some(len)=raw_len(s){span(&mut out,"string",&s[..len]);i+=len;continue}
            if s.starts_with("->"){span(&mut out,"punctuation","->");i+=2;continue}
            if s.starts_with("#[")||s.starts_with("#!["){
                let len=s.find(']').map(|p|p+1).unwrap_or(s.len());span(&mut out,"meta",&s[..len]);i+=len;continue
            }
            if s.starts_with('\''){
                let len=ident(&s[1..]);
                if len>0&&!s[1+len..].starts_with('\''){span(&mut out,"symbol",&s[..1+len]);i+=1+len;continue}
            }
        }
        let quote=if lang=="rust"&&(s.starts_with("b\"")||s.starts_with("b'")){Some((2,s.as_bytes()[1] as char))}
            else if ch=='"'||ch=='\''{Some((ch.len_utf8(),ch))}else{None};
        if let Some((offset,quote))=quote{
            let len=quote_len(s,offset,quote);span(&mut out,"string",&s[..len]);i+=len;continue
        }
        if ch.is_ascii_digit(){let len=number_len(s);span(&mut out,"number",&s[..len]);i+=len;continue}
        let len=ident(s);
        if len>0{
            let word=&s[..len];
            if lang=="rust"{
                if let Some(end)=declaration(&mut out,s,word,len){i+=end;continue}
                let n=if s[len..].starts_with('!'){len+1}else{len};
                let w=&s[..n];let tail=&s[n..];
                let scope=if n==len&&tail.starts_with("::"){
                    // A path segment has the reference grammar's reduced keyword set.
                    classify(lang,w).filter(|class|matches!(*class,"type"|"built_in")||w=="Self")
                }else if n==len&&tail.trim_start().starts_with('(')&&!matches!(word,"let"|"for"|"while"|"if"|"else"|"match"){
                    Some("title function_ invoke__")
                }else{classify(lang,w)};
                if let Some(class)=scope{span(&mut out,class,w)}else{out.push_str(&escaped(w))}
                i+=n;continue
            }
            if let Some(class)=classify(lang,word){span(&mut out,class,word)}else{out.push_str(&escaped(word))}
            i+=len;continue
        }
        out.push_str(&escaped(&s[..ch.len_utf8()]));i+=ch.len_utf8();
    }
    out
}
pub(super) fn block(info:&str,highlighted:&str,icon:&str)->String{
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
        let info=field(f)?;let code=field(f)?;
        let language=info.split_whitespace().next().unwrap_or("");
        emit(out,&block(info,&highlight(language,code),icon))?;
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
    #[test]fn highlights_path_and_config_regressions(){
        assert_eq!(highlight("rust","use super::*;"),"<span class=\"hljs-keyword\">use</span> super::*;");
        assert_eq!(highlight("rust","String::new()"),"<span class=\"hljs-type\">String</span>::<span class=\"hljs-title function_ invoke__\">new</span>()");
        assert_eq!(highlight("bash","cd ~/rust-learning/day01\n"),"<span class=\"hljs-built_in\">cd</span> ~/rust-learning/day01\n");
        assert_eq!(highlight("bash","rustc --edition=2024 main.rs\n"),"rustc --edition=2024 main.rs\n");
        assert_eq!(highlight("toml","[package]\nedition = \"2024\"\n"),"<span class=\"hljs-section\">[package]</span>\n<span class=\"hljs-attr\">edition</span> = <span class=\"hljs-string\">&quot;2024&quot;</span>\n");
    }
    #[test]fn wrapper_preserves_contract(){
        let h=block("rust error E0382","&lt;","ICON");
        assert!(h.contains("Rust <small>編譯反例</small>"));
        assert!(h.contains("data-copy hidden>ICON"));
        assert!(h.ends_with("</code></pre></div>\n"));
    }
}
