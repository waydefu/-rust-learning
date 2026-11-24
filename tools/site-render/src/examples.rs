//! Execute trusted, checked-in Rust lesson examples in a temporary working directory.
//! This is not an OS sandbox; never run unreviewed snippets with local credentials.
use super::{Result,count,emit,field};
use std::{collections::{BTreeMap,BTreeSet},env,fs,io,path::{Path,PathBuf},process::{Command,Stdio},thread,time::{Duration,Instant,SystemTime,UNIX_EPOCH}};

fn err(e:impl std::fmt::Display)->String{e.to_string()}
struct Scratch(PathBuf);
impl Scratch{
    fn new()->Result<Self>{
        let now=SystemTime::now().duration_since(UNIX_EPOCH).map_err(err)?.as_nanos();
        for i in 0..100{
            let p=env::temp_dir().join(format!("rust-lessons-{}-{now}-{i}",std::process::id()));
            match fs::create_dir(&p){Ok(())=>return Ok(Self(p)),Err(e) if e.kind()==io::ErrorKind::AlreadyExists=>continue,Err(e)=>return Err(err(e))}
        }
        Err("無法建立暫存目錄".into())
    }
}
impl Drop for Scratch{fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
struct Output{success:bool,code:Option<i32>,text:String}
fn command(program:&Path,args:&[&str],cwd:&Path,log:&Path,seconds:u64)->Result<Output>{
    let file=fs::File::create(log).map_err(err)?;
    let mut child=Command::new(program).args(args).current_dir(cwd).stdin(Stdio::null())
        .stdout(Stdio::from(file.try_clone().map_err(err)?)).stderr(Stdio::from(file)).spawn().map_err(err)?;
    let start=Instant::now();
    let status=loop{
        match child.try_wait(){
            Ok(Some(status))=>break status,
            Ok(None) if start.elapsed()<Duration::from_secs(seconds)=>thread::sleep(Duration::from_millis(15)),
            Ok(None)=>{let _=child.kill();let _=child.wait();return Err(format!("命令逾時：{}",program.display()))},
            Err(e)=>{let _=child.kill();let _=child.wait();return Err(err(e))}
        }
    };
    let data=fs::read(log).map_err(err)?;
    Ok(Output{success:status.success(),code:status.code(),text:String::from_utf8_lossy(&data).into_owned()})
}
fn success(output:Output)->Result<()>{
    if output.success{Ok(())}else{Err(if output.text.is_empty(){format!("程式結束狀態 {:?}",output.code)}else{output.text})}
}
fn main_line(source:&str,allow_async:bool)->bool{
    source.lines().any(|line|{
        let line=if allow_async{line.trim_start()}else{line};
        let line=if allow_async{line.strip_prefix("async").filter(|tail|tail.starts_with(char::is_whitespace)).map(str::trim_start).unwrap_or(line)}else{line};
        let Some(rest)=line.strip_prefix("fn")else{return false};
        rest.starts_with(char::is_whitespace)&&rest.trim_start().strip_prefix("main").is_some_and(|s|s.trim_start().starts_with('('))
    })
}
fn as_binary(source:&str)->String{if main_line(source,true){source.into()}else{format!("fn main() {{\n{source}\n}}\n")}}
fn diagnostics(text:&str)->BTreeSet<String>{
    // rustc --error-format=short emits location: error[Edddd]: message.
    // Ignore warnings, uncoded summary lines, and any message suffix.
    text.lines().filter_map(|line|{
        let (_,tail)=line.split_once(": error[")?;
        let (code,_)=tail.split_once("]:")?;
        if code.len()==5&&code.starts_with('E')&&code.as_bytes()[1..].iter().all(u8::is_ascii_digit){Some(code.to_owned())}else{None}
    }).collect()
}
// Explicit line-by-line output contract for deterministic run examples.
// No annotation means exit-status validation only. Output uses LF on all platforms.
fn expected_output(source: &str) -> Option<String> {
    let lines: Vec<_> = source.lines().filter_map(|line| {
        line.trim_start().strip_prefix("// 預期輸出: ").map(str::to_owned)
    }).collect();
    if lines.is_empty() { None } else { Some(lines.join("\n") + "\n") }
}
fn verify_output(expected: &str, actual: &str) -> Result<()> {
    let actual = actual.replace("\r\n", "\n");
    if actual == expected { Ok(()) } else {
        Err(format!("預期輸出 {expected:?}，實際 {actual:?}"))
    }
}
fn safe_name(s:&str)->bool{!s.is_empty()&&s.as_bytes()[0].is_ascii_lowercase()&&s.bytes().all(|b|b.is_ascii_lowercase()||b==b'_')}
fn safe_file(s:&str)->bool{
    let Some(p)=s.strip_prefix("src/")else{return false};
    let p=p.strip_prefix("bin/").unwrap_or(p);
    p.strip_suffix(".rs").is_some_and(|s|!s.is_empty()&&s.bytes().all(|b|b.is_ascii_lowercase()||b==b'_'))
}
#[derive(Default)]
struct Stats{compiled:usize,run:usize,error:usize,project:usize}
struct Runner{
    root:PathBuf,rustc:PathBuf,cargo:PathBuf,fixture:PathBuf,binary:PathBuf,log:PathBuf,
    projects:BTreeMap<String,BTreeMap<String,String>>,stats:Stats
}
impl Runner{
    fn snippet(&mut self,mode:&str,arg:&str,source:&str,previous:Option<&str>)->Result<()>{
        if mode=="project"{
            let (name,path)=arg.split_once(':').ok_or("專案範例需指定 name:src/file.rs")?;
            if !safe_name(name)||!safe_file(path){return Err("專案範例需指定 name:src/file.rs".into())}
            let files=self.projects.entry(name.into()).or_default();
            if files.contains_key(path){return Err(format!("重複的專案範例 {name}:{path}"))}
            files.insert(path.into(),if name=="external"{as_binary(source)}else{source.into()});
            return Ok(())
        }
        let joined;
        let source=if mode=="continuation"{
            joined=format!("{}\n{source}",previous.filter(|s|!s.is_empty()).ok_or("接續範例的前一個區塊必須是 Rust")?);
            &joined
        }else{source};
        if mode=="program"&&!main_line(source,false){return Err("完整程式缺少 fn main()".into())}
        let program=if mode=="test"{source.to_owned()}else{as_binary(source)};
        fs::write(&self.fixture,program).map_err(err)?;
        let binary=self.binary.to_str().ok_or("暫存路徑非 UTF-8")?;
        let fixture=self.fixture.to_str().ok_or("暫存路徑非 UTF-8")?;
        let mut flags=vec!["--edition=2024","-Awarnings"];
        if mode=="error"{
            flags.extend(["--emit=metadata","--error-format=short","-o",binary,fixture]);
            let expected:BTreeSet<String>=arg.split(',').map(str::to_owned).collect();
            if expected.is_empty()||expected.iter().any(|s|s.len()!=5||!s.starts_with('E')||!s.as_bytes()[1..].iter().all(u8::is_ascii_digit)){return Err("需指定預期診斷碼".into())}
            let result=command(&self.rustc,&flags,&self.root,&self.log,20)?;
            let actual=diagnostics(&result.text);
            if result.success||result.code.is_none()||expected!=actual{return Err(format!("預期 {arg}，實際 {actual:?}\n{}",result.text))}
            self.stats.error+=1;return Ok(())
        }
        if !matches!(mode,"run"|"test"|"fragment"|"continuation"|"program"){return Err(format!("未知的 Rust 範例類型 {mode}"))}
        if mode=="test"{flags.push("--test")}
        if matches!(mode,"fragment"|"continuation"|"program"){flags.push("--emit=metadata")}
        flags.extend(["-o",binary,fixture]);
        success(command(&self.rustc,&flags,&self.root,&self.log,20)?)?;
        self.stats.compiled+=1;
        if matches!(mode,"run"|"test"){
            let output=command(&self.binary,&[],&self.root,&self.log,20)?;
            if mode=="run" && output.success {
                if let Some(expected)=expected_output(source){verify_output(&expected,&output.text)?;}
            }
            success(output)?;self.stats.run+=1;
        }
        Ok(())
    }
    fn project(&self,name:&str,files:&BTreeMap<String,String>,project_root:&Path)->Result<()>{
        let project=self.root.join(name);fs::create_dir_all(&project).map_err(err)?;
        if name=="external"{
            for file in ["Cargo.toml","Cargo.lock"]{fs::copy(project_root.join("examples").join(file),project.join(file)).map_err(err)?;}
        }else{fs::write(project.join("Cargo.toml"),format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n")).map_err(err)?;}
        for (path,source) in files{
            let target=project.join(path);
            fs::create_dir_all(target.parent().ok_or("專案路徑無效")?).map_err(err)?;
            fs::write(target,source).map_err(err)?;
        }
        if name=="external"{
            success(command(&self.cargo,&["test","--locked","--all-targets","--quiet"],&project,&self.log,180)?)?;
            for bin in ["random","timer"]{success(command(&self.cargo,&["run","--locked","--quiet","--bin",bin],&project,&self.log,180)?)?;}
        }else{success(command(&self.cargo,&["test","--offline","--quiet"],&project,&self.log,60)?)?;}
        Ok(())
    }
}
pub(super) fn process_examples<'a>(n:usize,f:&mut impl Iterator<Item=&'a str>,out:&mut Vec<u8>)->Result<()>{
    // Parse the entire packet before running any command.
    let mut lessons=Vec::new();
    for _ in 0..n{
        let slug=field(f)?;
        let mut tokens=Vec::new();
        for _ in 0..count(f,5000)?{tokens.push((count(f,1_000_000)?,field(f)?,field(f)?));}
        lessons.push((slug,tokens));
    }
    if f.next().is_some(){return Err("範例資料包含多餘欄位".into())}
    let root=Scratch::new()?;
    let project_root=env::current_dir().map_err(err)?;
    fs::copy(project_root.join("rust-toolchain.toml"),root.0.join("rust-toolchain.toml")).map_err(err)?;
    let mut runner=Runner{
        root:root.0.clone(),rustc:env::var_os("RUSTC").map(PathBuf::from).unwrap_or_else(||"rustc".into()),
        cargo:env::var_os("CARGO").map(PathBuf::from).unwrap_or_else(||"cargo".into()),
        fixture:root.0.join("snippet.rs"),binary:root.0.join(if cfg!(windows){"snippet.exe"}else{"snippet"}),
        log:root.0.join("command.log"),projects:BTreeMap::new(),stats:Stats::default()
    };
    let version=command(&runner.rustc,&["--version"],&runner.root,&runner.log,20)?;
    let version_text=version.text.trim().to_owned();success(version)?;
    let mut failures=Vec::new();
    for (slug,tokens) in lessons{
        let mut previous=None;
        for (line,info,source) in tokens{
            let mut parts=info.split_whitespace();
            if parts.next()!=Some("rust"){previous=None;continue}
            let mode=parts.next().unwrap_or("");let arg=parts.next().unwrap_or("");
            if let Err(e)=runner.snippet(mode,arg,source,previous){failures.push(format!("{slug}.md:{line} {e}"))}
            previous=Some(source);
        }
    }
    let mut messages=Vec::new();
    for (name,files) in &runner.projects{
        match runner.project(name,files,&project_root){
            Ok(())=>{runner.stats.project+=files.len();messages.push(format!("Cargo 專案通過 {name}（{} 段）",files.len()))},
            Err(e)=>failures.push(format!("{name}: {e}"))
        }
    }
    if !failures.is_empty(){return Err(failures.join("\n\n"))}
    emit(out,&version_text)?;
    for value in [runner.stats.compiled,runner.stats.run,runner.stats.error,runner.stats.project]{emit(out,&value.to_string())?}
    emit(out,&messages.len().to_string())?;
    for message in messages{emit(out,&message)?;}
    Ok(())
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn wrap_and_paths(){
        assert_eq!(as_binary("let a=1;"),"fn main() {\nlet a=1;\n}\n");
        assert_eq!(as_binary("  async fn main() {}"),"  async fn main() {}");
        assert!(safe_file("src/bin/random.rs"));assert!(!safe_file("src/../main.rs"));
    }
    #[test]fn expected_output_contract(){
        assert_eq!(expected_output("// 預期輸出: a\n// 預期輸出: b\n").as_deref(),Some("a\nb\n"));
        assert_eq!(expected_output("let x = 1;"),None);
        assert!(verify_output("a\n","a\r\n").is_ok());
        assert!(verify_output("a\n","b\n").is_err());
        assert!(verify_output("a\n","a\nextra\n").is_err());
    }
    #[test]fn exact_error_codes(){
        let d=diagnostics("snippet.rs:1:2: error[E0382]: moved\nsnippet.rs:2:3: warning: ignored\nerror: aborting");
        assert_eq!(d,BTreeSet::from(["E0382".to_owned()]));
    }
}
