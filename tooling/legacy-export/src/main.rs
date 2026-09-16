use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

type AppResult<T> = Result<T, AppError>;
type Row = BTreeMap<String, String>;

#[derive(Debug)]
enum AppError {
    Message(String),
    Io(io::Error),
    Json(serde_json::Error),
    ReportedFailure(i32),
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Message(msg) => write!(f, "{msg}"),
            AppError::Io(err) => write!(f, "{err}"),
            AppError::Json(err) => write!(f, "{err}"),
            AppError::ReportedFailure(code) => {
                write!(f, "command reported failure with exit code {code}")
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    query_runner: QueryRunnerConfig,
    output: Option<OutputConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutputConfig {
    default_directory: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum QueryRunnerConfig {
    ShellTemplate {
        #[serde(alias = "command_template", rename = "commandTemplate")]
        command_template: String,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    func_id: String,
    feature_code: String,
    feature_name: String,
    main_entity_code: Option<String>,
    scenario_codes: Vec<String>,
    direct_entity_codes: Vec<String>,
    plugin_refs: Vec<PluginRef>,
    indirect_plugin_refs: Vec<IndirectPluginRef>,
    warnings: Vec<String>,
    exported_at_epoch_seconds: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PluginRef {
    event_source: String,
    event_name: String,
    owner_id: String,
    owner_secondary_id: String,
    raw_ref: String,
    library: String,
    namespace: String,
    class_name: String,
    method_name: String,
    params: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct IndirectPluginRef {
    bl_id: String,
    raw_ref: String,
    library: String,
    namespace: String,
    class_name: String,
    method_name: String,
    param_group_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectResult {
    func_id: String,
    feature_name: String,
    main_entity_code: Option<String>,
    scenario_count: usize,
    block_count: usize,
    block_field_count: usize,
    button_count: usize,
    event_count: usize,
    table_codes: Vec<String>,
    plugin_ref_count: usize,
    indirect_plugin_ref_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemExportManifest {
    exported_at_epoch_seconds: u64,
    sections: BTreeMap<String, usize>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScopeFile {
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    menus: Vec<String>,
    #[serde(default)]
    include_system_assets: bool,
}

fn main() {
    if let Err(err) = run() {
        if let AppError::ReportedFailure(code) = err {
            std::process::exit(code);
        }
        let payload = json!({
            "success": false,
            "error": {
                "message": err.to_string()
            }
        });
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&payload)
                .unwrap_or_else(|_| "{\"success\":false}".to_string())
        );
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let cli = Cli::parse(env::args().collect())?;
    match cli.command {
        ParsedCommand::Help { target } => print_help(target.as_deref()),
        ParsedCommand::ExportFunction { func_id } => export_function(cli.common, &func_id),
        ParsedCommand::InspectFunction { func_id } => inspect_function(cli.common, &func_id),
        ParsedCommand::BatchExport { func_file } => batch_export(cli.common, &func_file),
        ParsedCommand::ExportMenuSubtree { menu_name } => {
            export_menu_subtree(cli.common, &menu_name)
        }
        ParsedCommand::ExportScope { scope_file } => export_scope(cli.common, &scope_file),
        ParsedCommand::ExportSystem => export_system(cli.common),
        ParsedCommand::ExportSerials => export_serials(cli.common),
        ParsedCommand::ExportWorkflows => export_workflows(cli.common),
        ParsedCommand::ExportSecurity => export_security(cli.common),
        ParsedCommand::ExportUsers => export_users(cli.common),
        ParsedCommand::ExportOrg => export_org(cli.common),
        ParsedCommand::ExportMenus => export_menus(cli.common),
        ParsedCommand::ExportLists => export_lists(cli.common),
    }
}

#[derive(Clone)]
struct CommonArgs {
    config: PathBuf,
    out_dir: PathBuf,
}

enum ParsedCommand {
    Help { target: Option<String> },
    ExportFunction { func_id: String },
    InspectFunction { func_id: String },
    BatchExport { func_file: PathBuf },
    ExportMenuSubtree { menu_name: String },
    ExportScope { scope_file: PathBuf },
    ExportSystem,
    ExportSerials,
    ExportWorkflows,
    ExportSecurity,
    ExportUsers,
    ExportOrg,
    ExportMenus,
    ExportLists,
}

struct Cli {
    common: CommonArgs,
    command: ParsedCommand,
}

impl Cli {
    fn parse(args: Vec<String>) -> AppResult<Self> {
        if args.len() < 2 {
            return Ok(Self {
                common: CommonArgs {
                    config: PathBuf::from("config.json"),
                    out_dir: PathBuf::from("exports"),
                },
                command: ParsedCommand::Help { target: None },
            });
        }

        let command_name = args[1].clone();
        if matches!(command_name.as_str(), "--help" | "-h" | "help") {
            let target = args.get(2).cloned();
            return Ok(Self {
                common: CommonArgs {
                    config: PathBuf::from("config.json"),
                    out_dir: PathBuf::from("exports"),
                },
                command: ParsedCommand::Help { target },
            });
        }
        let mut config: Option<PathBuf> = None;
        let mut func_id: Option<String> = None;
        let mut out_dir: Option<PathBuf> = None;
        let mut func_file: Option<PathBuf> = None;
        let mut menu_name: Option<String> = None;
        let mut scope_file: Option<PathBuf> = None;

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--help" | "-h" => {
                    return Ok(Self {
                        common: CommonArgs {
                            config: PathBuf::from("config.json"),
                            out_dir: PathBuf::from("exports"),
                        },
                        command: ParsedCommand::Help {
                            target: Some(command_name.clone()),
                        },
                    });
                }
                "--config" => {
                    i += 1;
                    config = args.get(i).map(PathBuf::from);
                }
                "--func-id" => {
                    i += 1;
                    func_id = args.get(i).cloned();
                }
                "--out" => {
                    i += 1;
                    out_dir = args.get(i).map(PathBuf::from);
                }
                "--func-file" => {
                    i += 1;
                    func_file = args.get(i).map(PathBuf::from);
                }
                "--menu-name" => {
                    i += 1;
                    menu_name = args.get(i).cloned();
                }
                "--scope-file" => {
                    i += 1;
                    scope_file = args.get(i).map(PathBuf::from);
                }
                flag => {
                    return Err(AppError::Message(format!("unknown argument: {flag}")));
                }
            }
            i += 1;
        }

        let common = CommonArgs {
            config: config.ok_or_else(|| AppError::Message("--config is required".to_string()))?,
            out_dir: out_dir.unwrap_or_else(|| PathBuf::from("exports")),
        };

        let command = match command_name.as_str() {
            "export-function" => ParsedCommand::ExportFunction {
                func_id: func_id
                    .ok_or_else(|| AppError::Message("--func-id is required".to_string()))?,
            },
            "inspect-function" => ParsedCommand::InspectFunction {
                func_id: func_id
                    .ok_or_else(|| AppError::Message("--func-id is required".to_string()))?,
            },
            "batch-export" => ParsedCommand::BatchExport {
                func_file: func_file
                    .ok_or_else(|| AppError::Message("--func-file is required".to_string()))?,
            },
            "export-menu-subtree" => ParsedCommand::ExportMenuSubtree {
                menu_name: menu_name
                    .ok_or_else(|| AppError::Message("--menu-name is required".to_string()))?,
            },
            "export-scope" => ParsedCommand::ExportScope {
                scope_file: scope_file
                    .ok_or_else(|| AppError::Message("--scope-file is required".to_string()))?,
            },
            "export-system" => ParsedCommand::ExportSystem,
            "export-serials" => ParsedCommand::ExportSerials,
            "export-workflows" => ParsedCommand::ExportWorkflows,
            "export-security" => ParsedCommand::ExportSecurity,
            "export-users" => ParsedCommand::ExportUsers,
            "export-org" => ParsedCommand::ExportOrg,
            "export-menus" => ParsedCommand::ExportMenus,
            "export-lists" => ParsedCommand::ExportLists,
            other => {
                return Err(AppError::Message(format!(
                    "unsupported command: {other}. Run `legacy-export --help` for usage."
                )))
            }
        };

        Ok(Self { common, command })
    }
}

fn print_help(target: Option<&str>) -> AppResult<()> {
    let text = match target.unwrap_or("root") {
        "root" => ROOT_HELP,
        "export-function" => HELP_EXPORT_FUNCTION,
        "inspect-function" => HELP_INSPECT_FUNCTION,
        "batch-export" => HELP_BATCH_EXPORT,
        "export-menu-subtree" => HELP_EXPORT_MENU_SUBTREE,
        "export-scope" => HELP_EXPORT_SCOPE,
        "export-system" => HELP_EXPORT_SYSTEM,
        "export-serials" => HELP_EXPORT_SERIALS,
        "export-workflows" => HELP_EXPORT_WORKFLOWS,
        "export-security" => HELP_EXPORT_SECURITY,
        "export-users" => HELP_EXPORT_USERS,
        "export-org" => HELP_EXPORT_ORG,
        "export-menus" => HELP_EXPORT_MENUS,
        "export-lists" => HELP_EXPORT_LISTS,
        other => {
            return Err(AppError::Message(format!(
                "unknown help topic: {other}. Run `legacy-export --help` for available commands."
            )))
        }
    };
    println!("{text}");
    Ok(())
}

const ROOT_HELP: &str = r#"legacy-export

Old-system metadata export CLI for AI-assisted migration.

Usage:
  legacy-export <command> --config <config.json> [options]
  legacy-export --help
  legacy-export <command> --help

Commands:
  export-function      Export one feature/function package by FUNCID
  inspect-function     Inspect one feature/function without writing files
  batch-export         Export features listed in a text file
  export-menu-subtree  Export all exportable functions under a menu subtree
  export-scope         Export a stable migration scope from a JSON file
  export-system        Export full metadata/config/system-data package
  export-serials       Export serial rule assets only
  export-workflows     Export workflow assets only
  export-security      Export roles/permissions assets only
  export-users         Export user assets only
  export-org           Export organization assets only
  export-menus         Export menu assets only
  export-lists         Export list assets only

Global options:
  --config <path>      Path to config.json
  --out <dir>          Output directory, default: exports or config.output.defaultDirectory

Examples:
  legacy-export export-function --config config.json --func-id SD_T_SO --out exports
  legacy-export export-menu-subtree --config config.json --menu-name "站点配送" --out exports
  legacy-export export-scope --config config.json --scope-file scope.json --out exports
  legacy-export export-system --config config.json --out exports
"#;

const HELP_EXPORT_FUNCTION: &str = r#"export-function

Usage:
  legacy-export export-function --config <config.json> --func-id <FUNCID> [--out <dir>]

Output:
  functions/<FUNCID>/
    manifest.json
    raw/
    normalized/
    handoff/
    plugin_context/
"#;

const HELP_INSPECT_FUNCTION: &str = r#"inspect-function

Usage:
  legacy-export inspect-function --config <config.json> --func-id <FUNCID>

This command prints a JSON summary and does not write export files.
"#;

const HELP_BATCH_EXPORT: &str = r#"batch-export

Usage:
  legacy-export batch-export --config <config.json> --func-file <funcs.txt> [--out <dir>]

funcs.txt format:
  One FUNCID per line.
  Empty lines and lines starting with # are ignored.
"#;

const HELP_EXPORT_MENU_SUBTREE: &str = r#"export-menu-subtree

Usage:
  legacy-export export-menu-subtree --config <config.json> --menu-name <menu> [--out <dir>]

This command resolves SYS_Function tree nodes by IDNUM / DES1 / DES2,
collects the subtree, filters directory-only nodes, and exports each function.
"#;

const HELP_EXPORT_SCOPE: &str = r#"export-scope

Usage:
  legacy-export export-scope --config <config.json> --scope-file <scope.json> [--out <dir>]

scope.json example:
{
  "features": ["SD_T_SO", "SD_T_SHP"],
  "menus": ["站点配送"],
  "includeSystemAssets": true
}

This is the recommended stable way to define a migration range.
"#;

const HELP_EXPORT_SYSTEM: &str = r#"export-system

Usage:
  legacy-export export-system --config <config.json> [--out <dir>]

Exports metadata/configuration/system-data only.
Business transactional data is intentionally excluded.
"#;

const HELP_EXPORT_SERIALS: &str = "export-serials\n\nUsage:\n  legacy-export export-serials --config <config.json> [--out <dir>]\n";
const HELP_EXPORT_WORKFLOWS: &str = "export-workflows\n\nUsage:\n  legacy-export export-workflows --config <config.json> [--out <dir>]\n";
const HELP_EXPORT_SECURITY: &str = "export-security\n\nUsage:\n  legacy-export export-security --config <config.json> [--out <dir>]\n";
const HELP_EXPORT_USERS: &str =
    "export-users\n\nUsage:\n  legacy-export export-users --config <config.json> [--out <dir>]\n";
const HELP_EXPORT_ORG: &str =
    "export-org\n\nUsage:\n  legacy-export export-org --config <config.json> [--out <dir>]\n";
const HELP_EXPORT_MENUS: &str =
    "export-menus\n\nUsage:\n  legacy-export export-menus --config <config.json> [--out <dir>]\n";
const HELP_EXPORT_LISTS: &str =
    "export-lists\n\nUsage:\n  legacy-export export-lists --config <config.json> [--out <dir>]\n";

fn export_function(common: CommonArgs, func_id: &str) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let func_root = output_root.join("functions").join(func_id);
    let raw_root = func_root.join("raw");
    let normalized_root = func_root.join("normalized");
    let handoff_root = func_root.join("handoff");
    let plugin_context_root = func_root.join("plugin_context");
    fs::create_dir_all(&raw_root)?;
    fs::create_dir_all(&normalized_root)?;
    fs::create_dir_all(&handoff_root)?;
    fs::create_dir_all(&plugin_context_root)?;

    let runner = QueryRunner::new(config.query_runner);
    let legacy = export_legacy_function(&runner, func_id)?;
    write_raw_exports(&raw_root, &legacy)?;

    let normalized = normalize(&legacy);
    write_json_file(&func_root.join("manifest.json"), &normalized.manifest)?;
    write_json_file(&normalized_root.join("feature.json"), &normalized.feature)?;
    write_json_file(
        &normalized_root.join("scenarios.json"),
        &normalized.scenarios,
    )?;
    write_json_file(&normalized_root.join("entities.json"), &normalized.entities)?;
    write_json_file(
        &normalized_root.join("feature_fields.json"),
        &normalized.feature_fields,
    )?;
    write_json_file(
        &normalized_root.join("scenario_fields.json"),
        &normalized.scenario_fields,
    )?;
    write_json_file(&normalized_root.join("actions.json"), &normalized.actions)?;
    write_json_file(
        &normalized_root.join("plugin_refs.json"),
        &normalized.plugin_refs,
    )?;
    write_json_file(
        &normalized_root.join("indirect_plugin_refs.json"),
        &normalized.indirect_plugin_refs,
    )?;

    write_json_file(
        &handoff_root.join("create_feature_seed.json"),
        &normalized.create_feature_seed,
    )?;
    write_json_file(
        &handoff_root.join("scenario_update_seed.json"),
        &normalized.scenario_update_seed,
    )?;
    write_json_file(
        &handoff_root.join("scenario_update_seeds.json"),
        &normalized.scenario_update_seeds,
    )?;
    write_json_file(
        &handoff_root.join("plugin_analysis_seed.json"),
        &normalized.plugin_analysis_seed,
    )?;
    fs::write(handoff_root.join("summary.md"), normalized.summary_md)?;
    write_json_file(
        &plugin_context_root.join("call_context.json"),
        &normalized.plugin_call_context,
    )?;
    write_json_file(
        &plugin_context_root.join("entity_field_catalog.json"),
        &normalized.entity_field_catalog,
    )?;
    write_json_file(
        &plugin_context_root.join("related_table_catalog.json"),
        &normalized.related_table_catalog,
    )?;
    write_json_file(
        &plugin_context_root.join("plugin_bindings.json"),
        &normalized.plugin_bindings,
    )?;
    write_json_file(
        &plugin_context_root.join("sql_dependencies.json"),
        &normalized.sql_dependencies,
    )?;
    write_json_file(
        &plugin_context_root.join("plugin_rewrite_handoff.json"),
        &normalized.plugin_rewrite_handoff,
    )?;
    write_json_file(
        &func_root.join("migration_readiness.json"),
        &normalized.migration_readiness,
    )?;

    let payload = json!({
        "success": true,
        "command": "legacy-export.export-function",
        "data": {
            "funcId": func_id,
            "outputDirectory": func_root
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn inspect_function(common: CommonArgs, func_id: &str) -> AppResult<()> {
    let (config, _) = load_config_and_output_root(&common)?;
    let runner = QueryRunner::new(config.query_runner);
    let legacy = export_legacy_function(&runner, func_id)?;
    let function = legacy.function.first().cloned().unwrap_or_default();
    let plugin_refs = collect_plugin_refs(&legacy.events);
    let indirect_plugin_refs = collect_indirect_plugin_refs(&legacy.bl_defs);
    let result = InspectResult {
        func_id: func_id.to_string(),
        feature_name: preferred_name(&function),
        main_entity_code: infer_main_entity_code(&legacy, &function),
        scenario_count: legacy.groups.len(),
        block_count: legacy.blocks.len(),
        block_field_count: legacy.block_fields.len(),
        button_count: legacy.buttons.len(),
        event_count: legacy.events.len(),
        table_codes: legacy.tables.keys().cloned().collect(),
        plugin_ref_count: plugin_refs.len(),
        indirect_plugin_ref_count: indirect_plugin_refs.len(),
    };
    let payload = json!({
        "success": true,
        "command": "legacy-export.inspect-function",
        "data": result
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn batch_export(common: CommonArgs, func_file: &Path) -> AppResult<()> {
    let func_ids: Vec<String> = fs::read_to_string(func_file)?
        .lines()
        .map(strip_utf8_bom)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();

    let mut results = Vec::new();
    let mut failed_count = 0usize;
    for func_id in &func_ids {
        match export_function(common.clone(), func_id) {
            Ok(()) => results.push(json!({"funcId": func_id, "success": true})),
            Err(err) => {
                failed_count += 1;
                results
                    .push(json!({"funcId": func_id, "success": false, "error": err.to_string()}));
            }
        }
    }

    let success = failed_count == 0;
    let payload = json!({
        "success": success,
        "command": "legacy-export.batch-export",
        "data": {
            "count": func_ids.len(),
            "failedCount": failed_count,
            "results": results
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    if success {
        Ok(())
    } else {
        Err(AppError::ReportedFailure(1))
    }
}

fn export_menu_subtree(common: CommonArgs, menu_name: &str) -> AppResult<()> {
    let (config, _) = load_config_and_output_root(&common)?;
    let runner = QueryRunner::new(config.query_runner);
    let all_functions = runner.query("select * from SYS_Function order by PARENT,DSPIDX,IDNUM")?;
    let func_ids = resolve_menu_subtree_function_ids(&all_functions, menu_name)?;

    let mut results = Vec::new();
    for func_id in &func_ids {
        match export_function(common.clone(), func_id) {
            Ok(()) => results.push(json!({"funcId": func_id, "success": true})),
            Err(err) => {
                results.push(json!({"funcId": func_id, "success": false, "error": err.to_string()}))
            }
        }
    }

    let payload = json!({
        "success": true,
        "command": "legacy-export.export-menu-subtree",
        "data": {
            "menuName": menu_name,
            "count": func_ids.len(),
            "functionIds": func_ids,
            "results": results
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn export_scope(common: CommonArgs, scope_file: &Path) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let scope_text = fs::read_to_string(scope_file)?;
    let scope: ScopeFile = serde_json::from_str(strip_utf8_bom(&scope_text))?;
    let runner = QueryRunner::new(config.query_runner);
    let all_functions = runner.query("select * from SYS_Function order by PARENT,DSPIDX,IDNUM")?;

    let mut func_ids = BTreeSet::new();
    for func_id in scope.features {
        if !func_id.trim().is_empty() {
            func_ids.insert(func_id.trim().to_string());
        }
    }
    for menu_name in &scope.menus {
        for func_id in resolve_menu_subtree_function_ids(&all_functions, menu_name)? {
            func_ids.insert(func_id);
        }
    }

    let mut results = Vec::new();
    for func_id in &func_ids {
        match export_function(common.clone(), func_id) {
            Ok(()) => results.push(json!({"funcId": func_id, "success": true})),
            Err(err) => {
                results.push(json!({"funcId": func_id, "success": false, "error": err.to_string()}))
            }
        }
    }

    let mut system_result = Value::Null;
    if scope.include_system_assets {
        let system_export = perform_export_system(&common)?;
        system_result = json!({
            "success": true,
            "outputDirectory": system_export.output_directory,
            "manifest": system_export.manifest,
            "migrationReadiness": system_export.migration_readiness
        });
    }

    let scope_root = output_root.join("scopes");
    fs::create_dir_all(&scope_root)?;
    let scope_manifest = json!({
        "features": func_ids.iter().cloned().collect::<Vec<String>>(),
        "menus": scope.menus,
        "includeSystemAssets": scope.include_system_assets,
        "count": func_ids.len(),
        "results": results,
        "systemExport": system_result
    });
    write_json_file(scope_root.join("last_scope_manifest.json"), &scope_manifest)?;

    let payload = json!({
        "success": true,
        "command": "legacy-export.export-scope",
        "data": scope_manifest
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn resolve_menu_subtree_function_ids(
    all_functions: &[Row],
    menu_name: &str,
) -> AppResult<Vec<String>> {
    let target = menu_name.trim();
    let matched_roots: Vec<&Row> = all_functions
        .iter()
        .filter(|row| {
            let id = row_string(row, "IDNUM");
            let des1 = row_string(row, "DES1");
            let des2 = row_string(row, "DES2");
            id == target || des1 == target || des2 == target
        })
        .collect();

    if matched_roots.is_empty() {
        return Err(AppError::Message(format!("menu not found: {target}")));
    }

    let mut children_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut row_map: BTreeMap<String, &Row> = BTreeMap::new();
    for row in all_functions {
        let id = row_string(row, "IDNUM");
        let parent = row_string(row, "PARENT");
        row_map.insert(id.clone(), row);
        children_map.entry(parent).or_default().push(id);
    }

    let mut collected = Vec::new();
    let mut visited = BTreeSet::new();
    for root in matched_roots {
        let root_id = row_string(root, "IDNUM");
        collect_subtree_ids(&children_map, &root_id, &mut visited, &mut collected);
    }

    let exportable: Vec<String> = collected
        .into_iter()
        .filter(|func_id| {
            row_map
                .get(func_id)
                .map(|row| is_exportable_function(row, &children_map))
                .unwrap_or(false)
        })
        .collect();

    if exportable.is_empty() {
        return Err(AppError::Message(format!(
            "menu subtree resolved, but no exportable functions found under: {target}"
        )));
    }

    Ok(exportable)
}

fn collect_subtree_ids(
    children_map: &BTreeMap<String, Vec<String>>,
    root_id: &str,
    visited: &mut BTreeSet<String>,
    collected: &mut Vec<String>,
) {
    if !visited.insert(root_id.to_string()) {
        return;
    }
    collected.push(root_id.to_string());
    if let Some(children) = children_map.get(root_id) {
        for child in children {
            collect_subtree_ids(children_map, child, visited, collected);
        }
    }
}

fn is_exportable_function(row: &Row, children_map: &BTreeMap<String, Vec<String>>) -> bool {
    let id = row_string(row, "IDNUM");
    let has_children = children_map
        .get(&id)
        .map(|items| !items.is_empty())
        .unwrap_or(false);
    let has_data_binding = !row_string(row, "FUNCOBJ").is_empty();
    let fun_type = row_string(row, "FUNCTYPE");
    let status = row_string(row, "STATUS");

    if status == "0" {
        return false;
    }

    if has_data_binding {
        return true;
    }

    if !has_children && !fun_type.is_empty() {
        return true;
    }

    false
}

fn export_system(common: CommonArgs) -> AppResult<()> {
    let system_export = perform_export_system(&common)?;
    let payload = json!({
        "success": true,
        "command": "legacy-export.export-system",
        "data": {
            "outputDirectory": system_export.output_directory,
            "manifest": system_export.manifest,
            "migrationReadiness": system_export.migration_readiness
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

struct SystemExportResult {
    output_directory: PathBuf,
    manifest: SystemExportManifest,
    migration_readiness: Value,
}

fn perform_export_system(common: &CommonArgs) -> AppResult<SystemExportResult> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("system");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);

    let serials = export_serial_assets(&runner, &root.join("serials"))?;
    let workflows = export_workflow_assets(&runner, &root.join("workflows"))?;
    let security = export_security_assets(&runner, &root.join("security"))?;
    let users = export_user_assets(&runner, &root.join("users"))?;
    let org = export_org_assets(&runner, &root.join("org"))?;
    let menus = export_menu_assets(&runner, &root.join("menus"))?;
    let lists = export_list_assets(&runner, &root.join("lists"))?;
    let metadata = export_metadata_assets(&runner, &root.join("metadata"))?;
    let data_catalog = export_data_catalog_assets(&runner, &root.join("data_catalog"))?;
    let readiness = build_system_migration_readiness(&root)?;

    let sections = BTreeMap::from([
        ("serials".to_string(), serials),
        ("workflows".to_string(), workflows),
        ("security".to_string(), security),
        ("users".to_string(), users),
        ("org".to_string(), org),
        ("menus".to_string(), menus),
        ("lists".to_string(), lists),
        ("metadata".to_string(), metadata),
        ("data_catalog".to_string(), data_catalog),
    ]);

    let manifest = SystemExportManifest {
        exported_at_epoch_seconds: now_epoch_seconds(),
        sections,
        notes: vec![
            "This export contains metadata/configuration/system data only.".to_string(),
            "Business transactional data is intentionally excluded.".to_string(),
        ],
    };
    write_json_file(root.join("manifest.json"), &manifest)?;
    write_json_file(root.join("migration_readiness.json"), &readiness)?;
    Ok(SystemExportResult {
        output_directory: root,
        manifest,
        migration_readiness: readiness,
    })
}

fn export_serials(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("serials");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_serial_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-serials", root, count)
}

fn export_workflows(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("workflows");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_workflow_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-workflows", root, count)
}

fn export_security(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("security");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_security_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-security", root, count)
}

fn export_users(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("users");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_user_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-users", root, count)
}

fn export_org(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("org");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_org_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-org", root, count)
}

fn export_menus(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("menus");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_menu_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-menus", root, count)
}

fn export_lists(common: CommonArgs) -> AppResult<()> {
    let (config, output_root) = load_config_and_output_root(&common)?;
    let root = output_root.join("lists");
    fs::create_dir_all(&root)?;
    let runner = QueryRunner::new(config.query_runner);
    let count = export_list_assets(&runner, &root)?;
    print_simple_export_result("legacy-export.export-lists", root, count)
}

fn print_simple_export_result(command: &str, root: PathBuf, count: usize) -> AppResult<()> {
    let payload = json!({
        "success": true,
        "command": command,
        "data": {
            "outputDirectory": root,
            "count": count
        }
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn load_config_and_output_root(common: &CommonArgs) -> AppResult<(Config, PathBuf)> {
    let config_text = fs::read_to_string(&common.config)?;
    let config: Config = serde_json::from_str(strip_utf8_bom(&config_text))?;
    let output_root = if common.out_dir == PathBuf::from("exports") {
        config
            .output
            .as_ref()
            .and_then(|o| o.default_directory.as_ref())
            .map(PathBuf::from)
            .unwrap_or_else(|| common.out_dir.clone())
    } else {
        common.out_dir.clone()
    };
    Ok((config, output_root))
}

struct QueryRunner {
    config: QueryRunnerConfig,
}

impl QueryRunner {
    fn new(config: QueryRunnerConfig) -> Self {
        Self { config }
    }

    fn query(&self, sql: &str) -> AppResult<Vec<Row>> {
        match &self.config {
            QueryRunnerConfig::ShellTemplate { command_template } => {
                let temp_path = write_temp_sql(sql)?;
                let output = run_shell_template(command_template, &temp_path)?;
                let _ = fs::remove_file(&temp_path);

                if !output.status.success() {
                    return Err(AppError::Message(format!(
                        "query command failed: {}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    )));
                }

                let stdout = decode_query_output(&output.stdout)?;
                parse_tsv(&stdout)
            }
        }
    }

    fn query_optional(&self, sql: &str) -> AppResult<Vec<Row>> {
        match self.query(sql) {
            Ok(rows) => Ok(rows),
            Err(_) => Ok(Vec::new()),
        }
    }
}

fn write_temp_sql(sql: &str) -> AppResult<PathBuf> {
    let mut path = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Message("system time error".to_string()))?
        .as_millis();
    path.push(format!("legacy_export_{stamp}.sql"));
    fs::write(&path, format!("SET NOCOUNT ON;\n{sql}\n"))?;
    Ok(path)
}

fn render_shell_template(command_template: &str, sql_file: &Path) -> String {
    let escaped_path = shell_escape_path(sql_file);
    let raw_path = sql_file.to_string_lossy();
    command_template
        .replace("\"{sql_file}\"", &escaped_path)
        .replace("'{sql_file}'", &escaped_path)
        .replace("{sql_file}", &escaped_path)
        .replace("{sql_file_raw}", raw_path.as_ref())
}

fn parse_shell_template_args(
    command_template: &str,
    sql_file: &Path,
) -> io::Result<(String, Vec<String>)> {
    let mut parts = shell_words::split(command_template)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?;
    if parts.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "query command template is empty",
        ));
    }

    let raw_path = sql_file.to_string_lossy();
    for part in &mut parts {
        *part = part
            .replace("{sql_file}", raw_path.as_ref())
            .replace("{sql_file_raw}", raw_path.as_ref());
    }

    let program = parts.remove(0);
    Ok((program, parts))
}

fn run_shell_template(command_template: &str, sql_file: &Path) -> io::Result<Output> {
    if cfg!(target_os = "windows") {
        // Passing a complete command line as one `cmd /C` argument makes Rust and
        // cmd.exe quote it twice. In particular, sqlcmd then receives literal
        // escaped quotes around `-i`, such as \"C:\\...\\query.sql\". Parse the
        // configured template once and let Command encode each argument instead.
        let (program, args) = parse_shell_template_args(command_template, sql_file)?;
        Command::new(program).args(args).output()
    } else {
        let command_line = render_shell_template(command_template, sql_file);
        run_shell_command(&command_line)
    }
}

fn shell_escape_path(path: &Path) -> String {
    if cfg!(target_os = "windows") {
        let raw = path.to_string_lossy().replace('"', "\\\"");
        format!("\"{raw}\"")
    } else {
        let raw = path.to_string_lossy().replace('\'', "'\"'\"'");
        format!("'{raw}'")
    }
}

fn run_shell_command(command_line: &str) -> io::Result<Output> {
    if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(command_line).output()
    } else {
        Command::new("/bin/zsh")
            .arg("-lc")
            .arg(command_line)
            .output()
    }
}

fn decode_query_output(bytes: &[u8]) -> AppResult<String> {
    if bytes.starts_with(&[0xff, 0xfe]) {
        return decode_utf16_query_output(&bytes[2..], true);
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        return decode_utf16_query_output(&bytes[2..], false);
    }
    if bytes.len() >= 4 && bytes.len() % 2 == 0 {
        let even_zeroes = bytes.iter().step_by(2).filter(|byte| **byte == 0).count();
        let odd_zeroes = bytes
            .iter()
            .skip(1)
            .step_by(2)
            .filter(|byte| **byte == 0)
            .count();
        if odd_zeroes > bytes.len() / 8 || even_zeroes > bytes.len() / 8 {
            return decode_utf16_query_output(bytes, odd_zeroes >= even_zeroes);
        }
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn decode_utf16_query_output(bytes: &[u8], little_endian: bool) -> AppResult<String> {
    let units = bytes
        .chunks_exact(2)
        .map(|pair| {
            if little_endian {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        })
        .collect::<Vec<_>>();
    String::from_utf16(&units)
        .map_err(|error| AppError::Message(format!("invalid UTF-16 query output: {error}")))
}

fn parse_tsv(text: &str) -> AppResult<Vec<Row>> {
    let all_lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let (header_index, header_line, delimiter) = match find_header_line(&all_lines) {
        Some(value) => value,
        None => return Ok(Vec::new()),
    };
    let headers: Vec<String> = header_line
        .split(delimiter)
        .map(normalize_column_name)
        .collect();
    let expected_delimiters = headers.len().saturating_sub(1);
    let mut logical_lines = Vec::new();
    let mut current_line = String::new();
    for (index, line) in all_lines.iter().enumerate().skip(header_index + 1) {
        if index == header_index + 1 && is_separator_line(line, delimiter) {
            continue;
        }
        let delimiter_count = line.matches(delimiter).count();
        if delimiter_count >= expected_delimiters && !current_line.is_empty() {
            logical_lines.push(std::mem::take(&mut current_line));
        }
        if !current_line.is_empty() {
            current_line.push('\n');
        }
        current_line.push_str(line);
    }
    if !current_line.is_empty() {
        logical_lines.push(current_line);
    }

    let mut rows = Vec::new();
    for line in logical_lines {
        let values: Vec<&str> = line.splitn(headers.len(), delimiter).collect();
        let mut row = Row::new();
        for (column_index, header) in headers.iter().enumerate() {
            row.insert(
                header.clone(),
                values
                    .get(column_index)
                    .copied()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            );
        }
        rows.push(row);
    }
    Ok(rows)
}

fn find_header_line<'a>(lines: &'a [&'a str]) -> Option<(usize, &'a str, &'static str)> {
    for (index, line) in lines.iter().enumerate() {
        let delimiter = detect_column_delimiter(line).unwrap_or("\t");
        if lines
            .get(index + 1)
            .is_some_and(|next| is_separator_line(next, delimiter))
        {
            return Some((index, *line, delimiter));
        }
    }
    lines
        .first()
        .map(|line| (0, *line, detect_column_delimiter(line).unwrap_or("\t")))
}

fn detect_column_delimiter(header_line: &str) -> Option<&'static str> {
    if header_line.contains("\\t") {
        Some("\\t")
    } else if header_line.contains('\t') {
        Some("\t")
    } else if header_line.contains('|') {
        Some("|")
    } else if header_line.contains('\\') {
        Some("\\")
    } else {
        None
    }
}

fn normalize_column_name(value: &str) -> String {
    value
        .replace('\0', "")
        .trim()
        .trim_start_matches(['\u{feff}', '\u{fffd}'])
        .to_ascii_uppercase()
}

fn is_separator_line(line: &str, delimiter: &str) -> bool {
    line.replace(delimiter, "")
        .chars()
        .all(|ch| ch == '-' || ch == ' ' || ch == '\r')
}

#[derive(Debug)]
struct LegacyFunctionExport {
    function: Vec<Row>,
    function_ext: Vec<Row>,
    groups: Vec<Row>,
    blocks: Vec<Row>,
    events: Vec<Row>,
    regions: Vec<Row>,
    block_fields: Vec<Row>,
    buttons: Vec<Row>,
    block_field_overrides: Vec<Row>,
    tables: BTreeMap<String, LegacyTableExport>,
    bl_defs: Vec<Row>,
    bl_params: Vec<Row>,
}

#[derive(Debug)]
struct LegacyTableExport {
    table: Vec<Row>,
    fields: Vec<Row>,
    indexes: Vec<Row>,
    foreign_keys: Vec<Row>,
    ext: Vec<Row>,
}

fn export_legacy_function(runner: &QueryRunner, func_id: &str) -> AppResult<LegacyFunctionExport> {
    let function = runner.query(&format!(
        "select * from SYS_Function where IDNUM='{}'",
        escape_sql(func_id)
    ))?;
    if function.is_empty() {
        return Err(AppError::Message(format!("function not found: {func_id}")));
    }
    if row_string(&function[0], "IDNUM").is_empty() {
        let columns = function[0].keys().cloned().collect::<Vec<_>>().join(", ");
        return Err(AppError::Message(format!(
            "query output for {func_id} did not expose an IDNUM column (parsed columns: {columns}). Check sqlcmd encoding/separator and rebuild legacy-export from current source"
        )));
    }
    let function_ext = runner.query(&format!(
        "select * from SYS_EXTPROP where PROPTYPE='FUNCTION' and ID1='{}'",
        escape_sql(func_id)
    ))?;
    let groups = runner.query(&format!(
        "select * from SYS_Group where FUNCID='{}' order by DSPIDX",
        escape_sql(func_id)
    ))?;
    let blocks = runner.query(&format!(
        "select * from V_SYS_GroupBlock where FUNCID='{}' order by DSPIDX",
        escape_sql(func_id)
    ))?;
    let events = runner.query(&format!(
        "select * from SYS_EVENT where EVTOBJID1='{}'",
        escape_sql(func_id)
    ))?;
    let regions = runner.query(&format!(
        "select * from SYS_REGION where FUNCID='{}' order by DSPIDX",
        escape_sql(func_id)
    ))?;
    let block_fields = runner.query(&format!(
        "select * from V_SYS_GroupBlockField where FUNCID='{}'",
        escape_sql(func_id)
    ))?;
    let buttons = runner.query(&format!(
        "select * from V_SYS_GroupButton where FUNCID='{}' order by DSPIDX",
        escape_sql(func_id)
    ))?;
    let block_field_overrides = runner.query(&format!(
        "select * from SYS_BlockFieldOver where FUNCID='{}'",
        escape_sql(func_id)
    ))?;

    let mut table_codes = BTreeSet::new();
    if let Some(row) = function.first() {
        let code = row_first_string(row, &["FUNCOBJ", "MAINENTITY", "TABLEID", "DATATABLE"]);
        if !code.is_empty() {
            table_codes.insert(code);
        }
    }
    for row in &blocks {
        let code = legacy_table_code(row);
        if !code.is_empty() {
            table_codes.insert(code);
        }
    }

    let mut tables = BTreeMap::new();
    for table_code in table_codes {
        let table = runner.query(&format!(
            "select * from SYS_Table where IDNUM='{}'",
            escape_sql(&table_code)
        ))?;
        let fields = runner.query(&format!(
            "select * from SYS_TableField where IDNUM='{}' order by DSPIDX",
            escape_sql(&table_code)
        ))?;
        let indexes = runner.query(&format!(
            "select * from SYS_TableIndex where IDNUM='{}'",
            escape_sql(&table_code)
        ))?;
        let foreign_keys = runner.query(&format!(
            "select * from SYS_TableFK where IDNUM='{}'",
            escape_sql(&table_code)
        ))?;
        let ext = runner.query(&format!(
            "select * from SYS_EXTPROP where PROPTYPE in ('TABLE','TABLEFIELD') and ID1='{}'",
            escape_sql(&table_code)
        ))?;
        tables.insert(
            table_code,
            LegacyTableExport {
                table,
                fields,
                indexes,
                foreign_keys,
                ext,
            },
        );
    }

    let mut bl_ids = BTreeSet::new();
    for row in &events {
        if let Some(evt_std) = row.get("EVTSTD") {
            if let Some(rest) = evt_std.strip_prefix("@BL.") {
                if !rest.trim().is_empty() {
                    bl_ids.insert(rest.trim().to_string());
                }
            }
        }
    }

    let mut bl_defs = Vec::new();
    let mut bl_params = Vec::new();
    for bl_id in bl_ids {
        let defs = runner.query(&format!(
            "select * from SYS_BL where IDNUM='{}'",
            escape_sql(&bl_id)
        ))?;
        let pgid = defs
            .first()
            .and_then(|row| row.get("PGID"))
            .cloned()
            .unwrap_or_default();
        if !pgid.trim().is_empty() {
            let params = runner.query(&format!(
                "select * from SYS_BLP where PGID='{}'",
                escape_sql(&pgid)
            ))?;
            bl_params.extend(params);
        }
        bl_defs.extend(defs);
    }

    Ok(LegacyFunctionExport {
        function,
        function_ext,
        groups,
        blocks,
        events,
        regions,
        block_fields,
        buttons,
        block_field_overrides,
        tables,
        bl_defs,
        bl_params,
    })
}

fn write_raw_exports(root: &Path, legacy: &LegacyFunctionExport) -> AppResult<()> {
    write_json_file(&root.join("function.json"), &legacy.function)?;
    write_json_file(&root.join("function_ext.json"), &legacy.function_ext)?;
    write_json_file(&root.join("groups.json"), &legacy.groups)?;
    write_json_file(&root.join("blocks.json"), &legacy.blocks)?;
    write_json_file(&root.join("events.json"), &legacy.events)?;
    write_json_file(&root.join("regions.json"), &legacy.regions)?;
    write_json_file(&root.join("block_fields.json"), &legacy.block_fields)?;
    write_json_file(&root.join("buttons.json"), &legacy.buttons)?;
    write_json_file(
        &root.join("block_field_overrides.json"),
        &legacy.block_field_overrides,
    )?;
    write_json_file(&root.join("bl_defs.json"), &legacy.bl_defs)?;
    write_json_file(&root.join("bl_params.json"), &legacy.bl_params)?;

    let tables_root = root.join("tables");
    fs::create_dir_all(&tables_root)?;
    for (table_code, export) in &legacy.tables {
        let table_dir = tables_root.join(table_code);
        fs::create_dir_all(&table_dir)?;
        write_json_file(&table_dir.join("table.json"), &export.table)?;
        write_json_file(&table_dir.join("fields.json"), &export.fields)?;
        write_json_file(&table_dir.join("indexes.json"), &export.indexes)?;
        write_json_file(&table_dir.join("foreign_keys.json"), &export.foreign_keys)?;
        write_json_file(&table_dir.join("ext.json"), &export.ext)?;
    }
    Ok(())
}

fn export_serial_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let serials = runner.query("select * from SYS_ID order by IDNUM")?;
    let seeds = runner.query("select * from SYS_IDSEED order by IDNUM")?;
    let related_tables = runner.query("select * from SYS_Table where IDSER is not null and ltrim(rtrim(IDSER))<>'' order by IDNUM")?;
    write_json_file(root.join("sys_id.json"), &serials)?;
    write_json_file(root.join("sys_idseed.json"), &seeds)?;
    write_json_file(root.join("serial_related_tables.json"), &related_tables)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "serialRules": serials,
            "serialSeeds": seeds,
            "entityBindings": related_tables
                .iter()
                .map(|row| json!({
                    "entityCode": row_string(row, "IDNUM"),
                    "serialRuleCode": row_string(row, "IDSER"),
                    "idField": row_string(row, "IDFIELD")
                }))
                .collect::<Vec<Value>>()
        }),
    )?;
    Ok(serials.len() + seeds.len() + related_tables.len())
}

fn export_workflow_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let wf = runner.query("select * from SYS_WF order by IDNUM")?;
    let wf_stage = runner.query("select * from SYS_WFSTAGE order by IDNUM,DSPIDX")?;
    let wf_link = runner.query("select * from SYS_WFLINK order by IDNUM,STAGEID")?;
    let wf_act = runner.query("select * from SYS_WFACT order by IDNUM,DSPIDX")?;
    let wf_filters = runner.query_optional("select * from SYS_WFAGFILTER order by FUNCID")?;
    let scripts =
        runner.query("select * from SYS_SCRIPT where GROUPID='SYS_WF' order by SCRIPTID")?;
    let script_details = runner
        .query("select * from SYS_SCRIPTD where GROUPID='SYS_WF' order by SCRIPTID,EXESEQ")?;
    write_json_file(root.join("sys_wf.json"), &wf)?;
    write_json_file(root.join("sys_wfstage.json"), &wf_stage)?;
    write_json_file(root.join("sys_wflink.json"), &wf_link)?;
    write_json_file(root.join("sys_wfact.json"), &wf_act)?;
    write_json_file(root.join("sys_wfagfilter.json"), &wf_filters)?;
    write_json_file(root.join("sys_script_wf.json"), &scripts)?;
    write_json_file(root.join("sys_scriptd_wf.json"), &script_details)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "workflowDefinitions": wf,
            "stages": wf_stage,
            "links": wf_link,
            "actions": wf_act,
            "filters": wf_filters,
            "scripts": scripts,
            "scriptDetails": script_details
        }),
    )?;
    Ok(wf.len()
        + wf_stage.len()
        + wf_link.len()
        + wf_act.len()
        + wf_filters.len()
        + scripts.len()
        + script_details.len())
}

fn export_security_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let roles = runner.query("select * from SYS_Role order by IDNUM")?;
    let role_users = runner.query("select * from SYS_RoleUser order by ROLEID,USERID")?;
    let role_functions = runner.query("select * from SYS_RoleFunction order by ROLEID,FUNCID")?;
    let role_groups = runner.query("select * from SYS_RoleGroup order by ROLEID,FUNCID,GRPID")?;
    let role_folders =
        runner.query_optional("select * from SYS_RoleFolder order by ROLEID,FOLDERID")?;
    write_json_file(root.join("sys_role.json"), &roles)?;
    write_json_file(root.join("sys_roleuser.json"), &role_users)?;
    write_json_file(root.join("sys_rolefunction.json"), &role_functions)?;
    write_json_file(root.join("sys_rolegroup.json"), &role_groups)?;
    write_json_file(root.join("sys_rolefolder.json"), &role_folders)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "roles": roles,
            "roleUsers": role_users,
            "roleFunctions": role_functions,
            "roleGroups": role_groups,
            "roleFolders": role_folders
        }),
    )?;
    Ok(roles.len()
        + role_users.len()
        + role_functions.len()
        + role_groups.len()
        + role_folders.len())
}

fn export_user_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let users = runner.query("select * from SYS_USER order by IDNUM")?;
    let user_default_temp =
        runner.query_optional("select * from SYS_USERDEFAULTTEMP order by USERID,FUNCID")?;
    let user_view_temp =
        runner.query_optional("select * from SYS_USERVIEWTEMP order by USERID,FUNCID,TEMPID")?;
    let user_dash =
        runner.query_optional("select * from SYS_USERDASH order by USERID,FUNCID,GRPID")?;
    write_json_file(root.join("sys_user.json"), &users)?;
    write_json_file(root.join("sys_userdefaulttemp.json"), &user_default_temp)?;
    write_json_file(root.join("sys_userviewtemp.json"), &user_view_temp)?;
    write_json_file(root.join("sys_userdash.json"), &user_dash)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "users": users,
            "userDefaultTemplates": user_default_temp,
            "userViewTemplates": user_view_temp,
            "userDashboards": user_dash
        }),
    )?;
    Ok(users.len() + user_default_temp.len() + user_view_temp.len() + user_dash.len())
}

fn export_org_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let dept = runner.query_optional("select * from BD_ORG_DEPT order by DEPTID")?;
    let emp = runner.query_optional("select * from BD_ORG_EMP order by EMPID")?;
    let org_user_view = runner.query_optional("select * from DV_BD_ORG_USR order by USERID")?;
    let org_if_view = runner.query_optional("select * from DV_BD_GO_IF order by IFID")?;
    write_json_file(root.join("bd_org_dept.json"), &dept)?;
    write_json_file(root.join("bd_org_emp.json"), &emp)?;
    write_json_file(root.join("dv_bd_org_usr.json"), &org_user_view)?;
    write_json_file(root.join("dv_bd_go_if.json"), &org_if_view)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "departments": dept,
            "employees": emp,
            "orgUserView": org_user_view,
            "orgHierarchyView": org_if_view
        }),
    )?;
    Ok(dept.len() + emp.len() + org_user_view.len() + org_if_view.len())
}

fn export_menu_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let functions = runner.query("select * from SYS_Function order by PARENT,DSPIDX,IDNUM")?;
    let user_menu = runner.query_optional("select * from V_SYS_USERMENU order by USERID,DSPIDX")?;
    let web_user_menu =
        runner.query_optional("select * from WV_SYS_USERMENU order by USERID,DSPIDX")?;
    let menu_tree_seed = build_menu_tree_seed(&functions);
    write_json_file(root.join("sys_function_menu.json"), &functions)?;
    write_json_file(root.join("v_sys_usermenu.json"), &user_menu)?;
    write_json_file(root.join("wv_sys_usermenu.json"), &web_user_menu)?;
    write_json_file(root.join("menu_tree_seed.json"), &menu_tree_seed)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "menus": functions
                .iter()
                .map(|row| json!({
                    "code": row_string(row, "IDNUM"),
                    "name": preferred_name(row),
                    "parent": row_string(row, "PARENT"),
                    "icon": row_string(row, "IMAGEID"),
                    "displayOrder": row_string(row, "DSPIDX"),
                    "featureType": row_string(row, "FUNCTYPE")
                }))
                .collect::<Vec<Value>>(),
            "menuTreeSeed": menu_tree_seed,
            "userMenuView": user_menu,
            "webUserMenuView": web_user_menu
        }),
    )?;
    Ok(functions.len() + user_menu.len() + web_user_menu.len())
}

fn build_menu_tree_seed(functions: &[Row]) -> Value {
    let mut children_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut row_map: BTreeMap<String, &Row> = BTreeMap::new();
    for row in functions {
        row_map.insert(row_string(row, "IDNUM"), row);
        children_map
            .entry(row_string(row, "PARENT"))
            .or_default()
            .push(row_string(row, "IDNUM"));
    }

    let mut ordered_codes = Vec::new();
    let mut visited = BTreeSet::new();
    append_menu_tree_codes("", &children_map, &mut visited, &mut ordered_codes);
    for row in functions {
        let code = row_string(row, "IDNUM");
        if !visited.contains(&code) {
            visited.insert(code.clone());
            ordered_codes.push(code.clone());
            append_menu_tree_codes(&code, &children_map, &mut visited, &mut ordered_codes);
        }
    }

    let entries: Vec<Value> = ordered_codes
        .iter()
        .filter_map(|code| row_map.get(code).copied())
        .filter(|row| row_string(row, "STATUS") != "0")
        .map(|row| {
            let code = row_string(row, "IDNUM");
            let parent_code = non_empty(row_string(row, "PARENT"));
            let has_children = children_map.get(&code).map(|items| !items.is_empty()).unwrap_or(false);
            let feature_code = if is_exportable_function(row, &children_map) && !has_children {
                Some(code.clone())
            } else {
                None
            };
            let menu_type = if feature_code.is_some() { "scenario" } else { "group" };
            json!({
                "legacyCode": code,
                "parentLegacyCode": parent_code,
                "createRequest": {
                    "label": preferred_name(row),
                    "code": row_string(row, "IDNUM"),
                    "menuType": menu_type,
                    "orderIndex": parse_i64ish(&row_string(row, "DSPIDX")).unwrap_or(0),
                    "featureCode": feature_code,
                    "scenarioCode": if menu_type == "scenario" { Value::String("list".to_string()) } else { Value::Null }
                }
            })
        })
        .collect();

    json!({
        "version": 1,
        "notes": [
            "Create entries in listed order so parentLegacyCode can be resolved to the menu ID created earlier.",
            "For scenario menus, resolve featureCode and scenarioCode to new-system featureId/scenarioId before calling system create-menu.",
            "If a feature has no list scenario, replace scenarioCode with the target scenario code before import."
        ],
        "entries": entries
    })
}

fn append_menu_tree_codes(
    parent_code: &str,
    children_map: &BTreeMap<String, Vec<String>>,
    visited: &mut BTreeSet<String>,
    ordered_codes: &mut Vec<String>,
) {
    if let Some(children) = children_map.get(parent_code) {
        for child in children {
            if visited.insert(child.clone()) {
                ordered_codes.push(child.clone());
                append_menu_tree_codes(child, children_map, visited, ordered_codes);
            }
        }
    }
}

fn export_list_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let lists = runner.query("select * from SYS_List order by IDNUM")?;
    let list_details = runner.query("select * from SYS_LISTDETAIL order by IDNUM,IDLINE")?;
    write_json_file(root.join("sys_list.json"), &lists)?;
    write_json_file(root.join("sys_listdetail.json"), &list_details)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "lists": lists,
            "listDetails": list_details
        }),
    )?;
    Ok(lists.len() + list_details.len())
}

