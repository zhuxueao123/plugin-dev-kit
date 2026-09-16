use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::config::{
    load_config, normalize_base_url, resolve, save_config, ResolvedConfig, StoredConfig,
};
use crate::http::{ApiClient, DeleteResult};
use crate::output::print_success;

#[derive(Parser, Debug)]
#[command(name = "asapflow")]
#[command(about = "Unified CLI for AsapFlow capabilities")]
pub struct Cli {
    #[arg(long, global = true)]
    pub output: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Auth(AuthCommand),
    System(SystemCommand),
    Identity(IdentityCommand),
    Workflow(WorkflowCommand),
    NumberRule(NumberRuleCommand),
    Data(DataCommand),
    Plugin(PluginCommand),
    Configuration(ConfigurationCommand),
    DataSource(DataSourceCommand),
    Dictionary(DictionaryCommand),
    ScheduledJob(ScheduledJobCommand),
    ImportExport(ImportExportCommand),
    FeaturePackage(FeaturePackageCommand),
    Bi(BiCommand),
}

#[derive(Args, Debug)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub command: AuthSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum AuthSubcommands {
    Login(LoginArgs),
    UseToken(UseTokenArgs),
    #[command(name = "whoami")]
    WhoAmI(ConnectionArgs),
    HealthCheck(ConnectionArgs),
}

#[derive(Args, Debug)]
pub struct SystemCommand {
    #[command(subcommand)]
    pub command: SystemSubcommands,
}

#[derive(Args, Debug)]
pub struct IdentityCommand {
    #[command(subcommand)]
    pub command: IdentitySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum SystemSubcommands {
    CreateEntity(CreateJsonInputArgs),
    ListEntities(ListMetadataArgs),
    GetEntity(GetEntityArgs),
    UpdateEntity(UpdateEntityArgs),
    DeleteEntity(DeleteEntityArgs),
    AddEntityFields(AddEntityFieldsArgs),
    UpdateEntityField(UpdateEntityFieldArgs),
    CreateFeature(CreateJsonInputArgs),
    ListFeatures(ListMetadataArgs),
    GetFeature(GetFeatureArgs),
    AddFeatureFields(AddFeatureFieldsArgs),
    UpdateFeature(UpdateFeatureArgs),
    DeleteFeature(DeleteFeatureArgs),
    CreateScenario(FeatureJsonInputArgs),
    UpdateScenario(UpdateScenarioArgs),
    DeleteScenario(DeleteScenarioArgs),
    CreateAction(JsonInputArgs),
    ListActions(ListActionsArgs),
    UpdateAction(UpdateActionArgs),
    DeleteAction(DeleteActionArgs),
    CreateMenu(JsonInputArgs),
    ListMenus(ConnectionArgs),
    UpdateMenu(UpdateMenuArgs),
    DeleteMenu(DeleteMenuArgs),
    CreateActionMenu(JsonInputArgs),
}

#[derive(Subcommand, Debug)]
pub enum IdentitySubcommands {
    ListPermissions(ConnectionArgs),
    ListRoles(ConnectionArgs),
    CreateRole(JsonInputArgs),
    UpdateRole(UpdateRoleArgs),
    DeleteRole(DeleteIdentityRoleArgs),
    ListUsers(ConnectionArgs),
    CreateUser(JsonInputArgs),
    AssignUserRoles(AssignUserRolesArgs),
    GetUserPermissions(GetUserPermissionsArgs),
    GetPasswordPolicy(ConnectionArgs),
    SetPasswordPolicy(JsonInputArgs),
}

#[derive(Args, Debug)]
pub struct DataCommand {
    #[command(subcommand)]
    pub command: DataSubcommands,
}

#[derive(Args, Debug)]
pub struct WorkflowCommand {
    #[command(subcommand)]
    pub command: WorkflowSubcommands,
}

#[derive(Args, Debug)]
pub struct NumberRuleCommand {
    #[command(subcommand)]
    pub command: NumberRuleSubcommands,
}

#[derive(Args, Debug)]
pub struct ConfigurationCommand {
    #[command(subcommand)]
    pub command: ConfigurationSubcommands,
}

#[derive(Args, Debug)]
pub struct DataSourceCommand {
    #[command(subcommand)]
    pub command: DataSourceSubcommands,
}

#[derive(Args, Debug)]
pub struct DictionaryCommand {
    #[command(subcommand)]
    pub command: DictionarySubcommands,
}

#[derive(Args, Debug)]
pub struct ScheduledJobCommand {
    #[command(subcommand)]
    pub command: ScheduledJobSubcommands,
}

#[derive(Args, Debug)]
pub struct ImportExportCommand {
    #[command(subcommand)]
    pub command: ImportExportSubcommands,
}

#[derive(Args, Debug)]
pub struct FeaturePackageCommand {
    #[command(subcommand)]
    pub command: FeaturePackageSubcommands,
}

#[derive(Args, Debug)]
pub struct BiCommand {
    #[command(subcommand)]
    pub command: BiSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum WorkflowSubcommands {
    CreateDefinition(JsonInputArgs),
    UpdateDefinition(UpdateWorkflowDefinitionArgs),
    GetDefinition(GetWorkflowDefinitionArgs),
    ListDefinitions(ConnectionArgs),
    PublishDefinition(PublishWorkflowDefinitionArgs),
    DeleteDefinition(DeleteWorkflowDefinitionArgs),
    ListWorkbench(ListWorkflowWorkbenchArgs),
    StartInstance(StartWorkflowInstanceArgs),
    GetInstance(GetWorkflowInstanceArgs),
    ExecuteTaskAction(ExecuteWorkflowTaskActionArgs),
}

#[derive(Subcommand, Debug)]
pub enum NumberRuleSubcommands {
    Create(JsonInputArgs),
    Update(UpdateNumberRuleArgs),
    Get(GetNumberRuleArgs),
    List(ConnectionArgs),
    Preview(JsonInputArgs),
    BindField(BindNumberRuleFieldArgs),
}

#[derive(Subcommand, Debug)]
pub enum DataSubcommands {
    QueryRecords(EntityInputArgs),
    GetRecord(GetRecordArgs),
    CreateRecord(EntityInputArgs),
    UpdateRecord(UpdateRecordArgs),
    DeleteRecord(DeleteRecordArgs),
    ExecuteAction(ExecuteActionArgs),
}

#[derive(Subcommand, Debug)]
pub enum ConfigurationSubcommands {
    List(ListConfigurationArgs),
    Set(JsonInputArgs),
    Delete(DeleteConfigurationArgs),
    GetFileStorage(ConnectionArgs),
    SetFileStorage(JsonInputArgs),
}

#[derive(Subcommand, Debug)]
pub enum DataSourceSubcommands {
    List(ConnectionArgs),
    Get(GetCodeArgs),
    Validate(JsonInputArgs),
    Create(JsonInputArgs),
    Update(UpdateCodeArgs),
    Delete(DeleteCodeArgs),
}

#[derive(Subcommand, Debug)]
pub enum DictionarySubcommands {
    List(ConnectionArgs),
    Get(GetCodeArgs),
    Create(JsonInputArgs),
    Update(UpdateCodeArgs),
    Delete(DeleteCodeArgs),
}

#[derive(Subcommand, Debug)]
pub enum ScheduledJobSubcommands {
    List(ConnectionArgs),
    Get(GetScheduledJobArgs),
    Create(JsonInputArgs),
    Update(UpdateScheduledJobArgs),
    Delete(GetScheduledJobArgs),
    Run(GetScheduledJobArgs),
    Runs(ScheduledJobRunsArgs),
}

#[derive(Subcommand, Debug)]
pub enum ImportExportSubcommands {
    Config(ImportExportScenarioArgs),
    Template(ImportExportTemplateArgs),
    CreateImportTask(CreateImportTaskArgs),
    CreateExportTask(CreateExportTaskArgs),
    ListTasks(ListImportExportTasksArgs),
    Download(DownloadImportExportTaskArgs),
}

#[derive(Subcommand, Debug)]
pub enum FeaturePackageSubcommands {
    Export(DownloadJsonInputArgs),
    ImportPreview(UploadFileArgs),
    Import(FeaturePackageImportArgs),
    LegacyPreview(LegacyFeaturePackageUploadArgs),
    LegacyConvert(LegacyFeaturePackageConvertArgs),
    LegacyImport(LegacyFeaturePackageImportArgs),
}

#[derive(Subcommand, Debug)]
pub enum BiSubcommands {
    List(ConnectionArgs),
    Get(BiDashboardIdArgs),
    Create(JsonInputArgs),
    Update(BiDashboardJsonInputArgs),
    Delete(BiDashboardIdArgs),
    Publish(BiPublishArgs),
    ExecuteQuery(BiExecuteQueryArgs),
    ListPublished(ConnectionArgs),
    GetPublished(BiDashboardIdArgs),
    Homepage(ConnectionArgs),
    ListHomepageBindings(ConnectionArgs),
    SaveHomepageBinding(JsonInputArgs),
    DeleteHomepageBinding(BiHomepageBindingIdArgs),
}

#[derive(Args, Debug)]
pub struct BiDashboardIdArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub dashboard_id: String,
}

#[derive(Args, Debug)]
pub struct BiDashboardJsonInputArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub dashboard_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct BiPublishArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub dashboard_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct BiExecuteQueryArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub dashboard_id: String,
    #[arg(long, default_value_t = false)]
    pub published: bool,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct BiHomepageBindingIdArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub binding_id: String,
}

#[derive(Args, Debug)]
pub struct PluginCommand {
    #[command(subcommand)]
    pub command: PluginSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum PluginSubcommands {
    Init(PluginInitArgs),
    Validate(PluginValidateArgs),
    ManifestCheck(ManifestCheckArgs),
    BuildFrontend(PluginBuildFrontendArgs),
    Reload(PluginReloadArgs),
    List(ConnectionArgs),
    Pages(ConnectionArgs),
    Invoke(PluginInvokeArgs),
    Pack(PluginPackArgs),
    Publish(PluginPublishArgs),
    ReleaseStatus(PluginReleaseStatusArgs),
    Releases(PluginReleasesArgs),
    Rollback(PluginRollbackArgs),
    WorkspaceStatus(ConnectionArgs),
    PullPackage(PullPluginPackageArgs),
    Package(PackagePluginWorkspaceArgs),
    DeployPackage(DeployPluginPackageArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ConnectionArgs {
    #[arg(long)]
    pub base_url: Option<String>,
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ListMetadataArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub limit: Option<u32>,
}

#[derive(Args, Debug)]
pub struct LoginArgs {
    #[arg(long)]
    pub base_url: String,
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub password: String,
    #[arg(long, default_value_t = true)]
    pub save: bool,
}

#[derive(Args, Debug)]
pub struct UseTokenArgs {
    #[arg(long)]
    pub base_url: String,
    #[arg(long)]
    pub token: String,
}

#[derive(Args, Debug)]
pub struct JsonInputArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DownloadJsonInputArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long)]
    pub output_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct CreateJsonInputArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long, value_enum, default_value_t = IfExistsBehavior::Fail)]
    pub if_exists: IfExistsBehavior,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum IfExistsBehavior {
    Fail,
    Skip,
}

#[derive(Args, Debug)]
pub struct GetEntityArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub code: String,
}

#[derive(Args, Debug)]
pub struct UpdateEntityArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteEntityArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
}

#[derive(Args, Debug)]
pub struct AddEntityFieldsArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateEntityFieldArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub field_id: Option<String>,
    #[arg(long)]
    pub field_code: Option<String>,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct FeatureJsonInputArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct GetFeatureArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub code: String,
}

#[derive(Args, Debug)]
pub struct GetCodeArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub code: String,
}

#[derive(Args, Debug)]
pub struct UpdateCodeArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteCodeArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub code: String,
}

#[derive(Args, Debug)]
pub struct UpdateFeatureArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct AddFeatureFieldsArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteFeatureArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateScenarioArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: String,
    #[arg(long)]
    pub scenario_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteScenarioArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: String,
    #[arg(long)]
    pub scenario_id: String,
}

#[derive(Args, Debug)]
pub struct ListActionsArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_id: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateActionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub action_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteActionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub action_id: String,
}

#[derive(Args, Debug)]
pub struct UpdateMenuArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub menu_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteMenuArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub menu_id: String,
}

#[derive(Args, Debug)]
pub struct EntityInputArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateRoleArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub role_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteIdentityRoleArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub role_id: String,
}

#[derive(Args, Debug)]
pub struct AssignUserRolesArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub user_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct GetUserPermissionsArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub user_id: String,
}

#[derive(Args, Debug)]
pub struct GetRecordArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub record_id: String,
}

#[derive(Args, Debug)]
pub struct UpdateRecordArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub record_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateWorkflowDefinitionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub definition_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteWorkflowDefinitionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub definition_id: String,
}

#[derive(Args, Debug)]
pub struct GetWorkflowDefinitionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub definition_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
}

#[derive(Args, Debug)]
pub struct PublishWorkflowDefinitionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub definition_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListWorkflowWorkbenchArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long, value_enum, default_value_t = WorkflowWorkbenchView::Todo)]
    pub view: WorkflowWorkbenchView,
}

#[derive(Args, Debug)]
pub struct StartWorkflowInstanceArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
    #[arg(long = "cc-user-id")]
    pub cc_user_ids: Vec<String>,
}

#[derive(Args, Debug)]
pub struct GetWorkflowInstanceArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub instance_id: String,
}

#[derive(Args, Debug)]
pub struct ExecuteWorkflowTaskActionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub task_id: String,
    #[arg(long)]
    pub action_code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum WorkflowWorkbenchView {
    Todo,
    Started,
    Done,
    Cc,
    Finished,
}

impl WorkflowWorkbenchView {
    fn as_api_value(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Started => "started",
            Self::Done => "done",
            Self::Cc => "cc",
            Self::Finished => "finished",
        }
    }
}

#[derive(Args, Debug)]
pub struct ListConfigurationArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub prefix: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteConfigurationArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub config_key: String,
}

#[derive(Args, Debug)]
pub struct GetScheduledJobArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub job_id: String,
}

#[derive(Args, Debug)]
pub struct UpdateScheduledJobArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub job_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct ScheduledJobRunsArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub job_id: String,
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct ImportExportScenarioArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_code: String,
    #[arg(long)]
    pub scenario_code: String,
}

#[derive(Args, Debug)]
pub struct ImportExportTemplateArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_code: String,
    #[arg(long)]
    pub scenario_code: String,
    #[arg(long)]
    pub target_code: String,
    #[arg(long)]
    pub output_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct CreateImportTaskArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_code: String,
    #[arg(long)]
    pub scenario_code: String,
    #[arg(long)]
    pub target_code: String,
    #[arg(long, default_value = "auto")]
    pub number_mode: String,
    #[arg(long)]
    pub file: PathBuf,
}

#[derive(Args, Debug)]
pub struct CreateExportTaskArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub feature_code: String,
    #[arg(long)]
    pub scenario_code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListImportExportTasksArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long, default_value_t = 5)]
    pub page_size: u32,
}

