# 老系统到新系统元数据映射方案

## 1. 文档目标

本文定义 Asapment 老系统元数据模型到 AsapFlow 新系统元数据模型的映射规则，用于支持：

- 老系统元数据导出 CLI 的输出设计
- AI 基于导出结果分析插件源码与补全依赖
- 新系统 CLI 按标准语义创建实体、功能、场景、动作与插件绑定

本文关注“迁移语义映射”，不是逐字段复制，也不是最终导入协议。

## 2. 映射原则

### 2.1 三层语义

迁移过程分三层：

1. 老系统原始字段
2. 迁移标准语义
3. 新系统落库对象

不要直接把老系统字段名映射成新系统字段名。应先归一到迁移标准语义，再落到新系统模型。

### 2.2 真源边界

- 老系统 `SYS_Table*` 是结构真源
- 老系统 `SYS_Function / SYS_Group / Block / Event` 是功能与场景真源
- 老系统插件源码是隐式依赖真源
- 新系统 `entity_*` 是结构落点
- 新系统 `feature / scenario / action` 是功能落点
- 新系统插件 manifest 与 `pluginBinding` 是扩展落点

### 2.3 功能是迁移入口

迁移以功能为入口，不以单表为入口。

原因：

- 同一张表在不同场景下有不同展示与交互语义
- 事件、按钮、字段覆盖都挂在功能/场景上
- 插件依赖常超出功能主实体范围

因此导出单位应为“功能包”，表定义作为功能包的依赖资产。

## 3. 对象映射总览

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `SYS_Table` | `entity` | `entity_definition` |
| `SYS_TableField` | `entity_field` | `entity_field` |
| `SYS_TableIndex` | `entity_index` | `entity_index` |
| `SYS_TableFK` | `entity_relationship` | `entity_relationship` |
| `SYS_EXTPROP(PROPTYPE=TABLE/TABLEFIELD)` | `entity_metadata` | `entity_definition.metadata` / `entity_field.metadata` |
| `SYS_Function` | `feature` | `feature` |
| `SYS_Group` | `scenario` | `scenario` |
| `V_SYS_GroupBlock` | `scenario_view_binding` | `scenario.data_binding/default_view/metadata` |
| `V_SYS_GroupBlockField` | `feature_field + scenario_field` | `feature_field` / `scenario_field` |
| `SYS_BlockFieldOver` | `field_override` | `scenario_field.override_rules/metadata` |
| `V_SYS_GroupButton` | `action_binding` | `action` / `scenario_action` |
| `SYS_EVENT` | `event_binding` | `action.metadata` / `scenario_action.metadata` / `feature_field.source_config` / `metadata` |
| `SYS_REGION` | `layout_region` | `scenario.metadata` / `scenario_field.metadata` |
| `SYS_List / SYS_LISTDETAIL` | `lookup/list_source` | `feature_field.source_config` / `metadata` |
| `SYS_BL / SYS_BLP` | `indirect_plugin_binding` | `pluginBinding.params` 或迁移分析阶段展开 |
| 插件源码 | `behavior_source` | 新系统 Python 插件 + plugin manifest |

## 4. 实体层映射

### 4.1 `SYS_Table` -> `entity_definition`

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `IDNUM` | `entity_code` | `entity_definition.code` |
| `DES1` | `entity_name_zh` | `entity_definition.name` |
| `DES2` | `entity_name_alt` | `entity_definition.metadata.altName` |
| `IDFIELD` | `primary_field_code` | 由字段层 `is_primary=true` 体现，冗余保留到 `metadata.primaryFieldCode` |
| `IDSER` | `id_generator_code` | `entity_definition.metadata.idGeneratorCode` |
| `ISVIEW` | `storage_type` | `entity_definition.storage_type=view/table` |
| `STATUS` | `legacy_status` | `entity_definition.metadata.legacyStatus` |
| `OBJVER` | `legacy_version` | `entity_definition.metadata.legacyVersion` |
| `UPDDAT` | `legacy_updated_at` | `entity_definition.metadata.legacyUpdatedAt` |