fn export_metadata_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let tables = runner.query("select * from SYS_Table order by IDNUM")?;
    let table_fields = runner.query("select * from SYS_TableField order by IDNUM,DSPIDX")?;
    let table_indexes = runner.query("select * from SYS_TableIndex order by IDNUM")?;
    let table_fks = runner.query("select * from SYS_TableFK order by IDNUM")?;
    let functions = runner.query("select * from SYS_Function order by IDNUM")?;
    let groups = runner.query("select * from SYS_Group order by FUNCID,DSPIDX")?;
    let blocks = runner.query("select * from V_SYS_GroupBlock order by FUNCID,GRPID,DSPIDX")?;
    let block_fields =
        runner.query("select * from V_SYS_GroupBlockField order by FUNCID,GRPID,BLKID")?;
    let buttons =
        runner.query("select * from V_SYS_GroupButton order by FUNCID,GRPID,BLKID,DSPIDX")?;
    let events = runner.query("select * from SYS_EVENT order by EVTOBJID1,EVTSRC,EVTNAME")?;
    let regions =
        runner.query_optional("select * from SYS_REGION order by FUNCID,BLOCKID,DSPIDX")?;
    let block_field_overrides = runner
        .query_optional("select * from SYS_BlockFieldOver order by FUNCID,BLKID,FIELDNAME")?;
    let ext_props =
        runner.query_optional("select * from SYS_EXTPROP order by PROPTYPE,ID1,ID2,ID3")?;
    write_json_file(root.join("sys_table.json"), &tables)?;
    write_json_file(root.join("sys_tablefield.json"), &table_fields)?;
    write_json_file(root.join("sys_tableindex.json"), &table_indexes)?;
    write_json_file(root.join("sys_tablefk.json"), &table_fks)?;
    write_json_file(root.join("sys_function.json"), &functions)?;
    write_json_file(root.join("sys_group.json"), &groups)?;
    write_json_file(root.join("v_sys_groupblock.json"), &blocks)?;
    write_json_file(root.join("v_sys_groupblockfield.json"), &block_fields)?;
    write_json_file(root.join("v_sys_groupbutton.json"), &buttons)?;
    write_json_file(root.join("sys_event.json"), &events)?;
    write_json_file(root.join("sys_region.json"), &regions)?;
    write_json_file(root.join("sys_blockfieldover.json"), &block_field_overrides)?;
    write_json_file(root.join("sys_extprop.json"), &ext_props)?;
    write_json_file(
        root.join("normalized.json"),
        &json!({
            "entities": tables,
            "entityFields": table_fields,
            "entityIndexes": table_indexes,
            "entityRelationships": table_fks,
            "features": functions,
            "scenarios": groups,
            "blocks": blocks,
            "scenarioFields": block_fields,
            "actions": buttons,
            "events": events,
            "regions": regions,
            "blockFieldOverrides": block_field_overrides,
            "extProps": ext_props
        }),
    )?;
    Ok(tables.len()
        + table_fields.len()
        + table_indexes.len()
        + table_fks.len()
        + functions.len()
        + groups.len()
        + blocks.len()
        + block_fields.len()
        + buttons.len()
        + events.len()
        + regions.len()
        + block_field_overrides.len()
        + ext_props.len())
}