#[derive(Args, Debug)]
pub struct DownloadImportExportTaskArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub task_id: String,
    #[arg(long)]
    pub output_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct UploadFileArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub file: PathBuf,
}

#[derive(Args, Debug)]
pub struct FeaturePackageImportArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long, default_value = "Fail")]
    pub strategy: String,
}

#[derive(Args, Debug)]
pub struct LegacyFeaturePackageUploadArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long)]
    pub package_id: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub module: Option<String>,
}

#[derive(Args, Debug)]
pub struct LegacyFeaturePackageConvertArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long)]
    pub output_file: PathBuf,
    #[arg(long)]
    pub package_id: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub module: Option<String>,
}

#[derive(Args, Debug)]
pub struct LegacyFeaturePackageImportArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long, default_value = "Fail")]
    pub strategy: String,
    #[arg(long)]
    pub package_id: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub module: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateNumberRuleArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub rule_id: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct GetNumberRuleArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub rule_id: Option<String>,
    #[arg(long)]
    pub code: Option<String>,
}

#[derive(Args, Debug)]
pub struct BindNumberRuleFieldArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub field_code: String,
    #[arg(long)]
    pub rule_code: String,
    #[arg(long, default_value = "create")]
    pub trigger: String,
    #[arg(long, default_value_t = false)]
    pub allow_manual_override: bool,
}

#[derive(Args, Debug)]
pub struct DeleteRecordArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub record_id: String,
}

#[derive(Args, Debug)]
pub struct ExecuteActionArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub entity_code: String,
    #[arg(long)]
    pub action_code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum PluginKind {
    Backend,
    Frontend,
    Full,
}

#[derive(Args, Debug)]
pub struct PluginInitArgs {
    #[arg(long)]
    pub code: String,
    #[arg(long)]
    pub name: String,
    #[arg(long, value_enum, default_value_t = PluginKind::Full)]
    pub kind: PluginKind,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
}

#[derive(Args, Debug)]
pub struct PluginValidateArgs {
    #[arg(long)]
    pub code: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
}

#[derive(Args, Debug)]
pub struct ManifestCheckArgs {
    #[arg(long)]
    pub path: PathBuf,
}

#[derive(Args, Debug)]
pub struct PluginBuildFrontendArgs {
    #[arg(long)]
    pub working_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct PluginPackArgs {
    #[arg(long)]
    pub code: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub output_file: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub skip_frontend_build: bool,
}

#[derive(Args, Debug)]
pub struct PluginPublishArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub code: Option<String>,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub skip_frontend_build: bool,
    #[arg(long, default_value_t = true)]
    pub wait: bool,
}

#[derive(Args, Debug)]
pub struct PluginReleaseStatusArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub deployment_id: String,
}

#[derive(Args, Debug)]
pub struct PluginReleasesArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub plugin_code: String,
}

#[derive(Args, Debug)]
pub struct PluginRollbackArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub plugin_code: String,
    #[arg(long)]
    pub version: String,
    #[arg(long, default_value_t = true)]
    pub wait: bool,
}

#[derive(Args, Debug)]
pub struct PluginReloadArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub plugin_code: Option<String>,
}

#[derive(Args, Debug)]
pub struct PluginInvokeArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub plugin_code: String,
    #[arg(long)]
    pub capability_code: String,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: Option<String>,
}

#[derive(Args, Debug)]
pub struct PullPluginPackageArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub output_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct PackagePluginWorkspaceArgs {
    #[arg(long, default_value = "plugins")]
    pub source: PathBuf,
    #[arg(long)]
    pub output_file: PathBuf,
}

#[derive(Args, Debug)]
pub struct DeployPluginPackageArgs {
    #[command(flatten)]
    pub connection: ConnectionArgs,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long, default_value = "merge")]
    pub mode: String,
    #[arg(long, default_value_t = false)]
    pub build_frontend: bool,
    #[arg(long, default_value_t = false)]
    pub reload_backend: bool,
    #[arg(long, default_value_t = false)]
    pub strict_docs: bool,
    #[arg(long, default_value_t = true)]
    pub strict_validation: bool,
}

impl Cli {
    pub fn command_name(&self) -> String {
        match &self.command {
            Commands::Auth(command) => format!("auth.{}", auth_command_name(&command.command)),
            Commands::System(command) => {
                format!("system.{}", system_command_name(&command.command))
            }
            Commands::Identity(command) => {
                format!("identity.{}", identity_command_name(&command.command))
            }
            Commands::Workflow(command) => {
                format!("workflow.{}", workflow_command_name(&command.command))
            }
            Commands::NumberRule(command) => {
                format!("number_rule.{}", number_rule_command_name(&command.command))
            }
            Commands::Data(command) => format!("data.{}", data_command_name(&command.command)),
            Commands::Plugin(command) => {
                format!("plugin.{}", plugin_command_name(&command.command))
            }
            Commands::Configuration(command) => {
                format!(
                    "configuration.{}",
                    configuration_command_name(&command.command)
                )
            }
            Commands::DataSource(command) => {
                format!("data_source.{}", data_source_command_name(&command.command))
            }
            Commands::Dictionary(command) => {
                format!("dictionary.{}", dictionary_command_name(&command.command))
            }
            Commands::ScheduledJob(command) => {
                format!(
                    "scheduled_job.{}",
                    scheduled_job_command_name(&command.command)
                )
            }
            Commands::ImportExport(command) => {
                format!(
                    "import_export.{}",
                    import_export_command_name(&command.command)
                )
            }
            Commands::FeaturePackage(command) => {
                format!(
                    "feature_package.{}",
                    feature_package_command_name(&command.command)
                )
            }
            Commands::Bi(command) => format!("bi.{}", bi_command_name(&command.command)),
        }
    }
}

fn auth_command_name(command: &AuthSubcommands) -> &'static str {
    match command {
        AuthSubcommands::Login(_) => "login",
        AuthSubcommands::UseToken(_) => "use_token",
        AuthSubcommands::WhoAmI(_) => "whoami",
        AuthSubcommands::HealthCheck(_) => "health_check",
    }
}

fn system_command_name(command: &SystemSubcommands) -> &'static str {
    match command {
        SystemSubcommands::CreateEntity(_) => "create_entity",
        SystemSubcommands::ListEntities(_) => "list_entities",
        SystemSubcommands::GetEntity(_) => "get_entity",
        SystemSubcommands::UpdateEntity(_) => "update_entity",
        SystemSubcommands::DeleteEntity(_) => "delete_entity",
        SystemSubcommands::AddEntityFields(_) => "add_entity_fields",
        SystemSubcommands::UpdateEntityField(_) => "update_entity_field",
        SystemSubcommands::CreateFeature(_) => "create_feature",
        SystemSubcommands::ListFeatures(_) => "list_features",
        SystemSubcommands::GetFeature(_) => "get_feature",
        SystemSubcommands::AddFeatureFields(_) => "add_feature_fields",
        SystemSubcommands::UpdateFeature(_) => "update_feature",
        SystemSubcommands::DeleteFeature(_) => "delete_feature",
        SystemSubcommands::CreateScenario(_) => "create_scenario",
        SystemSubcommands::UpdateScenario(_) => "update_scenario",
        SystemSubcommands::DeleteScenario(_) => "delete_scenario",
        SystemSubcommands::CreateAction(_) => "create_action",
        SystemSubcommands::ListActions(_) => "list_actions",
        SystemSubcommands::UpdateAction(_) => "update_action",
        SystemSubcommands::DeleteAction(_) => "delete_action",
        SystemSubcommands::CreateMenu(_) => "create_menu",
        SystemSubcommands::ListMenus(_) => "list_menus",
        SystemSubcommands::UpdateMenu(_) => "update_menu",
        SystemSubcommands::DeleteMenu(_) => "delete_menu",
        SystemSubcommands::CreateActionMenu(_) => "create_action_menu",
    }
}

fn identity_command_name(command: &IdentitySubcommands) -> &'static str {
    match command {
        IdentitySubcommands::ListPermissions(_) => "list_permissions",
        IdentitySubcommands::ListRoles(_) => "list_roles",
        IdentitySubcommands::CreateRole(_) => "create_role",
        IdentitySubcommands::UpdateRole(_) => "update_role",
        IdentitySubcommands::DeleteRole(_) => "delete_role",
        IdentitySubcommands::ListUsers(_) => "list_users",
        IdentitySubcommands::CreateUser(_) => "create_user",
        IdentitySubcommands::AssignUserRoles(_) => "assign_user_roles",
        IdentitySubcommands::GetUserPermissions(_) => "get_user_permissions",
        IdentitySubcommands::GetPasswordPolicy(_) => "get_password_policy",
        IdentitySubcommands::SetPasswordPolicy(_) => "set_password_policy",
    }
}

fn workflow_command_name(command: &WorkflowSubcommands) -> &'static str {
    match command {
        WorkflowSubcommands::CreateDefinition(_) => "create_definition",
        WorkflowSubcommands::UpdateDefinition(_) => "update_definition",
        WorkflowSubcommands::GetDefinition(_) => "get_definition",
        WorkflowSubcommands::ListDefinitions(_) => "list_definitions",
        WorkflowSubcommands::PublishDefinition(_) => "publish_definition",
        WorkflowSubcommands::DeleteDefinition(_) => "delete_definition",
        WorkflowSubcommands::ListWorkbench(_) => "list_workbench",
        WorkflowSubcommands::StartInstance(_) => "start_instance",
        WorkflowSubcommands::GetInstance(_) => "get_instance",
        WorkflowSubcommands::ExecuteTaskAction(_) => "execute_task_action",
    }
}

fn number_rule_command_name(command: &NumberRuleSubcommands) -> &'static str {
    match command {
        NumberRuleSubcommands::Create(_) => "create",
        NumberRuleSubcommands::Update(_) => "update",
        NumberRuleSubcommands::Get(_) => "get",
        NumberRuleSubcommands::List(_) => "list",
        NumberRuleSubcommands::Preview(_) => "preview",
        NumberRuleSubcommands::BindField(_) => "bind_field",
    }
}

fn data_command_name(command: &DataSubcommands) -> &'static str {
    match command {
        DataSubcommands::QueryRecords(_) => "query_records",
        DataSubcommands::GetRecord(_) => "get_record",
        DataSubcommands::CreateRecord(_) => "create_record",
        DataSubcommands::UpdateRecord(_) => "update_record",
        DataSubcommands::DeleteRecord(_) => "delete_record",
        DataSubcommands::ExecuteAction(_) => "execute_action",
    }
}

fn plugin_command_name(command: &PluginSubcommands) -> &'static str {
    match command {
        PluginSubcommands::Init(_) => "init",
        PluginSubcommands::Validate(_) => "validate",
        PluginSubcommands::ManifestCheck(_) => "manifest_check",
        PluginSubcommands::BuildFrontend(_) => "build_frontend",
        PluginSubcommands::Reload(_) => "reload",
        PluginSubcommands::List(_) => "list",
        PluginSubcommands::Pages(_) => "pages",
        PluginSubcommands::Invoke(_) => "invoke",
        PluginSubcommands::Pack(_) => "pack",
        PluginSubcommands::Publish(_) => "publish",
        PluginSubcommands::ReleaseStatus(_) => "release_status",
        PluginSubcommands::Releases(_) => "releases",
        PluginSubcommands::Rollback(_) => "rollback",
        PluginSubcommands::WorkspaceStatus(_) => "workspace_status",
        PluginSubcommands::PullPackage(_) => "pull_package",
        PluginSubcommands::Package(_) => "package",
        PluginSubcommands::DeployPackage(_) => "deploy_package",
    }
}

fn configuration_command_name(command: &ConfigurationSubcommands) -> &'static str {
    match command {
        ConfigurationSubcommands::List(_) => "list",
        ConfigurationSubcommands::Set(_) => "set",
        ConfigurationSubcommands::Delete(_) => "delete",
        ConfigurationSubcommands::GetFileStorage(_) => "get_file_storage",
        ConfigurationSubcommands::SetFileStorage(_) => "set_file_storage",
    }
}

fn data_source_command_name(command: &DataSourceSubcommands) -> &'static str {
    match command {
        DataSourceSubcommands::List(_) => "list",
        DataSourceSubcommands::Get(_) => "get",
        DataSourceSubcommands::Validate(_) => "validate",
        DataSourceSubcommands::Create(_) => "create",
        DataSourceSubcommands::Update(_) => "update",
        DataSourceSubcommands::Delete(_) => "delete",
    }
}

fn dictionary_command_name(command: &DictionarySubcommands) -> &'static str {
    match command {
        DictionarySubcommands::List(_) => "list",
        DictionarySubcommands::Get(_) => "get",
        DictionarySubcommands::Create(_) => "create",
        DictionarySubcommands::Update(_) => "update",
        DictionarySubcommands::Delete(_) => "delete",
    }
}

fn scheduled_job_command_name(command: &ScheduledJobSubcommands) -> &'static str {
    match command {
        ScheduledJobSubcommands::List(_) => "list",
        ScheduledJobSubcommands::Get(_) => "get",
        ScheduledJobSubcommands::Create(_) => "create",
        ScheduledJobSubcommands::Update(_) => "update",
        ScheduledJobSubcommands::Delete(_) => "delete",
        ScheduledJobSubcommands::Run(_) => "run",
        ScheduledJobSubcommands::Runs(_) => "runs",
    }
}

fn import_export_command_name(command: &ImportExportSubcommands) -> &'static str {
    match command {
        ImportExportSubcommands::Config(_) => "config",
        ImportExportSubcommands::Template(_) => "template",
        ImportExportSubcommands::CreateImportTask(_) => "create_import_task",
        ImportExportSubcommands::CreateExportTask(_) => "create_export_task",
        ImportExportSubcommands::ListTasks(_) => "list_tasks",
        ImportExportSubcommands::Download(_) => "download",
    }
}

fn feature_package_command_name(command: &FeaturePackageSubcommands) -> &'static str {
    match command {
        FeaturePackageSubcommands::Export(_) => "export",
        FeaturePackageSubcommands::ImportPreview(_) => "import_preview",
        FeaturePackageSubcommands::Import(_) => "import",
        FeaturePackageSubcommands::LegacyPreview(_) => "legacy_preview",
        FeaturePackageSubcommands::LegacyConvert(_) => "legacy_convert",
        FeaturePackageSubcommands::LegacyImport(_) => "legacy_import",
    }
}