推荐映射：

- `module` 需要由导出上下文或功能归属推断，老系统表定义本身通常不提供稳定模块字段
- `physical_name` 默认先使用老系统 `IDNUM`
- `schema_name` 默认 `public`，除非迁移策略另行指定
- `data_source_code` 默认空，表示平台默认数据源
- `schema_managed` 默认 `true`，除非识别为外部只读对象
- `write_enabled` 对 `view` 默认建议为 `false`

### 4.2 `SYS_TableField` -> `entity_field`

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `FIELDNAME` | `field_code` | `entity_field.code` |
| `DES1` | `field_name_zh` | `entity_field.name` |
| `DES2` | `field_name_alt` | `entity_field.metadata.altName` |
| `FIELDTYPE` | `legacy_field_type` | 转换为 `entity_field.data_type`，原值保留到 `metadata.legacyFieldType` |
| `FLENGTH` | `length` | `entity_field.length` |
| `FSUBLENGTH` | `scale_or_sub_length` | `entity_field.scale` 或 `metadata.subLength` |
| `REQUIRED` | `is_required` | `entity_field.is_nullable = !REQUIRED` |
| `DEFVALUE` | `default_value` | `entity_field.default_value` |
| `DSPIDX` | `order_index` | `entity_field.order_index` |
| `SYSTEM` | `field_category` | `entity_field.category=system/business` |
| `UNIQUE` | `unique_hint` | `entity_field.metadata.unique=true`，必要时生成唯一索引 |
| `STATUS` | `legacy_status` | `entity_field.metadata.legacyStatus` |

`FIELDTYPE` 必须按旧平台实际物理语义输出：`1=int`、`2=decimal`、`3/4=string`、`5=datetime`、`6=boolean`、`7=bigint`、`8=text`、`9=binary`。其中旧平台将 `5` 命名为 Date，但实际创建 SQL Server `datetime` 列，不得降为新平台 `date`。`9` 仅表达为 `binary` 元数据，当前交付不包含二进制数据搬迁。

以下老系统字段不直接落到实体层，而主要用于功能层：

- `CTRLTYPE`
- `DSTYPE`
- `DS`
- `DSTEXT`
- `DSVALUE`
- `DSFILTER`
- `DSPARAM`
- `DSPFORMAT`
- `ALLOWSEARCH`
- `ALLOWCOPY`
- `OUTLINK`

这些字段建议保留到：

- `entity_field.metadata.legacyUi`
- 或在生成 `feature_field` 时转为功能层来源配置

### 4.3 `SYS_TableIndex` -> `entity_index`

基本映射：

- 索引名 -> `entity_index.index_name`
- 唯一性 -> `entity_index.is_unique`
- 类型 -> `entity_index.index_type`
- 字段顺序 -> `entity_index.fields`

若老系统索引定义字段不足以完整表达新系统结构，应：

- 先完整保留原始定义到 `entity_index.metadata.legacy`
- 再生成最接近的新系统索引定义

### 4.4 `SYS_TableFK` -> `entity_relationship`

推荐映射：

- 外键源表 -> `source_entity_id`
- 外键目标表 -> `target_entity_id`
- 源字段 -> `source_field_id`
- 目标字段 -> `target_field_id`
- 删除行为若老系统未显式声明，则默认 `restrict`
- 关系类型优先映射为 `many_to_one` / `one_to_many`

若老系统外键只表达数据库约束、不表达业务关系，可同时保留：

- `entity_relationship.metadata.legacyFk`

### 4.5 `SYS_EXTPROP` -> `metadata`

统一原则：

- 老系统扩展属性不应丢失
- 无法结构化映射的属性保留到对应对象的 `metadata.legacyExtProps`

推荐键结构：

```json
{
  "legacyExtProps": {
    "propName": "propValue"
  }
}
```

## 5. 功能层映射