fn export_data_catalog_assets(runner: &QueryRunner, root: &Path) -> AppResult<usize> {
    fs::create_dir_all(root)?;
    let tables = runner.query("select * from SYS_Table order by IDNUM")?;
    let table_fields = runner.query("select * from SYS_TableField order by IDNUM,DSPIDX")?;
    let table_fks = runner.query("select * from SYS_TableFK order by IDNUM")?;

    let entities: Vec<Value> = tables
        .iter()
        .map(|row| {
            let code = row_string(row, "IDNUM");
            json!({
                "entityCode": code,
                "entityName": preferred_name(row),
                "physicalName": row_string(row, "IDNUM"),
                "idField": row_string(row, "IDFIELD"),
                "serialRuleCode": row_string(row, "IDSER"),
                "isView": parse_boolish(&row_string(row, "ISVIEW")),
                "tableRole": infer_table_role(&row_string(row, "IDNUM"))
            })
        })
        .collect();

    let fields: Vec<Value> = table_fields
        .iter()
        .map(|row| {
            let field_code = row_string(row, "FIELDNAME");
            json!({
                "entityCode": row_string(row, "IDNUM"),
                "fieldCode": field_code,
                "fieldName": preferred_name(row),
                "dataType": map_legacy_data_type(&row_string(row, "FIELDTYPE")),
                "legacyFieldType": row_string(row, "FIELDTYPE"),
                "length": parse_i64ish(&row_string(row, "FLENGTH")),
                "scale": parse_i64ish(&row_string(row, "FSUBLENGTH")),
                "required": parse_boolish(&row_string(row, "REQUIRED")),
                "isSystem": parse_boolish(&row_string(row, "SYSTEM")),
                "isUnique": parse_boolish(&row_string(row, "UNIQUE")),
                "semanticHints": build_field_semantic_hints(&field_code)
            })
        })
        .collect();

    let relationships: Vec<Value> = table_fks
        .iter()
        .map(|row| {
            json!({
                "entityCode": row_string(row, "IDNUM"),
                "fkId": row_string(row, "FKID"),
                "sourceField": row_string(row, "FIELDNAME"),
                "targetEntityCode": row_string(row, "REFTABLE"),
                "targetField": row_string(row, "REFFIELD"),
                "metadata": row
            })
        })
        .collect();

    let table_semantics: Vec<Value> = tables
        .iter()
        .map(|row| {
            let table_code = row_string(row, "IDNUM");
            json!({
                "tableCode": table_code,
                "tableRole": infer_table_role(&table_code),
                "semanticHints": build_table_semantic_hints(&table_code)
            })
        })
        .collect();

    write_json_file(root.join("entities.json"), &entities)?;
    write_json_file(root.join("fields.json"), &fields)?;
    write_json_file(root.join("relationships.json"), &relationships)?;
    write_json_file(root.join("table_semantics.json"), &table_semantics)?;
    write_json_file(
        root.join("manifest.json"),
        &json!({
            "entities": entities.len(),
            "fields": fields.len(),
            "relationships": relationships.len(),
            "tableSemantics": table_semantics.len(),
            "notes": [
                "This catalog is intended for AI-assisted plugin rewriting.",
                "Use semantic hints to convert raw SQL intent into new-system entity/service operations."
            ]
        }),
    )?;
    Ok(entities.len() + fields.len() + relationships.len() + table_semantics.len())
}