fn bi_command_name(command: &BiSubcommands) -> &'static str {
    match command {
        BiSubcommands::List(_) => "list",
        BiSubcommands::Get(_) => "get",
        BiSubcommands::Create(_) => "create",
        BiSubcommands::Update(_) => "update",
        BiSubcommands::Delete(_) => "delete",
        BiSubcommands::Publish(_) => "publish",
        BiSubcommands::ExecuteQuery(_) => "execute_query",
        BiSubcommands::ListPublished(_) => "list_published",
        BiSubcommands::GetPublished(_) => "get_published",
        BiSubcommands::Homepage(_) => "homepage",
        BiSubcommands::ListHomepageBindings(_) => "list_homepage_bindings",
        BiSubcommands::SaveHomepageBinding(_) => "save_homepage_binding",
        BiSubcommands::DeleteHomepageBinding(_) => "delete_homepage_binding",
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredConfigView {
    config_path: String,
    base_url: String,
    token_preview: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginUserInfo {
    id: String,
    user_name: String,
    display_name: String,
    email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    token: Option<String>,
    user: Option<LoginUserInfo>,
    requires_initialization: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct WhoAmIResponse {
    is_authenticated: bool,
    auth_type: String,
    id: Option<String>,
    user_name: Option<String>,
    display_name: Option<String>,
    claims: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginSavedResponse {
    base_url: String,
    token_preview: String,
    user: Option<LoginUserInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginInitResponse {
    plugin_code: String,
    created_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestCheckResponse {
    kind: String,
    path: String,
    plugin_code: String,
    version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginValidateResponse {
    plugin_code: String,
    backend_manifest: Option<ManifestCheckResponse>,
    frontend_manifest: Option<ManifestCheckResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginBuildResponse {
    working_dir: String,
    exit_code: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileWriteResponse {
    path: String,
    bytes: usize,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Auth(command) => run_auth(command),
        Commands::System(command) => run_system(command),
        Commands::Identity(command) => run_identity(command),
        Commands::Workflow(command) => run_workflow(command),
        Commands::NumberRule(command) => run_number_rule(command),
        Commands::Data(command) => run_data(command),
        Commands::Plugin(command) => run_plugin(command),
        Commands::Configuration(command) => run_configuration(command),
        Commands::DataSource(command) => run_data_source(command),
        Commands::Dictionary(command) => run_dictionary(command),
        Commands::ScheduledJob(command) => run_scheduled_job(command),
        Commands::ImportExport(command) => run_import_export(command),
        Commands::FeaturePackage(command) => run_feature_package(command),
        Commands::Bi(command) => run_bi(command),
    }
}

fn run_auth(command: AuthCommand) -> Result<()> {
    match command.command {
        AuthSubcommands::Login(args) => {
            let client = ApiClient::new(ResolvedConfig {
                base_url: normalize_base_url(&args.base_url),
                token: String::new(),
            })?;
            let payload = json!({
                "userName": args.username,
                "password": args.password,
            });
            let result: LoginResponse = client.post_json("/api/identity/login", &payload)?;
            let token = result
                .token
                .clone()
                .ok_or_else(|| anyhow!("Login succeeded but token is missing."))?;

            if args.save {
                let config = StoredConfig {
                    base_url: Some(normalize_base_url(&args.base_url)),
                    token: Some(token.clone()),
                };
                save_config(&config)?;
            }

            print_success(
                "auth.login",
                LoginSavedResponse {
                    base_url: normalize_base_url(&args.base_url),
                    token_preview: preview_token(&token),
                    user: result.user,
                },
            )
        }
        AuthSubcommands::UseToken(args) => {
            let config = StoredConfig {
                base_url: Some(normalize_base_url(&args.base_url)),
                token: Some(args.token.clone()),
            };
            let path = save_config(&config)?;
            let view = StoredConfigView {
                config_path: path.display().to_string(),
                base_url: config.base_url.unwrap_or_default(),
                token_preview: preview_token(&args.token),
            };
            print_success("auth.use_token", view)
        }
        AuthSubcommands::WhoAmI(args) => {
            let client = client_from_connection(&args)?;
            let result: WhoAmIResponse = match client.get_json("/api/identity/whoami") {
                Ok(result) => result,
                Err(error) => {
                    let message = error.to_string();
                    if message.contains("status 401") {
                        return Err(anyhow!(
                            "auth whoami returned 401. Verify --token/--base-url, or run `asapflow auth use-token --base-url <url> --token <token>` and retry."
                        ));
                    }
                    return Err(error);
                }
            };
            print_success("auth.whoami", result)
        }
        AuthSubcommands::HealthCheck(args) => {
            let client = client_for_health(&args)?;
            let result: HealthResponse = client.get_anonymous_json("/health")?;
            print_success("auth.health_check", result)
        }
    }
}

fn run_system(command: SystemCommand) -> Result<()> {
    match command.command {
        SystemSubcommands::CreateEntity(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_entity_payload(raw)?;
            if args.if_exists == IfExistsBehavior::Skip {
                if let Some(existing) = find_existing_by_payload_code(
                    &client,
                    &payload,
                    "entity",
                    "/api/meta/entities",
                )? {
                    return print_success(
                        "system.create_entity",
                        skipped_existing_response("entity", existing),
                    );
                }
            }
            let result: Value =
                client.post_json("/api/admin-sdk/entities/create-with-fields", &payload)?;
            print_success("system.create_entity", result)
        }
        SystemSubcommands::ListEntities(args) => {
            let client = client_from_connection(&args.connection)?;
            let path = metadata_list_path("/api/meta/entities", args.limit);
            let result: Value = client.get_json(&path)?;
            print_success("system.list_entities", result)
        }
        SystemSubcommands::GetEntity(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!("/api/meta/entities/{}", args.code))?;
            print_success("system.get_entity", result)
        }
        SystemSubcommands::UpdateEntity(args) => {
            let client = client_from_connection(&args.connection)?;
            let entity_id = if let Some(entity_id) = args.entity_id {
                entity_id
            } else if let Some(code) = args.code {
                let entity: Value = client.get_json(&format!("/api/meta/entities/{}", code))?;
                entity
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .ok_or_else(|| anyhow!("Entity '{}' does not expose an id.", code))?
            } else {
                return Err(anyhow!(
                    "system update-entity requires either --entity-id or --code."
                ));
            };
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.put_json(&format!("/api/meta/entities/{}", entity_id), &payload)?;
            print_success("system.update_entity", result)
        }
        SystemSubcommands::DeleteEntity(args) => {
            let client = client_from_connection(&args.connection)?;
            let entity_id = resolve_entity_id(&client, args.entity_id, args.code)?;
            let result: DeleteResult =
                client.delete_empty(&format!("/api/meta/entities/{}", entity_id))?;
            print_success("system.delete_entity", result)
        }
        SystemSubcommands::AddEntityFields(args) => {
            let client = client_from_connection(&args.connection)?;
            let entity: Value =
                client.get_json(&format!("/api/meta/entities/{}", args.entity_code))?;
            let entity_id = entity
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("Entity '{}' does not expose an id.", args.entity_code))?;

            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let fields = normalize_add_entity_fields_payload(payload)?;
            let mut created = Vec::new();
            for field in fields {
                let result: Value = client
                    .post_json(&format!("/api/meta/entities/{}/fields", entity_id), &field)?;
                created.push(result);
            }

            print_success(
                "system.add_entity_fields",
                json!({ "entityCode": args.entity_code, "fields": created }),
            )
        }
        SystemSubcommands::UpdateEntityField(args) => {
            let client = client_from_connection(&args.connection)?;
            let entity: Value =
                client.get_json(&format!("/api/meta/entities/{}", args.entity_code))?;
            let entity_id = entity
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("Entity '{}' does not expose an id.", args.entity_code))?;

            let field_id = if let Some(field_id) = args.field_id {
                field_id
            } else if let Some(field_code) = args.field_code {
                entity
                    .get("fields")
                    .and_then(Value::as_array)
                    .and_then(|fields| {
                        fields.iter().find(|field| {
                            field
                                .get("code")
                                .and_then(Value::as_str)
                                .map(|code| code.eq_ignore_ascii_case(&field_code))
                                .unwrap_or(false)
                        })
                    })
                    .and_then(|field| field.get("id"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .ok_or_else(|| {
                        anyhow!(
                            "Field '{}' not found in entity '{}'.",
                            field_code,
                            args.entity_code
                        )
                    })?
            } else {
                return Err(anyhow!(
                    "system update-entity-field requires either --field-id or --field-code."
                ));
            };

            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!("/api/meta/entities/{}/fields/{}", entity_id, field_id),
                &payload,
            )?;
            print_success("system.update_entity_field", result)
        }
        SystemSubcommands::CreateFeature(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_feature_payload(raw)?;
            if args.if_exists == IfExistsBehavior::Skip {
                if let Some(existing) = find_existing_by_payload_code(
                    &client,
                    &payload,
                    "feature",
                    "/api/meta/features",
                )? {
                    return print_success(
                        "system.create_feature",
                        skipped_existing_response("feature", existing),
                    );
                }
            }
            let result: Value = client.post_json(
                "/api/admin-sdk/features/create-with-default-scenarios",
                &payload,
            )?;
            print_success("system.create_feature", result)
        }
        SystemSubcommands::ListFeatures(args) => {
            let client = client_from_connection(&args.connection)?;
            let path = metadata_list_path("/api/meta/features", args.limit);
            let result: Value = client.get_json(&path)?;
            print_success("system.list_features", result)
        }
        SystemSubcommands::GetFeature(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!("/api/meta/features/{}", args.code))?;
            print_success("system.get_feature", result)
        }
        SystemSubcommands::AddFeatureFields(args) => {
            let client = client_from_connection(&args.connection)?;
            let feature_id = resolve_feature_id(&client, args.feature_id, args.code)?;
            let existing_feature = get_feature_by_id(&client, &feature_id)?;

            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let fields = normalize_add_feature_fields_payload(payload)?;
            let synced = sync_feature_fields(&client, &feature_id, &existing_feature, fields)?;

            print_success(
                "system.add_feature_fields",
                json!({ "featureId": feature_id, "fields": synced }),
            )
        }
        SystemSubcommands::UpdateFeature(args) => {
            let client = client_from_connection(&args.connection)?;
            let existing_feature: Value = if let Some(feature_id) = args.feature_id.clone() {
                get_feature_by_id(&client, &feature_id)?
            } else if let Some(code) = args.code.clone() {
                client.get_json(&format!("/api/meta/features/{}", code))?
            } else {
                return Err(anyhow!(
                    "system update-feature requires either --feature-id or --code."
                ));
            };
            let feature_id = existing_feature
                .get("id")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("Feature does not expose an id."))?;

            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let fields_to_sync = normalize_update_feature_field_changes(&raw)?;
            let payload = normalize_update_feature_payload(&existing_feature, raw)?;
            let result: Value =
                client.put_json(&format!("/api/meta/features/{}", feature_id), &payload)?;

            if !fields_to_sync.is_empty() {
                let _ =
                    sync_feature_fields(&client, &feature_id, &existing_feature, fields_to_sync)?;
            }

            print_success("system.update_feature", result)
        }
        SystemSubcommands::DeleteFeature(args) => {
            let client = client_from_connection(&args.connection)?;
            let feature_id = resolve_feature_id(&client, args.feature_id, args.code)?;
            let result: DeleteResult =
                client.delete_empty(&format!("/api/meta/features/{}", feature_id))?;
            print_success("system.delete_feature", result)
        }
        SystemSubcommands::CreateScenario(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json(
                &format!("/api/meta/features/{}/scenarios", args.feature_id),
                &payload,
            )?;
            print_success("system.create_scenario", result)
        }
        SystemSubcommands::UpdateScenario(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!(
                    "/api/meta/features/{}/scenarios/{}",
                    args.feature_id, args.scenario_id
                ),
                &payload,
            )?;
            print_success("system.update_scenario", result)
        }
        SystemSubcommands::DeleteScenario(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/meta/features/{}/scenarios/{}",
                args.feature_id, args.scenario_id
            ))?;
            print_success("system.delete_scenario", result)
        }
        SystemSubcommands::CreateAction(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_action_payload(raw)?;
            let result: Value = client.post_json("/api/meta/actions", &payload)?;
            print_success("system.create_action", result)
        }
        SystemSubcommands::ListActions(args) => {
            let client = client_from_connection(&args.connection)?;
            let path = if let Some(feature_id) = args.feature_id {
                format!("/api/meta/actions?featureId={feature_id}")
            } else {
                "/api/meta/actions".to_string()
            };
            let result: Value = client.get_json(&path)?;
            print_success("system.list_actions", result)
        }
        SystemSubcommands::UpdateAction(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_action_payload(raw)?;
            let result: Value =
                client.put_json(&format!("/api/meta/actions/{}", args.action_id), &payload)?;
            print_success("system.update_action", result)
        }
        SystemSubcommands::DeleteAction(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult =
                client.delete_empty(&format!("/api/meta/actions/{}", args.action_id))?;
            print_success("system.delete_action", result)
        }
        SystemSubcommands::CreateMenu(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_menu_payload(raw)?;
            let result: Value = client.post_json("/api/system/menus", &payload)?;
            print_success("system.create_menu", result)
        }
        SystemSubcommands::ListMenus(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/system/menus")?;
            print_success("system.list_menus", result)
        }
        SystemSubcommands::UpdateMenu(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_menu_payload(raw)?;
            let result: Value =
                client.put_json(&format!("/api/system/menus/{}", args.menu_id), &payload)?;
            print_success("system.update_menu", result)
        }
        SystemSubcommands::DeleteMenu(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult =
                client.delete_empty(&format!("/api/system/menus/{}", args.menu_id))?;
            print_success("system.delete_menu", result)
        }
        SystemSubcommands::CreateActionMenu(args) => {
            let client = client_from_connection(&args.connection)?;
            let raw = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let payload = normalize_create_action_menu_payload(raw)?;
            let result: Value =
                client.post_json("/api/admin-sdk/actions/create-with-menu", &payload)?;
            print_success("system.create_action_menu", result)
        }
    }
}

fn run_identity(command: IdentityCommand) -> Result<()> {
    match command.command {
        IdentitySubcommands::ListPermissions(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/identity/permissions")?;
            print_success("identity.list_permissions", result)
        }
        IdentitySubcommands::ListRoles(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/identity/roles")?;
            print_success("identity.list_roles", result)
        }
        IdentitySubcommands::CreateRole(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/identity/roles", &payload)?;
            print_success("identity.create_role", result)
        }
        IdentitySubcommands::UpdateRole(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.put_json(&format!("/api/identity/roles/{}", args.role_id), &payload)?;
            print_success("identity.update_role", result)
        }
        IdentitySubcommands::DeleteRole(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult =
                client.delete_empty(&format!("/api/identity/roles/{}", args.role_id))?;
            print_success("identity.delete_role", result)
        }
        IdentitySubcommands::ListUsers(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/identity/users")?;
            print_success("identity.list_users", result)
        }
        IdentitySubcommands::CreateUser(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/identity/users", &payload)?;
            print_success("identity.create_user", result)
        }
        IdentitySubcommands::AssignUserRoles(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: DeleteResult = client.put_empty(
                &format!("/api/identity/users/{}/roles", args.user_id),
                &payload,
            )?;
            print_success("identity.assign_user_roles", result)
        }
        IdentitySubcommands::GetUserPermissions(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value =
                client.get_json(&format!("/api/identity/users/{}/permissions", args.user_id))?;
            print_success("identity.get_user_permissions", result)
        }
        IdentitySubcommands::GetPasswordPolicy(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/identity/password-policy")?;
            print_success("identity.get_password_policy", result)
        }
        IdentitySubcommands::SetPasswordPolicy(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json("/api/identity/password-policy", &payload)?;
            print_success("identity.set_password_policy", result)
        }
    }
}

fn run_data(command: DataCommand) -> Result<()> {
    match command.command {
        DataSubcommands::QueryRecords(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.post_json(&format!("/api/data/{}/query", args.entity_code), &payload)?;
            print_success("data.query_records", result)
        }
        DataSubcommands::GetRecord(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/data/{}/{}",
                args.entity_code, args.record_id
            ))?;
            print_success("data.get_record", result)
        }
        DataSubcommands::CreateRecord(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.post_json(&format!("/api/data/{}", args.entity_code), &payload)?;
            print_success("data.create_record", result)
        }
        DataSubcommands::UpdateRecord(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!("/api/data/{}/{}", args.entity_code, args.record_id),
                &payload,
            )?;
            print_success("data.update_record", result)
        }
        DataSubcommands::DeleteRecord(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/data/{}/{}",
                args.entity_code, args.record_id
            ))?;
            print_success("data.delete_record", result)
        }
        DataSubcommands::ExecuteAction(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json(
                &format!(
                    "/api/data/{}/actions/{}/execute",
                    args.entity_code, args.action_code
                ),
                &payload,
            )?;
            print_success("data.execute_action", result)
        }
    }
}

