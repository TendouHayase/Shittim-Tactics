//! Copies skill and state sources from the leaf crates into `core`.
//!
//! `Skill` is a closed enum that `core` itself refers to, so it has to live in `core`. The
//! implementations live in `students` and `bosses`, which `core` cannot depend on without a
//! cycle. This copies them in, and generates the `define_skill!` call from what it finds.
//!
//! Sources are `students/src/**` and `bosses/src/**`. Outputs are `core/src/skills/**`,
//! `core/src/states.rs`, `core/src/boss_macros.rs` and `core/src/skill_defs.rs`; all four are
//! rewritten from scratch on every run, and none of them are in the repository.
//!
//! Skill struct names get their module as a prefix, because files reuse generic names like
//! `ExSkill` and they all land in one enum. State structs keep their names, because
//! `use crate::states::KeiState;` has to resolve identically before and after the copy; a
//! collision fails the run instead of silently merging.
//!
//! `MAX_EXTRA_STATE_SIZE` is appended to `states.rs` as a `const` block rather than a number,
//! since xtask cannot know field layout. Writing it anywhere else is pointless: these files
//! are overwritten wholesale.

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
    let bosses_src = workspace_root.join("crates/bosses/src");
    let core_src = workspace_root.join("crates/core/src");

    // states를 먼저 만든다: skills 쪽이 `crate::states::...`를 참조하기 때문.
    let state_structs = process_states(
        &[students_src.join("states.rs"), bosses_src.join("states.rs")],
        &core_src.join("states.rs"),
    )?;

    // states가 모듈 디렉터리에서 파일 하나로 바뀌기 전에 생성됐던 산출물을 치운다.
    // 남겨두면 `core/src/states.rs`의 mod 선언이 사라진 뒤에도 파일만 떠돌아 헷갈린다.
    let stale_states_dir = core_src.join("states");
    if stale_states_dir.is_dir() {
        fs::remove_dir_all(&stale_states_dir)?;
    }

    let skills_out = core_src.join("skills");
    let (mut skill_modules, mut skill_structs) = process_tree(
        &students_src.join("skills"),
        &skills_out,
        Prefixing::ByFileName,
    )?;

    // 보스 스킬이 `create_boss_skill!`을 쓰므로 매크로 정의를 먼저 복제해둔다.
    copy_boss_macros(
        &bosses_src.join("macros.rs"),
        &core_src.join("boss_macros.rs"),
    )?;

    let (boss_modules, boss_structs) = process_boss_tree(&bosses_src, &skills_out)?;

    // 학생과 보스가 같은 `core/src/skills/` 아래로 들어가므로 모듈명이 겹치면
    // 한쪽 파일이 조용히 덮어써진다. 겹치는 순간 실패시킨다.
    for module in &boss_modules {
        if skill_modules.contains(module) {
            return Err(format!(
                "module name `{module}` is used by both students/src/skills and bosses/src"
            )
            .into());
        }
    }

    skill_modules.extend(boss_modules);
    skill_structs.extend(boss_structs);
    skill_modules.sort();
    skill_structs.sort_by(|a, b| a.0.cmp(&b.0));

    write_mod_aggregator(&core_src.join("skills.rs"), &skill_modules)?;

    ensure_mods_declared(
        &core_src.join("lib.rs"),
        &["states", "skills", "boss_macros"],
    )?;
    write_skill_defs(&core_src.join("skill_defs.rs"), &skill_structs)?;

    println!(
        "gen-skills done: {} skill file(s) / {} struct(s), {} state struct(s)",
        skill_modules.len(),
        skill_structs.len(),
        state_structs.len(),
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

/// Whether top-level struct names get the file name as a prefix.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Prefixing {
    /// `ExSkill` in `kei.rs` becomes `KeiExSkill`.
    ByFileName,
    Off,
}

/// Copies every `students/src/<kind>/*.rs` into `core/src/<kind>/*.rs`.
///
/// Returns the module names and the (struct ident, module) pairs, both sorted. `fs::read_dir`
/// order varies by filesystem, and an unsorted result would reshuffle `define_skill!`'s
/// arguments on every run.
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

