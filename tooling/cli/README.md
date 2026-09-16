# AsapFlow CLI

这是 AsapFlow 的统一对外 CLI 工程。

当前目标：

- 为 AI 和终端用户提供统一能力入口
- 通过平台 API 调用系统能力
- 在本地提供插件工程辅助命令

## 当前状态

已完成：

- Rust CLI 工程与统一命令入口 `asapflow`
- 配置读取与本地 token 保存
- 认证与环境命令：
  - `auth login`
  - `auth use-token`
  - `auth whoami`
  - `auth health-check`
- 系统结构命令：
  - `system create-entity`
  - `system get-entity`
  - `system add-entity-fields`
  - `system create-feature`
  - `system get-feature`
  - `system add-feature-fields`
  - `system update-feature`（当输入包含 `fields` 时，会自动按 `fieldKey` 对 feature 字段做新增/更新）
  - `system create-action`
  - `system create-menu`
  - `system create-action-menu`
- 流程命令：
  - `workflow create-definition`
  - `workflow update-definition`
  - `workflow get-definition`
  - `workflow publish-definition`
  - `workflow list-workbench`
  - `workflow start-instance`
- `workflow get-instance`
  - 返回审批上下文，不是普通实体详情
- `workflow execute-task-action`
- 身份与权限命令：
  - `identity list-permissions`
  - `identity list-roles`
  - `identity create-role`
  - `identity update-role`
  - `identity delete-role`
  - `identity list-users`
  - `identity create-user`
  - `identity assign-user-roles`
  - `identity get-user-permissions`
  - `identity get-password-policy`
  - `identity set-password-policy`
- 数据命令：
  - `data query-records`
  - `data get-record`
  - `data create-record`
  - `data update-record`
  - `data delete-record`
  - `data execute-action`
- 插件命令：
  - `plugin init`
  - `plugin validate`
  - `plugin manifest-check`
  - `plugin build-frontend`
  - `plugin reload`
  - `plugin list`
  - `plugin pages`
  - `plugin invoke`
  - `plugin pack`
  - `plugin publish`
  - `plugin release-status`
  - `plugin releases`
  - `plugin rollback`
  - `plugin workspace-status`
  - `plugin pull-package`
  - `plugin package`
  - `plugin deploy-package`
- 配置与运维命令：
  - `configuration list`
  - `configuration set`
  - `configuration delete`
  - `configuration get-file-storage`
  - `configuration set-file-storage`
  - `data-source list`
  - `data-source get`
  - `data-source validate`
  - `data-source create`
  - `data-source update`
  - `data-source delete`
  - `dictionary list`
  - `dictionary get`
  - `dictionary create`
  - `dictionary update`
  - `dictionary delete`
  - `scheduled-job list`
  - `scheduled-job get`
  - `scheduled-job create`
  - `scheduled-job update`
  - `scheduled-job delete`
  - `scheduled-job run`
  - `scheduled-job runs`
  - `import-export config`
  - `import-export template`
  - `import-export create-import-task`
  - `import-export create-export-task`
  - `import-export list-tasks`
  - `import-export download`
  - `feature-package export`
  - `feature-package import-preview`
  - `feature-package import`
  - `feature-package legacy-preview`
  - `feature-package legacy-convert`
  - `feature-package legacy-import`
  - `bi list`
  - `bi get`
  - `bi create`
  - `bi update`
  - `bi delete`
  - `bi publish`
  - `bi execute-query`
  - `bi list-published`
  - `bi get-published`
  - `bi homepage`
  - `bi list-homepage-bindings`
  - `bi save-homepage-binding`
  - `bi delete-homepage-binding`

未完成：

- 插件测试、打包、发布类命令
- 更丰富的错误码与 JSON Schema 约束
- 多看板标签页、看板模板与历史版本回滚命令
- BI 部门继承、用户个人看板副本和标签排序
- BI 标准功能查询、存储过程等更多数据源类型及更细插件数据权限协议

## 示例命令