fn run_workflow(command: WorkflowCommand) -> Result<()> {
    match command.command {
        WorkflowSubcommands::CreateDefinition(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/workflows/definitions", &payload)?;
            print_success("workflow.create_definition", result)
        }
        WorkflowSubcommands::UpdateDefinition(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!("/api/workflows/definitions/{}", args.definition_id),
                &payload,
            )?;
            print_success("workflow.update_definition", result)
        }
        WorkflowSubcommands::GetDefinition(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = if let Some(definition_id) = args.definition_id {
                client.get_json(&format!("/api/workflows/definitions/{}", definition_id))?
            } else if let Some(code) = args.code {
                let definitions: Value = client.get_json("/api/workflows/definitions")?;
                select_definition_by_code(definitions, &code)?
            } else {
                return Err(anyhow!(
                    "workflow get-definition requires either --definition-id or --code."
                ));
            };

            print_success("workflow.get_definition", result)
        }
        WorkflowSubcommands::ListDefinitions(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/workflows/definitions")?;
            print_success("workflow.list_definitions", result)
        }
        WorkflowSubcommands::PublishDefinition(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = match (args.input.as_deref(), args.json.as_deref()) {
                (None, None) => json!({ "versionNotes": "" }),
                _ => load_json_input(args.input.as_deref(), args.json.as_deref())?,
            };
            let result: Value = client.post_json(
                &format!("/api/workflows/definitions/{}/publish", args.definition_id),
                &payload,
            )?;
            print_success("workflow.publish_definition", result)
        }
        WorkflowSubcommands::DeleteDefinition(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/workflows/definitions/{}",
                args.definition_id
            ))?;
            print_success("workflow.delete_definition", result)
        }
        WorkflowSubcommands::ListWorkbench(args) => {
            let client = client_from_connection(&args.connection)?;
            let path = format!("/api/workflows/workbench?view={}", args.view.as_api_value());
            let result: Value = client.get_json(&path)?;
            print_success("workflow.list_workbench", result)
        }
        WorkflowSubcommands::StartInstance(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = with_workflow_cc_user_ids(
                load_json_input(args.input.as_deref(), args.json.as_deref())?,
                &args.cc_user_ids,
            )?;
            let result: Value = client.post_json("/api/workflows/instances/start", &payload)?;
            print_success("workflow.start_instance", result)
        }
        WorkflowSubcommands::GetInstance(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/workflows/instances/{}/approval-context",
                args.instance_id
            ))?;
            print_success("workflow.get_instance", result)
        }
        WorkflowSubcommands::ExecuteTaskAction(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json(
                &format!(
                    "/api/workflows/tasks/{}/actions/{}",
                    args.task_id, args.action_code
                ),
                &payload,
            )?;
            print_success("workflow.execute_task_action", result)
        }
    }
}

fn run_number_rule(command: NumberRuleCommand) -> Result<()> {
    match command.command {
        NumberRuleSubcommands::Create(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/number-rules", &payload)?;
            print_success("number_rule.create", result)
        }
        NumberRuleSubcommands::Update(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.put_json(&format!("/api/number-rules/{}", args.rule_id), &payload)?;
            print_success("number_rule.update", result)
        }
        NumberRuleSubcommands::Get(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = if let Some(rule_id) = args.rule_id {
                client.get_json(&format!("/api/number-rules/{}", rule_id))?
            } else if let Some(code) = args.code {
                let rules: Value = client.get_json("/api/number-rules")?;
                select_rule_by_code(rules, &code)?
            } else {
                return Err(anyhow!(
                    "number-rule get requires either --rule-id or --code."
                ));
            };

            print_success("number_rule.get", result)
        }
        NumberRuleSubcommands::List(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/number-rules")?;
            print_success("number_rule.list", result)
        }
        NumberRuleSubcommands::Preview(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/number-rules/preview", &payload)?;
            print_success("number_rule.preview", result)
        }
        NumberRuleSubcommands::BindField(args) => {
            let client = client_from_connection(&args.connection)?;
            let entity: Value =
                client.get_json(&format!("/api/meta/entities/{}", args.entity_code))?;
            let bind_result = bind_number_rule_to_field(
                &client,
                &entity,
                &args.entity_code,
                &args.field_code,
                &args.rule_code,
                &args.trigger,
                args.allow_manual_override,
            )?;
            print_success("number_rule.bind_field", bind_result)
        }
    }
}

fn run_plugin(command: PluginCommand) -> Result<()> {
    match command.command {
        PluginSubcommands::Init(args) => {
            let mut created = Vec::new();
            let workspace = if args.root.join("plugin-workspace").is_dir() {
                args.root.join("plugin-workspace")
            } else {
                args.root.join("plugins")
            };
            if matches!(args.kind, PluginKind::Backend | PluginKind::Full) {
                let backend_dir = workspace.join("backend").join(&args.code);
                create_backend_plugin(&backend_dir, &args.code, &args.name)?;
                created.push(backend_dir.display().to_string());
            }
            if matches!(args.kind, PluginKind::Frontend | PluginKind::Full) {
                let frontend_dir = workspace
                    .join("frontend")
                    .join("src")
                    .join("pages")
                    .join(&args.code);
                create_frontend_plugin(&frontend_dir, &args.code, &args.name)?;
                created.push(frontend_dir.display().to_string());
            }

            print_success(
                "plugin.init",
                PluginInitResponse {
                    plugin_code: args.code,
                    created_paths: created,
                },
            )
        }
        PluginSubcommands::Validate(args) => {
            let workspace = resolve_plugin_workspace_for_code(&args.root, &args.code)?;
            let backend_manifest_path = workspace
                .join("backend")
                .join(&args.code)
                .join("manifest.json");
            let frontend_manifest_path = workspace
                .join("frontend")
                .join("src")
                .join("pages")
                .join(&args.code)
                .join("manifest.json");

            let backend_manifest = if backend_manifest_path.exists() {
                Some(validate_backend_manifest(&backend_manifest_path)?)
            } else {
                None
            };

            let frontend_manifest = if frontend_manifest_path.exists() {
                Some(validate_frontend_manifest(&frontend_manifest_path)?)
            } else {
                None
            };

            if backend_manifest.is_none() && frontend_manifest.is_none() {
                return Err(anyhow!(
                    "Plugin '{}' not found under plugins/backend or plugins/frontend.",
                    args.code
                ));
            }

            print_success(
                "plugin.validate",
                PluginValidateResponse {
                    plugin_code: args.code,
                    backend_manifest,
                    frontend_manifest,
                },
            )
        }
        PluginSubcommands::ManifestCheck(args) => {
            let result = validate_manifest_by_path(&args.path)?;
            print_success("plugin.manifest_check", result)
        }
        PluginSubcommands::BuildFrontend(args) => {
            let working_dir = resolve_frontend_working_dir(args.working_dir.as_deref())?;
            let status = Command::new(npm_program())
                .arg("run")
                .arg("build")
                .current_dir(&working_dir)
                .status()
                .with_context(|| format!("Failed to run npm build in {}", working_dir.display()))?;

            if !status.success() {
                return Err(anyhow!(
                    "Frontend plugin build failed with exit code {:?}.",
                    status.code()
                ));
            }

            print_success(
                "plugin.build_frontend",
                PluginBuildResponse {
                    working_dir: working_dir.display().to_string(),
                    exit_code: status.code().unwrap_or(0),
                },
            )
        }
        PluginSubcommands::Reload(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = if let Some(plugin_code) = args.plugin_code {
                client.post_json(&format!("/api/plugins/reload/{}", plugin_code), &json!({}))?
            } else {
                client.post_json("/api/plugins/reload", &json!({}))?
            };
            print_success("plugin.reload", result)
        }
        PluginSubcommands::List(args) => {
            let client = client_from_connection(&args)?;
            let manifests: Value = client.get_json("/api/plugins/capabilities")?;
            print_success("plugin.list", manifests)
        }
        PluginSubcommands::Pages(args) => {
            let client = client_from_connection(&args)?;
            let pages: Value = client.get_json("/api/plugins/pages")?;
            print_success("plugin.pages", pages)
        }
        PluginSubcommands::Invoke(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json(
                &format!(
                    "/api/plugins/{}/capabilities/{}/invoke",
                    encode_path_segment(&args.plugin_code),
                    encode_path_segment(&args.capability_code)
                ),
                &payload,
            )?;
            print_success("plugin.invoke", result)
        }
        PluginSubcommands::Pack(args) => {
            let result = build_plugin_release_package(
                &args.root,
                &args.code,
                args.output_file.as_deref(),
                !args.skip_frontend_build,
            )?;
            print_success("plugin.pack", result)
        }
        PluginSubcommands::Publish(args) => {
            let client = client_from_connection(&args.connection)?;
            let mut temporary_package = None;
            let package_path = if let Some(file) = args.file {
                file
            } else {
                let code = args
                    .code
                    .as_deref()
                    .ok_or_else(|| anyhow!("Either --file or --code is required."))?;
                let temp_dir = std::env::temp_dir().join(format!(
                    "asapflow-plugin-publish-{}-{}",
                    std::process::id(),
                    unix_timestamp()
                ));
                fs::create_dir_all(&temp_dir)?;
                let result = build_plugin_release_package(
                    &args.root,
                    code,
                    Some(&temp_dir.join("release.afplugin")),
                    !args.skip_frontend_build,
                )?;
                let path = PathBuf::from(result.path);
                temporary_package = Some(temp_dir);
                path
            };
            let queued: Value =
                client.post_multipart_json("/api/plugins/releases", "file", &package_path, &[])?;
            let result = if args.wait {
                wait_for_plugin_deployment(&client, &queued)?
            } else {
                queued
            };
            if let Some(temp_dir) = temporary_package {
                let _ = fs::remove_dir_all(temp_dir);
            }
            print_success("plugin.publish", result)
        }
        PluginSubcommands::ReleaseStatus(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/plugins/releases/{}",
                encode_path_segment(&args.deployment_id)
            ))?;
            print_success("plugin.release_status", result)
        }
        PluginSubcommands::Releases(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/plugins/{}/releases",
                encode_path_segment(&args.plugin_code)
            ))?;
            print_success("plugin.releases", result)
        }
        PluginSubcommands::Rollback(args) => {
            let client = client_from_connection(&args.connection)?;
            let queued: Value = client.post_json(
                &format!(
                    "/api/plugins/{}/releases/{}/rollback",
                    encode_path_segment(&args.plugin_code),
                    encode_path_segment(&args.version)
                ),
                &json!({}),
            )?;
            let result = if args.wait {
                wait_for_plugin_deployment(&client, &queued)?
            } else {
                queued
            };
            print_success("plugin.rollback", result)
        }
        PluginSubcommands::WorkspaceStatus(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/plugins/workspace/status")?;
            print_success("plugin.workspace_status", result)
        }
        PluginSubcommands::PullPackage(args) => {
            let client = client_from_connection(&args.connection)?;
            let bytes = client.get_bytes("/api/plugins/package/download")?;
            let written = write_binary_output(&args.output_file, bytes)?;
            print_success("plugin.pull_package", written)
        }
        PluginSubcommands::Package(args) => {
            let written = build_plugin_workspace_package(&args.source, &args.output_file)?;
            print_success("plugin.package", written)
        }
        PluginSubcommands::DeployPackage(args) => {
            let client = client_from_connection(&args.connection)?;
            let fields = vec![
                ("mode", args.mode),
                ("buildFrontend", args.build_frontend.to_string()),
                ("reloadBackend", args.reload_backend.to_string()),
                ("strictDocs", args.strict_docs.to_string()),
                ("strictValidation", args.strict_validation.to_string()),
            ];
            let result: Value = client.post_multipart_json(
                "/api/plugins/package/deploy",
                "file",
                &args.file,
                &fields,
            )?;
            print_success("plugin.deploy_package", result)
        }
    }
}

fn run_configuration(command: ConfigurationCommand) -> Result<()> {
    match command.command {
        ConfigurationSubcommands::List(args) => {
            let client = client_from_connection(&args.connection)?;
            let path = match args.prefix {
                Some(prefix) => format!(
                    "/api/system/configuration?prefix={}",
                    encode_query_value(&prefix)
                ),
                None => "/api/system/configuration".to_string(),
            };
            let result: Value = client.get_json(&path)?;
            print_success("configuration.list", result)
        }
        ConfigurationSubcommands::Set(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/system/configuration", &payload)?;
            print_success("configuration.set", result)
        }
        ConfigurationSubcommands::Delete(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/system/configuration/{}",
                encode_path_segment(&args.config_key)
            ))?;
            print_success("configuration.delete", result)
        }
        ConfigurationSubcommands::GetFileStorage(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/system/configuration/file-storage")?;
            print_success("configuration.get_file_storage", result)
        }
        ConfigurationSubcommands::SetFileStorage(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.put_json("/api/system/configuration/file-storage", &payload)?;
            print_success("configuration.set_file_storage", result)
        }
    }
}

fn run_data_source(command: DataSourceCommand) -> Result<()> {
    match command.command {
        DataSourceSubcommands::List(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/system/configuration/data-sources")?;
            print_success("data_source.list", result)
        }
        DataSourceSubcommands::Get(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/system/configuration/data-sources/{}",
                encode_path_segment(&args.code)
            ))?;
            print_success("data_source.get", result)
        }
        DataSourceSubcommands::Validate(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.post_json("/api/system/configuration/data-sources/validate", &payload)?;
            print_success("data_source.validate", result)
        }
        DataSourceSubcommands::Create(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.post_json("/api/system/configuration/data-sources", &payload)?;
            print_success("data_source.create", result)
        }
        DataSourceSubcommands::Update(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!(
                    "/api/system/configuration/data-sources/{}",
                    encode_path_segment(&args.code)
                ),
                &payload,
            )?;
            print_success("data_source.update", result)
        }
        DataSourceSubcommands::Delete(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/system/configuration/data-sources/{}",
                encode_path_segment(&args.code)
            ))?;
            print_success("data_source.delete", result)
        }
    }
}

fn run_dictionary(command: DictionaryCommand) -> Result<()> {
    match command.command {
        DictionarySubcommands::List(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/system/dictionaries")?;
            print_success("dictionary.list", result)
        }
        DictionarySubcommands::Get(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/system/dictionaries/{}",
                encode_path_segment(&args.code)
            ))?;
            print_success("dictionary.get", result)
        }
        DictionarySubcommands::Create(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/system/dictionaries", &payload)?;
            print_success("dictionary.create", result)
        }
        DictionarySubcommands::Update(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!(
                    "/api/system/dictionaries/{}",
                    encode_path_segment(&args.code)
                ),
                &payload,
            )?;
            print_success("dictionary.update", result)
        }
        DictionarySubcommands::Delete(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/system/dictionaries/{}",
                encode_path_segment(&args.code)
            ))?;
            print_success("dictionary.delete", result)
        }
    }
}

