# AsapFlow CLI 使用说明

## 1. 配置文件

CLI 会优先读取可执行文件同目录下的：

- `config.json`

最小配置格式：

```json
{
  "baseUrl": "http://127.0.0.1:5189",
  "token": "replace-with-user-service-token"
}
```

说明：

- `baseUrl` 填后端 API 地址
- `token` 填平台生成的 Service Token
- Service Token 由当前登录用户在后台创建
- Service Token 调用时会动态继承创建者当前的角色权限、菜单权限和数据范围
- 普通用户只能管理自己的 token，管理员可以管理全部 token

## 1.1 术语

- `feature`：功能，例如“供应商管理”“订单管理”
- `scenario`：场景，是功能下的页面/视图，例如 `list`、`detail`
- `action`：动作，是场景中的按钮或操作，例如“新建”“编辑”“删除”“查看”

说明：

- `action` 不等于 `scenario`
- 默认情况下，一个功能可以包含多个场景；一个场景可以绑定多个动作

## 2. 调用方式

CLI 对外只保留按模块分组的命令，不再提供单独的 `run <capability>` 别名层。

### 2.1 分组命令

适合人工使用：

```bash
asapflow auth whoami
asapflow system create-entity --input customer.entity.json
asapflow system create-entity --if-exists skip --input customer.entity.json
asapflow system add-feature-fields --code customer_management --input add_feature_fields.json
asapflow system create-scenario --feature-id <feature-id> --input order-in-progress.scenario.json
asapflow identity create-role --input sales-role.json
asapflow identity set-password-policy --input set_password_policy.json
asapflow number-rule bind-field --entity-code sales_order --field-code order_no --rule-code sales_order_no
asapflow workflow create-definition --input create_workflow.json
asapflow workflow publish-definition --definition-id <id> --input publish_workflow.json
asapflow workflow list-workbench --view cc
asapflow workflow start-instance --input start_workflow_instance.json --cc-user-id <user-id>
asapflow number-rule create --input create_number_rule.json
asapflow number-rule preview --input preview_number_rule.json
asapflow bi create --input create_bi_dashboard.json
asapflow bi execute-query --dashboard-id <dashboard-id> --input execute_bi_query.json
asapflow bi publish --dashboard-id <dashboard-id> --input publish_bi_dashboard.json
asapflow data query-records --entity-code customer --input customer.query.json
asapflow plugin init --code supplier_guard --name "供应商校验插件"
```

如果在 Windows PowerShell 中从当前目录运行，可执行文件应写成：

```powershell
.\asapflow.exe auth whoami
.\asapflow.exe auth health-check
```

原因：

