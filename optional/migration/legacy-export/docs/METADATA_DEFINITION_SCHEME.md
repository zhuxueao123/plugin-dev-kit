# 元数据定义方案（实体/功能/场景/插件）

## 1. 文档目标与范围

本文描述 AsapFlow 当前实现中的“实体 + 功能 + 场景 + 插件”元数据方案，重点覆盖：

- 实体定义如何落库（表名、字段、约束）
- 功能与场景如何组织、绑定字段与动作
- 插件元数据如何声明与被平台消费
- 字段、关系、索引、版本、变更日志的元数据结构
- 系统保留字段与可变更边界
- 数据源与物理表映射规则

说明：本文基于当前代码实现整理，不是理想化草案。

## 2. 总体模型

实体建模相关核心表如下：

- `entity_definition`：实体主定义（逻辑实体 -> 物理对象映射）
- `entity_field`：实体字段定义
- `entity_relationship`：实体间关系定义
- `entity_index`：索引定义
- `entity_version`：发布版本与DDL快照
- `entity_change_log`：发布/回滚执行日志
- `entity_data_record`：实体数据记录（JSONB）

## 3. 实体主定义：entity_definition

### 3.1 表职责

`entity_definition` 用于描述一个业务实体的元信息，并确定它对应到哪个数据源、schema、物理表名。

### 3.2 字段定义

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `code` | `varchar(128)` | 是 | - | 实体编码（逻辑标识，唯一） |
| `name` | `varchar(128)` | 是 | - | 实体名称（展示用） |
| `module` | `varchar(64)` | 是 | - | 所属模块 |
| `storage_type` | `varchar(32)` | 是 | `table` | 存储类型（例如 `table`/`view`） |
| `schema_name` | `varchar(64)` | 是 | `public` | 物理 schema |
| `physical_name` | `varchar(128)` | 是 | - | 物理表名/视图名 |
| `data_source_code` | `varchar(64)` | 否 | `null` | 绑定数据源编码 |
| `schema_managed` | `boolean` | 是 | `true` | 是否由平台管理表结构 |
| `write_enabled` | `boolean` | 是 | `true` | 是否允许写操作 |
| `status` | `varchar(16)` | 是 | `draft` | 状态（如 `draft`/`published`） |
| `version` | `int` | 是 | `1` | 当前版本号 |
| `description` | `text` | 否 | `null` | 描述 |
| `last_version_id` | `uuid` | 否 | `null` | 最近版本ID |
| `metadata` | `jsonb` | 是 | `{}` | 扩展元数据 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |
| `published_at` | `timestamptz` | 否 | `null` | 发布时间 |

### 3.3 关键约束

- `code` 全局唯一
- `(data_source_code, schema_name, physical_name)` 组合唯一

这意味着同一数据源下不能重复映射到同一个物理对象。

## 4. 实体字段定义：entity_field

### 4.1 表职责

`entity_field` 描述一个实体的字段模型（字段编码、类型、精度、是否主键、顺序等）。

### 4.2 字段定义

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `entity_id` | `uuid` | 是 | - | 所属实体ID，外键到 `entity_definition.id` |
| `code` | `varchar(128)` | 是 | - | 字段编码 |
| `name` | `varchar(128)` | 是 | - | 字段名称 |
| `data_type` | `varchar(64)` | 是 | - | 逻辑数据类型 |
| `length` | `int` | 否 | `null` | 长度 |
| `precision` | `int` | 否 | `null` | 精度 |
| `scale` | `int` | 否 | `null` | 小数位 |
| `is_nullable` | `boolean` | 是 | `true` | 是否可空 |
| `is_primary` | `boolean` | 是 | `false` | 是否主键字段 |
| `default_value` | `text` | 否 | `null` | 默认值表达式 |
| `order_index` | `int` | 是 | `0` | 排序 |
| `category` | `varchar(32)` | 是 | `business` | 字段类别（`business`/`system`） |
| `metadata` | `jsonb` | 是 | `{}` | 扩展元数据 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

### 4.3 关键约束

- `(entity_id, code)` 组合唯一
- `entity_id` 外键级联删除（删除实体时删除字段）

## 5. 系统保留字段规则

当实体属于平台默认数据源（即 `data_source_code` 为空或默认源）时，系统自动注入以下字段：