fn run_scheduled_job(command: ScheduledJobCommand) -> Result<()> {
    match command.command {
        ScheduledJobSubcommands::List(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/scheduled-jobs")?;
            print_success("scheduled_job.list", result)
        }
        ScheduledJobSubcommands::Get(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!("/api/scheduled-jobs/{}", args.job_id))?;
            print_success("scheduled_job.get", result)
        }
        ScheduledJobSubcommands::Create(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/scheduled-jobs", &payload)?;
            print_success("scheduled_job.create", result)
        }
        ScheduledJobSubcommands::Update(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value =
                client.put_json(&format!("/api/scheduled-jobs/{}", args.job_id), &payload)?;
            print_success("scheduled_job.update", result)
        }
        ScheduledJobSubcommands::Delete(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult =
                client.delete_empty(&format!("/api/scheduled-jobs/{}", args.job_id))?;
            print_success("scheduled_job.delete", result)
        }
        ScheduledJobSubcommands::Run(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.post_json(
                &format!("/api/scheduled-jobs/{}/run", args.job_id),
                &json!({}),
            )?;
            print_success("scheduled_job.run", result)
        }
        ScheduledJobSubcommands::Runs(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/scheduled-jobs/{}/runs?limit={}",
                args.job_id, args.limit
            ))?;
            print_success("scheduled_job.runs", result)
        }
    }
}

fn run_import_export(command: ImportExportCommand) -> Result<()> {
    match command.command {
        ImportExportSubcommands::Config(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/import-export/features/{}/scenarios/{}/config",
                encode_path_segment(&args.feature_code),
                encode_path_segment(&args.scenario_code)
            ))?;
            print_success("import_export.config", result)
        }
        ImportExportSubcommands::Template(args) => {
            let client = client_from_connection(&args.connection)?;
            let bytes = client.get_bytes(&format!(
                "/api/import-export/features/{}/scenarios/{}/import-targets/{}/template",
                encode_path_segment(&args.feature_code),
                encode_path_segment(&args.scenario_code),
                encode_path_segment(&args.target_code)
            ))?;
            let written = write_binary_output(&args.output_file, bytes)?;
            print_success("import_export.template", written)
        }
        ImportExportSubcommands::CreateImportTask(args) => {
            let client = client_from_connection(&args.connection)?;
            let fields = vec![
                ("targetCode", args.target_code),
                ("numberMode", args.number_mode),
            ];
            let result: Value = client.post_multipart_json(
                &format!(
                    "/api/import-export/features/{}/scenarios/{}/import-tasks",
                    encode_path_segment(&args.feature_code),
                    encode_path_segment(&args.scenario_code)
                ),
                "file",
                &args.file,
                &fields,
            )?;
            print_success("import_export.create_import_task", result)
        }
        ImportExportSubcommands::CreateExportTask(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json(
                &format!(
                    "/api/import-export/features/{}/scenarios/{}/export-tasks",
                    encode_path_segment(&args.feature_code),
                    encode_path_segment(&args.scenario_code)
                ),
                &payload,
            )?;
            print_success("import_export.create_export_task", result)
        }
        ImportExportSubcommands::ListTasks(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/import-export/tasks?page={}&pageSize={}",
                args.page, args.page_size
            ))?;
            print_success("import_export.list_tasks", result)
        }
        ImportExportSubcommands::Download(args) => {
            let client = client_from_connection(&args.connection)?;
            let bytes = client.get_bytes(&format!(
                "/api/import-export/tasks/{}/download",
                args.task_id
            ))?;
            let written = write_binary_output(&args.output_file, bytes)?;
            print_success("import_export.download", written)
        }
    }
}

fn run_feature_package(command: FeaturePackageCommand) -> Result<()> {
    match command.command {
        FeaturePackageSubcommands::Export(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let bytes = client.post_json_bytes("/api/feature-packages/export", &payload)?;
            let written = write_binary_output(&args.output_file, bytes)?;
            print_success("feature_package.export", written)
        }
        FeaturePackageSubcommands::ImportPreview(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.post_multipart_json(
                "/api/feature-packages/import/preview",
                "file",
                &args.file,
                &[],
            )?;
            print_success("feature_package.import_preview", result)
        }
        FeaturePackageSubcommands::Import(args) => {
            let client = client_from_connection(&args.connection)?;
            let fields = vec![("strategy", args.strategy)];
            let result: Value = client.post_multipart_json(
                "/api/feature-packages/import",
                "file",
                &args.file,
                &fields,
            )?;
            print_success("feature_package.import", result)
        }
        FeaturePackageSubcommands::LegacyPreview(args) => {
            let client = client_from_connection(&args.connection)?;
            let fields = legacy_convert_fields(
                args.package_id.as_deref(),
                args.name.as_deref(),
                args.version.as_deref(),
                args.module.as_deref(),
            );
            let result: Value = client.post_multipart_json(
                "/api/feature-packages/legacy/import/preview",
                "file",
                &args.file,
                &fields,
            )?;
            print_success("feature_package.legacy_preview", result)
        }
        FeaturePackageSubcommands::LegacyConvert(args) => {
            let client = client_from_connection(&args.connection)?;
            let fields = legacy_convert_fields(
                args.package_id.as_deref(),
                args.name.as_deref(),
                args.version.as_deref(),
                args.module.as_deref(),
            );
            let bytes = client.post_multipart_bytes(
                "/api/feature-packages/legacy/convert",
                "file",
                &args.file,
                &fields,
            )?;
            let written = write_binary_output(&args.output_file, bytes)?;
            print_success("feature_package.legacy_convert", written)
        }
        FeaturePackageSubcommands::LegacyImport(args) => {
            let client = client_from_connection(&args.connection)?;
            let mut fields = vec![("strategy", args.strategy)];
            fields.extend(legacy_convert_fields(
                args.package_id.as_deref(),
                args.name.as_deref(),
                args.version.as_deref(),
                args.module.as_deref(),
            ));
            let result: Value = client.post_multipart_json(
                "/api/feature-packages/legacy/import",
                "file",
                &args.file,
                &fields,
            )?;
            print_success("feature_package.legacy_import", result)
        }
    }
}

fn run_bi(command: BiCommand) -> Result<()> {
    match command.command {
        BiSubcommands::List(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/bi/dashboards")?;
            print_success("bi.list", result)
        }
        BiSubcommands::Get(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/bi/dashboards/{}",
                encode_path_segment(&args.dashboard_id)
            ))?;
            print_success("bi.get", result)
        }
        BiSubcommands::Create(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.post_json("/api/bi/dashboards", &payload)?;
            print_success("bi.create", result)
        }
        BiSubcommands::Update(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json(
                &format!(
                    "/api/bi/dashboards/{}",
                    encode_path_segment(&args.dashboard_id)
                ),
                &payload,
            )?;
            print_success("bi.update", result)
        }
        BiSubcommands::Delete(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/bi/dashboards/{}",
                encode_path_segment(&args.dashboard_id)
            ))?;
            print_success("bi.delete", result)
        }
        BiSubcommands::Publish(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = if args.input.is_some() || args.json.is_some() {
                load_json_input(args.input.as_deref(), args.json.as_deref())?
            } else {
                json!({})
            };
            let result: Value = client.post_json(
                &format!(
                    "/api/bi/dashboards/{}/publish",
                    encode_path_segment(&args.dashboard_id)
                ),
                &payload,
            )?;
            print_success("bi.publish", result)
        }
        BiSubcommands::ExecuteQuery(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let path = if args.published {
                format!(
                    "/api/bi/runtime/dashboards/{}/queries/execute",
                    encode_path_segment(&args.dashboard_id)
                )
            } else {
                format!(
                    "/api/bi/dashboards/{}/queries/execute?draft=true",
                    encode_path_segment(&args.dashboard_id)
                )
            };
            let result: Value = client.post_json(&path, &payload)?;
            print_success("bi.execute_query", result)
        }
        BiSubcommands::ListPublished(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/bi/runtime/dashboards")?;
            print_success("bi.list_published", result)
        }
        BiSubcommands::GetPublished(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: Value = client.get_json(&format!(
                "/api/bi/runtime/dashboards/{}",
                encode_path_segment(&args.dashboard_id)
            ))?;
            print_success("bi.get_published", result)
        }
        BiSubcommands::Homepage(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/bi/homepage")?;
            print_success("bi.homepage", result)
        }
        BiSubcommands::ListHomepageBindings(args) => {
            let client = client_from_connection(&args)?;
            let result: Value = client.get_json("/api/bi/homepage/bindings")?;
            print_success("bi.list_homepage_bindings", result)
        }
        BiSubcommands::SaveHomepageBinding(args) => {
            let client = client_from_connection(&args.connection)?;
            let payload = load_json_input(args.input.as_deref(), args.json.as_deref())?;
            let result: Value = client.put_json("/api/bi/homepage/bindings", &payload)?;
            print_success("bi.save_homepage_binding", result)
        }
        BiSubcommands::DeleteHomepageBinding(args) => {
            let client = client_from_connection(&args.connection)?;
            let result: DeleteResult = client.delete_empty(&format!(
                "/api/bi/homepage/bindings/{}",
                encode_path_segment(&args.binding_id)
            ))?;
            print_success("bi.delete_homepage_binding", result)
        }
    }
}

fn client_from_connection(args: &ConnectionArgs) -> Result<ApiClient> {
    let config = resolve(args.base_url.as_deref(), args.token.as_deref())?;
    ApiClient::new(config)
}

fn write_binary_output(path: &Path, bytes: Vec<u8>) -> Result<FileWriteResponse> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }
    }

    let byte_count = bytes.len();
    fs::write(path, bytes)
        .with_context(|| format!("Failed to write output file: {}", path.display()))?;
    Ok(FileWriteResponse {
        path: path.display().to_string(),
        bytes: byte_count,
    })
}