struct NormalizedExport {
    manifest: Manifest,
    feature: Value,
    scenarios: Value,
    entities: Value,
    feature_fields: Value,
    scenario_fields: Value,
    actions: Value,
    plugin_refs: Value,
    indirect_plugin_refs: Value,
    create_feature_seed: Value,
    scenario_update_seed: Value,
    scenario_update_seeds: Value,
    plugin_analysis_seed: Value,
    plugin_call_context: Value,
    entity_field_catalog: Value,
    related_table_catalog: Value,
    plugin_bindings: Value,
    sql_dependencies: Value,
    plugin_rewrite_handoff: Value,
    migration_readiness: Value,
    summary_md: String,
}

fn normalize(legacy: &LegacyFunctionExport) -> NormalizedExport {
    let function = legacy.function.first().cloned().unwrap_or_default();
    let func_id = row_string(&function, "IDNUM");
    let feature_name = preferred_name(&function);
    let main_entity_code = infer_main_entity_code(legacy, &function);
    let plugin_refs = collect_plugin_refs(&legacy.events);
    let indirect_plugin_refs = collect_indirect_plugin_refs(&legacy.bl_defs);

    let mut warnings = Vec::new();
    if !legacy.regions.is_empty() {
        warnings.push("legacy regions were preserved in metadata only".to_string());
    }
    if legacy
        .events
        .iter()
        .any(|row| row_string(row, "EVTSRC") == "BLOCKFIELD")
    {
        warnings.push("block field events require plugin/source adaptation".to_string());
    }
    if !indirect_plugin_refs.is_empty() {
        warnings
            .push("@BL references were expanded but still need plugin source analysis".to_string());
    }

    let scenarios: Vec<Value> = legacy
        .groups
        .iter()
        .map(|row| {
            let scenario_code = legacy_group_code(row);
            let related_blocks: Vec<Value> = legacy
                .blocks
                .iter()
                .filter(|block| legacy_group_code(block) == scenario_code)
                .map(|block| {
                    json!({
                        "blockCode": legacy_block_code(block),
                        "tableCode": legacy_table_code(block),
                        "orderBy": row_string(block, "ORDERBY"),
                        "allowCreate": parse_boolish(&row_string(block, "ALLOWCREATE")),
                        "allowEdit": parse_boolish(&row_string(block, "ALLOWEDIT")),
                        "allowDelete": parse_boolish(&row_string(block, "ALLOWDELETE"))
                    })
                })
                .collect();
            json!({
                "code": scenario_code,
                "name": preferred_name(row),
                "scenarioType": infer_scenario_type(row, legacy),
                "metadata": {
                    "legacyStatus": row_string(row, "STATUS"),
                    "legacyDisplayOrder": row_string(row, "DSPIDX"),
                    "blocks": related_blocks
                }
            })
        })
        .collect();

    let entities: Vec<Value> = legacy
        .tables
        .iter()
        .map(|(table_code, table_export)| {
            let table_row = table_export.table.first().cloned().unwrap_or_default();
            let fields: Vec<Value> = table_export
                .fields
                .iter()
                .map(|field| {
                    json!({
                        "code": legacy_field_code(field),
                        "name": preferred_name(field),
                        "legacyFieldType": row_string(field, "FIELDTYPE"),
                        "dataType": map_legacy_data_type(&row_string(field, "FIELDTYPE")),
                        "required": parse_boolish(&row_string(field, "REQUIRED")),
                        "length": parse_i64ish(&row_string(field, "FLENGTH")),
                        "scale": parse_i64ish(&row_string(field, "FSUBLENGTH"))
                    })
                })
                .collect();
            json!({
                "code": table_code,
                "name": preferred_name(&table_row),
                "idField": row_string(&table_row, "IDFIELD"),
                "isView": parse_boolish(&row_string(&table_row, "ISVIEW")),
                "fields": fields
            })
        })
        .collect();

    let feature_field_defs = build_feature_fields(legacy, &main_entity_code);
    let scenario_field_defs = build_scenario_fields(legacy, &feature_field_defs);
    let actions = build_actions(legacy, &plugin_refs);
    let unresolved_action_scenarios = actions
        .iter()
        .filter(|row| {
            row.get("metadata")
                .and_then(|metadata| metadata.get("scenarioResolution"))
                .and_then(Value::as_str)
                == Some("fallback")
        })
        .count();
    if unresolved_action_scenarios > 0 {
        warnings.push(format!(
            "{unresolved_action_scenarios} action(s) used fallback scenario mapping; check normalized/actions.json metadata.legacyScenarioCode"
        ));
    }

    let manifest = Manifest {
        func_id: func_id.clone(),
        feature_code: func_id.clone(),
        feature_name: feature_name.clone(),
        main_entity_code: main_entity_code.clone(),
        scenario_codes: legacy
            .groups
            .iter()
            .map(legacy_group_code)
            .filter(|code| !code.is_empty())
            .collect(),
        direct_entity_codes: legacy.tables.keys().cloned().collect(),
        plugin_refs: plugin_refs.clone(),
        indirect_plugin_refs: indirect_plugin_refs.clone(),
        warnings,
        exported_at_epoch_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    let feature = json!({
        "code": func_id,
        "name": feature_name,
        "mainEntityCode": main_entity_code,
        "fieldCount": feature_field_defs.len(),
        "scenarioCount": scenarios.len(),
        "fields": feature_field_defs,
        "scenarios": scenarios,
        "metadata": {
            "legacyParent": row_string(&function, "PARENT"),
            "legacyFuncType": row_string(&function, "FUNCTYPE"),
            "legacyImageId": row_string(&function, "IMAGEID"),
            "legacyKeepLog": row_string(&function, "KEEPLOG"),
            "legacyExtProps": ext_props_to_map(&legacy.function_ext)
        }
    });

    let create_feature_seed = json!({
        "code": manifest.feature_code,
        "name": manifest.feature_name,
        "module": infer_module_code(&manifest.feature_code),
        "entityCode": manifest.main_entity_code,
        "includeDefaultListActions": false,
        "fields": feature_field_defs
            .iter()
            .filter(|field| field.get("sourceType").and_then(Value::as_str) == Some("entity"))
            .map(|field| {
                json!({
                    "displayName": field.get("displayName").cloned().unwrap_or(Value::Null),
                    "dataType": field.get("dataType").cloned().unwrap_or(Value::String("string".to_string())),
                    "sourceField": field.get("sourceField").cloned().unwrap_or(Value::Null),
                    "isIdentifier": field.get("isIdentifier").cloned().unwrap_or(Value::Bool(false))
                })
            })
            .collect::<Vec<Value>>()
    });

    let scenario_update_seeds = scenarios
        .iter()
        .map(|scenario| {
            build_scenario_update_seed(
                legacy,
                scenario,
                &scenario_field_defs,
                &actions,
                &feature_field_defs,
                &main_entity_code,
            )
        })
        .collect::<Vec<_>>();

    let default_scenario_index = scenarios
        .iter()
        .position(|item| item.get("scenarioType").and_then(Value::as_str) == Some("form"))
        .or_else(|| {
            scenarios
                .iter()
                .position(|item| item.get("scenarioType").and_then(Value::as_str) == Some("list"))
        })
        .unwrap_or(0);
    let scenario_update_seed = scenario_update_seeds
        .get(default_scenario_index)
        .cloned()
        .unwrap_or_else(|| json!({}));

    let plugin_analysis_seed = json!({
        "funcId": manifest.func_id,
        "featureCode": manifest.feature_code,
        "mainEntityCode": manifest.main_entity_code,
        "directEntityCodes": manifest.direct_entity_codes,
        "pluginRefs": manifest.plugin_refs,
        "indirectPluginRefs": manifest.indirect_plugin_refs,
        "notes": [
            "Analyze plugin source code to find helper methods and extra SQL table dependencies.",
            "Do not assume metadata-bound tables are the full write/read dependency set."
        ]
    });

    let plugin_call_context = json!({
        "feature": {
            "code": manifest.feature_code,
            "name": manifest.feature_name,
            "mainEntityCode": manifest.main_entity_code
        },
        "scenarios": scenarios,
        "actions": actions,
        "pluginRefs": manifest.plugin_refs,
        "indirectPluginRefs": manifest.indirect_plugin_refs,
        "notes": [
            "Use this file together with plugin source code to reconstruct trigger points and call paths.",
            "Event-owner mappings are legacy metadata references, not direct new-system runtime bindings."
        ]
    });

    let entity_field_catalog = build_entity_field_catalog(legacy);
    let related_table_catalog = build_related_table_catalog(legacy);
    let plugin_bindings = build_plugin_bindings(legacy, &plugin_refs, &indirect_plugin_refs);
    let sql_dependencies = build_sql_dependencies(legacy, &plugin_refs, &indirect_plugin_refs);
    let plugin_rewrite_handoff = build_plugin_rewrite_handoff(
        &manifest,
        &entity_field_catalog,
        &related_table_catalog,
        &plugin_bindings,
        &sql_dependencies,
    );
    let migration_readiness =
        build_function_migration_readiness(legacy, &manifest, &sql_dependencies, &plugin_bindings);

    let summary_md = build_summary_md(
        &manifest,
        &feature_field_defs,
        &scenario_field_defs,
        &actions,
    );

    NormalizedExport {
        manifest,
        feature,
        scenarios: Value::Array(scenarios),
        entities: Value::Array(entities),
        feature_fields: Value::Array(feature_field_defs),
        scenario_fields: Value::Array(scenario_field_defs),
        actions: Value::Array(actions),
        plugin_refs: serde_json::to_value(plugin_refs).unwrap_or_else(|_| Value::Array(Vec::new())),
        indirect_plugin_refs: serde_json::to_value(indirect_plugin_refs)
            .unwrap_or_else(|_| Value::Array(Vec::new())),
        create_feature_seed,
        scenario_update_seed,
        scenario_update_seeds: Value::Array(scenario_update_seeds),
        plugin_analysis_seed,
        plugin_call_context,
        entity_field_catalog,
        related_table_catalog,
        plugin_bindings,
        sql_dependencies,
        plugin_rewrite_handoff,
        migration_readiness,
        summary_md,
    }
}

fn build_entity_field_catalog(legacy: &LegacyFunctionExport) -> Value {
    let items: Vec<Value> = legacy
        .tables
        .iter()
        .flat_map(|(table_code, table_export)| {
            table_export.fields.iter().map(move |field| {
                let field_code = row_string(field, "FIELDNAME");
                json!({
                    "tableCode": table_code,
                    "fieldCode": field_code,
                    "fieldName": preferred_name(field),
                    "dataType": map_legacy_data_type(&row_string(field, "FIELDTYPE")),
                    "legacyFieldType": row_string(field, "FIELDTYPE"),
                    "length": parse_i64ish(&row_string(field, "FLENGTH")),
                    "scale": parse_i64ish(&row_string(field, "FSUBLENGTH")),
                    "required": parse_boolish(&row_string(field, "REQUIRED")),
                    "isSystem": parse_boolish(&row_string(field, "SYSTEM")),
                    "isUnique": parse_boolish(&row_string(field, "UNIQUE")),
                    "semanticHints": build_field_semantic_hints(&field_code)
                })
            })
        })
        .collect();
    Value::Array(items)
}

fn build_related_table_catalog(legacy: &LegacyFunctionExport) -> Value {
    let main_entity_code = legacy
        .function
        .first()
        .and_then(|row| infer_main_entity_code(legacy, row))
        .unwrap_or_default();
    let items: Vec<Value> = legacy
        .tables
        .iter()
        .map(|(table_code, table_export)| {
            let table_row = table_export.table.first().cloned().unwrap_or_default();
            json!({
                "tableCode": table_code,
                "tableName": preferred_name(&table_row),
                "isMainEntity": !main_entity_code.is_empty() && table_code == &main_entity_code,
                "isView": parse_boolish(&row_string(&table_row, "ISVIEW")),
                "idField": row_string(&table_row, "IDFIELD"),
                "serialRuleCode": row_string(&table_row, "IDSER"),
                "tableRole": infer_table_role(table_code),
                "fieldCount": table_export.fields.len()
            })
        })
        .collect();
    Value::Array(items)
}

fn build_plugin_bindings(
    legacy: &LegacyFunctionExport,
    plugin_refs: &[PluginRef],
    indirect_plugin_refs: &[IndirectPluginRef],
) -> Value {
    let direct = plugin_refs
        .iter()
        .map(|plugin| {
            json!({
                "bindingType": "event",
                "triggerSource": plugin.event_source,
                "triggerName": plugin.event_name,
                "ownerId": plugin.owner_id,
                "ownerSecondaryId": plugin.owner_secondary_id,
                "pluginRef": plugin.raw_ref,
                "library": plugin.library,
                "namespace": plugin.namespace,
                "className": plugin.class_name,
                "methodName": plugin.method_name,
                "legacyParams": plugin.params
            })
        })
        .collect::<Vec<Value>>();

    let indirect = legacy
        .events
        .iter()
        .filter(|row| row_string(row, "EVTSTD").starts_with("@BL."))
        .map(|row| {
            let bl_id = row_string(row, "EVTSTD")
                .trim_start_matches("@BL.")
                .to_string();
            let target = indirect_plugin_refs.iter().find(|item| item.bl_id == bl_id);
            json!({
                "bindingType": "business-logic",
                "triggerSource": row_string(row, "EVTSRC"),
                "triggerName": row_string(row, "EVTNAME"),
                "ownerId": row_string(row, "EVTOBJID2"),
                "ownerSecondaryId": row.get("EVTOBJID3").cloned().unwrap_or_default(),
                "blId": bl_id,
                "pluginRef": target.as_ref().map(|item| item.raw_ref.clone()),
                "library": target.as_ref().map(|item| item.library.clone()),
                "namespace": target.as_ref().map(|item| item.namespace.clone()),
                "className": target.as_ref().map(|item| item.class_name.clone()),
                "methodName": target.as_ref().map(|item| item.method_name.clone()),
                "paramGroupId": target.and_then(|item| item.param_group_id.clone())
            })
        })
        .collect::<Vec<Value>>();

    json!({
        "directBindings": direct,
        "indirectBindings": indirect
    })
}

fn build_sql_dependencies(
    legacy: &LegacyFunctionExport,
    plugin_refs: &[PluginRef],
    indirect_plugin_refs: &[IndirectPluginRef],
) -> Value {
    let related_tables = legacy
        .tables
        .keys()
        .map(|table_code| {
            json!({
                "tableCode": table_code,
                "dependencyType": "metadata-direct",
                "analysisRequired": true,
                "reason": "This table is directly referenced by feature metadata and should be considered when rewriting SQL."
            })
        })
        .collect::<Vec<Value>>();

    let plugin_entries = plugin_refs
        .iter()
        .map(|plugin| {
            json!({
                "pluginRef": plugin.raw_ref,
                "entryMethod": {
                    "library": plugin.library,
                    "namespace": plugin.namespace,
                    "className": plugin.class_name,
                    "methodName": plugin.method_name
                },
                "candidateTables": legacy.tables.keys().cloned().collect::<Vec<String>>(),
                "analysisRequired": true,
                "notes": [
                    "Static SQL extraction is not performed by this CLI.",
                    "Use plugin source code plus data_catalog to determine actual read/write tables."
                ]
            })
        })
        .collect::<Vec<Value>>();

    let indirect_entries = indirect_plugin_refs
        .iter()
        .map(|plugin| {
            json!({
                "pluginRef": plugin.raw_ref,
                "entryMethod": {
                    "library": plugin.library,
                    "namespace": plugin.namespace,
                    "className": plugin.class_name,
                    "methodName": plugin.method_name
                },
                "candidateTables": legacy.tables.keys().cloned().collect::<Vec<String>>(),
                "analysisRequired": true,
                "notes": [
                    "This plugin is referenced through @BL indirection.",
                    "Plugin source analysis is required to identify actual SQL dependencies."
                ]
            })
        })
        .collect::<Vec<Value>>();

    json!({
        "relatedTables": related_tables,
        "directPluginEntries": plugin_entries,
        "indirectPluginEntries": indirect_entries
    })
}

fn build_plugin_rewrite_handoff(
    manifest: &Manifest,
    entity_field_catalog: &Value,
    related_table_catalog: &Value,
    plugin_bindings: &Value,
    sql_dependencies: &Value,
) -> Value {
    json!({
        "featureCode": manifest.feature_code,
        "featureName": manifest.feature_name,
        "mainEntityCode": manifest.main_entity_code,
        "pluginRefs": manifest.plugin_refs,
        "indirectPluginRefs": manifest.indirect_plugin_refs,
        "pluginBindings": plugin_bindings,
        "sqlDependencies": sql_dependencies,
        "entityFieldCatalog": entity_field_catalog,
        "relatedTableCatalog": related_table_catalog,
        "rewriteGuidance": [
            "Do not translate legacy SQL literally into new plugins.",
            "Rewrite SQL intent into new-system entity, repository, or service operations.",
            "Use table and field semantic hints to infer join purpose, status checks, and master-data lookups."
        ],
        "manualReviewRequired": [
            "Helper method call chains inside plugin source code",
            "Cross-feature writeback tables",
            "Direct transaction management and UI/session interactions"
        ]
    })
}

fn build_function_migration_readiness(
    legacy: &LegacyFunctionExport,
    manifest: &Manifest,
    sql_dependencies: &Value,
    plugin_bindings: &Value,
) -> Value {
    let has_plugins = !manifest.plugin_refs.is_empty() || !manifest.indirect_plugin_refs.is_empty();
    let has_indirect = !manifest.indirect_plugin_refs.is_empty();
    let has_regions = !legacy.regions.is_empty();
    let has_events = !legacy.events.is_empty();
    let risk_level = if has_indirect {
        "high"
    } else if has_plugins || has_events || has_regions {
        "medium"
    } else {
        "low"
    };
    let risk_flags: Vec<&str> = [
        if has_plugins {
            Some("event_uses_plugin")
        } else {
            None
        },
        if has_indirect {
            Some("event_uses_bl")
        } else {
            None
        },
        if has_regions {
            Some("legacy_region_metadata_present")
        } else {
            None
        },
        if has_events {
            Some("event_binding_present")
        } else {
            None
        },
    ]
    .into_iter()
    .flatten()
    .collect();
    json!({
        "featureCode": manifest.feature_code,
        "readyForMetadataMigration": true,
        "readyForPluginRewrite": has_plugins,
        "riskLevel": risk_level,
        "riskFlags": risk_flags,
        "summary": {
            "directEntityCount": manifest.direct_entity_codes.len(),
            "pluginRefCount": manifest.plugin_refs.len(),
            "indirectPluginRefCount": manifest.indirect_plugin_refs.len(),
            "scenarioCount": manifest.scenario_codes.len()
        },
        "requiredNextSteps": [
            "Use plugin source repository to resolve helper method call paths.",
            "Review sql_dependencies.json before generating new-system plugins.",
            "Use new-system CLI to create entities/features/scenarios first, then rewrite plugins."
        ],
        "pluginBindings": plugin_bindings,
        "sqlDependencies": sql_dependencies
    })
}

fn build_system_migration_readiness(root: &Path) -> AppResult<Value> {
    let metadata = read_json_value(root.join("metadata/normalized.json"))?;
    let workflows = read_json_value(root.join("workflows/normalized.json"))?;
    let security = read_json_value(root.join("security/normalized.json"))?;
    let serials = read_json_value(root.join("serials/normalized.json"))?;
    let menus = read_json_value(root.join("menus/normalized.json"))?;

    let feature_count = metadata
        .get("features")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let workflow_count = workflows
        .get("workflowDefinitions")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let role_count = security
        .get("roles")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let serial_count = serials
        .get("serialRules")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let menu_count = menus
        .get("menus")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let risk_flags: Vec<&str> = [
        if workflow_count > 0 {
            Some("workflow_assets_present")
        } else {
            None
        },
        if role_count > 0 {
            Some("security_assets_present")
        } else {
            None
        },
        if serial_count > 0 {
            Some("serial_rules_present")
        } else {
            None
        },
    ]
    .into_iter()
    .flatten()
    .collect();

    Ok(json!({
        "readyForSystemMetadataMigration": true,
        "riskLevel": if workflow_count > 0 || role_count > 0 { "medium" } else { "low" },
        "summary": {
            "featureCount": feature_count,
            "workflowCount": workflow_count,
            "roleCount": role_count,
            "serialRuleCount": serial_count,
            "menuCount": menu_count
        },
        "riskFlags": risk_flags,
        "requiredNextSteps": [
            "Migrate entities before features and scenarios.",
            "Apply security, menu, workflow, and serial assets after base metadata exists.",
            "Validate plugin bindings separately with exported function packages."
        ]
    }))
}

fn build_field_semantic_hints(field_code: &str) -> Vec<String> {
    let upper = field_code.to_ascii_uppercase();
    let mut hints = Vec::new();
    if upper == "STATUS" || upper.ends_with("_STATUS") {
        hints.push("status".to_string());
    }
    if upper == "ID" || upper.starts_with("ID_") {
        hints.push("identifier".to_string());
    }
    if upper.starts_with("CODE_") || upper == "CODE" {
        hints.push("reference-code".to_string());
    }
    if upper.contains("ORG") || upper.contains("DEPT") {
        hints.push("organization".to_string());
    }
    if upper.contains("DATE") || upper.ends_with("DAT") {
        hints.push("date-time".to_string());
    }
    if upper.starts_with("CRE") || upper.starts_with("UPD") {
        hints.push("audit".to_string());
    }
    hints
}

fn infer_table_role(table_code: &str) -> &'static str {
    let upper = table_code.to_ascii_uppercase();
    if upper.starts_with("SYS_WF") {
        "workflow"
    } else if upper.starts_with("SYS_") {
        "system"
    } else if upper.contains("_LOG") || upper.contains("HIST") {
        "log-or-history"
    } else if upper.starts_with("BD_") {
        "master-data"
    } else {
        "business-structure"
    }
}