/// Copies every `bosses/src/<boss>/skills.rs` into `core/src/skills/<boss>.rs`.
///
/// The module name comes from the directory rather than the file name, which is always
/// `skills`. Return value and sorting match [`process_tree`].
fn process_boss_tree(
    bosses_src: &Path,
    out_dir: &Path,
) -> Result<(Vec<String>, Vec<(Ident, String)>), Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;

    let mut modules = Vec::new();
    let mut structs = Vec::new();

    for entry in fs::read_dir(bosses_src)? {
        let dir = entry?.path();
        let skills_rs = dir.join("skills.rs");
        if !dir.is_dir() || !skills_rs.is_file() {
            continue;
        }
        let module_name = dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("failed to read boss directory name")?
            .to_string();

        let (idents, source) = process_file(&skills_rs, &module_name, Prefixing::ByFileName)?;
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

/// Concatenates both `states.rs` sources into `core/src/states.rs`.
///
/// Unlike skills these are not split into per-file modules, because
/// `use crate::states::KeiState;` in a source file has to resolve the same way after the copy.
/// Duplicate names therefore fail the run instead of one quietly winning.
///
/// Returns the struct idents, sorted.
fn process_states(
    sources: &[PathBuf],
    out_file: &Path,
) -> Result<Vec<Ident>, Box<dyn std::error::Error>> {
    let mut idents: Vec<Ident> = Vec::new();
    let mut body = String::new();

    for src in sources {
        // 아직 state가 필요 없는 크레이트는 파일 자체가 없을 수 있다.
        if !src.is_file() {
            continue;
        }

        let (file_idents, source) = process_file(src, "states", Prefixing::Off)?;

        for ident in &file_idents {
            if idents.contains(ident) {
                return Err(format!(
                    "state struct `{ident}` is defined more than once; \
                     state structs from all crates share one `core::states` module, \
                     so prefix them with the owner name (e.g. `KeiState`)"
                )
                .into());
            }
        }

        idents.extend(file_idents);
        body.push_str(&source);
        body.push('\n');
    }

    // `fs::read_dir`와 달리 소스 목록은 고정이지만, struct 순서는 파일 내 작성 순서를
    // 따라가므로 상수 본문이 흔들리지 않도록 정렬한다.
    idents.sort();
    body.push_str(&max_extra_state_size_const(&idents));

    write_formatted_with_header(out_file, STATES_HEADER, &body)?;
    Ok(idents)
}

/// Copies `bosses/src/macros.rs` to `core/src/boss_macros.rs` verbatim.
///
/// No AST round trip: prettyplease prints `macro_rules!` bodies token by token and would
/// destroy their hand-tuned formatting.
fn copy_boss_macros(src: &Path, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(src)?;
    let header =
        "// === xtask gen-skills: copied from bosses/src/macros.rs, do not edit by hand ===\n\n";
    fs::write(out, format!("{header}{content}"))?;
    Ok(())
}

/// Parses one file, renames its structs if asked, rewrites `core::` paths to `crate::`, and
/// returns the struct idents along with the generated source.
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
            // 보스 스킬 struct는 `create_boss_skill!(Name, ...)`이 만들어내므로
            // `Item::Struct`로는 보이지 않는다. 호출부의 첫 인자가 struct 이름이다.
            Item::Macro(item_macro) => boss_skill_macro_name(&item_macro.mac),
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

/// Returns `Name` if this is a `create_boss_skill!(Name, ...)` invocation.
///
/// Matched on the last path segment, since it can also be called as
/// `crate::create_boss_skill!`.
fn boss_skill_macro_name(mac: &syn::Macro) -> Option<String> {
    let is_boss_skill = mac
        .path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "create_boss_skill");
    if !is_boss_skill {
        return None;
    }

    match mac.tokens.clone().into_iter().next() {
        Some(proc_macro2::TokenTree::Ident(ident)) => Some(ident.to_string()),
        _ => None,
    }
}

/// Renames struct identifiers and rewrites a leading `core::` path segment to `crate::`.
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
        if let UseTree::Path(use_path) = tree
            && use_path.ident == "core"
        {
            use_path.ident = Ident::new("crate", use_path.ident.span());
        }
        syn::visit_mut::visit_use_tree_mut(self, tree);
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if let Some(first) = path.segments.first_mut()
            && first.ident == "core"
        {
            first.ident = Ident::new("crate", first.ident.span());
        }
        syn::visit_mut::visit_path_mut(self, path);
    }

    // syn은 매크로 호출의 토큰 스트림 안으로 들어가지 않음(문법을 알 수 없으므로). 여기서는
    // `vec![SkillEffect { .. }]` 안의 리네이밍과 경로 치환이 조용히 빠지게 되므로, 본문을
    // 쉼표로 구분된 식 목록으로 다시 파싱해 훑고 되돌려씀
    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        self.visit_path_mut(&mut mac.path);
        if let Ok(mut exprs) = mac.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for expr in exprs.iter_mut() {
                self.visit_expr_mut(expr);
            }
            mac.tokens = quote::quote! { #exprs };
            return;
        }

        // Expr 목록으로 파싱되지 않는 본문도 있다. `create_boss_skill!`은 마지막 인자로
        // `SkillOps` 메서드들이 든 `{ fn apply(&self, ..) { .. } }` 블록을 받는데, 이건
        // 식이 아니라 item 목록이다. 여기서 그냥 포기하면 struct 리네이밍과 `core::` ->
        // `crate::` 치환이 조용히 건너뛰어져 생성된 코드가 컴파일되지 않는다.
        // 최선책으로 토큰을 직접 훑어 ident만 치환한다.
        mac.tokens = self.rewrite_tokens(mac.tokens.clone());
    }
}