fn build_plugin_workspace_package(source: &Path, output_file: &Path) -> Result<FileWriteResponse> {
    if !source.exists() {
        return Err(anyhow!(
            "Plugin workspace source does not exist: {}",
            source.display()
        ));
    }

    if let Some(parent) = output_file.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }
    }

    let file = fs::File::create(output_file)
        .with_context(|| format!("Failed to create archive: {}", output_file.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for entry_name in ["backend", "frontend", "locales", "docs", "README.md"] {
        let source_path = source.join(entry_name);
        if source_path.is_dir() {
            for entry in WalkDir::new(&source_path)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }

                let relative = path
                    .strip_prefix(source)
                    .with_context(|| {
                        format!("Failed to resolve relative path for {}", path.display())
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");
                zip.start_file(relative, options)?;
                let mut input = fs::File::open(path).with_context(|| {
                    format!("Failed to read plugin workspace file: {}", path.display())
                })?;
                std::io::copy(&mut input, &mut zip)?;
            }
        } else if source_path.is_file() {
            zip.start_file(entry_name, options)?;
            let mut input = fs::File::open(&source_path).with_context(|| {
                format!(
                    "Failed to read plugin workspace file: {}",
                    source_path.display()
                )
            })?;
            std::io::copy(&mut input, &mut zip)?;
        }
    }

    zip.finish()?;
    let bytes = fs::metadata(output_file)
        .with_context(|| format!("Failed to read archive metadata: {}", output_file.display()))?
        .len() as usize;
    Ok(FileWriteResponse {
        path: output_file.display().to_string(),
        bytes,
    })
}

fn build_plugin_release_package(
    root: &Path,
    plugin_code: &str,
    output_file: Option<&Path>,
    build_frontend: bool,
) -> Result<FileWriteResponse> {
    let workspace = resolve_plugin_workspace_for_code(root, plugin_code)?;
    let backend_dir = workspace.join("backend").join(plugin_code);
    let backend_manifest = backend_dir.join("manifest.json");
    let frontend_dir = workspace.join("frontend");
    let frontend_source_manifest = frontend_dir
        .join("src")
        .join("pages")
        .join(plugin_code)
        .join("manifest.json");

    let backend_check = if backend_manifest.exists() {
        if !backend_dir.join("main.py").exists() {
            return Err(anyhow!(
                "Backend plugin '{}' is missing main.py.",
                plugin_code
            ));
        }
        Some(validate_backend_manifest(&backend_manifest)?)
    } else {
        None
    };
    let frontend_check = if frontend_source_manifest.exists() {
        Some(validate_frontend_manifest(&frontend_source_manifest)?)
    } else {
        None
    };
    if backend_check.is_none() && frontend_check.is_none() {
        return Err(anyhow!(
            "Plugin '{}' was not found in {}.",
            plugin_code,
            workspace.display()
        ));
    }

    let version = backend_check
        .as_ref()
        .map(|item| item.version.clone())
        .or_else(|| frontend_check.as_ref().map(|item| item.version.clone()))
        .unwrap_or_default();
    if let (Some(backend), Some(frontend)) = (&backend_check, &frontend_check) {
        if backend.version != frontend.version {
            return Err(anyhow!(
                "Frontend version '{}' and backend version '{}' must match.",
                frontend.version,
                backend.version
            ));
        }
    }

    if frontend_check.is_some() && build_frontend {
        ensure_frontend_dependencies(&frontend_dir)?;
        run_checked_command(
            &frontend_dir,
            npm_program(),
            &["run", "build"],
            "Frontend plugin build",
        )?;
    }

    let frontend_dist_manifest = frontend_dir
        .join("dist")
        .join("manifests")
        .join(format!("{}.json", plugin_code));
    let frontend_assets = frontend_dir.join("dist").join(plugin_code);
    if frontend_check.is_some() && (!frontend_dist_manifest.exists() || !frontend_assets.is_dir()) {
        return Err(anyhow!(
            "Frontend build output for '{}' is missing. Run without --skip-frontend-build.",
            plugin_code
        ));
    }

    let output = output_file.map(Path::to_path_buf).unwrap_or_else(|| {
        root.join("dist")
            .join(format!("{}-{}.afplugin", plugin_code, version))
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_dir = std::env::temp_dir().join(format!(
        "asapflow-plugin-pack-{}-{}",
        std::process::id(),
        unix_timestamp()
    ));
    fs::create_dir_all(&temp_dir)?;
    let result = (|| -> Result<FileWriteResponse> {
        let mut components = serde_json::Map::new();

        if frontend_check.is_some() {
            let frontend_zip = temp_dir.join("frontend.zip");
            let mut frontend_entries = vec![
                (
                    frontend_dist_manifest.as_path(),
                    format!("manifests/{}.json", plugin_code),
                ),
                (frontend_assets.as_path(), plugin_code.to_string()),
            ];
            let shared_dir = frontend_dir.join("dist").join("shared");
            let assets_dir = frontend_dir.join("dist").join("assets");
            if shared_dir.is_dir() {
                frontend_entries.push((shared_dir.as_path(), "shared".to_string()));
            }
            if assets_dir.is_dir() {
                frontend_entries.push((assets_dir.as_path(), "assets".to_string()));
            }
            create_zip(&frontend_zip, &frontend_entries)?;
            components.insert(
                "frontend".to_string(),
                json!({ "file": "frontend.zip", "sha256": sha256_file(&frontend_zip)? }),
            );
        }

        if backend_check.is_some() {
            let backend_zip = temp_dir.join("backend.zip");
            create_zip(&backend_zip, &[(backend_dir.as_path(), String::new())])?;
            components.insert(
                "backend".to_string(),
                json!({ "file": "backend.zip", "sha256": sha256_file(&backend_zip)? }),
            );
        }

        let source_manifest = if backend_manifest.exists() {
            &backend_manifest
        } else {
            &frontend_source_manifest
        };
        let source_value = load_manifest_value(source_manifest)?;
        let release = json!({
            "schemaVersion": "1.0",
            "pluginCode": plugin_code,
            "pluginName": source_value.get("pluginName").cloned().unwrap_or(Value::Null),
            "version": version,
            "components": Value::Object(components)
        });

        let file = fs::File::create(&output)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);
        zip.start_file("release.json", options)?;
        zip.write_all(serde_json::to_string_pretty(&release)?.as_bytes())?;
        for component in ["frontend.zip", "backend.zip"] {
            let path = temp_dir.join(component);
            if path.exists() {
                zip.start_file(component, options)?;
                let mut input = fs::File::open(path)?;
                std::io::copy(&mut input, &mut zip)?;
            }
        }
        zip.finish()?;

        let bytes = fs::metadata(&output)?.len() as usize;
        Ok(FileWriteResponse {
            path: output.display().to_string(),
            bytes,
        })
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn resolve_plugin_workspace_for_code(root: &Path, plugin_code: &str) -> Result<PathBuf> {
    for candidate in [
        root.join("plugins"),
        root.join("plugin-workspace"),
        root.to_path_buf(),
        root.join("plugins/examples"),
        root.join("plugin-workspace/examples"),
        root.join("examples"),
    ] {
        let backend_manifest = candidate
            .join("backend")
            .join(plugin_code)
            .join("manifest.json");
        let frontend_manifest = candidate
            .join("frontend")
            .join("src")
            .join("pages")
            .join(plugin_code)
            .join("manifest.json");
        if backend_manifest.is_file() || frontend_manifest.is_file() {
            return Ok(candidate);
        }
    }
    Err(anyhow!(
        "Plugin '{}' was not found under {}, {}/plugins, {}/plugin-workspace, or their examples directories.",
        plugin_code,
        root.display(),
        root.display(),
        root.display()
    ))
}

fn ensure_frontend_dependencies(frontend_dir: &Path) -> Result<()> {
    if frontend_dir.join("node_modules").is_dir() {
        return Ok(());
    }
    let install_args = if frontend_dir.join("package-lock.json").exists() {
        vec!["ci"]
    } else {
        vec!["install"]
    };
    run_checked_command(
        frontend_dir,
        npm_program(),
        &install_args,
        "Frontend dependency installation",
    )
}

fn npm_program() -> &'static str {
    if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    }
}

fn resolve_frontend_working_dir(requested: Option<&Path>) -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to resolve current directory.")?;
    resolve_frontend_working_dir_from(&current_dir, requested)
}

fn resolve_frontend_working_dir_from(
    current_dir: &Path,
    requested: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = requested {
        if path.join("package.json").is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(anyhow!(
            "Frontend working directory {} does not contain package.json.",
            path.display()
        ));
    }

    for candidate in [
        current_dir.join("plugin-workspace/frontend"),
        current_dir.join("plugins/frontend"),
        current_dir.join("frontend"),
        current_dir.to_path_buf(),
    ] {
        if candidate.join("package.json").is_file() {
            return Ok(candidate);
        }
    }

    Err(anyhow!(
        "Frontend workspace was not found. Run the command from the source/development-kit root, or pass --working-dir explicitly."
    ))
}

fn metadata_list_path(base_path: &str, limit: Option<u32>) -> String {
    match limit {
        Some(value) => format!("{base_path}?limit={value}"),
        None => base_path.to_string(),
    }
}

fn run_checked_command(
    working_dir: &Path,
    program: &str,
    args: &[&str],
    label: &str,
) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .status()
        .with_context(|| format!("Failed to run {} in {}.", label, working_dir.display()))?;
    if !status.success() {
        return Err(anyhow!(
            "{} failed with exit code {:?}.",
            label,
            status.code()
        ));
    }
    Ok(())
}

fn create_zip(output: &Path, entries: &[(&Path, String)]) -> Result<()> {
    let file = fs::File::create(output)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for (source, prefix) in entries {
        if source.is_file() {
            add_file_to_zip(&mut zip, source, prefix, options)?;
            continue;
        }
        for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() || should_exclude_release_file(path) {
                continue;
            }
            let relative = path.strip_prefix(source)?;
            let archive_path = if prefix.is_empty() {
                relative.to_string_lossy().replace('\\', "/")
            } else {
                format!(
                    "{}/{}",
                    prefix,
                    relative.to_string_lossy().replace('\\', "/")
                )
            };
            add_file_to_zip(&mut zip, path, &archive_path, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn add_file_to_zip(
    zip: &mut ZipWriter<fs::File>,
    source: &Path,
    archive_path: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    zip.start_file(archive_path, options)?;
    let mut input = fs::File::open(source)?;
    std::io::copy(&mut input, zip)?;
    Ok(())
}

fn should_exclude_release_file(path: &Path) -> bool {
    path.components().any(|part| {
        matches!(
            part.as_os_str().to_str(),
            Some("__pycache__" | ".git" | ".venv" | "node_modules")
        )
    }) || matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("pyc" | "pyo")
    )
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn wait_for_plugin_deployment(client: &ApiClient, queued: &Value) -> Result<Value> {
    let deployment_id = queued
        .get("deploymentId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Publish response is missing deploymentId."))?;
    let started = Instant::now();
    loop {
        let result: Value = client.get_json(&format!(
            "/api/plugins/releases/{}",
            encode_path_segment(deployment_id)
        ))?;
        match result.get("status").and_then(Value::as_str) {
            Some("success") => return Ok(result),
            Some("failed") => {
                let message = result
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Plugin publish failed.");
                return Err(anyhow!(message.to_string()));
            }
            _ if started.elapsed() >= Duration::from_secs(300) => {
                return Err(anyhow!(
                    "Plugin deployment did not finish within 300 seconds. Deployment ID: {}",
                    deployment_id
                ));
            }
            _ => thread::sleep(Duration::from_secs(1)),
        }
    }
}

fn legacy_convert_fields(
    package_id: Option<&str>,
    name: Option<&str>,
    version: Option<&str>,
    module: Option<&str>,
) -> Vec<(&'static str, String)> {
    let mut fields = Vec::new();
    if let Some(value) = package_id.filter(|value| !value.trim().is_empty()) {
        fields.push(("packageId", value.to_string()));
    }
    if let Some(value) = name.filter(|value| !value.trim().is_empty()) {
        fields.push(("name", value.to_string()));
    }
    if let Some(value) = version.filter(|value| !value.trim().is_empty()) {
        fields.push(("version", value.to_string()));
    }
    if let Some(value) = module.filter(|value| !value.trim().is_empty()) {
        fields.push(("module", value.to_string()));
    }
    fields
}

fn encode_path_segment(value: &str) -> String {
    percent_encode(value, true)
}

fn encode_query_value(value: &str) -> String {
    percent_encode(value, false)
}

fn percent_encode(value: &str, keep_dot: bool) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let is_unreserved = byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'_')
            || (keep_dot && byte == b'.')
            || (!keep_dot && byte == b'.');
        if is_unreserved {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn client_for_health(args: &ConnectionArgs) -> Result<ApiClient> {
    let stored = load_config().unwrap_or_default();
    let base_url = args
        .base_url
        .clone()
        .or_else(|| std::env::var("ASAPFLOW_BASE_URL").ok())
        .or(stored.base_url)
        .unwrap_or_else(|| "http://localhost:5000".to_string());

    ApiClient::new(ResolvedConfig {
        base_url: normalize_base_url(&base_url),
        token: args
            .token
            .clone()
            .or_else(|| std::env::var("ASAPFLOW_TOKEN").ok())
            .or(stored.token)
            .unwrap_or_default(),
    })
}

fn resolve_entity_id(
    client: &ApiClient,
    entity_id: Option<String>,
    code: Option<String>,
) -> Result<String> {
    if let Some(entity_id) = entity_id {
        return Ok(entity_id);
    }

    if let Some(code) = code {
        let entity: Value = client.get_json(&format!("/api/meta/entities/{}", code))?;
        return entity
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("Entity '{}' does not expose an id.", code));
    }

    Err(anyhow!(
        "system delete-entity requires either --entity-id or --code."
    ))
}

fn resolve_feature_id(
    client: &ApiClient,
    feature_id: Option<String>,
    code: Option<String>,
) -> Result<String> {
    if let Some(feature_id) = feature_id {
        return Ok(feature_id);
    }

    if let Some(code) = code {
        let feature: Value = client.get_json(&format!("/api/meta/features/{}", code))?;
        return feature
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| anyhow!("Feature '{}' does not expose an id.", code));
    }

    Err(anyhow!(
        "system delete-feature requires either --feature-id or --code."
    ))
}

fn find_existing_by_payload_code(
    client: &ApiClient,
    payload: &Value,
    object_key: &str,
    endpoint: &str,
) -> Result<Option<Value>> {
    let code = payload_code(payload, object_key)?;
    client.get_optional_json(&format!("{}/{}", endpoint, code))
}

fn get_feature_by_id(client: &ApiClient, feature_id: &str) -> Result<Value> {
    let features: Value = client.get_json("/api/meta/features")?;
    let items = features
        .as_array()
        .ok_or_else(|| anyhow!("Expected feature list array response."))?;
    items
        .iter()
        .find(|feature| {
            feature
                .get("id")
                .and_then(Value::as_str)
                .map(|id| id.eq_ignore_ascii_case(feature_id))
                .unwrap_or(false)
        })
        .cloned()
        .ok_or_else(|| anyhow!("Feature '{}' not found.", feature_id))
}

fn payload_code(payload: &Value, object_key: &str) -> Result<String> {
    payload
        .get(object_key)
        .and_then(|value| value.get("code"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("{} payload does not expose a code.", object_key))
}

fn skipped_existing_response(kind: &str, existing: Value) -> Value {
    json!({
        "skipped": true,
        "reason": "already_exists",
        "kind": kind,
        "existing": existing
    })
}

fn load_json_input(path: Option<&Path>, inline: Option<&str>) -> Result<Value> {
    if let Some(raw) = inline {
        return parse_inline_json(raw).context("Failed to parse --json payload.");
    }

    let input_path = path.ok_or_else(|| anyhow!("Either --input or --json is required."))?;
    let raw = if input_path == Path::new("-") {
        use std::io::Read;

        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read JSON payload from stdin.")?;
        buffer
    } else {
        fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?
    };
    let raw = strip_utf8_bom(&raw);
    serde_json::from_str(raw).with_context(|| {
        if input_path == Path::new("-") {
            "Failed to parse JSON payload from stdin.".to_string()
        } else {
            format!("Failed to parse JSON file: {}", input_path.display())
        }
    })
}

fn with_workflow_cc_user_ids(mut payload: Value, cc_user_ids: &[String]) -> Result<Value> {
    if cc_user_ids.is_empty() {
        return Ok(payload);
    }

    let object = payload
        .as_object_mut()
        .ok_or_else(|| anyhow!("workflow start-instance payload must be a JSON object."))?;

    let merged = cc_user_ids
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| Value::String(item.to_string()))
        .collect::<Vec<_>>();

    object.insert("ccUserIds".to_string(), Value::Array(merged));
    Ok(payload)
}

fn parse_inline_json(raw: &str) -> Result<Value> {
    let trimmed = strip_utf8_bom(raw).trim();

    for candidate in inline_json_candidates(trimmed) {
        if let Ok(value) = serde_json::from_str::<Value>(&candidate) {
            return Ok(value);
        }
    }

    Err(anyhow!("Failed to parse --json payload."))
}

fn strip_utf8_bom(value: &str) -> &str {
    value.strip_prefix('\u{feff}').unwrap_or(value)
}

fn inline_json_candidates(value: &str) -> Vec<String> {
    let mut candidates = vec![value.to_string()];
    if let Some(unwrapped) = strip_wrapping_quotes(value) {
        candidates.push(unwrapped.to_string());
    }

    let escaped_candidates = candidates
        .iter()
        .filter(|candidate| candidate.contains("\\\""))
        .map(|candidate| candidate.replace("\\\"", "\""))
        .collect::<Vec<_>>();
    candidates.extend(escaped_candidates);
    candidates
}

fn strip_wrapping_quotes(value: &str) -> Option<&str> {
    if value.len() < 2 {
        return None;
    }

    let bytes = value.as_bytes();
    let first = bytes.first().copied()?;
    let last = bytes.last().copied()?;

    let wrapped_in_single = first == b'\'' && last == b'\'';
    let wrapped_in_double = first == b'"' && last == b'"';

    if wrapped_in_single || wrapped_in_double {
        return Some(&value[1..value.len() - 1]);
    }

    None
}

fn select_definition_by_code(value: Value, code: &str) -> Result<Value> {
    let items = value
        .as_array()
        .ok_or_else(|| anyhow!("Expected workflow definitions array response."))?;

    items
        .iter()
        .find(|item| {
            item.get("code")
                .and_then(Value::as_str)
                .map(|current| current.eq_ignore_ascii_case(code))
                .unwrap_or(false)
        })
        .cloned()
        .ok_or_else(|| anyhow!("Workflow definition '{}' not found.", code))
}

fn select_rule_by_code(value: Value, code: &str) -> Result<Value> {
    let items = value
        .as_array()
        .ok_or_else(|| anyhow!("Expected number rule array response."))?;

    items
        .iter()
        .find(|item| {
            item.get("code")
                .and_then(Value::as_str)
                .map(|current| current.eq_ignore_ascii_case(code))
                .unwrap_or(false)
        })
        .cloned()
        .ok_or_else(|| anyhow!("Number rule '{}' not found.", code))
}

fn bind_number_rule_to_field(
    client: &ApiClient,
    entity: &Value,
    entity_code: &str,
    field_code: &str,
    rule_code: &str,
    trigger: &str,
    allow_manual_override: bool,
) -> Result<Value> {
    let entity_id = entity
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Entity '{}' does not expose an id.", entity_code))?;

    let field = entity
        .get("fields")
        .and_then(Value::as_array)
        .and_then(|fields| {
            fields.iter().find(|field| {
                field
                    .get("code")
                    .and_then(Value::as_str)
                    .map(|value| value.eq_ignore_ascii_case(field_code))
                    .unwrap_or(false)
            })
        })
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "Field '{}' not found in entity '{}'.",
                field_code,
                entity_code
            )
        })?;

    let field_id = field
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Field '{}' does not expose an id.", field_code))?;

    let metadata_text = field
        .get("metadata")
        .and_then(Value::as_str)
        .unwrap_or("{}");
    let mut metadata = parse_json_object_or_default(metadata_text)?;
    metadata.insert(
        "generator".to_string(),
        json!({
            "type": "number_rule",
            "ruleCode": rule_code,
            "trigger": trigger,
            "allowManualOverride": allow_manual_override
        }),
    );

    let payload = json!({
        "code": field.get("code").cloned().unwrap_or(Value::Null),
        "name": field.get("name").cloned().unwrap_or(Value::Null),
        "dataType": field.get("dataType").cloned().unwrap_or(Value::Null),
        "length": field.get("length").cloned().unwrap_or(Value::Null),
        "precision": field.get("precision").cloned().unwrap_or(Value::Null),
        "scale": field.get("scale").cloned().unwrap_or(Value::Null),
        "isNullable": field.get("isNullable").cloned().unwrap_or(Value::Bool(true)),
        "isPrimary": field.get("isPrimary").cloned().unwrap_or(Value::Bool(false)),
        "defaultValue": field.get("defaultValue").cloned().unwrap_or(Value::Null),
        "orderIndex": field.get("orderIndex").cloned().unwrap_or_else(|| json!(0)),
        "category": field
            .get("category")
            .cloned()
            .unwrap_or_else(|| Value::String("business".to_string())),
        "metadata": Value::String(Value::Object(metadata).to_string())
    });

    let updated: Value = client.put_json(
        &format!("/api/meta/entities/{}/fields/{}", entity_id, field_id),
        &payload,
    )?;

    Ok(json!({
        "entityCode": entity_code,
        "fieldCode": field_code,
        "ruleCode": rule_code,
        "trigger": trigger,
        "allowManualOverride": allow_manual_override,
        "field": updated
    }))
}