fn build_table_semantic_hints(table_code: &str) -> Vec<String> {
    let upper = table_code.to_ascii_uppercase();
    let mut hints = Vec::new();
    if upper.starts_with("SYS_") {
        hints.push("system".to_string());
    }
    if upper.starts_with("BD_") {
        hints.push("master-data".to_string());
    }
    if upper.starts_with("DV_") || upper.starts_with("V_") || upper.starts_with("WV_") {
        hints.push("view".to_string());
    }
    if upper.starts_with("SYS_WF") {
        hints.push("workflow".to_string());
    }
    if upper.contains("LOG") {
        hints.push("log".to_string());
    }
    if upper.contains("HIST") {
        hints.push("history".to_string());
    }
    hints
}

fn build_feature_fields(
    legacy: &LegacyFunctionExport,
    main_entity_code: &Option<String>,
) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in &legacy.block_fields {
        let field_name = legacy_field_code(row);
        let table_code = block_field_table_code(legacy, row);
        if field_name.is_empty() {
            continue;
        }
        let field_key = if Some(table_code.clone()) == *main_entity_code || table_code.is_empty() {
            field_name.clone()
        } else {
            format!("{table_code}.{field_name}")
        };
        if seen.contains(&field_key) {
            continue;
        }
        seen.insert(field_key.clone());
        result.push(json!({
            "fieldKey": field_key,
            "displayName": preferred_name(row),
            "dataType": infer_field_data_type(legacy, &table_code, &field_name),
            "sourceType": "entity",
            "sourceField": field_name,
            "sourceConfig": {
                "entityCode": table_code,
                "fieldCode": field_name
            },
            "isIdentifier": is_identifier_candidate(&field_name),
            "metadata": {
                "legacyRegion": legacy_region_code(row),
                "legacyGStatus": row_string(row, "GSTATUS"),
                "legacyBStatus": row_string(row, "BSTATUS")
            }
        }));
    }
    result
}