impl RelocateVisitor {
    /// Walks a token stream and substitutes identifiers.
    ///
    /// Without a grammar every `core` is rewritten, not just leading path segments. Skill
    /// sources only use it as a crate name, but a variable or field named `core` passed to a
    /// macro would be rewritten wrongly here.
    fn rewrite_tokens(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        tokens
            .into_iter()
            .map(|tree| match tree {
                proc_macro2::TokenTree::Ident(ident) => {
                    let name = ident.to_string();
                    let replacement = match self.rename_map.get(&name) {
                        Some(renamed) => renamed.to_string(),
                        None if name == "core" => "crate".to_string(),
                        None => return proc_macro2::TokenTree::Ident(ident),
                    };
                    proc_macro2::TokenTree::Ident(Ident::new(&replacement, ident.span()))
                }
                proc_macro2::TokenTree::Group(group) => {
                    let mut rewritten = proc_macro2::Group::new(
                        group.delimiter(),
                        self.rewrite_tokens(group.stream()),
                    );
                    rewritten.set_span(group.span());
                    proc_macro2::TokenTree::Group(rewritten)
                }
                other => other,
            })
            .collect()
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

/// Builds the `MAX_EXTRA_STATE_SIZE` definition.
///
/// The size is left to const evaluation. xtask knows neither field layout nor alignment, so a
/// literal written here would be wrong. Yields 0 when there are no state structs.
fn max_extra_state_size_const(state_structs: &[Ident]) -> String {
    let mut body = String::new();
    for ident in state_structs {
        body.push_str(&format!(
            "    {{\n        \
             let size = ::std::mem::size_of::<{ident}>();\n        \
             if size > max {{\n            max = size;\n        }}\n    }}\n"
        ));
    }

    format!(
        "/// 모든 state 구조체를 담을 수 있는 `StateData::extra`의 최소 크기.\n\
         ///\n\
         /// Generated by xtask from both `states.rs` sources. Do not edit by hand.\n\
         pub const MAX_EXTRA_STATE_SIZE: usize = {{\n    \
         let mut max = 0usize;\n{body}    max\n}};\n"
    )
}

fn ensure_mods_declared(
    lib_rs: &Path,
    mod_names: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
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

const SKILL_DEFS_HEADER: &str = "// === xtask gen-skills: generated, do not edit by hand ===\n\
                                 // `core/src/skill.rs`가 `define_skill!` 정의 뒤에서 \
                                 `include!`한다.\n\n";

/// Writes the imports and the `define_skill!` call to `core/src/skill_defs.rs`.
///
/// Kept out of `skill.rs` so that a list which grows with every student does not sit in a
/// hand-written file. `skill.rs` is now untouched by xtask.
///
/// No prettyplease: `define_skill!` is a macro invocation, and an AST round trip reprints its
/// arguments as bare tokens on one line. Written as text and left to rustfmt.
fn write_skill_defs(
    path: &Path,
    skill_structs: &[(Ident, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut by_module: BTreeMap<&str, Vec<&Ident>> = BTreeMap::new();
    for (ident, module) in skill_structs {
        by_module.entry(module.as_str()).or_default().push(ident);
    }

    let mut generated = String::from(SKILL_DEFS_HEADER);
    for (module, idents) in &by_module {
        // ident는 이미 접두어가 붙은 이름이라(process_file이 정의 자체를 리네이밍) 그대로 import
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

    fs::write(path, &generated)?;
    let _ = Command::new("rustfmt").arg(path).output();
    Ok(())
}

const STATES_HEADER: &str = "// === xtask gen-skills: merged from students/src/states.rs and \
                             bosses/src/states.rs, do not edit by hand ===\n\n";

fn write_formatted(path: &Path, source: &str) -> Result<(), Box<dyn std::error::Error>> {
    write_formatted_with_header(path, "", source)
}

fn write_formatted_with_header(
    path: &Path,
    header: &str,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let formatted = match syn::parse_file(source) {
        Ok(file) => prettyplease::unparse(&file),
        Err(_) => source.to_string(),
    };
    fs::write(path, format!("{header}{formatted}"))?;

    // prettyplease가 실패했을 경우를 대비해 rustfmt로 한 번 더 정리를 시도한다 (실패해도 무시).
    let _ = Command::new("rustfmt").arg(path).output();

    Ok(())
}