fn parse_json_object_or_default(raw: &str) -> Result<serde_json::Map<String, Value>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(serde_json::Map::new());
    }

    match serde_json::from_str::<Value>(trimmed)? {
        Value::Object(map) => Ok(map),
        Value::Null => Ok(serde_json::Map::new()),
        _ => Err(anyhow!("Expected metadata JSON object.")),
    }
}

fn extract_array_payload(value: Value, key: &str) -> Result<Vec<Value>> {
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(map) => map
            .get(key)
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("Expected array payload or object containing '{}'.", key)),
        _ => Err(anyhow!("Expected array JSON payload.")),
    }
}

fn normalize_create_entity_payload(value: Value) -> Result<Value> {
    if value.get("entity").is_some() {
        return Ok(value);
    }

    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("create_entity expects a JSON object."))?;

    let code = required_string(&Value::Object(object.clone()), "code")?;
    let name = required_string(&Value::Object(object.clone()), "name")?;
    let module = required_string(&Value::Object(object.clone()), "module")?;

    let fields = object
        .get("fields")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("create_entity requires a 'fields' array."))?
        .iter()
        .enumerate()
        .map(|(index, field)| normalize_entity_field(field, index + 1))
        .collect::<Result<Vec<_>>>()?;

    let mut entity = serde_json::Map::new();
    entity.insert("code".to_string(), Value::String(code.clone()));
    entity.insert("name".to_string(), Value::String(name));
    entity.insert("module".to_string(), Value::String(module));
    entity.insert(
        "storageType".to_string(),
        object
            .get("storageType")
            .cloned()
            .unwrap_or_else(|| Value::String("table".to_string())),
    );
    entity.insert(
        "schemaName".to_string(),
        object
            .get("schemaName")
            .cloned()
            .unwrap_or_else(|| Value::String("public".to_string())),
    );
    entity.insert(
        "physicalName".to_string(),
        object
            .get("physicalName")
            .cloned()
            .unwrap_or_else(|| Value::String(code)),
    );
    entity.insert(
        "writeEnabled".to_string(),
        object
            .get("writeEnabled")
            .cloned()
            .unwrap_or(Value::Bool(true)),
    );
    entity.insert(
        "description".to_string(),
        object
            .get("description")
            .cloned()
            .unwrap_or(Value::String(String::new())),
    );
    entity.insert(
        "metadata".to_string(),
        object
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::String("{}".to_string())),
    );
    if let Some(data_source_code) = object.get("dataSourceCode").cloned() {
        entity.insert("dataSourceCode".to_string(), data_source_code);
    }
    entity.insert("fields".to_string(), Value::Array(fields));

    Ok(json!({ "entity": Value::Object(entity) }))
}

fn normalize_create_feature_payload(value: Value) -> Result<Value> {
    if value.get("feature").is_some() {
        return Ok(value);
    }

    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("create_feature expects a JSON object."))?;

    let code = required_string(&Value::Object(object.clone()), "code")?;
    let name = required_string(&Value::Object(object.clone()), "name")?;
    let module = required_string(&Value::Object(object.clone()), "module")?;
    let entity_code = object
        .get("entityCode")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let fields = object
        .get("fields")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("create_feature requires a 'fields' array."))?
        .iter()
        .map(normalize_feature_field)
        .collect::<Result<Vec<_>>>()?;

    let mut feature = serde_json::Map::new();
    feature.insert("code".to_string(), Value::String(code));
    feature.insert("name".to_string(), Value::String(name));
    feature.insert("module".to_string(), Value::String(module));
    feature.insert(
        "dataSourceType".to_string(),
        object
            .get("dataSourceType")
            .cloned()
            .unwrap_or_else(|| Value::String("table".to_string())),
    );
    feature.insert(
        "dataSourceName".to_string(),
        object
            .get("dataSourceName")
            .cloned()
            .or_else(|| entity_code.clone().map(Value::String))
            .unwrap_or(Value::Null),
    );
    feature.insert(
        "description".to_string(),
        object
            .get("description")
            .cloned()
            .unwrap_or(Value::String(String::new())),
    );
    feature.insert(
        "isActive".to_string(),
        object.get("isActive").cloned().unwrap_or(Value::Bool(true)),
    );
    feature.insert(
        "metadata".to_string(),
        object
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::String("{}".to_string())),
    );
    if let Some(data_source_code) = object.get("dataSourceCode").cloned() {
        feature.insert("dataSourceCode".to_string(), data_source_code);
    }
    feature.insert("fields".to_string(), Value::Array(fields));
    let has_fields = feature
        .get("fields")
        .and_then(Value::as_array)
        .map(|fields| !fields.is_empty())
        .unwrap_or(false);

    Ok(json!({
        "feature": Value::Object(feature),
        "initializeDefaultScenarios": object
            .get("initializeDefaultScenarios")
            .cloned()
            .unwrap_or(Value::Bool(has_fields)),
        "includeDefaultListActions": object
            .get("includeDefaultListActions")
            .cloned()
            .unwrap_or(Value::Bool(has_fields))
    }))
}

fn normalize_update_feature_payload(existing: &Value, value: Value) -> Result<Value> {
    let input = if let Some(feature) = value.get("feature") {
        feature
    } else {
        &value
    };

    let existing_object = existing
        .as_object()
        .ok_or_else(|| anyhow!("Existing feature payload must be a JSON object."))?;
    let input_object = input
        .as_object()
        .ok_or_else(|| anyhow!("update_feature expects a JSON object."))?;

    let mut payload = serde_json::Map::new();

    for key in [
        "name",
        "module",
        "dataSourceType",
        "dataSourceName",
        "dataSourceCode",
        "description",
        "isActive",
        "metadata",
    ] {
        if let Some(value) = input_object.get(key).cloned() {
            payload.insert(key.to_string(), value);
        } else if let Some(value) = existing_object.get(key).cloned() {
            payload.insert(key.to_string(), value);
        }
    }

    if let Some(entity_code) = input_object
        .get("entityCode")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        payload.insert(
            "dataSourceName".to_string(),
            Value::String(entity_code.to_string()),
        );
    }

    if let Some(fields) = input_object.get("fields").and_then(Value::as_array) {
        let normalized_fields = fields
            .iter()
            .map(normalize_feature_field)
            .collect::<Result<Vec<_>>>()?;
        payload.insert("fields".to_string(), Value::Array(normalized_fields));
    }

    Ok(Value::Object(payload))
}

fn normalize_create_action_menu_payload(value: Value) -> Result<Value> {
    if value.get("action").is_some() || value.get("menu").is_some() {
        return Ok(value);
    }

    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("create_action_menu expects a JSON object."))?;

    let feature_id = required_string(&Value::Object(object.clone()), "featureId")?;
    let scenario_id = required_string(&Value::Object(object.clone()), "scenarioId")?;
    let action_code = required_string(&Value::Object(object.clone()), "actionCode")?;
    let action_name = required_string(&Value::Object(object.clone()), "actionName")?;
    let menu_code = required_string(&Value::Object(object.clone()), "menuCode")?;
    let menu_label = required_string(&Value::Object(object.clone()), "menuLabel")?;

    let action = json!({
        "featureId": feature_id,
        "code": action_code,
        "name": action_name,
        "actionType": object
            .get("actionType")
            .cloned()
            .unwrap_or_else(|| Value::String("custom".to_string())),
        "executionMode": object
            .get("executionMode")
            .cloned()
            .unwrap_or_else(|| Value::String("sync".to_string())),
        "permissionCode": object.get("permissionCode").cloned().unwrap_or(Value::Null),
        "description": object
            .get("actionDescription")
            .cloned()
            .or_else(|| object.get("description").cloned())
            .unwrap_or(Value::String(String::new())),
        "isSystem": object.get("isSystem").cloned().unwrap_or(Value::Bool(false)),
        "metadata": object
            .get("actionMetadata")
            .cloned()
            .or_else(|| object.get("metadata").cloned())
            .unwrap_or_else(|| Value::String("{}".to_string()))
    });

    let menu = json!({
        "label": menu_label,
        "code": menu_code,
        "menuType": object
            .get("menuType")
            .cloned()
            .unwrap_or_else(|| Value::String("scenario".to_string())),
        "icon": object.get("icon").cloned().unwrap_or(Value::Null),
        "featureId": object.get("featureId").cloned().unwrap_or(Value::Null),
        "scenarioId": scenario_id,
        "description": object
            .get("menuDescription")
            .cloned()
            .or_else(|| object.get("description").cloned())
            .unwrap_or(Value::String(String::new())),
        "metadata": object
            .get("menuMetadata")
            .cloned()
            .or_else(|| object.get("metadata").cloned())
            .unwrap_or_else(|| Value::String("{}".to_string())),
        "isActive": object.get("isActive").cloned().unwrap_or(Value::Bool(true))
    });

    Ok(json!({
        "action": action,
        "menu": menu
    }))
}

fn normalize_add_entity_fields_payload(value: Value) -> Result<Vec<Value>> {
    let fields = extract_array_payload(value, "fields")?;
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| normalize_entity_field(field, index + 1))
        .collect()
}

fn normalize_add_feature_fields_payload(value: Value) -> Result<Vec<Value>> {
    let fields = extract_array_payload(value, "fields")?;
    fields.iter().map(normalize_feature_field).collect()
}

fn normalize_update_feature_field_changes(value: &Value) -> Result<Vec<Value>> {
    let root = if let Some(feature) = value.get("feature") {
        feature
    } else {
        value
    };

    let Some(fields) = root.get("fields").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    fields.iter().map(normalize_feature_field).collect()
}

fn sync_feature_fields(
    client: &ApiClient,
    feature_id: &str,
    existing_feature: &Value,
    fields: Vec<Value>,
) -> Result<Vec<Value>> {
    let mut field_id_map: HashMap<String, String> = existing_feature
        .get("fields")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|field| {
            let key = field
                .get("fieldKey")
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            let id = field.get("id").and_then(Value::as_str)?.trim().to_string();
            if key.is_empty() || id.is_empty() {
                return None;
            }
            Some((key, id))
        })
        .collect();

    let mut results = Vec::new();
    for field in fields {
        let field_key = required_string(&field, "fieldKey")?;
        if let Some(existing_id) = field_id_map.get(&field_key).cloned() {
            let updated: Value = client.put_json(
                &format!("/api/meta/features/{}/fields/{}", feature_id, existing_id),
                &field,
            )?;
            results.push(json!({ "operation": "updated", "field": updated }));
            continue;
        }

        let created: Value =
            client.post_json(&format!("/api/meta/features/{}/fields", feature_id), &field)?;
        if let Some(created_id) = created.get("id").and_then(Value::as_str) {
            field_id_map.insert(field_key, created_id.to_string());
        }
        results.push(json!({ "operation": "created", "field": created }));
    }

    Ok(results)
}

fn normalize_create_action_payload(value: Value) -> Result<Value> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("create_action expects a JSON object."))?;
    let object_value = Value::Object(object.clone());

    let code = required_string(&object_value, "code")?;
    let name = required_string(&object_value, "name")?;

    Ok(json!({
        "featureId": object.get("featureId").cloned().unwrap_or(Value::Null),
        "code": code,
        "name": name,
        "actionType": object
            .get("actionType")
            .cloned()
            .unwrap_or_else(|| Value::String("custom".to_string())),
        "executionMode": object
            .get("executionMode")
            .cloned()
            .unwrap_or_else(|| Value::String("sync".to_string())),
        "permissionCode": object
            .get("permissionCode")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new())),
        "description": object
            .get("description")
            .cloned()
            .unwrap_or(Value::String(String::new())),
        "isSystem": object.get("isSystem").cloned().unwrap_or(Value::Bool(false)),
        "metadata": object
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::String("{}".to_string()))
    }))
}

fn normalize_create_menu_payload(value: Value) -> Result<Value> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("create_menu expects a JSON object."))?;
    let object_value = Value::Object(object.clone());
    let label = required_string(&object_value, "label")?;

    Ok(json!({
        "label": label,
        "code": object.get("code").cloned().unwrap_or(Value::Null),
        "menuType": object
            .get("menuType")
            .cloned()
            .unwrap_or_else(|| Value::String("group".to_string())),
        "parentId": object.get("parentId").cloned().unwrap_or(Value::Null),
        "orderIndex": object.get("orderIndex").cloned().unwrap_or(Value::Null),
        "icon": object.get("icon").cloned().unwrap_or(Value::Null),
        "featureId": object.get("featureId").cloned().unwrap_or(Value::Null),
        "scenarioId": object.get("scenarioId").cloned().unwrap_or(Value::Null),
        "customRoute": object.get("customRoute").cloned().unwrap_or(Value::Null),
        "externalUrl": object.get("externalUrl").cloned().unwrap_or(Value::Null),
        "reportCode": object.get("reportCode").cloned().unwrap_or(Value::Null),
        "description": object
            .get("description")
            .cloned()
            .unwrap_or(Value::String(String::new())),
        "metadata": object
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::String("{}".to_string())),
        "isActive": object.get("isActive").cloned().unwrap_or(Value::Bool(true))
    }))
}

fn normalize_entity_field(field: &Value, default_order_index: usize) -> Result<Value> {
    let object = field
        .as_object()
        .ok_or_else(|| anyhow!("Entity field must be a JSON object."))?;
    let field_value = Value::Object(object.clone());
    let code = required_string(&field_value, "code")?;
    let name = required_string(&field_value, "name")?;
    let data_type = required_string(&field_value, "dataType")?;

    Ok(json!({
        "code": code,
        "name": name,
        "dataType": data_type,
        "length": object.get("length").cloned().unwrap_or(Value::Null),
        "precision": object.get("precision").cloned().unwrap_or(Value::Null),
        "scale": object.get("scale").cloned().unwrap_or(Value::Null),
        "isNullable": object.get("isNullable").cloned().unwrap_or(Value::Bool(true)),
        "isPrimary": object.get("isPrimary").cloned().unwrap_or(Value::Bool(false)),
        "defaultValue": object.get("defaultValue").cloned().unwrap_or(Value::Null),
        "orderIndex": object
            .get("orderIndex")
            .cloned()
            .unwrap_or_else(|| json!(default_order_index)),
        "category": object
            .get("category")
            .cloned()
            .unwrap_or_else(|| Value::String("business".to_string())),
        "metadata": object
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::String("{}".to_string()))
    }))
}