fn build_scenario_fields(legacy: &LegacyFunctionExport, feature_fields: &[Value]) -> Vec<Value> {
    let feature_field_keys: BTreeMap<(String, String), String> = feature_fields
        .iter()
        .filter_map(|row| {
            let field_key = row.get("fieldKey")?.as_str()?.to_string();
            let source_config = row.get("sourceConfig")?;
            let entity = source_config.get("entityCode")?.as_str()?.to_string();
            let field = source_config.get("fieldCode")?.as_str()?.to_string();
            Some(((entity, field), field_key))
        })
        .collect();

    let mut result = Vec::new();
    for row in &legacy.block_fields {
        let table_code = block_field_table_code(legacy, row);
        let field_name = legacy_field_code(row);
        let scenario_code = legacy_group_code(row);
        if let Some(feature_field_key) =
            feature_field_keys.get(&(table_code.clone(), field_name.clone()))
        {
            let view_context = infer_view_context(&legacy_block_code(row));
            let status = compute_block_field_status(row);
            let component_type = legacy_component_type(row);
            let suggested_component = legacy_suggested_component(row);
            result.push(json!({
                "scenarioCode": scenario_code,
                "featureFieldKey": feature_field_key,
                "viewContext": view_context,
                "orderIndex": parse_i64ish(&row_string(row, "DSPIDX")).unwrap_or(0),
                "isVisible": status != "disable",
                "isReadonly": status == "readonly",
                "isRequired": parse_boolish(&row_string(row, "REQUIRED")),
                "columnWidth": Value::Null,
                "filterable": parse_boolish(&row_string(row, "ALLOWSEARCH")),
                "componentType": component_type,
                "metadata": {
                    "legacyBlockId": legacy_block_code(row),
                    "legacyRegion": legacy_region_code(row),
                    "legacyStatus": status,
                    "legacyControlType": row_string(row, "CTRLTYPE"),
                    "legacyDataSourceType": row_string(row, "DSTYPE"),
                    "legacyDataSource": row_first_string(row, &["DSVALUE", "DATASOURCE", "LISTID"]),
                    "suggestedComponent": suggested_component,
                    "migrationStatus": legacy_dependency_status(row)
                }
            }));
        }
    }
    result
}

fn build_scenario_update_seed(
    legacy: &LegacyFunctionExport,
    scenario: &Value,
    scenario_fields: &[Value],
    actions: &[Value],
    feature_fields: &[Value],
    main_entity_code: &Option<String>,
) -> Value {
    let scenario_code = scenario
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("detail");
    let scenario_type = scenario
        .get("scenarioType")
        .and_then(Value::as_str)
        .unwrap_or("form");

    let field_groups = scenario_fields
        .iter()
        .filter(|row| row.get("scenarioCode").and_then(Value::as_str) == Some(scenario_code))
        .filter(|row| {
            row.get("featureFieldKey")
                .and_then(Value::as_str)
                .map(|key| !key.contains('.'))
                .unwrap_or(false)
        })
        .map(|row| {
            let context = row
                .get("viewContext")
                .and_then(Value::as_str)
                .unwrap_or("form");
            let mut view = serde_json::Map::new();
            view.insert(
                "enabled".to_string(),
                row.get("isVisible").cloned().unwrap_or(Value::Bool(true)),
            );
            view.insert(
                "visible".to_string(),
                row.get("isVisible").cloned().unwrap_or(Value::Bool(true)),
            );
            view.insert(
                "readonly".to_string(),
                row.get("isReadonly").cloned().unwrap_or(Value::Bool(false)),
            );
            view.insert(
                "required".to_string(),
                row.get("isRequired").cloned().unwrap_or(Value::Bool(false)),
            );
            view.insert(
                "filterable".to_string(),
                row.get("filterable").cloned().unwrap_or(Value::Bool(false)),
            );
            view.insert(
                "componentType".to_string(),
                row.get("componentType")
                    .cloned()
                    .unwrap_or(Value::String("q-input".to_string())),
            );
            view.insert(
                "extraMetadata".to_string(),
                row.get("metadata").cloned().unwrap_or_else(|| json!({})),
            );

            let mut group = serde_json::Map::new();
            group.insert(
                "featureFieldKey".to_string(),
                row.get("featureFieldKey").cloned().unwrap_or(Value::Null),
            );
            group.insert(
                "orderIndex".to_string(),
                row.get("orderIndex")
                    .cloned()
                    .unwrap_or(Value::Number(0.into())),
            );
            let target_context = if context == "list" {
                "list"
            } else if context == "detail" {
                "detail"
            } else {
                "form"
            };
            group.insert(target_context.to_string(), Value::Object(view));
            Value::Object(group)
        })
        .collect::<Vec<_>>();

    let form_layout = build_form_layout(legacy, scenario_code, feature_fields, main_entity_code);
    let detail_tables = build_detail_tables(legacy, scenario_code, main_entity_code);
    let mut metadata = serde_json::Map::new();
    if !form_layout.is_null() {
        metadata.insert("formLayout".to_string(), form_layout);
    }
    if !detail_tables.is_empty() {
        metadata.insert("detailTables".to_string(), Value::Array(detail_tables));
    }

    json!({
        "code": scenario_code,
        "name": scenario.get("name").cloned().unwrap_or(Value::String(scenario_code.to_string())),
        "scenarioType": scenario_type,
        "fieldGroups": field_groups,
        "actions": actions
            .iter()
            .filter(|row| row.get("scenarioCode").and_then(Value::as_str) == Some(scenario_code))
            .map(|row| json!({
                "actionId": "replace-with-action-id",
                "displayName": row.get("displayName").cloned().unwrap_or(Value::Null),
                "orderIndex": row.get("orderIndex").cloned().unwrap_or(Value::Number(0.into())),
                "layoutSlot": row.get("layoutSlot").cloned().unwrap_or(Value::String("toolbar".to_string()))
            }))
            .collect::<Vec<_>>(),
        "metadata": Value::Object(metadata)
    })
}

fn build_form_layout(
    legacy: &LegacyFunctionExport,
    scenario_code: &str,
    feature_fields: &[Value],
    main_entity_code: &Option<String>,
) -> Value {
    let feature_keys: BTreeMap<(String, String), String> = feature_fields
        .iter()
        .filter_map(|field| {
            let key = field.get("fieldKey")?.as_str()?.to_string();
            let source = field.get("sourceConfig")?;
            Some((
                (
                    source.get("entityCode")?.as_str()?.to_string(),
                    source.get("fieldCode")?.as_str()?.to_string(),
                ),
                key,
            ))
        })
        .collect();
    let mut tab_order = Vec::new();
    let mut tab_fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &legacy.block_fields {
        if legacy_group_code(row) != scenario_code {
            continue;
        }
        let table = block_field_table_code(legacy, row);
        if main_entity_code.as_deref() != Some(table.as_str())
            || infer_view_context(&legacy_block_code(row)) == "list"
        {
            continue;
        }
        let region = legacy_region_code(row);
        let tab_key = if region.is_empty() {
            "basic".to_string()
        } else {
            region
        };
        if !tab_fields.contains_key(&tab_key) {
            tab_order.push(tab_key.clone());
        }
        if let Some(field_key) = feature_keys.get(&(table, legacy_field_code(row))) {
            tab_fields
                .entry(tab_key)
                .or_default()
                .push(field_key.clone());
        }
    }
    let tabs = tab_order
        .into_iter()
        .filter_map(|key| {
            let fields = tab_fields.remove(&key).unwrap_or_default();
            if fields.is_empty() {
                return None;
            }
            let label = legacy
                .regions
                .iter()
                .find(|region| legacy_region_code(region) == key)
                .map(preferred_name)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| key.clone());
            Some(json!({ "key": key, "label": label, "fields": fields }))
        })
        .collect::<Vec<_>>();
    if tabs.is_empty() {
        Value::Null
    } else {
        json!({ "type": "tabs", "tabs": tabs })
    }
}

