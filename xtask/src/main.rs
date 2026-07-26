//! `cargo run -p xtask`
//!
//! `core::skill::Skill`은 `define_skill!` 매크로로 생성되는 닫힌 enum이고,
//! `core`의 여러 타입(`Student::skills`, `Character::skill_list`, `Simulator` 등)이
//! 이 enum을 직접 참조하기 때문에 enum 정의는 반드시 `core` 크레이트 안에 있어야 한다.
//!
//! 반면 실제 스킬 구현체는 학생별로 `students/src/skills/*.rs`에, 그 스킬들이 쓰는
//! 상태 구조체는 `students/src/states/*.rs`에 작성한다(leaf 크레이트, `bosses`와 같은 레벨).
//! `core`는 `students`에 의존할 수 없으므로(순환 의존), 이 xtask가 두 디렉터리를 읽어
//! `core/src/skills/*.rs`, `core/src/states/*.rs`로 복제하고
//! `core/src/skill.rs` 끝에 `define_skill!(...)` 호출을 생성해 넣는다.
//!
//! `students/src/**`가 소스이고 `core/src/{skills,states}/**`는 매번 재생성되는
//! 산출물이다. `students` 쪽 파일은 건드리지 않는다.
//!
//! `core/src/states.rs`에는 mod 선언과 함께 `MAX_EXTRA_STATE_SIZE` 상수를 생성한다.
//! 이 값은 모든 state 구조체를 담을 수 있는 `StateData::extra`의 최소 크기이므로
//! state 구조체 전체를 봐야 정해진다. xtask는 레이아웃을 모르니 `size_of`를 직접
//! 계산하지 않고, 컴파일러가 const 평가로 최댓값을 구하도록 코드를 뱉는다.
//!
//! 이 상수를 `states.rs`가 아닌 곳에 손으로 적어두면 안 된다. 이 파일들은 매번
//! 통째로 덮어써지므로 손으로 넣은 정의는 다음 실행 때 사라진다.
//! `students` 쪽에서 참조할 때는 `core::states::MAX_EXTRA_STATE_SIZE`로 쓴다
//! (복제 시 `core::` -> `crate::`로 치환되어 양쪽 크레이트에서 모두 성립한다).
//!
//! skills와 states의 처리 방식 차이:
//! - skills: 모든 학생의 스킬 구조체가 하나의 `Skill` enum으로 합쳐지므로 이름이 충돌한다
//!   (파일마다 `ExSkill`, `BasicSkill` 같은 범용 이름을 쓴다). 그래서 파일명을 접두어로
//!   붙인다: `kei.rs`의 `ExSkill` -> `KeiExSkill`.
//! - states: enum으로 합쳐지지 않고 모듈별로 그대로 남으므로 충돌하지 않는다. 이름을
//!   바꾸지 않아야 `skills/kei.rs`의 `use crate::states::kei::SubSkillState;`가
//!   복제 후에도 그대로 성립한다.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use heck::ToUpperCamelCase;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Ident, Item, UseTree};

fn main() {
    if let Err(err) = run() {
        eprintln!("gen-skills failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;

    let students_src = workspace_root.join("crates/students/src");
    let core_src = workspace_root.join("crates/core/src");

    // states를 먼저 만든다: skills 쪽이 `crate::states::...`를 참조하기 때문.
    let (state_modules, state_structs) = process_tree(
        &students_src.join("states"),
        &core_src.join("states"),
        Prefixing::Off,
    )?;
    write_states_aggregator(&core_src.join("states.rs"), &state_modules, &state_structs)?;

    let (skill_modules, skill_structs) = process_tree(
        &students_src.join("skills"),
        &core_src.join("skills"),
        Prefixing::ByFileName,
    )?;
    write_mod_aggregator(&core_src.join("skills.rs"), &skill_modules)?;

    ensure_mods_declared(&core_src.join("lib.rs"), &["states", "skills"])?;
    rewrite_skill_rs(&core_src.join("skill.rs"), &skill_structs)?;

    println!(
        "gen-skills done: {} skill file(s) / {} struct(s), {} state file(s)",
        skill_modules.len(),
        skill_structs.len(),
        state_modules.len(),
    );

    Ok(())
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = Path::new(manifest_dir)
        .parent()
        .ok_or("failed to locate workspace root")?
        .to_path_buf();
    Ok(root)
}

/// 최상위 struct 이름에 파일명 접두어를 붙일지 여부.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prefixing {
    /// `kei.rs`의 `ExSkill` -> `KeiExSkill`
    ByFileName,
    /// 이름을 그대로 둔다.
    Off,
}

/// `students/src/<kind>/*.rs`를 전부 읽어 `core/src/<kind>/*.rs`로 복제한다.
///
/// 반환값은 (모듈 이름 목록, (최상위 struct ident, 모듈명) 목록)이며 둘 다 정렬되어 있다.
/// `fs::read_dir` 순서는 OS/파일시스템마다 다르므로, 정렬하지 않으면 실행할 때마다
/// `define_skill!(...)`의 인자 순서가 바뀌어 불필요한 diff가 생긴다.
fn process_tree(
    src_dir: &Path,
    out_dir: &Path,
    prefixing: Prefixing,
) -> Result<(Vec<String>, Vec<(Ident, String)>), Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;

    let mut modules = Vec::new();
    let mut structs = Vec::new();

    for entry in fs::read_dir(src_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("failed to read file stem")?
            .to_string();

        let (idents, source) = process_file(&path, &module_name, prefixing)?;
        write_formatted(&out_dir.join(format!("{module_name}.rs")), &source)?;

        for ident in idents {
            structs.push((ident, module_name.clone()));
        }
        modules.push(module_name);
    }

    modules.sort();
    structs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((modules, structs))
}