### 5.1 `SYS_Function` -> `feature`

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `IDNUM` | `feature_code` | `feature.code` |
| `DES1` | `feature_name_zh` | `feature.name` |
| `DES2` | `feature_name_alt` | `feature.metadata.altName` |
| `FUNCOBJ` | `main_entity_code` | `feature.data_source_name` / `feature.data_source_code` |
| `FUNCTYPE` | `legacy_feature_type` | `feature.metadata.legacyFuncType` |
| `PARENT` | `legacy_parent_feature` | `feature.metadata.parentFeatureCode` |
| `IMAGEID` | `legacy_icon` | `feature.metadata.icon` |
| `KEEPLOG` | `keep_log` | `feature.metadata.keepLog` |
| `STATUS` | `legacy_status` | `feature.is_active` + `feature.metadata.legacyStatus` |
| `OBJVER` | `legacy_version` | `feature.metadata.legacyVersion` |
| `DSPIDX` | `display_order` | `feature.metadata.displayOrder` |
| `SIZE` | `legacy_size` | `feature.metadata.legacySize` |

推荐规则：

- 当 `FUNCOBJ` 指向实体表时，`feature.data_source_type=table`
- `feature.data_source_name` 建议取主实体物理名
- `feature.data_source_code` 建议取主实体编码
- 老系统功能层未稳定表达的新系统菜单、权限、导航归属信息，先放 `feature.metadata`

### 5.2 `SYS_Group` -> `scenario`

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `IDNUM` | `scenario_code` | `scenario.code` |
| `DES1` | `scenario_name_zh` | `scenario.name` |
| `DES2` | `scenario_name_alt` | `scenario.metadata.altName` |
| `STATUS` | `legacy_status` | `scenario.metadata.legacyStatus` |
| `DSPIDX` | `display_order` | `scenario.metadata.displayOrder` |

`scenario.scenario_type` 老系统没有一一对应的强字段，建议按场景下 block 结构和交互意图推断：

- 主列表场景 -> `list`
- 新建/编辑/维护类场景 -> `form`
- 详情只读类场景 -> `detail` 或 `form`

默认规则：

- 第一版迁移优先收敛到 `list` 与 `form`
- 只有明确是只读详情展示时才落 `detail`

### 5.3 `V_SYS_GroupBlock` -> `scenario.data_binding/default_view/metadata`

老系统 `Block` 在当前新系统中不是一级元数据模型，应降级映射到场景配置和字段配置中。

建议映射：

- `TABLEID` -> `scenario.data_binding.primaryEntityCode` 或子视图绑定
- `GFILTER / GPARAM / BFILTER / BPARAM` -> `scenario.data_binding` 或 `scenario.metadata.legacyFilter`
- `ALLOWCREATE / ALLOWEDIT / ALLOWDELETE` -> 推导系统动作是否创建，以及动作/字段默认只读策略
- `ORDERBY` -> `scenario.default_view.defaultSort`
- `PKLINK / TREELINK / PARENT` -> `scenario.metadata.navigation`
- `BLKTYPE` -> `scenario.metadata.blockType`
- `DES` / `BLKID` -> `scenario.metadata.blocks[]`

推荐保留原始 block 结构：

```json
{
  "blocks": [
    {
      "blockCode": "BMAINBLOCK",
      "tableCode": "SD_T_SO",
      "blockType": 1
    }
  ]
}
```

这能保证后续若新系统引入独立 layout 模型时，可再次转换。

## 6. 字段与视图层映射

### 6.1 `V_SYS_GroupBlockField` -> `feature_field`

`feature_field` 是功能字段池，应该以“功能内唯一字段语义”为单位生成，而不是按场景重复生成。

推荐生成键：

- 优先使用 `TABLEID.FIELDNAME`
- 若新系统希望主实体字段可直接裸名，可对主实体字段保留裸名并在 `metadata.entityFieldCode` 中保留全限定名