fn normalize_feature_field(field: &Value) -> Result<Value> {
    let object = field
        .as_object()
        .ok_or_else(|| anyhow!("Feature field must be a JSON object."))?;
    let object_value = Value::Object(object.clone());
    let display_name = required_string(&object_value, "displayName")?;
    let data_type = required_string(&object_value, "dataType")?;
    let source_type = object
        .get("sourceType")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("entity")
        .to_string();
    let source_field = object
        .get("sourceField")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let field_key = object
        .get("fieldKey")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            if source_type == "entity" {
                source_field.clone()
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Feature field requires a 'fieldKey'. For entity fields, you can omit 'fieldKey' and provide 'sourceField' only."))?;

    let source_config = if let Some(value) = object.get("sourceConfig").cloned() {
        value
    } else if let Some(source_field) = source_field.as_ref() {
        Value::String(json!({ "field": source_field }).to_string())
    } else {
        Value::String(json!({ "field": field_key.clone() }).to_string())
    };

    Ok(json!({
        "fieldKey": field_key,
        "displayName": display_name,
        "dataType": data_type,
        "sourceType": Value::String(source_type),
        "sourceConfig": source_config,
        "isIdentifier": object.get("isIdentifier").cloned().unwrap_or(Value::Bool(false)),
        "defaultVisible": object.get("defaultVisible").cloned().unwrap_or(Value::Bool(true)),
        "defaultEditable": object.get("defaultEditable").cloned().unwrap_or(Value::Bool(true)),
        "validationRules": object.get("validationRules").cloned().unwrap_or(Value::Null),
        "metadata": object
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::String("{}".to_string()))
    }))
}

fn required_string(value: &Value, key: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("Missing required string field '{}'.", key))
}

fn create_backend_plugin(target_dir: &Path, code: &str, name: &str) -> Result<()> {
    fs::create_dir_all(target_dir).with_context(|| {
        format!(
            "Failed to create backend plugin directory: {}",
            target_dir.display()
        )
    })?;

    let manifest = json!({
        "pluginCode": code,
        "pluginName": name,
        "version": "0.1.0",
        "capabilities": [
            {
                "code": "pre_submit",
                "displayName": "提交前校验",
                "slot": "scenario.submit.pre",
                "description": "由 CLI 初始化的后端插件示例",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                },
                "defaultParams": {},
                "permissions": [],
                "timeoutSeconds": 3,
                "auditLevel": "basic"
            }
        ]
    });

    let main_py = r#"from plugins_sdk import PluginApp, PluginContext
from plugins_sdk.result import ok

app = PluginApp()


@app.capability("pre_submit")
def pre_submit(context: PluginContext, payload: bytes):
    return ok({
        "business": {
            "message": "plugin initialized",
            "model": context.model.to_dict() if context.model else {}
        }
    })


PLUGIN_APP = app
"#;

    fs::write(
        target_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )
    .with_context(|| {
        format!(
            "Failed to write {}",
            target_dir.join("manifest.json").display()
        )
    })?;
    fs::write(target_dir.join("main.py"), main_py)
        .with_context(|| format!("Failed to write {}", target_dir.join("main.py").display()))?;
    Ok(())
}

fn create_frontend_plugin(target_dir: &Path, code: &str, name: &str) -> Result<()> {
    fs::create_dir_all(target_dir).with_context(|| {
        format!(
            "Failed to create frontend plugin directory: {}",
            target_dir.display()
        )
    })?;

    let page_code = format!("{}-page", code.replace('_', "-"));
    let route_name = format!("plugin-{}", code.replace('_', "-"));
    let route_path = format!("/plugins/{}", code.replace('_', "-"));
    let file_name = format!("{}.vue", code.replace('_', "-"));
    let bundle_name = format!("{}.es.js", code.replace('_', "-"));

    let manifest = json!({
        "pluginCode": code,
        "pluginName": name,
        "version": "0.1.0",
        "pages": [
            {
                "code": page_code,
                "displayName": name,
                "route": {
                    "name": route_name,
                    "path": route_path
                },
                "entry": format!("./{}", file_name),
                "bundle": bundle_name,
                "entryExport": "default",
                "propsSchema": {
                    "type": "object"
                },
                "permissions": [],
                "hostFeatures": [],
                "layout": {
                    "fullWidth": true
                },
                "metadata": {}
            }
        ]
    });

    let vue_file = format!(
        "<template>\n  <section class=\"plugin-page\">\n    <h1>{}</h1>\n    <p>Plugin page generated by AsapFlow CLI.</p>\n  </section>\n</template>\n\n<script setup>\n</script>\n\n<style scoped>\n.plugin-page {{\n  padding: 24px;\n}}\n</style>\n",
        name
    );

    fs::write(
        target_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )
    .with_context(|| {
        format!(
            "Failed to write {}",
            target_dir.join("manifest.json").display()
        )
    })?;
    fs::write(target_dir.join(file_name), vue_file).with_context(|| {
        format!(
            "Failed to write Vue component under {}",
            target_dir.display()
        )
    })?;
    Ok(())
}

fn validate_manifest_by_path(path: &Path) -> Result<ManifestCheckResponse> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
    let value: Value = serde_json::from_str(strip_utf8_bom(&raw))
        .with_context(|| format!("Failed to parse manifest JSON: {}", path.display()))?;
    if value.get("pages").is_some() {
        validate_frontend_manifest(path)
    } else {
        validate_backend_manifest(path)
    }
}

fn validate_backend_manifest(path: &Path) -> Result<ManifestCheckResponse> {
    let value = load_manifest_value(path)?;
    let plugin_code = required_string(&value, "pluginCode")?;
    let version = required_string(&value, "version")?;
    let capabilities = value
        .get("capabilities")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Backend manifest must contain 'capabilities' array."))?;
    if capabilities.is_empty() {
        return Err(anyhow!(
            "Backend manifest must contain at least one capability."
        ));
    }

    for capability in capabilities {
        required_string(capability, "code")?;
        required_string(capability, "slot")?;
    }

    Ok(ManifestCheckResponse {
        kind: "backend".to_string(),
        path: path.display().to_string(),
        plugin_code,
        version,
    })
}

fn validate_frontend_manifest(path: &Path) -> Result<ManifestCheckResponse> {
    let value = load_manifest_value(path)?;
    let plugin_code = required_string(&value, "pluginCode")?;
    let version = required_string(&value, "version")?;
    let pages = value
        .get("pages")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Frontend manifest must contain 'pages' array."))?;
    if pages.is_empty() {
        return Err(anyhow!("Frontend manifest must contain at least one page."));
    }

    for page in pages {
        required_string(page, "code")?;
        let route = page
            .get("route")
            .ok_or_else(|| anyhow!("Frontend page must contain 'route' object."))?;
        required_string(route, "name")?;
        required_string(route, "path")?;
    }

    Ok(ManifestCheckResponse {
        kind: "frontend".to_string(),
        path: path.display().to_string(),
        plugin_code,
        version,
    })
}

fn load_manifest_value(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
    serde_json::from_str(strip_utf8_bom(&raw))
        .with_context(|| format!("Failed to parse manifest JSON: {}", path.display()))
}

fn preview_token(token: &str) -> String {
    if token.len() <= 12 {
        return token.to_string();
    }

    let prefix = &token[..8];
    let suffix = &token[token.len() - 4..];
    format!("{}...{}", prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_feature_preserves_existing_binding_when_input_omits_it() {
        let existing = json!({
            "id": "feature-1",
            "name": "订单管理",
            "module": "sales",
            "dataSourceType": "table",
            "dataSourceName": "sales_order",
            "description": "原始描述",
            "isActive": true,
            "metadata": "{}"
        });
        let input = json!({
            "name": "订单管理V2",
            "module": "sales",
            "description": "更新后的描述"
        });

        let payload = normalize_update_feature_payload(&existing, input).unwrap();

        assert_eq!(payload["name"], "订单管理V2");
        assert_eq!(payload["description"], "更新后的描述");
        assert_eq!(payload["dataSourceName"], "sales_order");
        assert_eq!(payload["dataSourceType"], "table");
    }

    #[test]
    fn update_feature_maps_entity_code_and_normalizes_fields() {
        let existing = json!({
            "id": "feature-1",
            "name": "订单管理",
            "module": "sales",
            "dataSourceType": "table",
            "dataSourceName": "sales_order",
            "description": "",
            "isActive": true,
            "metadata": "{}"
        });
        let input = json!({
            "name": "产品管理",
            "module": "catalog",
            "entityCode": "product",
            "fields": [
                {
                    "displayName": "产品名称",
                    "dataType": "string",
                    "sourceField": "product_name"
                }
            ]
        });

        let payload = normalize_update_feature_payload(&existing, input).unwrap();

        assert_eq!(payload["dataSourceName"], "product");
        assert_eq!(payload["fields"][0]["fieldKey"], "product_name");
        assert_eq!(payload["fields"][0]["sourceType"], "entity");
    }

    #[test]
    fn json_input_accepts_utf8_bom() {
        let value = parse_inline_json("\u{feff}{\"pageSize\":20}").unwrap();

        assert_eq!(value["pageSize"], 20);
    }

    #[test]
    fn manifest_file_accepts_utf8_bom() {
        let path = std::env::temp_dir().join(format!(
            "asapflow-bom-manifest-{}-{}.json",
            std::process::id(),
            unix_timestamp()
        ));
        fs::write(
            &path,
            "\u{feff}{\"pluginCode\":\"bom_plugin\",\"version\":\"0.1.0\",\"capabilities\":[{\"code\":\"run\",\"slot\":\"test.run\"}]}",
        )
        .unwrap();

        let result = validate_manifest_by_path(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(result.plugin_code, "bom_plugin");
    }

    #[test]
    fn system_list_commands_accept_limit() {
        for command in ["list-entities", "list-features"] {
            let parsed =
                Cli::try_parse_from(["asapflow", "system", command, "--limit", "1"]).unwrap();
            match parsed.command {
                Commands::System(SystemCommand {
                    command: SystemSubcommands::ListEntities(args),
                })
                | Commands::System(SystemCommand {
                    command: SystemSubcommands::ListFeatures(args),
                }) => assert_eq!(args.limit, Some(1)),
                _ => panic!("expected metadata list command"),
            }
        }
        assert_eq!(
            metadata_list_path("/api/meta/entities", Some(1)),
            "/api/meta/entities?limit=1"
        );
    }

    #[test]
    fn frontend_working_dir_resolves_development_kit_layout() {
        let root = std::env::temp_dir().join(format!(
            "asapflow-frontend-workspace-{}-{}",
            std::process::id(),
            unix_timestamp()
        ));
        let frontend = root.join("plugin-workspace/frontend");
        fs::create_dir_all(&frontend).unwrap();
        fs::write(frontend.join("package.json"), "{}").unwrap();

        let resolved = resolve_frontend_working_dir_from(&root, None).unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(resolved, frontend);
    }

    #[test]
    fn plugin_root_falls_back_to_examples_for_requested_code() {
        let root = std::env::temp_dir().join(format!(
            "asapflow-plugin-root-{}-{}",
            std::process::id(),
            unix_timestamp()
        ));
        let examples = root.join("plugin-workspace/examples");
        let plugin_dir = examples.join("backend/supplier_guard");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("manifest.json"), "{}").unwrap();

        let resolved = resolve_plugin_workspace_for_code(&root, "supplier_guard").unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(resolved, examples);
    }

    #[test]
    fn inline_json_accepts_powershell_escaped_quotes() {
        let value = parse_inline_json(r#"{\"pageSize\":20}"#).unwrap();

        assert_eq!(value["pageSize"], 20);
    }

    #[test]
    fn inline_json_accepts_wrapped_powershell_escaped_quotes() {
        let value = parse_inline_json(r#"'{\"pageSize\":20}'"#).unwrap();

        assert_eq!(value["pageSize"], 20);
    }

    #[test]
    fn create_feature_disables_default_scenarios_for_empty_fields() {
        let payload = normalize_create_feature_payload(json!({
            "code": "BI_REPORT",
            "name": "BI 报表",
            "module": "qms",
            "fields": []
        }))
        .unwrap();

        assert_eq!(payload["initializeDefaultScenarios"], false);
        assert_eq!(payload["includeDefaultListActions"], false);
    }

    #[test]
    fn add_feature_fields_accepts_object_or_array_payload() {
        let object_payload = normalize_add_feature_fields_payload(json!({
            "fields": [
                {
                    "fieldKey": "sku",
                    "displayName": "SKU",
                    "dataType": "string"
                }
            ]
        }))
        .unwrap();
        assert_eq!(object_payload.len(), 1);
        assert_eq!(object_payload[0]["fieldKey"], "sku");

        let array_payload = normalize_add_feature_fields_payload(json!([
            {
                "fieldKey": "name",
                "displayName": "名称",
                "dataType": "string"
            }
        ]))
        .unwrap();
        assert_eq!(array_payload.len(), 1);
        assert_eq!(array_payload[0]["fieldKey"], "name");
    }

    #[test]
    fn update_feature_extracts_field_changes_from_root_or_feature_node() {
        let root = normalize_update_feature_field_changes(&json!({
            "name": "产品管理",
            "fields": [
                {
                    "fieldKey": "test_a",
                    "displayName": "测试A",
                    "dataType": "string"
                }
            ]
        }))
        .unwrap();
        assert_eq!(root.len(), 1);
        assert_eq!(root[0]["fieldKey"], "test_a");

        let nested = normalize_update_feature_field_changes(&json!({
            "feature": {
                "name": "产品管理",
                "fields": [
                    {
                        "fieldKey": "test_b",
                        "displayName": "测试B",
                        "dataType": "string"
                    }
                ]
            }
        }))
        .unwrap();
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0]["fieldKey"], "test_b");
    }

    #[test]
    fn bi_commands_parse_with_expected_names_and_published_flag() {
        let create = Cli::try_parse_from([
            "asapflow",
            "bi",
            "create",
            "--json",
            r#"{"code":"sales","name":"Sales","definition":{"queries":[],"components":[]}}"#,
        ])
        .unwrap();
        assert_eq!(create.command_name(), "bi.create");

        let execute = Cli::try_parse_from([
            "asapflow",
            "bi",
            "execute-query",
            "--dashboard-id",
            "dashboard-id",
            "--published",
            "--json",
            r#"{"queryId":"summary","parameters":{}}"#,
        ])
        .unwrap();
        assert_eq!(execute.command_name(), "bi.execute_query");
        match execute.command {
            Commands::Bi(BiCommand {
                command: BiSubcommands::ExecuteQuery(args),
            }) => assert!(args.published),
            _ => panic!("expected BI execute-query command"),
        }

        let save_homepage = Cli::try_parse_from([
            "asapflow",
            "bi",
            "save-homepage-binding",
            "--json",
            r#"{"scopeType":"global","scopeId":null,"dashboardId":"00000000-0000-0000-0000-000000000001","isActive":true}"#,
        ])
        .unwrap();
        assert_eq!(save_homepage.command_name(), "bi.save_homepage_binding");
    }
}