/// 파일 하나를 파싱해 struct 이름을 (필요하면) 리네이밍하고 `core::` 경로를 `crate::`로
/// 바꾼 뒤, (최상위 struct ident 목록, 생성된 소스)를 반환한다.
fn process_file(
    path: &Path,
    module_name: &str,
    prefixing: Prefixing,
) -> Result<(Vec<Ident>, String), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut file = syn::parse_file(&content).map_err(|e| format!("{}: {e}", path.display()))?;

    let struct_names: Vec<String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
            _ => None,
        })
        .collect();

    let rename_map: BTreeMap<String, Ident> = match prefixing {
        Prefixing::Off => BTreeMap::new(),
        Prefixing::ByFileName => {
            let prefix = module_name.to_upper_camel_case();
            struct_names
                .iter()
                .map(|name| {
                    let renamed =
                        Ident::new(&format!("{prefix}{name}"), proc_macro2::Span::call_site());
                    (name.clone(), renamed)
                })
                .collect()
        }
    };

    RelocateVisitor {
        rename_map: rename_map.clone(),
    }
    .visit_file_mut(&mut file);

    let idents = struct_names
        .iter()
        .map(|name| match rename_map.get(name) {
            Some(renamed) => renamed.clone(),
            None => Ident::new(name, proc_macro2::Span::call_site()),
        })
        .collect();

    Ok((idents, quote! { #file }.to_string()))
}

/// struct 식별자를 리네이밍하고, `core::` 경로 선두를 `crate::`로 바꾼다.
struct RelocateVisitor {
    rename_map: BTreeMap<String, Ident>,
}

impl VisitMut for RelocateVisitor {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if let Some(renamed) = self.rename_map.get(&ident.to_string()) {
            *ident = Ident::new(&renamed.to_string(), ident.span());
        }
    }

    fn visit_use_tree_mut(&mut self, tree: &mut UseTree) {
        if let UseTree::Path(use_path) = tree {
            if use_path.ident == "core" {
                use_path.ident = Ident::new("crate", use_path.ident.span());
            }
        }
        syn::visit_mut::visit_use_tree_mut(self, tree);
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Some(first) = path.segments.first_mut() {
            if first.ident == "core" {
                first.ident = Ident::new("crate", first.ident.span());
            }
        }
        syn::visit_mut::visit_path_mut(self, path);
    }

    // syn does not walk into a macro invocation's token stream (e.g. `vec![...]`),
    // since it can't know the callee's grammar in general. Renames/path rewrites
    // inside macro args (very common here via `vec![SkillEffect { .. }]`) would
    // otherwise be silently skipped. Best-effort: try to reparse the body as a
    // comma-separated expression list, visit that, and write it back.
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        self.visit_path_mut(&mut mac.path);
        if let Ok(mut exprs) = mac.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for expr in exprs.iter_mut() {
                self.visit_expr_mut(expr);
            }
            mac.tokens = quote::quote! { #exprs };
        }
    }
}

fn mod_declarations(module_names: &[String]) -> String {
    let mut content = String::new();
    for name in module_names {
        content.push_str(&format!("pub mod {name};\n"));
    }
    content
}

fn write_mod_aggregator(
    path: &Path,
    module_names: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, mod_declarations(module_names))?;
    Ok(())
}