```bash
asapflow auth login --base-url http://localhost:5000 --username admin --password admin123
asapflow auth use-token --base-url http://localhost:5000 --token asap_pat_xxx
asapflow auth whoami
asapflow auth health-check
asapflow --output whoami.json auth whoami
asapflow system create-entity --input customer.entity.json
asapflow workflow create-definition --input create_workflow.json
asapflow workflow publish-definition --definition-id <id> --input publish_workflow.json
asapflow workflow list-workbench --view cc
asapflow workflow start-instance --input start_workflow_instance.json --cc-user-id <user-id> --cc-user-id <user-id>
asapflow workflow start-instance --input start_workflow_instance.json
asapflow identity set-password-policy --input set_password_policy.json
asapflow data query-records --entity-code customer --input customer.query.json
asapflow plugin init --code supplier_guard --name "供应商校验插件"
asapflow plugin pack --code supplier_guard
asapflow plugin publish --code supplier_guard
asapflow plugin releases --plugin-code supplier_guard
asapflow plugin rollback --plugin-code supplier_guard --version 0.1.0
asapflow scheduled-job create --input scheduled_job.json
asapflow import-export list-tasks --page 1 --page-size 10
asapflow feature-package export --input feature_package_export.json --output-file package.zip
asapflow bi create --input create_bi_dashboard.json
asapflow bi execute-query --dashboard-id <dashboard-id> --input execute_bi_query.json
asapflow bi publish --dashboard-id <dashboard-id> --input publish_bi_dashboard.json
asapflow bi homepage
asapflow bi save-homepage-binding --input save_bi_homepage_binding.json
```

## 建模输入约定

首批建模命令已经开始对底层 REST DTO 做收口。

推荐直接使用面向业务对象的简化 JSON，而不是手写底层包裹结构：

`system create-entity` 示例：

```json
{
  "code": "customer",
  "name": "客户",
  "module": "crm",
  "description": "客户主数据",
  "fields": [
    {
      "code": "name",
      "name": "客户名称",
      "dataType": "string",
      "length": 128,
      "isNullable": false
    },
    {
      "code": "mobile",
      "name": "手机号",
      "dataType": "string",
      "length": 32
    }
  ]
}
```

`system create-feature` 示例：

```json
{
  "code": "customer_management",
  "name": "客户管理",
  "module": "crm",
  "entityCode": "customer",
  "fields": [
    {
      "fieldKey": "name",
      "displayName": "客户名称",
      "dataType": "string",
      "sourceField": "name",
      "isIdentifier": true
    }
  ]
}
```

说明：

- 默认不需要显式提供 `dataSourceCode`
- 如果 feature 对应某个实体，优先传 `entityCode`，CLI 会自动把它映射为 feature 的实体绑定
- 对于 `sourceType = entity` 的功能字段，统一使用实体原始字段名；如果只传 `sourceField`，CLI 会默认用它作为 `fieldKey`
- 系统使用类命令一直只面向 `entityCode`
- 如确实存在建模期绑定特定业务数据源的场景，仍可在建模输入里显式提供 `dataSourceCode`
- `system add-entity-fields` 也支持直接传字段数组或 `{ "fields": [...] }`
- `system add-feature-fields` 也支持直接传字段数组或 `{ "fields": [...] }`
- `system create-action`、`system create-menu` 支持简化业务输入，CLI 会自动补齐默认字段
- `workflow start-instance` 除了可在 JSON 中传 `ccUserIds`，也支持通过多个 `--cc-user-id` 参数覆盖注入

## 配置来源

CLI 当前支持三种配置来源，优先级从高到低：

1. 命令参数
2. 环境变量
3. 本地配置文件

环境变量：

```bash
ASAPFLOW_BASE_URL=http://localhost:5000
ASAPFLOW_TOKEN=asap_pat_xxx
```

本地配置文件路径：

- 优先读取可执行文件同目录的 `config.json`
- 如果同目录不存在，则回退到用户配置目录
- macOS 用户配置目录通常为 `~/Library/Application Support/asapflow/config.json`
- Linux 用户配置目录通常为 `~/.config/asapflow/config.json`
- Windows 用户配置目录通常为 `%APPDATA%\\asapflow\\config.json`

## 文件输出

如果调用环境无法稳定捕获 stdout，可给任意命令增加全局参数：

```bash
asapflow --output result.json auth whoami
```

CLI 会把成功或失败的 JSON 结果写入指定文件。

## 发布说明

跨平台发布方案见：

- [CLI_RELEASE_PLAN.md](/Users/mac4/Workspace/AsapFlow/docs/CLI_RELEASE_PLAN.md)