推荐映射：

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `FIELDNAME` | `field_key` | `feature_field.field_key` |
| 字段描述 | `display_name` | `feature_field.display_name` |
| 字段类型 | `data_type` | `feature_field.data_type` |
| 表来源 | `entity_source` | `feature_field.source_type=entity` |
| 数据源配置 | `source_config` | `feature_field.source_config` |
| 字段校验/控件信息 | `field_behavior` | `feature_field.validation_rules` / `feature_field.metadata` |

`feature_field.source_type` 推荐规则：

- 实体原生字段 -> `entity`
- 关联实体映射字段 -> `related`
- 表达式计算字段 -> `expression`
- 过程返回字段 -> `procedure`
- 插件提供字段 -> `plugin`

字段来源配置建议：

```json
{
  "entityCode": "sd_t_so",
  "fieldCode": "code_cust"
}
```

### 6.2 `V_SYS_GroupBlockField + SYS_BlockFieldOver` -> `scenario_field`

老系统字段行为大量依赖场景覆盖，这部分应落到 `scenario_field`。

映射关系：

- 场景 -> `scenario_id`
- 功能字段 -> `feature_field_id`
- 上下文 -> `view_context`
- 显示/隐藏 -> `is_visible`
- 只读/可编辑 -> `is_readonly`
- 必填 -> `is_required`
- 排序 -> `order_index`
- 控件类型 -> `component_type`
- 筛选能力 -> `filterable`
- 其他覆盖 -> `override_rules` / `metadata`

#### `view_context` 推荐落法

- 列表块字段 -> `list`
- 表单录入块字段 -> `form`
- 详情只读块字段 -> `detail`

#### 字段状态映射

老系统 `BlockField` 通过 `GSTATUS/BSTATUS` 计算实际状态。

推荐映射：

- `Disable` -> `is_visible=false`
- `Readonly` -> `is_visible=true, is_readonly=true`
- `Editable` -> `is_visible=true, is_readonly=false`
- `EditableNew` -> `is_visible=true, is_readonly=false, override_rules.editModeOnly=true`

原始状态值保留到：

- `scenario_field.metadata.legacyStatus`

### 6.3 老系统字段数据源映射

老系统字段上的：

- `DSTYPE`
- `DS`
- `DSTEXT`
- `DSVALUE`
- `DSFILTER`
- `DSPARAM`
- `DSPFORMAT`
- `OUTLINK`

应优先转换到：

- `feature_field.source_type`
- `feature_field.source_config`
- `scenario_field.metadata.lookup`

`DSTYPE` 的已确认语义及 handoff 转换规则：

- `1`：系统列表/字典，`DS` 作为字典编码，转换为 `q-select + dictionary` option source；
- `2`：表参照，`DS` 作为目标实体，`DSTEXT` 作为源字段，`DSVALUE` 作为本地接收字段，转换为 `relation-picker-field + relation` option source；
- `3`：SQL 数据源，保留原始表达式并标记需要迁移，不伪造可用选项；
- `4`：分号分隔的常量列表，转换为 `q-select + static` option source；
- `5`：用户数据源，保留原始表达式并标记需要迁移。

参照配置同时输出目标实体、取值字段、显示字段、搜索字段、本地接收字段以及可推导的自动回填映射。`DSFILTER`/`DSPARAM` 原样保留，供迁移阶段转换为新平台过滤器和参数。

若第一版无法完全结构化，则至少保留：

```json
{
  "legacyDataSource": {
    "type": 1,
    "ds": "XXX",
    "text": "NAME",
    "value": "CODE"
  }
}
```

## 7. 按钮、动作与事件映射

### 7.1 `V_SYS_GroupButton` -> `action + scenario_action`

老系统按钮本体建议拆成两层：

- 功能动作定义 -> `action`
- 场景挂接与展示 -> `scenario_action`

推荐映射：

| 老系统 | 迁移标准语义 | 新系统 |
|---|---|---|
| `BTNID` | `action_code` | `action.code` |
| `DES1` | `action_name_zh` | `action.name` / `scenario_action.display_name` |
| `DES2` | `action_name_alt` | `action.metadata.altName` |
| `NEEDCONFIRM` | `needs_confirmation` | `scenario_action.confirmation` |
| `BUTTONICON` | `action_icon` | `action.metadata.icon` |
| `GSTATUS/BSTATUS` | `enabled_visibility` | `scenario_action.visibility_rule` / `metadata` |