fn build_detail_tables(
    legacy: &LegacyFunctionExport,
    scenario_code: &str,
    main_entity_code: &Option<String>,
) -> Vec<Value> {
    let main_code = main_entity_code.as_deref().unwrap_or("");
    legacy
        .blocks
        .iter()
        .filter(|block| legacy_group_code(block) == scenario_code)
        .filter_map(|block| {
            let entity_code = legacy_table_code(block);
            if entity_code.is_empty()
                || entity_code == main_code
                || is_main_block(block)
                || infer_view_context(&legacy_block_code(block)) == "list"
            {
                return None;
            }
            let block_id = legacy_block_code(block);
            let (parent_key, child_key) = infer_relation_keys(legacy, main_code, &entity_code);
            let columns = legacy
                .block_fields
                .iter()
                .filter(|field| legacy_group_code(field) == scenario_code)
                .filter(|field| legacy_block_code(field) == block_id)
                .map(|field| json!({
                    "fieldKey": legacy_field_code(field),
                    "label": preferred_name(field),
                    "component": legacy_component_type(field),
                    "orderIndex": parse_i64ish(&row_string(field, "DSPIDX")).unwrap_or(0),
                    "required": parse_boolish(&row_string(field, "REQUIRED")),
                    "readonly": compute_block_field_status(field) == "readonly",
                    "persist": true,
                    "extraMetadata": {
                        "legacyControlType": row_string(field, "CTRLTYPE"),
                        "legacyDataSourceType": row_string(field, "DSTYPE"),
                        "legacyDataSource": row_first_string(field, &["DSVALUE", "DATASOURCE", "LISTID"]),
                        "suggestedComponent": legacy_suggested_component(field),
                        "migrationStatus": legacy_dependency_status(field)
                    }
                }))
                .collect::<Vec<_>>();
            Some(json!({
                "key": if block_id.is_empty() { entity_code.clone() } else { block_id },
                "title": preferred_name(block),
                "entityCode": entity_code,
                "relation": { "parentKey": parent_key, "childKey": child_key },
                "allowAdd": parse_boolish(&row_string(block, "ALLOWCREATE")),
                "allowDelete": parse_boolish(&row_string(block, "ALLOWDELETE")),
                "columns": columns,
                "migrationWarning": "Verify parentKey/childKey before applying this seed."
            }))
        })
        .collect()
}

fn legacy_table_id_field(legacy: &LegacyFunctionExport, table_code: &str) -> String {
    legacy
        .tables
        .get(table_code)
        .and_then(|table| table.table.first())
        .map(|row| row_string(row, "IDFIELD"))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "id".to_string())
}

fn infer_relation_keys(
    legacy: &LegacyFunctionExport,
    parent_table: &str,
    child_table: &str,
) -> (String, String) {
    let parent_fields = table_field_codes(legacy, parent_table);
    let child_fields = table_field_codes(legacy, child_table);
    let parent_id = legacy_table_id_field(legacy, parent_table);
    let parent_suffix = parent_table
        .to_ascii_uppercase()
        .trim_start_matches("BD_")
        .trim_start_matches("B_")
        .to_string();
    let expected_business_key = format!("CODE_{parent_suffix}");

    let mut common = parent_fields
        .intersection(&child_fields)
        .cloned()
        .collect::<Vec<_>>();
    common.sort_by_key(|field| {
        let upper = field.to_ascii_uppercase();
        if upper == expected_business_key {
            0
        } else if upper.starts_with("CODE_") || upper == "CODE" {
            1
        } else if field.eq_ignore_ascii_case(&parent_id) {
            2
        } else if upper.starts_with("ID_") || upper == "ID" {
            3
        } else {
            4
        }
    });
    if let Some(shared_key) = common.first() {
        return (shared_key.clone(), shared_key.clone());
    }

    let child_key = child_fields
        .iter()
        .find(|field| field.eq_ignore_ascii_case(&parent_id))
        .cloned()
        .unwrap_or_else(|| parent_id.clone());
    (parent_id, child_key)
}

fn table_field_codes(legacy: &LegacyFunctionExport, table_code: &str) -> BTreeSet<String> {
    let mut fields = legacy
        .tables
        .get(table_code)
        .map(|table| {
            table
                .fields
                .iter()
                .map(legacy_field_code)
                .filter(|field| !field.is_empty())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    fields.extend(
        legacy
            .block_fields
            .iter()
            .filter(|field| block_field_table_code(legacy, field) == table_code)
            .map(legacy_field_code)
            .filter(|field| !field.is_empty()),
    );
    fields
}

fn legacy_component_type(row: &Row) -> String {
    match row_string(row, "CTRLTYPE").as_str() {
        "3" => "q-checkbox",
        "4" => "q-date",
        "5" => "q-textarea",
        "7" => "q-decimal",
        "9" | "916" => "image-upload-field",
        "914" => "attachment-upload-field",
        "915" => "q-editor",
        // A list/relation without a migrated target is deliberately kept
        // editable. The suggested advanced component remains in metadata.
        "2" | "8" | "918" => "q-input",
        _ => "q-input",
    }
    .to_string()
}

fn legacy_suggested_component(row: &Row) -> String {
    match (
        row_string(row, "CTRLTYPE").as_str(),
        row_string(row, "DSTYPE").as_str(),
    ) {
        ("2" | "8" | "918", "2") => "relation-picker-field".to_string(),
        ("2" | "8" | "918", _) => "q-select".to_string(),
        _ => legacy_component_type(row),
    }
}

fn legacy_dependency_status(row: &Row) -> &'static str {
    match (
        row_string(row, "CTRLTYPE").as_str(),
        row_string(row, "DSTYPE").as_str(),
    ) {
        ("2" | "8" | "918", "2") => "requires-relation-mapping",
        ("2" | "8" | "918", _) => "requires-option-source-mapping",
        _ => "ready",
    }
}

fn build_actions(legacy: &LegacyFunctionExport, plugin_refs: &[PluginRef]) -> Vec<Value> {
    let scenario_codes: BTreeSet<String> = legacy.groups.iter().map(legacy_group_code).collect();
    let block_scenario_map: BTreeMap<String, String> = legacy
        .blocks
        .iter()
        .filter_map(|row| {
            let block_id = legacy_block_code(row);
            let scenario_code = legacy_group_code(row);
            if block_id.is_empty() || !scenario_codes.contains(&scenario_code) {
                None
            } else {
                Some((block_id, scenario_code))
            }
        })
        .collect();
    let fallback_scenario_code = legacy
        .groups
        .iter()
        .find(|row| infer_scenario_type(row, legacy) == "list")
        .or_else(|| legacy.groups.first())
        .map(legacy_group_code)
        .unwrap_or_else(|| "list".to_string());

    legacy
        .buttons
        .iter()
        .map(|row| {
            let legacy_scenario_code = legacy_group_code(row);
            let block_id = legacy_block_code(row);
            let (scenario_code, scenario_resolution) =
                if scenario_codes.contains(&legacy_scenario_code) {
                    (legacy_scenario_code.clone(), "direct")
                } else if let Some(mapped) = block_scenario_map.get(&block_id) {
                    (mapped.clone(), "block")
                } else {
                    (fallback_scenario_code.clone(), "fallback")
                };
            let button_id = row_string(row, "BTNID");
            let binding = plugin_refs.iter().find(|plugin| {
                plugin.event_source.eq_ignore_ascii_case("BUTTON")
                    && plugin.owner_id == block_id
                    && plugin.owner_secondary_id == button_id
            });
            json!({
                "scenarioCode": scenario_code,
                "code": button_id,
                "name": preferred_name(row),
                "actionType": infer_action_type(&row_string(row, "BTNID")),
                "executionMode": "sync",
                "displayName": preferred_name(row),
                "layoutSlot": "toolbar",
                "orderIndex": parse_i64ish(&row_string(row, "DSPIDX")).unwrap_or(0),
                "metadata": {
                    "legacyScenarioCode": legacy_scenario_code,
                    "legacyBlockId": block_id,
                    "scenarioResolution": scenario_resolution,
                    "icon": row_string(row, "BUTTONICON"),
                    "needsConfirmation": parse_boolish(&row_string(row, "NEEDCONFIRM")),
                    "pluginBinding": binding.map(|item| {
                        json!({
                            "pluginCode": build_plugin_code(item),
                            "capabilityCode": build_capability_code(item),
                            "triggerPhase": "before",
                            "params": {
                                "legacyParams": item.params
                            }
                        })
                    })
                }
            })
        })
        .collect()
}

fn collect_plugin_refs(events: &[Row]) -> Vec<PluginRef> {
    let mut refs = Vec::new();
    for row in events {
        let raw_ref = row_string(row, "EVTSTD");
        if raw_ref.is_empty() || raw_ref.starts_with("@BL.") {
            continue;
        }
        let parts: Vec<&str> = raw_ref.split('.').collect();
        if parts.len() < 3 {
            continue;
        }
        let (library, namespace, class_name, method_name) = if parts.len() >= 4 {
            (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            )
        } else {
            (
                parts[0].to_string(),
                String::new(),
                parts[1].to_string(),
                parts[2].to_string(),
            )
        };
        refs.push(PluginRef {
            event_source: row_string(row, "EVTSRC"),
            event_name: row_string(row, "EVTNAME"),
            owner_id: row_string(row, "EVTOBJID2"),
            owner_secondary_id: row
                .get("EVTOBJID3")
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string(),
            raw_ref: raw_ref.clone(),
            library,
            namespace,
            class_name,
            method_name,
            params: row_string(row, "EVTSTDP"),
        });
    }
    refs
}

fn collect_indirect_plugin_refs(bl_defs: &[Row]) -> Vec<IndirectPluginRef> {
    let mut refs = Vec::new();
    for row in bl_defs {
        let raw_ref = row_string(row, "REFPLUGIN");
        let parts: Vec<&str> = raw_ref.split('.').collect();
        if parts.len() < 3 {
            continue;
        }
        let (library, namespace, class_name, method_name) = if parts.len() >= 4 {
            (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            )
        } else {
            (
                parts[0].to_string(),
                String::new(),
                parts[1].to_string(),
                parts[2].to_string(),
            )
        };
        refs.push(IndirectPluginRef {
            bl_id: row_string(row, "IDNUM"),
            raw_ref: raw_ref.clone(),
            library,
            namespace,
            class_name,
            method_name,
            param_group_id: non_empty(row_string(row, "PGID")),
        });
    }
    refs
}

fn infer_scenario_type(group: &Row, legacy: &LegacyFunctionExport) -> String {
    let group_id = legacy_group_code(group);
    let block_ids: Vec<String> = legacy
        .blocks
        .iter()
        .filter(|row| legacy_group_code(row) == group_id)
        .map(|row| legacy_block_code(row).to_ascii_lowercase())
        .collect();
    if block_ids.iter().any(|id| id.contains("list")) {
        "list".to_string()
    } else if block_ids
        .iter()
        .any(|id| id.contains("detail") || id.contains("mainblock"))
    {
        "form".to_string()
    } else {
        "form".to_string()
    }
}

fn compute_block_field_status(row: &Row) -> &'static str {
    match (
        row_string(row, "GSTATUS").as_str(),
        row_string(row, "BSTATUS").as_str(),
    ) {
        ("1", _) => "readonly",
        ("2", "1") => "readonly",
        ("2", "2") | ("2", "3") | ("3", "2") | ("3", "3") => "editable",
        ("3", "1") => "readonly",
        _ => "disable",
    }
}

fn infer_view_context(block_id: &str) -> &'static str {
    let block_id = block_id.to_ascii_lowercase();
    if block_id.contains("list") {
        "list"
    } else if block_id.contains("detail") {
        "detail"
    } else {
        "form"
    }
}

fn infer_field_data_type(
    legacy: &LegacyFunctionExport,
    table_code: &str,
    field_name: &str,
) -> String {
    legacy
        .tables
        .get(table_code)
        .and_then(|table| {
            table
                .fields
                .iter()
                .find(|field| legacy_field_code(field) == field_name)
                .cloned()
        })
        .map(|field| map_legacy_data_type(&row_string(&field, "FIELDTYPE")))
        .unwrap_or_else(|| "string".to_string())
}

fn map_legacy_data_type(legacy_type: &str) -> String {
    match legacy_type {
        "1" => "int",
        "2" => "decimal",
        "3" | "4" => "string",
        "5" => "date",
        "6" => "bool",
        "7" => "bigint",
        "8" => "text",
        "9" => "binary",
        _ => "string",
    }
    .to_string()
}

fn infer_module_code(feature_code: &str) -> String {
    feature_code
        .split('_')
        .next()
        .unwrap_or("legacy")
        .to_ascii_lowercase()
}

fn infer_action_type(button_id: &str) -> &'static str {
    let upper = button_id.to_ascii_uppercase();
    if upper.contains("SAVE") {
        "save"
    } else if upper.contains("NEW") || upper.contains("ADD") || upper.contains("CREATE") {
        "create"
    } else if upper.contains("EDIT") || upper.contains("UPD") {
        "update"
    } else if upper.contains("DEL") || upper.contains("DELETE") {
        "delete"
    } else if upper.contains("VIEW") || upper.contains("DETAIL") {
        "view"
    } else {
        "execute"
    }
}

fn is_identifier_candidate(field_name: &str) -> bool {
    let upper = field_name.to_ascii_uppercase();
    upper.starts_with("ID_") || upper == "ID" || upper.starts_with("CODE_") || upper == "CODE"
}

fn build_plugin_code(plugin: &PluginRef) -> String {
    if plugin.class_name.is_empty() {
        format!("legacy_{}", plugin.library.to_ascii_lowercase())
    } else {
        format!(
            "legacy_{}_{}",
            plugin.library.to_ascii_lowercase(),
            plugin.class_name.to_ascii_lowercase()
        )
    }
}

fn build_capability_code(plugin: &PluginRef) -> String {
    plugin.method_name.to_ascii_lowercase()
}