| code | name | type | nullable | primary | category | order_index |
|---|---|---|---|---|---|---|
| `id` | Id | `uuid` | 否 | 是 | `system` | -100 |
| `created_at` | 创建时间 | `timestamp` | 否 | 否 | `system` | -90 |
| `created_by` | 创建人 | `uuid` | 否 | 否 | `system` | -80 |
| `updated_at` | 更新时间 | `timestamp` | 是 | 否 | `system` | -70 |
| `updated_by` | 更新人 | `uuid` | 是 | 否 | `system` | -60 |

同时有以下限制：

- 不允许新建与保留字段同名的业务字段
- 不允许修改系统字段
- 不允许删除系统字段

## 6. 关系定义：entity_relationship

### 6.1 表职责

`entity_relationship` 描述实体间关系（如一对多、多对多）及删除行为。

### 6.2 字段定义

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `source_entity_id` | `uuid` | 是 | - | 源实体 |
| `target_entity_id` | `uuid` | 是 | - | 目标实体 |
| `relationship_type` | `varchar(32)` | 是 | - | 关系类型 |
| `source_field_id` | `uuid` | 否 | `null` | 源字段 |
| `target_field_id` | `uuid` | 否 | `null` | 目标字段 |
| `bridge_entity_id` | `uuid` | 否 | `null` | 多对多桥表实体 |
| `on_delete_behavior` | `varchar(32)` | 是 | `restrict` | 删除行为 |
| `metadata` | `jsonb` | 是 | `{}` | 扩展元数据 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

## 7. 索引定义：entity_index

### 7.1 表职责

`entity_index` 描述实体上的二级索引信息。

### 7.2 字段定义

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `entity_id` | `uuid` | 是 | - | 实体ID |
| `index_name` | `varchar(128)` | 是 | - | 索引名 |
| `is_unique` | `boolean` | 是 | `false` | 是否唯一索引 |
| `index_type` | `varchar(32)` | 是 | `btree` | 索引类型 |
| `fields` | `jsonb` | 是 | - | 索引字段列表（有序） |
| `filter_condition` | `text` | 否 | `null` | 过滤条件（部分索引） |
| `metadata` | `jsonb` | 是 | `{}` | 扩展元数据 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

### 7.3 关键约束

- `(entity_id, index_name)` 组合唯一

## 8. 版本与变更日志

### 8.1 entity_version

记录实体版本发布信息与DDL快照。

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `entity_id` | `uuid` | 是 | - | 实体ID |
| `version_number` | `int` | 是 | - | 版本号 |
| `status` | `varchar(16)` | 是 | `pending` | 状态（如 `pending`/`applied`） |
| `ddl_script` | `text` | 是 | - | DDL脚本 |
| `definition_snapshot` | `jsonb` | 是 | - | 定义快照 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `applied_at` | `timestamptz` | 否 | `null` | 应用时间 |
| `rolled_back_at` | `timestamptz` | 否 | `null` | 回滚时间 |

约束：`(entity_id, version_number)` 组合唯一。

### 8.2 entity_change_log

记录发布/回滚执行结果。

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `entity_id` | `uuid` | 是 | - | 实体ID |
| `version_id` | `uuid` | 否 | `null` | 对应版本ID |
| `environment` | `varchar(64)` | 是 | - | 环境（dev/test/prod） |
| `action` | `varchar(32)` | 是 | - | 动作（publish/rollback） |
| `executed_by` | `uuid` | 是 | - | 执行人 |
| `executed_at` | `timestamptz` | 是 | `current_timestamp` | 执行时间 |
| `success` | `boolean` | 是 | - | 是否成功 |
| `message` | `text` | 否 | `null` | 日志消息 |
| `applied_ddl` | `text` | 否 | `null` | 实际执行DDL |

## 9. 数据记录：entity_data_record

### 9.1 表职责

`entity_data_record` 以 JSONB 形式承载实体数据记录，用于动态实体数据存储。

