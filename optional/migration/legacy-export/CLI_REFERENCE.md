# legacy-export

老系统元数据导出 CLI。

当前版本目标：

- 读取 `config.json`
- 按 `funcId`、菜单目录或范围文件导出老系统功能包
- 产出 `raw/`、`normalized/`、`handoff/`、`plugin_context/`、`migration_readiness.json`
- 支持系统级配置与系统数据导出
- 为下一步 AI 重写插件提供插件绑定、SQL 依赖提示和重写交接文件

## 配置文件

请基于 `config.example.json` 创建本地 `config.json`，不要把真实连接配置随交付包分发。

`config.example.json` 示例：

```json
{
  "queryRunner": {
    "type": "shell-template",
    "commandTemplate": "sqlcmd -S 127.0.0.1 -d LegacyDb -U sa -P secret -W -w 65535 -f 65001 -s \"|\" -i {sql_file}"
  },
  "output": {
    "defaultDirectory": "exports"
  }
}
```

说明：

- 当前版本通过外部 SQL 客户端执行查询
- `commandTemplate` 必须包含 `{sql_file}`
- Windows `sqlcmd` 建议使用 `-f 65001 -s "|"`，避免中文字段名乱码以及 `\t` 被当作普通文本分隔符
- `config.json`、scope JSON 和工具读取的中间 JSON 均支持 UTF-8 BOM；查询输出兼容 UTF-8、UTF-16 和 SQLCMD 的 `|`、Tab、反斜杠分隔
- `{sql_file}` 会由 CLI 按当前平台自动加引号；模板可写成 `-i {sql_file}` 或 `-i "{sql_file}"`，CLI 会避免重复加引号
- CLI 会把 SQL 写到临时文件后再执行
- 查询命令需要输出带表头的制表符分隔结果
- 已兼容 `sqlcmd` 常见的“表头 + 分隔线 + 数据行”格式

## 用法

```bash
./legacy-export --help
./legacy-export export-function --help
./legacy-export export-function --config config.json --func-id SD_T_SO --out exports
./legacy-export inspect-function --config config.json --func-id SD_T_SO
./legacy-export batch-export --config config.json --func-file funcs.txt --out exports
./legacy-export export-menu-subtree --config config.json --menu-name "站点配送" --out exports
./legacy-export export-scope --config config.json --scope-file scope.json --out exports
./legacy-export export-system --config config.json --out exports
./legacy-export export-serials --config config.json --out exports
./legacy-export export-workflows --config config.json --out exports
./legacy-export export-security --config config.json --out exports
./legacy-export export-users --config config.json --out exports
./legacy-export export-org --config config.json --out exports
./legacy-export export-menus --config config.json --out exports
./legacy-export export-lists --config config.json --out exports
```

构建：

交付目录中的可执行文件：

```bash
./legacy-export
```

## 输出结构

```text
exports/
  functions/
    SD_T_SO/
      manifest.json
      migration_readiness.json
      raw/
      normalized/
      handoff/
      plugin_context/
  scopes/
    last_scope_manifest.json
  system/
    manifest.json
    migration_readiness.json
    metadata/
    menus/
    serials/
    workflows/
    security/
    users/
    org/
    lists/
    data_catalog/
```

说明：

- `raw/`：老系统原始查询结果
- `normalized/`：标准化后的 feature/scenario/entity/action/plugin 语义
- `handoff/`：给下一步 AI 编排直接使用的草稿文件
- `plugin_context/`：给 AI 重写旧插件用的上下文与数据语义辅助文件
- `system/`：全系统配置元数据与系统基础数据导出，不包含业务交易数据

当前 `handoff/` 会生成：

- `create_feature_seed.json`
- `scenario_update_seed.json`
- `scenario_update_seeds.json`
- `plugin_analysis_seed.json`
- `summary.md`

`scenario_update_seed.json` 优先给出表单场景草稿；`scenario_update_seeds.json` 保留全部场景草稿。表单草稿会尽量生成 `metadata.formLayout`、`metadata.detailTables` 和控件迁移提示。关系、字典以及主从表关联键仍必须在应用前人工复核。

归一化时会优先把名称包含 `MAINBLOCK`（包括 `BMAINBLOCK`）的 block 识别为主信息块，并从该 block 推导主实体；主信息块不会进入 `detailTables`。明细列可在 block field 未重复携带 `TABLEID` 时通过 block 反查实体。主从表存在同名业务键时优先使用 `CODE_* = CODE_*`，其次才使用 ID 字段或兜底值。