#### `action_type` 推荐映射

- 系统新建按钮 -> `create`
- 系统编辑按钮 -> `update`
- 系统删除按钮 -> `delete`
- 系统查看按钮 -> `view`
- 系统保存按钮 -> `save`
- 业务扩展按钮 -> `execute`

#### `execution_mode`

默认：

- `sync`

只有明确识别为可异步执行的长耗时动作时再映射为：

- `async`

### 7.2 `SYS_EVENT` -> 插件绑定 / 字段联动 / 元数据保留

老系统 `SYS_EVENT` 是最复杂的迁移对象，不建议强行一一映射为新系统同名事件表，因为新系统当前通过动作绑定插件，不存在老系统式统一事件表。

因此推荐分流处理。

#### A. 按钮事件

若事件宿主是按钮：

- 优先落到 `action.metadata.pluginBinding`
- 或 `scenario_action.metadata.pluginBinding`

结构：

```json
{
  "pluginBinding": {
    "pluginCode": "legacy_xxx",
    "capabilityCode": "btn_submit",
    "triggerPhase": "before",
    "params": {}
  }
}
```

#### B. Block 级事件

例如：

- `BEFORE_SAVE`
- `AFTER_SAVE`
- `BEFORE_DELETE`

推荐落法：

- 优先映射为系统动作上的插件绑定
- 例如保存前事件 -> `save` 动作的 `triggerPhase=before`
- 保存后事件 -> `save` 动作的 `triggerPhase=after`
- 删除前后 -> `delete` 动作的 before/after

#### C. BlockField 级事件

老系统字段修改前后事件，在新系统中没有独立一级事件模型时，推荐映射到：

- `feature_field.source_type=plugin`
- 或 `feature_field.metadata.fieldPluginBinding`
- 或 `scenario_field.override_rules`

建议先落到 metadata，避免过早固化错误模型。

#### D. 无法直接落地的事件

对无法可靠落入新系统动作模型的事件，统一保留：

- `feature.metadata.legacyEvents`
- `scenario.metadata.legacyEvents`
- `scenario_field.metadata.legacyEvents`

同时导出 CLI 需将其标记为：

- `requires_plugin_analysis=true`

### 7.3 `@BL` 间接调用

老系统事件 `EVTSTD` 以 `@BL.xxx` 开头时，需额外查：

- `SYS_BL`
- `SYS_BLP`

迁移规则：

1. 在导出阶段展开得到最终插件引用和参数定义
2. 结果保留到导出包 `indirect_plugin_refs`
3. 新系统落地时按普通 `pluginBinding` 处理

若参数表达式复杂，优先保留到：

- `pluginBinding.params`
- `metadata.legacyParamExpressions`

## 8. 区域与布局映射

### 8.1 `SYS_REGION`

当前新系统没有独立 `block/section/region` 模型，因此：

- 区域不是一级对象
- 区域信息应沉淀到 `scenario.metadata` 或 `scenario_field.metadata`

建议结构：

```json
{
  "layout": {
    "regions": [
      {
        "regionCode": "RGN_MAIN",
        "name": "主信息"
      }
    ]
  }
}
```

### 8.2 Block 布局语义

老系统 block 的以下语义建议保留到 layout metadata：

- block 编号
- block 类型
- 区域归属
- 列表/表单上下文
- 是否主块/子块
- 主子表关系

## 9. 列表与字典映射

### 9.1 `SYS_List / SYS_LISTDETAIL`

建议映射到：

- `feature_field.source_config`
- `feature_field.metadata.lookup`
- 或 `scenario_field.metadata.lookup`

推荐结构：

```json
{
  "lookup": {
    "sourceType": "static_list",
    "listCode": "STATUS_LIST",
    "items": [
      {
        "label": "启用",
        "value": "1"
      }
    ]
  }
}
```