### 9.2 字段定义

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `entity_id` | `uuid` | 是 | - | 实体ID |
| `data_json` | `jsonb` | 是 | - | 实体记录内容 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_at` | `timestamptz` | 是 | `current_timestamp` | 更新时间 |

## 10. 命名与入库规范（当前实现）

### 10.1 归一化规则

- 标识类字段（如 `code`、`field_key`）当前只做 `Trim()`，不会自动强制小写
- 类型类字段（如 `storage_type`、`data_type`、`category`）入库时会转小写
- 可空字符串字段（如 `data_source_code`）会按空白归一为 `null`

### 10.2 推荐规范

尽管当前实现不强制，建议统一：

- 实体编码、字段编码、物理表名使用小写蛇形命名
- 避免大小写混用，降低跨数据库/脚本工具链歧义

## 11. 数据源与表结构托管策略

实体是否由平台托管物理结构，取决于 `data_source_code`：

- 当 `data_source_code` 为空或为默认数据源：
  - `schema_managed = true`
  - 系统自动补齐保留字段
  - `write_enabled` 最终会被视为 `true`
- 当 `data_source_code` 为非默认数据源：
  - `schema_managed = false`
  - 不自动补系统字段
  - `write_enabled` 按请求值生效

## 12. 生命周期与一致性

### 12.1 创建实体

1. 写入元数据（实体+字段）
2. 调用 schema 同步器创建/对齐物理结构
3. 若 schema 同步失败，回滚已写入元数据

### 12.2 字段变更

- 新增字段：先写元数据，再同步物理结构；失败则回滚元数据
- 修改字段：先更新元数据，再同步结构；失败则恢复原字段定义
- 删除字段：先删元数据，再删物理字段；失败则补回元数据

### 12.3 删除实体

- 会尝试先删除物理结构，再删除元数据
- 即使物理删除失败，仍会继续删除元数据（以清理业务侧定义）

## 13. 与功能元数据的关系

- `feature.data_source_type/name/code` 可绑定实体定义形成业务功能
- `feature_field`/`scenario_field` 负责界面层字段行为，不替代 `entity_field` 的物理结构定义

建议把“实体元数据”作为结构层真源，把“功能/场景元数据”作为视图与交互层真源。

## 14. 可直接复用的检查清单

- 实体编码是否唯一
- 同一数据源+schema+物理名是否唯一
- 字段编码在实体内是否唯一
- 是否误用了系统保留字段编码
- `storage_type`/`data_type`/`category` 是否符合预期小写值
- 外部数据源实体是否明确配置 `write_enabled`
- 发布与变更日志是否完整记录

## 15. 功能元数据（Feature Layer）

### 15.1 feature

`feature` 是业务功能容器，承接菜单、场景、字段池、权限归属。

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `code` | `varchar(64)` | 是 | - | 功能编码（全局唯一） |
| `name` | `varchar(128)` | 是 | - | 功能名称 |
| `module` | `varchar(64)` | 是 | - | 模块 |
| `data_source_type` | `varchar(32)` | 是 | `table` | 数据源类型 |
| `data_source_name` | `varchar(128)` | 否 | `null` | 数据源对象名（表/视图/API） |
| `data_source_code` | `varchar(64)` | 否 | `null` | 数据源编码 |
| `description` | `text` | 否 | `null` | 描述 |
| `is_active` | `boolean` | 是 | `true` | 是否启用 |
| `metadata` | `jsonb` | 是 | `{}` | 扩展配置（如 scenarioMappings） |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

关键约束：`code` 唯一。

### 15.2 feature_field

`feature_field` 是功能字段池，作为场景字段的复用源。

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `feature_id` | `uuid` | 是 | - | 所属功能 |
| `field_key` | `varchar(128)` | 是 | - | 字段键 |
| `display_name` | `varchar(128)` | 是 | - | 展示名 |
| `data_type` | `varchar(32)` | 是 | - | 数据类型 |
| `source_type` | `varchar(32)` | 是 | `entity` | 字段来源（如 `entity`/`related`/`expression`/`procedure`/`plugin`） |
| `source_config` | `jsonb` | 是 | `{}` | 来源配置 |
| `is_identifier` | `boolean` | 是 | `false` | 是否标识字段 |
| `default_visible` | `boolean` | 是 | `true` | 默认可见 |
| `default_editable` | `boolean` | 是 | `true` | 默认可编辑 |
| `validation_rules` | `jsonb` | 否 | `null` | 校验规则 |
| `metadata` | `jsonb` | 是 | `{}` | 扩展配置 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

关键约束：`(feature_id, field_key)` 组合唯一。

## 16. 场景元数据（Scenario Layer）

### 16.1 scenario

`scenario` 描述功能下的业务视图（列表、详情、报表等）。

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `feature_id` | `uuid` | 是 | - | 所属功能 |
| `code` | `varchar(64)` | 是 | - | 场景编码 |
| `name` | `varchar(128)` | 是 | - | 场景名称 |
| `scenario_type` | `varchar(32)` | 是 | `list` | 场景类型 |
| `description` | `text` | 否 | `null` | 描述 |
| `data_binding` | `jsonb` | 否 | `null` | 数据绑定条件 |
| `default_view` | `jsonb` | 否 | `null` | 默认视图配置 |
| `visibility_rule` | `jsonb` | 否 | `null` | 可见规则 |
| `is_default` | `boolean` | 是 | `false` | 是否默认场景 |
| `metadata` | `jsonb` | 是 | `{}` | 扩展配置（如 search） |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

关键约束：`(feature_id, code)` 组合唯一。

### 16.2 scenario_field

`scenario_field` 表示某场景对功能字段在某视图上下文的覆盖。

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `scenario_id` | `uuid` | 是 | - | 所属场景 |
| `feature_field_id` | `uuid` | 是 | - | 关联功能字段 |
| `view_context` | `varchar(32)` | 是 | `list` | 视图上下文（list/form/detail） |
| `order_index` | `int` | 是 | `0` | 排序 |
| `is_visible` | `boolean` | 是 | `true` | 是否显示 |
| `is_readonly` | `boolean` | 是 | `false` | 是否只读 |
| `is_required` | `boolean` | 是 | `false` | 是否必填 |
| `component_type` | `varchar(64)` | 否 | `null` | 组件类型 |
| `column_width` | `int` | 否 | `null` | 列宽 |
| `filterable` | `boolean` | 是 | `false` | 是否可筛选 |
| `override_rules` | `jsonb` | 否 | `null` | 覆盖规则 |
| `metadata` | `jsonb` | 是 | `{}` | 视图模式扩展元数据 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

关键约束：`(scenario_id, feature_field_id, view_context)` 组合唯一。

### 16.3 action 与 scenario_action

`action` 定义动作本体；`scenario_action` 把动作挂到场景并配置展示/执行细节。

`action` 字段：

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `feature_id` | `uuid` | 否 | `null` | 所属功能（为空代表系统动作） |
| `code` | `varchar(64)` | 是 | - | 动作编码 |
| `name` | `varchar(128)` | 是 | - | 动作名称 |
| `action_type` | `varchar(32)` | 是 | - | 动作类型 |
| `execution_mode` | `varchar(32)` | 是 | - | 执行模式 |
| `permission_code` | `varchar(128)` | 是 | - | 权限码 |
| `description` | `text` | 否 | `null` | 描述 |
| `is_system` | `boolean` | 是 | `false` | 是否系统动作 |
| `metadata` | `jsonb` | 是 | `{}` | 扩展配置（可含 pluginBinding） |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

关键约束：`(feature_id, code)` 组合唯一。

`scenario_action` 字段：

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `id` | `uuid` | 是 | - | 主键 |
| `scenario_id` | `uuid` | 是 | - | 场景ID |
| `action_id` | `uuid` | 是 | - | 动作ID |
| `display_name` | `varchar(128)` | 是 | - | 展示名 |
| `order_index` | `int` | 是 | `0` | 排序 |
| `layout_slot` | `varchar(32)` | 是 | `toolbar` | 布局位置 |
| `visibility_rule` | `jsonb` | 否 | `null` | 显示规则 |
| `confirmation` | `jsonb` | 否 | `null` | 二次确认配置 |
| `metadata` | `jsonb` | 是 | `{}` | 动作扩展配置 |
| `created_by` | `uuid` | 是 | - | 创建人 |
| `created_at` | `timestamptz` | 是 | `current_timestamp` | 创建时间 |
| `updated_by` | `uuid` | 否 | `null` | 更新人 |
| `updated_at` | `timestamptz` | 否 | `null` | 更新时间 |

关键约束：`(scenario_id, action_id)` 组合唯一。

### 16.4 筛选与数据范围表

用于场景可复用筛选与授权范围：

- `filter_preset`：`(feature_id, code)` 唯一
- `scenario_filter`：`(scenario_id, filter_preset_id)` 唯一
- `data_scope`：`(feature_id, code)` 唯一
- `scenario_scope`：`(scenario_id, data_scope_id)` 唯一

## 17. 插件元数据定义（Plugin Layer）

### 17.1 总体原则

当前实现中，插件 Manifest 不落数据库主表，主要通过文件系统加载并在内存聚合：

- 后端插件源码：`plugin-workspace/backend/<pluginCode>/manifest.json`
- 前端插件源码：`plugin-workspace/frontend/src/pages/<pluginCode>/manifest.json`
- 前端插件聚合产物：`plugin-workspace/frontend/dist/manifests/<pluginCode>.json`

平台在请求时汇总为统一 `PluginManifest`：

- `pluginCode`
- `pluginName`
- `version`
- `frontendVersion`
- `capabilities[]`
- `pages[]`
- `metadata`

### 17.2 后端能力元数据（PluginCapability）

| 字段 | 说明 |
|---|---|
| `code` | 能力编码 |
| `displayName` | 展示名 |
| `slot` | 能力插槽 |
| `description` | 描述 |
| `inputSchema` | 入参Schema |
| `defaultParams` | 默认参数 |
| `permissions` | 权限要求集合 |
| `timeoutSeconds` | 超时时间 |
| `auditLevel` | 审计级别 |

### 17.3 前端页面元数据（PluginPage）

| 字段 | 说明 |
|---|---|
| `code` | 页面编码 |
| `displayName` | 展示名 |
| `routeName` | 路由名 |
| `routePath` | 路由路径 |
| `modulePath` | 入口模块路径（兼容旧模式） |
| `bundlePath` | 打包产物路径（新模式） |
| `entryExport` | 导出名，默认 `default` |
| `propsSchema` | 页面入参Schema |
| `permissions` | 页面权限要求 |
| `hostFeatures` | 允许挂载的宿主功能 |
| `layout` | 布局信息 |
| `metadata` | 其他扩展 |

### 17.4 场景动作与插件绑定约定

动作可通过 `scenario_action.metadata` 或 `action.metadata` 挂接插件能力，当前解析约定为：

```json
{
  "pluginBinding": {
    "pluginCode": "order_check",
    "capabilityCode": "pre_submit",
    "triggerPhase": "before",
    "params": {
      "strict": true
    }
  }
}
```

规则：

- `pluginCode`、`capabilityCode` 必填
- `triggerPhase` 支持 `before`/`after`（默认 `before`）
- `params` 会与能力 `defaultParams` 合并
- 运行时会把 `pluginCode`、`capabilityCode`、`slot`、`requiredPermissions` 等补充进执行元数据

### 17.5 字段与插件联动约定

在 `feature_field.source_type = plugin` 时，可通过 `source_config` 指向插件能力（例如字段探测、动态计算）。建议结构：

```json
{
  "pluginCode": "order_check",
  "capabilityCode": "field_probe",
  "paramMapping": {
    "entityCode": "order"
  }
}
```

## 18. 联合建模建议（实体/功能/场景/插件）

1. 实体层负责“结构真源”：字段类型、主外键、物理落库。
2. 功能层负责“业务域真源”：数据源绑定、字段池、默认交互。
3. 场景层负责“视图真源”：展示字段、动作编排、筛选范围。
4. 插件层负责“扩展真源”：能力注册、页面注册、运行时执行。
5. 对插件相关信息，优先持久化“绑定关系”，不要把 Manifest 全量复制进数据库。

## 19. 代码实证补充（本轮问题）

本节仅给出“能从当前代码直接确认”的结论，并标注约束强度。

### 19.1 scenario_type 实际集合

已确认：`list`、`form`、`detail`。

- 后端默认值：`scenario.scenario_type` 默认 `list`。
- 后端默认场景创建：会创建一个 `list` 场景和一个 `form` 场景（代码名 `detail`，类型仍为 `form`）。
- 前端配置项：提供 `list/form/detail` 三种可选值。

约束强度：弱约束（数据库与请求模型是 `varchar + MaxLength`，未做枚举白名单校验）。

推荐：

- 列表场景统一使用 `list`。
- 明细/编辑类场景统一使用 `form` 或 `detail`，并在前端按“表单类场景”处理。

### 19.2 action_type、execution_mode 可选值与推荐用法

已确认可用值（来自系统种子 + 前端配置）：

- `action_type`：`execute`、`create`、`update`、`delete`、`view`、`save`
- `execution_mode`：`sync`、`async`

已确认事实：

- 系统内置动作种子包含 `create/update/delete/view/save`，且 `execution_mode` 全为 `sync`。
- 后端请求模型仅限制长度，不限制枚举。

约束强度：弱约束（以约定为主，非后端强校验）。

推荐：

- 标准 CRUD 场景优先使用：
  - `create`（新建）
  - `update`（编辑）
  - `delete`（删除）
  - `view`（查看）
  - `save`（表单保存）
- 插件型或通用执行入口使用 `execute`。
- `execution_mode` 默认用 `sync`；仅在动作可异步化且前端有状态回传方案时使用 `async`。

### 19.3 view_context 的 list/form/detail 边界

已确认：`scenario_field.view_context` 的运行约定是 `list/form/detail`。

- 后端 `NormalizeViewContext`：空值默认 `list`，非空仅做 `trim + lower`，不做白名单。
- 后端场景字段分组构建：会按 `list`、`form`、`detail` 三个上下文写入。
- 前端行为边界：`form` 与 `detail` 被统一视为“表单类场景”；`list` 独立。

约束强度：中等（虽然无枚举强校验，但上下文构建与UI逻辑都围绕三值运行）。

推荐：

- 列表展示字段放 `list`。
- 表单录入字段放 `form`。
- 详情只读展示字段放 `detail`（若不区分可先复用 `form`）。

### 19.4 feature.data_source_type 枚举与语义

已确认：

- 前端提供选项：`table`、`view`、`procedure`、`plugin`。
- 后端字段探测能力 `DetectColumns` 仅支持：`table/view/procedure`，其他值会报 `Unsupported data source type`。

约束强度：分层不一致。

- 持久化层：弱约束（`varchar`）。
- 能力层（字段探测）：强约束到 `table/view/procedure`。

语义建议：

- `table`：标准可写业务表。
- `view`：查询型数据源，通常只读。
- `procedure`：存储过程返回结构，字段来自探测结果。
- `plugin`：插件提供数据能力；当前不走后端通用字段探测链路，需要插件自描述字段或单独适配。

### 19.5 插件执行上下文默认注入参数

已确认默认上下文字段（后端触发 + runtime 解析）：

- 后端执行上下文基础键：`feature`、`scenario`、`action`、`tags`、`config`。
- 条件注入键：`user`（有用户上下文时）、`params`（默认参数或绑定参数存在时）、`metadata`（有绑定/能力元数据时）、`record`（针对记录操作且有 recordId 时）。
- Runtime unary 路径会追加 `request_id`，并解析：`params`、`metadata`、`raw_context`、`selection`、`config` 到 `PluginContext`。
- Runtime stream 路径构建 context_map 时默认带：`request_id`、`feature`、`scenario`、`action`、`user`、`tags`、`params={}`、`metadata={}`。

结论：插件可稳定依赖 `meta(feature/scenario/action/user/tags)` 与 `params/metadata`；`record/raw_context/selection` 属于场景相关增强上下文。

### 19.6 block/section/region 是否有独立元数据模型

当前代码库未发现独立的 `block/section/region` 实体表或领域模型。

- 在领域实体与初始化建表中，未看到对应独立表。
- 当前更接近“布局配置内嵌在 metadata/json 中”的模式（例如场景字段 metadata 中的 `layout/span`）。

结论：现阶段它们不是一级元数据模型，而是附着在场景/字段扩展配置上的布局语义。

### 19.7 待你确认的口径（建议）

以下项当前代码可跑通，但不是强约束，建议在规范中明确后续是否升级为后端硬校验：

1. 是否把 `scenario_type` 固定收敛为 `list/form/detail`（拒绝其他值）。
2. 是否把 `action_type` 固定收敛为 `execute/create/update/delete/view/save`。
3. 是否把 `execution_mode` 固定收敛为 `sync/async`。
4. `feature.data_source_type=plugin` 的字段模型来源是统一插件 schema，还是每插件自描述。
5. 是否引入独立 `layout_block/layout_section/layout_region` 模型，替代 metadata 内嵌布局。
