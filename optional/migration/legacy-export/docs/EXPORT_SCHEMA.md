# Export Schema

本文档描述 `legacy-export` 主要输出文件的结构意图，帮助 AI 和人工快速消费导出结果。

## 功能级目录

路径：

```text
exports/functions/<FUNCID>/
```

核心文件：

- `manifest.json`
  - 功能级总索引，包含 `featureCode`、`mainEntityCode`、场景、直接实体、插件引用、风险警告
- `migration_readiness.json`
  - 迁移风险分级、风险标记、下一步建议
- `raw/`
  - 老系统原始查询结果，字段名保持原库口径
- `normalized/`
  - 新系统迁移语义视图
- `handoff/`
  - 供下一步 AI 编排直接读取的草稿产物
- `plugin_context/`
  - 供 AI 结合插件代码库重写插件时使用的上下文包

### `normalized/feature.json`

```json
{
  "code": "SD_T_SO",
  "name": "销售订单",
  "mainEntityCode": "SD_T_SO",
  "metadata": {
    "legacyParent": "SD",
    "legacyFuncType": "1"
  }
}
```

### `normalized/scenarios.json`

```json
[
  {
    "code": "GEDIT",
    "name": "编辑",
    "scenarioType": "form",
    "metadata": {
      "blocks": [
        {
          "blockCode": "MAINBLOCK",
          "tableCode": "SD_T_SO"
        }
      ]
    }
  }
]
```

### `normalized/plugin_refs.json`

```json
[
  {
    "eventSource": "BUTTON",
    "eventName": "CLICK",
    "ownerId": "MAINBLOCK",
    "ownerSecondaryId": "BTN_SAVE",
    "rawRef": "SM.SO.BlockBeforeSave",
    "library": "SM",
    "namespace": "",
    "className": "SO",
    "methodName": "BlockBeforeSave",
    "params": "@WINBOX,@BLOCK"
  }
]
```

### `plugin_context/plugin_bindings.json`

```json
{
  "directBindings": [
    {
      "bindingType": "event",
      "triggerSource": "BUTTON",
      "triggerName": "CLICK",
      "pluginRef": "SM.SO.BlockBeforeSave"
    }
  ],
  "indirectBindings": []
}
```

### `plugin_context/sql_dependencies.json`

```json
{
  "relatedTables": [
    {
      "tableCode": "SD_T_SO",
      "dependencyType": "metadata-direct",
      "analysisRequired": true
    }
  ],
  "directPluginEntries": [
    {
      "pluginRef": "SM.SO.BlockBeforeSave",
      "candidateTables": ["SD_T_SO", "SD_T_SOD"],
      "analysisRequired": true
    }
  ]
}
```

### `plugin_context/plugin_rewrite_handoff.json`

```json
{
  "featureCode": "SD_T_SO",
  "pluginRefs": [],
  "pluginBindings": {},
  "sqlDependencies": {},
  "entityFieldCatalog": [],
  "relatedTableCatalog": [],
  "rewriteGuidance": [
    "Do not translate legacy SQL literally into new plugins."
  ]
}
```

### `handoff/create_feature_seed.json`

该文件用于创建与主实体绑定的功能。对多实体旧功能，`fields` 只包含 `manifest.mainEntityCode` 对应的主表字段；子表字段不会平铺进 feature，而是保留在场景 `metadata.detailTables[].columns` 中。这可避免多个实体的同名 `sourceField` 产生歧义。

### `handoff/scenario_update_seed.json`

该文件是 `system update-scenario` 的请求主体草稿，不包含 `featureId` / `scenarioId` 占位字段。动作列表只保留新平台稳定接收的最小字段：

```json
{
  "code": "list",
  "name": "列表",
  "scenarioType": "list",
  "metadata": "{\"formLayout\":{\"type\":\"tabs\",\"tabs\":[]},\"detailTables\":[]}",
  "fieldGroups": [],
  "actions": [
    {
      "actionId": "replace-with-action-id",
      "displayName": "新增",
      "orderIndex": 1,
      "layoutSlot": "toolbar"
    }
  ]
}
```