当前 `plugin_context/` 会生成：

- `call_context.json`
- `entity_field_catalog.json`
- `related_table_catalog.json`
- `plugin_bindings.json`
- `sql_dependencies.json`
- `plugin_rewrite_handoff.json`

额外会生成：

- `migration_readiness.json`
  - 功能级或系统级迁移风险分级与下一步建议

说明：

- 这里包含的是插件入口元数据与调用上下文，不包含插件源码文件本身
- 建议把老系统插件代码库与本 CLI 导出结果一起提供给 AI
- 当前插件入口引用来源于老系统元数据中的 `库名.类.方法名` 或兼容变体
- 后续 AI 应根据这些入口信息到插件代码库中定位真实源码并继续分析方法调用链
- `sql_dependencies.json` 当前是基于元数据和插件入口的保守依赖提示，不做插件源码级 SQL 静态分析
- `plugin_rewrite_handoff.json` 是给 AI 重写插件时直接消费的总入口文件

## `export-system` 范围

当前系统级导出包含：

- `metadata/`
  - 实体、功能、场景、字段、按钮、事件、扩展属性
- `menus/`
  - 功能树菜单、`menu_tree_seed.json` 与可选用户菜单视图
- `serials/`
  - 编号规则、编号种子、实体与编号规则绑定
- `workflows/`
  - 流程定义、阶段、连线、动作、流程脚本
- `security/`
  - 角色、角色用户、角色功能、角色场景、角色目录
- `users/`
  - 用户与部分用户级配置
- `org/`
  - 组织、人员及相关组织视图
- `lists/`
  - 系统列表与列表明细
- `data_catalog/`
  - 面向 AI 重写旧插件 SQL 的实体、字段、关系、表语义目录

明确不包含：

- 订单、采购、库存、财务等业务交易数据
- 流程实例运行数据
- 日志、历史、锁表、临时业务表

## 范围导出

推荐使用 `export-scope` 定义稳定迁移范围。

`scope.json` 示例：

```json
{
  "features": ["SD_T_SO", "SD_T_SHP"],
  "menus": ["站点配送"],
  "includeSystemAssets": true
}
```

说明：

- `features`：显式指定功能编码
- `menus`：按菜单子树追加功能范围
- `includeSystemAssets`：是否同时执行 `export-system`

## 批量导出失败语义

`batch-export` 会自动剥离 `--func-file` 首行 UTF-8 BOM，并对每行功能编码做首尾空白清理。任一子项失败时，顶层 JSON 输出 `success: false`、`failedCount > 0`，进程退出码为非 0。

## 菜单树 seed

`export-system` 与 `export-menus` 会额外输出 `menus/menu_tree_seed.json`。其中 `entries` 按父菜单优先顺序排列，保留 `legacyCode`、`parentLegacyCode` 与 `createRequest`。导入新平台时先创建父级 `group` 菜单，再将 `scenario` 菜单里的 `featureCode` / `scenarioCode` 解析成目标环境的 `featureId` / `scenarioId`。

## 帮助信息

CLI 已内置帮助：

```bash
./legacy-export --help
./legacy-export export-function --help
./legacy-export export-scope --help
```

## Schema 与样例

请同时参考：

- `docs/EXPORT_SCHEMA.md`
- `examples/sample-output/`

## 当前交付边界

当前版本适合作为“迁移编排输入包”交付，建议一起交付给 AI 的内容包括：

1. 本 CLI 二进制与 `config.example.json`
2. 本 CLI 导出的 `exports/`
3. 老系统插件代码库
4. 新系统 CLI 文档与建模文档

当前版本已经覆盖：

- 老系统配置元数据导出
- 系统基础数据导出
- 功能级插件入口引用导出
- 面向插件 SQL 重写的数据语义辅助目录

当前版本尚不直接提供：

- 插件源码打包
- 插件方法调用链静态分析结果
- 业务交易数据迁移

因此，推荐联调方式是：

1. 先用本 CLI 导出范围内元数据和系统数据
2. 把导出结果与插件代码库一起交给 AI
3. 由 AI 使用新系统 CLI 建模并重写插件

## 按菜单目录导出

`export-menu-subtree` 会：

1. 在 `SYS_Function` 中按 `IDNUM / DES1 / DES2` 匹配菜单名称
2. 递归收集该菜单节点下所有子功能
3. 过滤掉仅作为目录节点、没有实际功能绑定的菜单项
4. 逐个复用 `export-function` 导出功能包