- PowerShell 默认不会从当前目录自动解析可执行文件
- 如果 `asapflow.exe` 不在 `PATH` 中，就必须显式写 `.\`

如果你同时放置了 `asapflow.ps1` 包装脚本，PowerShell 中更推荐：

```powershell
.\asapflow.ps1 auth whoami
.\asapflow.ps1 data query-records --entity-code customer --input query_records.json
```

说明：

- `asapflow.ps1` 会自动调用同目录下的 `asapflow.exe`
- 如果未显式传 `--output`，脚本会自动使用系统临时目录中的临时文件接收 JSON，再把内容输出回 PowerShell
- 这比直接运行原生 `.exe` 更适合部分 AI 工具或 VS Code 终端工具层

macOS / Linux 下如果同时提供了 `asapflow.sh`，也推荐优先调用：

```bash
./asapflow.sh auth whoami
./asapflow.sh data query-records --entity-code customer --input -
```

## 3. 输入方式

所有命令优先使用 JSON 输入。

支持两种方式：

1. 文件输入

```bash
asapflow system create-entity --input customer.entity.json
```

2. 内联 JSON

```bash
asapflow data query-records --entity-code customer --json "{\"pageSize\":20}"
```

说明：

- CLI 现在会尽量容忍外层再包一层单引号或双引号的写法
- 但在 Windows 终端和 AI 工具场景下，仍然优先推荐 `--input <json-file>`
- 复杂 JSON 不建议长期依赖内联字符串
- 如果输入只是一次性临时数据，推荐使用 `--input -` 从标准输入读取，避免在插件工作区生成临时 JSON 文件

Windows PowerShell 示例：

```powershell
@'
{
  "pageSize": 20
}
'@ | .\asapflow.ps1 data query-records --entity-code customer --input -
```

macOS / Linux 示例：

```bash
cat <<'EOF' | ./asapflow.sh data query-records --entity-code customer --input -
{
  "pageSize": 20
}
EOF
```

## 4. 输出方式

默认情况下，CLI 将 JSON 结果输出到标准输出。

如果 AI 工具或终端环境无法稳定捕获 Windows `.exe` 的标准输出，推荐统一使用：

```powershell
.\asapflow.exe --output result.json auth whoami
.\asapflow.exe --output result.json data query-records --entity-code customer --input query_records.json
```

如果同时提供了 `asapflow.ps1`，也可以直接让 AI 优先调用：

```powershell
.\asapflow.ps1 auth whoami
.\asapflow.ps1 system create-entity --input create_entity.json
```

说明：

- `--output` 是全局参数，所有命令都支持
- 所有通过 `--input` 读取的 JSON 以及插件 manifest 均支持 UTF-8 BOM
- `system list-entities --limit 1` 和 `system list-features --limit 1` 可用于快速连通性检查，避免输出完整元数据集合
- 成功时写入成功 JSON
- 失败时写入失败 JSON
- 这样 AI 工具只需要读取文件，不依赖终端 stdout 捕获
- 如果必须显式指定 `--output`，建议写入系统临时目录，不要写到插件工作区根目录
- Windows PowerShell 5.1 读取 `--output` 文件时建议显式指定 UTF-8：

```powershell
Get-Content -Path result.json -Raw -Encoding UTF8 | ConvertFrom-Json
```

## 5. 输出格式

CLI 统一输出 JSON。

成功示例：

```json
{
  "success": true,
  "command": "system.create_entity",
  "data": {}
}
```

失败示例：

```json
{
  "success": false,
  "command": "system.create_entity",
  "error": {
    "message": "..."
  }
}
```

## 5.1 迁移编排建议顺序

如果目标是把老系统功能迁到 AsapFlow，推荐让 AI 按这个顺序调用：

1. `system create-entity`（迁移重跑时可加 `--if-exists skip`）
2. `system add-entity-fields`
3. `system update-entity` / `system update-entity-field` / `system delete-entity-field`
4. `system create-feature`（迁移重跑时可加 `--if-exists skip`）
5. `system add-feature-fields`（推荐专门用于给已有 feature 增量补字段）
6. `system update-feature`
7. `system create-scenario` / `system update-scenario`
8. `system create-action` / `system update-action`
9. `system create-menu` / `system update-menu`
10. `number-rule create`
11. `number-rule bind-field`
12. `identity create-role`
13. `identity create-user`
14. `identity assign-user-roles`
15. `identity set-password-policy`

实体字段输入规则：`add-entity-fields` 会把对象型 `metadata` 序列化为后端契约要求的 JSON 字符串，并按数据类型忽略无效的 `length/precision/scale`；批量失败时错误会指明字段编码和输入序号。删除普通字段可使用 `system delete-entity-field --entity-code <code> --field-code <field>`，该操作会同步删除物理列。

## 6. 重要规则

1. 创建实体、功能、菜单等建模能力，默认不要求显式指定 `dataSourceCode`
2. `workflow start-instance` 支持两种抄送写法：
   - 在 JSON 请求体中写 `ccUserIds`
   - 在命令行追加多个 `--cc-user-id`
3. `workflow list-workbench --view` 当前允许值为 `todo`、`started`、`done`、`cc`、`finished`
4. 创建 feature 时，如果它对应某个实体，应传 `entityCode`
5. 创建 feature 时，如果包含字段且未显式关闭默认场景初始化，系统会默认创建 `list` 和 `detail` 场景；如果 `fields` 为空，CLI 默认跳过默认场景和默认动作初始化
6. 默认情况下，`list` 场景会自动附带四个系统动作：`create`、`edit`、`delete`、`view`
5. 如果客户明确要求不要这四个系统动作，应在 `create-feature` 输入中传 `includeDefaultListActions = false`
6. 查询和写入已有实体的数据时，不需要关心业务库
7. 业务库路由由服务端根据元数据自动解析
8. 插件命令只操作本地工作目录，除 `plugin reload` 这类运行管理命令外，不直接作用于服务器源码
9. 创建流程时，优先走 `workflow create-definition -> workflow publish-definition`
10. 流程实例跳转到业务页面时，建议在发起输入中补齐 `featureCode`、`scenarioCode`、`mode`
11. 需要自动编号时，先创建编号规则，再在实体字段 `metadata.generator` 中引用 `ruleCode`
12. `create-feature` 只会创建默认 `list` / `detail` 场景，不会自动把子表实体挂成表单明细区
13. 如果旧功能存在主表 + 子表结构，必须额外执行 `system update-scenario`，在 `detail` 场景的 `metadata.detailTables` 中显式配置子表
14. `metadata.detailTables` 至少要包含 `entityCode`、`relation.parentKey`、`relation.childKey` 和 `columns`
15. 更新场景或菜单前必须先读取当前对象整包；`system update-scenario` / `system update-menu` 更接近整包覆盖，不能只凭想象发送局部片段
16. 表单场景如果要支持提交保存，必须显式为该场景配置 `save` 动作；仅有 `create/edit/view` 跳转并不会让表单页自动出现保存按钮
17. 如果表单字段很多，或旧平台已经有字段块/分组，应在表单场景 `metadata.formLayout` 中配置 `type = tabs`；示例见 `examples/update_detail_scenario_tabs.json`
18. 删除菜单、动作、场景、功能和实体时，应按引用关系从外到内处理，例如先删除菜单，再删除功能，最后删除实体
19. `system delete-entity` / `system delete-feature` 支持用 `--code` 自动解析 ID；`system delete-menu` / `system delete-action` / `system delete-scenario` 使用 ID 删除
20. 给已有 feature 补字段时，优先使用 `system add-feature-fields`；如果走 `system update-feature` 并包含 `fields`，CLI 也会按 `fieldKey` 自动做新增/更新同步
21. `bi update` 会整包更新看板草稿；修改前必须先执行 `bi get` 并保留未修改的查询和组件
22. `bi execute-query` 默认执行草稿查询，只有显式增加 `--published` 才执行发布版本
23. BI 查询支持 SQL、插件接口和静态数据；SQL 必须使用参数绑定，CLI 不在本地执行 SQL，数据库类型识别和安全校验都由后端完成
24. BI 主页绑定使用 `bi save-homepage-binding`，不要写进看板 `definition` JSON

## 7. 修改已有功能/场景时的建议

1. 先读取当前定义，再修改
2. 修改 feature 时，保留正确的 `entityCode` / 实体绑定；如果使用更新命令，输入中仍建议显式提供 `entityCode` 或当前 `dataSourceName`
3. 修改 scenario 时，优先参考：
   - `examples/update_feature_reference.json`
   - `examples/update_scenario_reference.json`
4. 不要只凭想象发送残缺的 `actions` 或 `fieldGroups`
5. 如果 feature 有主从表，不要假设子表实体创建完成后，表单页会自动出现明细列表
6. 如果编辑的是 `form/detail` 场景，确认 `actions` 中包含 `save`；场景配置页里看到建议勾选并不代表已经落库
7. `system update-feature --feature-id <id>` 场景已兼容；CLI 会内部解析 feature 列表后再执行更新，不再依赖后端按 id 的读取接口
8. 如果编辑的是字段较多的 `form/detail` 场景，确认 `metadata.formLayout` 已保留或补齐；缺失时运行页会回退为平铺表单
