# 示例说明

本目录下的 JSON 文件是各命令的最小输入形状，不保证都能在空环境中直接执行。

## 可直接作为起点的示例

- `create_entity.json`：创建实体。
- `create_feature.json`：创建功能。若对应已有实体，请先创建实体并在输入中填写 `entityCode`。
- `add_feature_fields.json`：给已有 feature 增量补字段。支持数组或 `{ "fields": [...] }`。
- `update_detail_scenario_tabs.json`：更新表单/详情场景，为大量字段配置标签页分组。
- `create_menu.json`：创建菜单。若是场景菜单，请先创建对应 feature/scenario 并填写返回的 ID。
- `query_records.json`：查询数据。执行前需要已有对应 `entityCode`。

## 依赖已有对象的示例

- `preview_number_rule.json`：依赖已存在的编号规则，先执行 `number-rule create --input create_number_rule.json`，再把 `ruleCode` 改成实际规则编码。
- `create_role.json`：依赖权限编码已存在，先执行 `identity list-permissions` 获取可用权限后再替换 `permissionCodes`。
- `publish_workflow.json`：依赖已存在的流程定义，先执行 `workflow create-definition --input create_workflow.json` 并使用返回的定义 ID。
- `execute_workflow_action.json`：依赖已存在的待办任务和动作编码，先通过 `workflow list-workbench --view todo` 获取任务。
- `start_workflow_instance.json`：可直接补充 `ccUserIds`，或在命令行追加 `--cc-user-id` 来指定抄送用户。
- `assign_user_roles.json`：依赖已存在用户和角色，先执行 `identity list-users` 与 `identity list-roles`。
- `set_password_policy.json`：管理员配置密码强度与失效期限；留空 `expireDays` 表示不过期。
- `create_bi_dashboard.json`：示例 SQL 依赖目标数据源中已经存在 `sales_order` 表及对应字段；它不会创建表或数据源。
- `create_bi_dashboard_plugin.json`：示例插件接口依赖目标插件已安装并暴露对应能力。
- `create_bi_dashboard_static.json`：示例静态数据直接写在看板定义中，适合少量手工目标值或说明性数据。
- `execute_bi_query.json`：需要先创建示例看板并使用返回的 dashboard id。
- `publish_bi_dashboard.json`：需要看板内所有 SQL 已通过实际数据库类型的校验；插件接口与静态数据也必须满足对应定义校验。

BI 主页绑定使用 `bi save-homepage-binding` 单独维护，不写入看板定义 JSON。示例暂未覆盖多标签、部门继承和个人副本。

复杂迁移场景建议优先使用文件输入或 `--input -`，不要长期依赖内联 `--json`。