fn build_summary_md(
    manifest: &Manifest,
    feature_fields: &[Value],
    scenario_fields: &[Value],
    actions: &[Value],
) -> String {
    let mut text = String::new();
    text.push_str(&format!("# {}\n\n", manifest.feature_code));
    text.push_str(&format!("- 功能名称：{}\n", manifest.feature_name));
    text.push_str(&format!(
        "- 主实体：{}\n",
        manifest
            .main_entity_code
            .clone()
            .unwrap_or_else(|| "-".to_string())
    ));
    text.push_str(&format!("- 场景数：{}\n", manifest.scenario_codes.len()));
    text.push_str(&format!(
        "- 直接实体：{}\n",
        manifest.direct_entity_codes.join(", ")
    ));
    text.push_str(&format!("- 插件入口：{}\n", manifest.plugin_refs.len()));
    text.push_str(&format!(
        "- 间接插件入口：{}\n",
        manifest.indirect_plugin_refs.len()
    ));
    if !manifest.warnings.is_empty() {
        text.push_str("- 风险提示：\n");
        for warning in &manifest.warnings {
            text.push_str(&format!("  - {}\n", warning));
        }
    }
    text.push_str("\n## 字段池\n\n");
    for field in feature_fields.iter().take(20) {
        text.push_str(&format!(
            "- `{}`: {}\n",
            field.get("fieldKey").and_then(Value::as_str).unwrap_or(""),
            field
                .get("displayName")
                .and_then(Value::as_str)
                .unwrap_or("")
        ));
    }
    text.push_str("\n## 场景字段\n\n");
    for field in scenario_fields.iter().take(20) {
        text.push_str(&format!(
            "- `{}` / `{}` / `{}`\n",
            field
                .get("scenarioCode")
                .and_then(Value::as_str)
                .unwrap_or(""),
            field
                .get("featureFieldKey")
                .and_then(Value::as_str)
                .unwrap_or(""),
            field
                .get("viewContext")
                .and_then(Value::as_str)
                .unwrap_or("")
        ));
    }
    text.push_str("\n## 动作\n\n");
    for action in actions {
        text.push_str(&format!(
            "- `{}` / `{}`\n",
            action
                .get("scenarioCode")
                .and_then(Value::as_str)
                .unwrap_or(""),
            action.get("code").and_then(Value::as_str).unwrap_or("")
        ));
    }
    text
}

fn preferred_name(row: &Row) -> String {
    let des1 = row_string(row, "DES1");
    if !des1.is_empty() {
        return des1;
    }
    let name = row_string(row, "NAME");
    if !name.is_empty() {
        return name;
    }
    row_string(row, "IDNUM")
}

fn row_string(row: &Row, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default().trim().to_string()
}

fn row_first_string(row: &Row, keys: &[&str]) -> String {
    keys.iter()
        .map(|key| row_string(row, key))
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

fn legacy_group_code(row: &Row) -> String {
    row_first_string(row, &["GRPID", "GROUPID", "IDNUM"])
}

fn legacy_block_code(row: &Row) -> String {
    row_first_string(row, &["BLKID", "BLOCKID", "BLOCKCODE", "IDNUM"])
}

fn legacy_table_code(row: &Row) -> String {
    row_first_string(row, &["TABLEID", "TBLID", "TABLECODE", "TABLENAME"])
}

fn legacy_field_code(row: &Row) -> String {
    row_first_string(row, &["FIELDNAME", "FIELDID", "FLDNAME", "FIELDCODE"])
}

fn legacy_region_code(row: &Row) -> String {
    row_first_string(row, &["RGNID", "REGIONID", "REGID", "IDNUM"])
}

fn is_main_block(row: &Row) -> bool {
    let block_code = legacy_block_code(row).to_ascii_uppercase();
    block_code.contains("MAINBLOCK")
        || block_code == "BMAIN"
        || parse_boolish(&row_first_string(row, &["ISMAIN", "MAINBLOCK"]))
}

fn infer_main_entity_code(legacy: &LegacyFunctionExport, function: &Row) -> Option<String> {
    legacy
        .blocks
        .iter()
        .find(|block| is_main_block(block))
        .map(legacy_table_code)
        .filter(|code| !code.is_empty())
        .or_else(|| {
            non_empty(row_first_string(
                function,
                &["FUNCOBJ", "MAINENTITY", "TABLEID", "DATATABLE"],
            ))
        })
        .or_else(|| {
            legacy
                .blocks
                .iter()
                .map(legacy_table_code)
                .find(|code| !code.is_empty())
        })
}

fn block_field_table_code(legacy: &LegacyFunctionExport, field: &Row) -> String {
    let direct = legacy_table_code(field);
    if !direct.is_empty() {
        return direct;
    }
    let block_code = legacy_block_code(field);
    let group_code = legacy_group_code(field);
    legacy
        .blocks
        .iter()
        .find(|block| {
            legacy_block_code(block) == block_code
                && (group_code.is_empty() || legacy_group_code(block) == group_code)
        })
        .map(legacy_table_code)
        .unwrap_or_default()
}

fn strip_utf8_bom(input: &str) -> &str {
    input.strip_prefix('\u{feff}').unwrap_or(input)
}

fn parse_boolish(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "y" | "yes"
    )
}

fn parse_i64ish(value: &str) -> Option<i64> {
    value.trim().parse::<i64>().ok()
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn ext_props_to_map(rows: &[Row]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for row in rows {
        let key = row_string(row, "PROPNAME");
        if !key.is_empty() {
            map.insert(key, row_string(row, "PROPVALUE"));
        }
    }
    map
}

fn escape_sql(input: &str) -> String {
    input.replace('\'', "''")
}

fn write_json_file<P: AsRef<Path>, T: Serialize>(path: P, value: &T) -> AppResult<()> {
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, text)?;
    Ok(())
}

fn read_json_value<P: AsRef<Path>>(path: P) -> AppResult<Value> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(strip_utf8_bom(&text))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_row(values: &[(&str, &str)]) -> Row {
        values
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect()
    }

    #[test]
    fn strip_utf8_bom_removes_only_leading_marker() {
        assert_eq!(strip_utf8_bom("\u{feff}CK_B_FAILURE"), "CK_B_FAILURE");
        assert_eq!(strip_utf8_bom("CK_B_FAILURE"), "CK_B_FAILURE");
    }

    #[test]
    fn config_file_accepts_utf8_bom() {
        let path = env::temp_dir().join(format!(
            "legacy-export-bom-config-{}-{}.json",
            std::process::id(),
            now_epoch_seconds()
        ));
        fs::write(
            &path,
            "\u{feff}{\"queryRunner\":{\"type\":\"shell-template\",\"commandTemplate\":\"sqlcmd -i {sql_file}\"},\"output\":{\"defaultDirectory\":\"bom-output\"}}",
        )
        .unwrap();
        let common = CommonArgs {
            config: path.clone(),
            out_dir: PathBuf::from("exports"),
        };

        let (_, output_root) = load_config_and_output_root(&common).unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(output_root, PathBuf::from("bom-output"));
    }

    #[test]
    fn render_shell_template_does_not_double_quote_quoted_placeholder() {
        let path = PathBuf::from("/tmp/legacy export.sql");
        let rendered = render_shell_template("sqlcmd -i \"{sql_file}\"", &path);
        assert_eq!(rendered.matches("legacy export.sql").count(), 1);
        assert!(!rendered.contains("\"'"));
        assert!(!rendered.contains("'\""));
    }

    #[test]
    fn parse_shell_template_passes_sql_file_as_one_unquoted_argument() {
        let path = PathBuf::from(r"C:\Users\Plugin Developer\AppData\Local\Temp\query.sql");
        let (program, args) = parse_shell_template_args(
            r#"sqlcmd -S 127.0.0.1 -d LegacyDb -s "\t" -i "{sql_file}""#,
            &path,
        )
        .unwrap();

        assert_eq!(program, "sqlcmd");
        assert_eq!(args.last().unwrap(), path.to_string_lossy().as_ref());
        assert!(!args.last().unwrap().contains(r#"\""#));
    }

    #[test]
    fn parse_tsv_accepts_sqlcmd_backslash_separator() {
        let rows =
            parse_tsv("IDNUM\\DES1\\FUNCOBJ\n-----\\-----\\-------\nBD_ITEM\\物料信息\\ITM\n")
                .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(row_string(&rows[0], "IDNUM"), "BD_ITEM");
        assert_eq!(row_string(&rows[0], "DES1"), "物料信息");
        assert_eq!(row_string(&rows[0], "FUNCOBJ"), "ITM");
    }

    #[test]
    fn parse_tsv_accepts_pipe_separator() {
        let rows = parse_tsv("IDNUM|DES1\n------|------\nBD_ITEM|物料信息\n").unwrap();
        assert_eq!(row_string(&rows[0], "IDNUM"), "BD_ITEM");
        assert_eq!(row_string(&rows[0], "DES1"), "物料信息");
    }

    #[test]
    fn parse_tsv_accepts_literal_backslash_t_and_bom_header() {
        let rows = parse_tsv(
            "\u{feff}IDNUM\\tDES1\\tFUNCOBJ\n------\\t------\\t------\nBD_ITEM\\t物料信息\\tITM\n",
        )
        .unwrap();
        assert_eq!(row_string(&rows[0], "IDNUM"), "BD_ITEM");
        assert_eq!(row_string(&rows[0], "DES1"), "物料信息");
        assert_eq!(row_string(&rows[0], "FUNCOBJ"), "ITM");
    }

    #[test]
    fn query_output_decoder_accepts_utf16le() {
        let text = "IDNUM|DES1\r\n------|------\r\nBD_ITEM|物料信息\r\n";
        let mut bytes = vec![0xff, 0xfe];
        for unit in text.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let decoded = decode_query_output(&bytes).unwrap();
        let rows = parse_tsv(&decoded).unwrap();
        assert_eq!(row_string(&rows[0], "IDNUM"), "BD_ITEM");
        assert_eq!(row_string(&rows[0], "DES1"), "物料信息");
    }

    #[test]
    fn parse_tsv_preserves_multiline_column_as_valid_value() {
        let rows = parse_tsv(
            "IDNUM|DSVALUE|DES1\n------|-------|----\nCODE_LOC|select code,\nname from location|位置\n",
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(row_string(&rows[0], "IDNUM"), "CODE_LOC");
        assert_eq!(
            row_string(&rows[0], "DSVALUE"),
            "select code,\nname from location"
        );
        assert_eq!(row_string(&rows[0], "DES1"), "位置");
        assert!(serde_json::to_string(&rows).is_ok());
    }

    #[test]
    fn normalize_builds_form_tabs_detail_tables_and_safe_control_fallbacks() {
        let mut tables = BTreeMap::new();
        tables.insert(
            "ITM".to_string(),
            LegacyTableExport {
                table: vec![test_row(&[("IDFIELD", "ID_ITEM"), ("DES1", "物料")])],
                fields: vec![test_row(&[("FIELDNAME", "CODE_ITEM"), ("FIELDTYPE", "4")])],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                ext: Vec::new(),
            },
        );
        tables.insert(
            "BD_B_ITEMUM".to_string(),
            LegacyTableExport {
                table: vec![test_row(&[("IDFIELD", "ID_ITEMUM"), ("DES1", "单位")])],
                fields: vec![
                    test_row(&[("FIELDNAME", "ID_ITEM"), ("FIELDTYPE", "4")]),
                    test_row(&[("FIELDNAME", "CODE_ITEM"), ("FIELDTYPE", "4")]),
                    test_row(&[("FIELDNAME", "CODE_UM"), ("FIELDTYPE", "4")]),
                ],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                ext: Vec::new(),
            },
        );
        let legacy = LegacyFunctionExport {
            function: vec![test_row(&[
                ("IDNUM", "BD_ITEM"),
                ("DES1", "物料信息"),
                ("FUNCOBJ", "ITM"),
            ])],
            function_ext: Vec::new(),
            groups: vec![test_row(&[("GRPID", "detail"), ("DES1", "详情")])],
            blocks: vec![
                test_row(&[
                    ("GRPID", "detail"),
                    ("BLKID", "BMAINBLOCK"),
                    ("TABLEID", "ITM"),
                ]),
                test_row(&[
                    ("GRPID", "detail"),
                    ("BLKID", "BUOM"),
                    ("TABLEID", "BD_B_ITEMUM"),
                    ("DES1", "计量单位"),
                    ("ALLOWCREATE", "1"),
                    ("ALLOWDELETE", "1"),
                ]),
            ],
            events: Vec::new(),
            regions: vec![test_row(&[("REGID", "BD"), ("DES1", "基本信息")])],
            block_fields: vec![
                test_row(&[
                    ("GRPID", "detail"),
                    ("BLKID", "BMAINBLOCK"),
                    ("FIELDNAME", "CODE_ITEM"),
                    ("RGNID", "BD"),
                    ("CTRLTYPE", "8"),
                    ("DSTYPE", "2"),
                    ("DSVALUE", "BD_CATEGORY"),
                    ("GSTATUS", "2"),
                    ("BSTATUS", "2"),
                ]),
                test_row(&[
                    ("GRPID", "detail"),
                    ("BLKID", "BUOM"),
                    ("FIELDNAME", "CODE_UM"),
                    ("CTRLTYPE", "2"),
                    ("DSTYPE", "1"),
                    ("GSTATUS", "2"),
                    ("BSTATUS", "2"),
                ]),
            ],
            buttons: Vec::new(),
            block_field_overrides: Vec::new(),
            tables,
            bl_defs: Vec::new(),
            bl_params: Vec::new(),
        };

        let normalized = normalize(&legacy);
        assert_eq!(normalized.manifest.main_entity_code.as_deref(), Some("ITM"));
        assert_eq!(normalized.feature["fieldCount"], 2);
        assert_eq!(normalized.feature["scenarioCount"], 1);
        let seed = normalized.scenario_update_seed;
        assert_eq!(seed["code"], "detail");
        assert_eq!(seed["metadata"]["formLayout"]["type"], "tabs");
        assert_eq!(
            seed["metadata"]["detailTables"][0]["entityCode"],
            "BD_B_ITEMUM"
        );
        assert_eq!(
            seed["metadata"]["detailTables"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            seed["metadata"]["detailTables"][0]["columns"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            seed["metadata"]["detailTables"][0]["relation"]["parentKey"],
            "CODE_ITEM"
        );
        assert_eq!(
            seed["metadata"]["detailTables"][0]["relation"]["childKey"],
            "CODE_ITEM"
        );
        assert_eq!(seed["fieldGroups"][0]["form"]["componentType"], "q-input");
        assert_eq!(
            seed["fieldGroups"][0]["form"]["extraMetadata"]["suggestedComponent"],
            "relation-picker-field"
        );
    }

    #[test]
    fn menu_tree_seed_lists_parents_before_children() {
        let mut parent = Row::new();
        parent.insert("IDNUM".to_string(), "QMS".to_string());
        parent.insert("DES1".to_string(), "QMS".to_string());
        parent.insert("PARENT".to_string(), "".to_string());
        parent.insert("DSPIDX".to_string(), "1".to_string());

        let mut child = Row::new();
        child.insert("IDNUM".to_string(), "QMS_IQC".to_string());
        child.insert("DES1".to_string(), "IQC".to_string());
        child.insert("PARENT".to_string(), "QMS".to_string());
        child.insert("DSPIDX".to_string(), "1".to_string());
        child.insert("FUNCOBJ".to_string(), "QMS_IQC".to_string());

        let seed = build_menu_tree_seed(&[child, parent]);
        let entries = seed.get("entries").and_then(Value::as_array).unwrap();
        assert_eq!(
            entries[0].get("legacyCode").and_then(Value::as_str),
            Some("QMS")
        );
        assert_eq!(
            entries[1].get("legacyCode").and_then(Value::as_str),
            Some("QMS_IQC")
        );
    }
}