对于共享列表，可在导出结果中单独形成：

- `lists/<listCode>.json`

## 10. 插件映射

### 10.1 老系统插件引用 -> 新系统插件能力

老系统插件引用格式按你确认的口径拆分：

- 库名
- 命名空间
- 类名
- 方法名

迁移标准对象建议：

```json
{
  "legacyRef": "SM.SM.SO.BlockBeforeSave",
  "library": "SM",
  "namespace": "SM",
  "className": "SO",
  "methodName": "BlockBeforeSave"
}
```

新系统建议：

- 每个迁移后的插件作为一个后端 plugin
- 每个可调用迁移入口作为一个 capability

推荐命名：

- `pluginCode`: 以功能或模块归组，例如 `legacy_sm_so`
- `capabilityCode`: 以方法语义命名，例如 `before_save`

### 10.2 新系统 `pluginBinding` 的生成原则

插件绑定时，优先持久化“绑定关系”，不要复制老系统源码信息到业务元数据主结构中。

推荐：

- `pluginCode`
- `capabilityCode`
- `triggerPhase`
- `params`

附加保留：

- `metadata.legacyPluginRef`

### 10.3 运行时上下文映射

新系统插件运行时默认可依赖：

- `feature`
- `scenario`
- `action`
- `user`
- `tags`
- `params`
- `metadata`
- 视情况包含 `record`

因此老系统插件里依赖 `WindowBox/WindowSession/当前记录/当前场景` 的逻辑，迁移时应改写为基于新系统 `PluginContext` 的读取。

## 11. 导出包建议结构

建议老系统导出 CLI 生成如下结构：

```text
exports/
  functions/
    <feature_code>/
      manifest.json
      feature.json
      scenarios.json
      feature_fields.json
      scenario_fields.json
      actions.json
      plugin_refs.json
      indirect_plugin_refs.json
      entities/
      lists/
      summary.md
```

其中：

- `manifest.json` 是 AI 与新系统 CLI 的主入口
- `entities/` 放结构资产
- `plugin_refs.json` 只放插件入口引用，不放源码
- 源码由外部补充后交给 AI 进一步分析

## 12. 待插件源码分析补全的内容

以下内容不能仅靠数据库元数据得出，必须在 AI 分析插件源码后补齐：

- 插件实际访问的额外表
- 插件方法调用链
- 事务边界
- 历史表/日志表/状态表回写
- 外部系统调用
- 文件、消息、网络等副作用

因此迁移流程必须是：

1. 导出元数据
2. 定位插件入口
3. AI 分析源码并补全依赖
4. 形成迁移计划
5. 调用新系统 CLI 创建对象
6. 生成新插件

## 13. 不确定项与当前口径

### 13.1 已采用的当前口径

- `scenario_type` 当前按 `list/form/detail` 理解
- `action_type` 当前按 `execute/create/update/delete/view/save` 理解
- `execution_mode` 当前按 `sync/async` 理解
- `block/section/region` 当前不作为一级模型，落到 `metadata`
- `feature.data_source_type=plugin` 视为特殊来源，不走通用字段探测

### 13.2 仍需后续规范确认但不阻塞文档使用

- 是否把上述枚举升级为后端硬校验
- `detail` 是否长期保留为独立 `scenario_type`
- 是否未来引入独立布局元数据模型
- `plugin` 类型数据源的字段自描述标准是否统一

## 14. 结论

当前可稳定执行的迁移策略是：

1. 以功能为迁移入口
2. 以实体为结构资产
3. 以场景承接旧系统 Group 与 Block 的业务视图语义
4. 以 `action + scenario_action + pluginBinding` 承接按钮与事件
5. 以 `metadata` 承接当前新系统尚未一级建模的老系统能力
6. 通过 AI 分析插件源码补足隐式表依赖与行为迁移

这套映射足以支撑老系统导出 CLI 与新系统创建 CLI 之间的中间标准设计。