`ScenarioRequest.metadata` 在新平台接口中是 JSON 字符串，因此 `formLayout` 和 `detailTables` 位于该字符串的内层 JSON 中。CLI 会将整个 seed 原样提交；它们不应移到请求顶层，否则后端会忽略。

复杂功能还会生成 `handoff/scenario_update_seeds.json`，其中包含全部列表/表单场景。表单场景尽量附带：

- `metadata.formLayout.tabs`：从旧平台 region 和主表字段分组推导；
- `metadata.detailTables`：从非主表 block 和 block field 推导；
- `MAINBLOCK`/`BMAINBLOCK` 被视为主信息块并排除在 `detailTables` 外；block field 缺少 `TABLEID` 时通过 block 反查；
- 主从表共有 `CODE_*` 字段时，关系键优先保留为同名业务键，例如 `CODE_ITEM = CODE_ITEM`；
- 明细列会合并 `SYS_TableField`、`V_SYS_GroupBlockField` 和 `SYS_BlockFieldOver`，覆盖值优先；因此表字段上的名称、图片/附件控件类型不会因 block 视图列为空而丢失；
- 明细列仅保留旧平台 `GSTATUS/BSTATUS` 可见的非系统字段；主从关联键（如 `CODE_ITEM`）及在主表中同样存在、且在明细块中不是明确可编辑状态的父级上下文字段（如 `DESC_ITEM`、`MODEL_ITEM`、`SPEC_ITEM`）不再重复显示为明细列；主从关联键仍保留在 `relation.parentKey/childKey`，明确可编辑的同名子表业务字段仍保留；
- `componentType` 与 `extraMetadata`：保留旧 `CTRLTYPE`、`DSTYPE` 和完整数据源表达式；`DSTYPE=1` 转换为字典源，`DSTYPE=2` 转换为关系源，`DSTYPE=4` 转换为静态选项；
- 同一场景的同一 `featureFieldKey` 只输出一个 `fieldGroups` 条目，`list/form/detail` 上下文合并在该条目内；重复来源优先保留可见且数据源配置更完整的版本；
- `migrationWarning`：提醒应用前复核主从表关联键。

下拉和参照不再静默降级为 `q-input`。无法结构化的 SQL、脚本或缺失依赖的数据源会保留完整旧元数据，并通过 `migrationStatus` 标记后续处理，不伪造可用选项。

### `migration_readiness.json`

```json
{
  "featureCode": "SD_T_SO",
  "readyForMetadataMigration": true,
  "readyForPluginRewrite": true,
  "riskLevel": "medium",
  "riskFlags": ["event_uses_plugin"]
}
```

## 系统级目录

路径：

```text
exports/system/
```

核心文件：

- `manifest.json`
- `migration_readiness.json`
- `metadata/normalized.json`
- `menus/normalized.json`
- `serials/normalized.json`
- `workflows/normalized.json`
- `security/normalized.json`
- `users/normalized.json`
- `org/normalized.json`
- `lists/normalized.json`
- `data_catalog/*.json`

### `data_catalog/entities.json`

```json
[
  {
    "entityCode": "SD_T_SO",
    "entityName": "销售订单",
    "physicalName": "SD_T_SO",
    "idField": "ID_SO",
    "serialRuleCode": "SO_NO",
    "isView": false,
    "tableRole": "business-structure"
  }
]
```

### `data_catalog/fields.json`

```json
[
  {
    "entityCode": "SD_T_SO",
    "fieldCode": "STATUS",
    "fieldName": "状态",
    "dataType": "string",
    "semanticHints": ["status"]
  }
]
```

### `data_catalog/relationships.json`

```json
[
  {
    "entityCode": "SD_T_SOD",
    "sourceField": "ID_SO",
    "targetEntityCode": "SD_T_SO",
    "targetField": "ID_SO"
  }
]
```

### `migration_readiness.json`

```json
{
  "readyForSystemMetadataMigration": true,
  "riskLevel": "medium",
  "summary": {
    "featureCount": 120,
    "workflowCount": 8,
    "roleCount": 16
  }
}
```

## 说明

- `raw/` 是事实层，优先保留原始字段名。
- `normalized/` 是语义层，给迁移逻辑和 AI 使用。
- `plugin_context/` 是插件重写专用层。
- `migration_readiness.json` 是排期和风险判断层。