/// `core/src/states.rs`를 생성한다. mod 선언에 더해 `MAX_EXTRA_STATE_SIZE`를 넣는다.
fn write_states_aggregator(
    path: &Path,
    module_names: &[String],
    state_structs: &[(Ident, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = mod_declarations(module_names);
    content.push('\n');
    content.push_str(&max_extra_state_size_const(state_structs));
    fs::write(path, content)?;
    let _ = Command::new("rustfmt").arg(path).output();
    Ok(())
}

/// `MAX_EXTRA_STATE_SIZE` 정의를 만든다.
///
/// 크기 계산은 컴파일러에게 맡긴다. xtask는 필드 레이아웃과 정렬을 알 수 없으므로
/// 여기서 숫자를 직접 박으면 틀린다. state 구조체가 하나도 없으면 0이 된다.
fn max_extra_state_size_const(state_structs: &[(Ident, String)]) -> String {
    let mut body = String::new();
    for (ident, module) in state_structs {
        body.push_str(&format!(
            "    {{\n        \
             let size = ::std::mem::size_of::<{module}::{ident}>();\n        \
             if size > max {{\n            max = size;\n        }}\n    }}\n"
        ));
    }

    format!(
        "/// 모든 state 구조체를 담을 수 있는 `StateData::extra`의 최소 크기.\n\
         ///\n\
         /// xtask가 `students/src/states/**`를 보고 생성한다. 손으로 고치지 말 것.\n\
         pub const MAX_EXTRA_STATE_SIZE: usize = {{\n    \
         let mut max = 0usize;\n{body}    max\n}};\n"
    )
}

fn ensure_mods_declared(lib_rs: &Path, mod_names: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = fs::read_to_string(lib_rs)?;
    let mut changed = false;

    for name in mod_names {
        let decl = format!("pub mod {name};");
        if content.lines().any(|l| l.trim() == decl) {
            continue;
        }
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&decl);
        content.push('\n');
        changed = true;
    }

    if changed {
        fs::write(lib_rs, content)?;
    }
    Ok(())
}

const GENERATED_MARKER: &str = "// === xtask gen-skills: generated below, do not edit by hand ===";

/// `core/src/skill.rs`는 사람이 손으로 쓴 `macro_rules! define_skill { ... }` 정의와
/// xtask가 생성하는 `use` + `define_skill!(...)` 호출부가 한 파일에 섞여 있다.
/// 이 파일 전체를 syn -> prettyplease로 왕복시키면(다른 완전 생성 파일들과 달리)
/// 사람이 formatting해둔 macro_rules! 본문까지 다시 출력되면서 스페이싱이 망가진다
/// (prettyplease는 macro 본문을 토큰 단위로만 찍어내지 Rust-aware하게 정리하지 않음).
/// 그래서 이 파일은 절대 AST로 왕복시키지 않고, 마커 주석 뒤쪽 텍스트만 잘라내고
/// 새 텍스트를 이어붙이는 방식으로 처리한다 (원본 텍스트는 바이트 단위로 그대로 유지).
fn rewrite_skill_rs(
    path: &Path,
    skill_structs: &[(Ident, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    let base = match content.find(GENERATED_MARKER) {
        Some(idx) => content[..idx].trim_end(),
        None => strip_demo_section(&content),
    };

    let mut by_module: BTreeMap<&str, Vec<&Ident>> = BTreeMap::new();
    for (ident, module) in skill_structs {
        by_module.entry(module.as_str()).or_default().push(ident);
    }

    let mut generated = String::new();
    generated.push_str(GENERATED_MARKER);
    generated.push('\n');
    for (module, idents) in &by_module {
        // idents are already the prefixed names as they appear in `crate::skills::<module>`
        // (process_file renamed the struct defs themselves), so a plain import is enough.
        let imports = idents
            .iter()
            .map(|ident| ident.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        generated.push_str(&format!("use crate::skills::{module}::{{{imports}}};\n"));
    }
    generated.push('\n');
    let all_idents = skill_structs
        .iter()
        .map(|(ident, _)| ident.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    generated.push_str(&format!("define_skill!({all_idents});\n"));

    let new_content = format!("{base}\n\n{generated}");
    fs::write(path, &new_content)?;
    let _ = Command::new("rustfmt").arg(path).output();
    Ok(())
}

/// 첫 실행 시 남아있는 `struct A {} define_skill!(A);` 데모 섹션을 제거한다.
fn strip_demo_section(content: &str) -> &str {
    if let Some(idx) = content.find("struct A {}") {
        content[..idx].trim_end()
    } else {
        content.trim_end()
    }
}

fn write_formatted(path: &Path, source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = match syn::parse_file(source) {
        Ok(file) => prettyplease::unparse(&file),
        Err(_) => source.to_string(),
    };
    fs::write(path, formatted)?;

    // prettyplease가 실패했을 경우를 대비해 rustfmt로 한 번 더 정리를 시도한다 (실패해도 무시).
    let _ = Command::new("rustfmt").arg(path).output();

    Ok(())
}
