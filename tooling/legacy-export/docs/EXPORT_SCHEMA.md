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

### `handoff/scenario_update_seed.json`

该文件是 `system update-scenario` 的请求主体草稿，不包含 `featureId` / `scenarioId` 占位字段。动作列表只保留新平台稳定接收的最小字段：

```json
{
  "code": "list",
  "name": "列表",
  "scenarioType": "list",
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

复杂功能还会生成 `handoff/scenario_update_seeds.json`，其中包含全部列表/表单场景。表单场景尽量附带：

- `metadata.formLayout.tabs`：从旧平台 region 和主表字段分组推导；
- `metadata.detailTables`：从非主表 block 和 block field 推导；
- `MAINBLOCK`/`BMAINBLOCK` 被视为主信息块并排除在 `detailTables` 外；block field 缺少 `TABLEID` 时通过 block 反查；
- 主从表共有 `CODE_*` 字段时，关系键优先保留为同名业务键，例如 `CODE_ITEM = CODE_ITEM`；
- `componentType` 与 `extraMetadata`：保留旧 `CTRLTYPE`、`DSTYPE`、数据源表达式和推荐控件；
- `migrationWarning`：提醒应用前复核主从表关联键。

当字典或关系目标尚未迁移时，导出器使用可录入的 `q-input` 作为安全降级，并把推荐的 `q-select` 或 `relation-picker-field` 写入 `suggestedComponent`，避免生成不可操作表单。

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
