# 插件 UI 返回协议

本文定义后端插件通过 `business.ui` 返回给宿主前端的标准 UI 控制协议。

目标：

- 插件作者不需要读前端源码
- AI 可以稳定生成 UI 控制结果
- 表单页、明细表、列表页、动作区采用统一结构

## 1. 总体结构

插件成功返回时，UI 控制统一放在：

```json
{
  "business": {
    "ui": {
      "fields": {},
      "detailTables": {},
      "actions": {}
    }
  }
}
```

说明：

- `system` 由 runtime 注入，插件不要自己构造
- 如果没有 UI 控制需要，可不返回 `business.ui`
- UI 覆盖是“按 key 覆盖”，不是前端完整状态快照

## 2. fields

`business.ui.fields` 用于控制主表字段或列表列。

```json
{
  "business": {
    "ui": {
      "fields": {
        "supplier_name": {
          "visible": true,
          "readonly": false,
          "required": true,
          "disabled": false,
          "label": "供应商名称",
          "sortable": true
        }
      }
    }
  }
}
```

字段说明：

- `visible`
  - `boolean`
  - 表单页/列表页都支持
- `readonly`
  - `boolean`
  - 仅表单页支持
- `required`
  - `boolean`
  - 仅表单页支持
- `disabled`
  - `boolean`
  - 仅表单页支持
- `label`
  - `string`
  - 仅列表页支持，用于列标题覆盖
- `sortable`
  - `boolean`
  - 仅列表页支持

字段 key 约定：

- 统一使用实体原始字段名
- 不要使用前端别名或驼峰字段名

## 3. detailTables

`business.ui.detailTables` 仅表单页消费，用于控制明细表及其列。

```json
{
  "business": {
    "ui": {
      "detailTables": {
        "items": {
          "visible": true,
          "readonly": true,
          "allowAdd": false,
          "allowDelete": false,
          "columns": {
            "price": {
              "readonly": true
            },
            "remark": {
              "visible": false
            }
          }
        }
      }
    }
  }
}
```

表级字段说明：

- `visible`
  - 是否显示整张明细表
- `readonly`
  - 是否只读
- `allowAdd`
  - 是否允许新增行
- `allowDelete`
  - 是否允许删除行

列级字段说明：

- `visible`
- `readonly`
- `required`

说明：

- `columns` 的 key 统一使用明细实体原始字段名

## 4. actions

`business.ui.actions` 用于控制表单页和列表页动作。

```json
{
  "business": {
    "ui": {
      "actions": {
        "save": {
          "visible": true,
          "disabled": true,
          "displayName": "重新保存"
        },
        "view": {
          "displayName": "查看详情"
        }
      }
    }
  }
}
```

字段说明：

- `visible`
  - `boolean`
- `disabled`
  - `boolean`
- `displayName`
  - `string`

动作 key 约定：

- 优先使用动作编码
- 系统动作通常是：`create`、`edit`、`delete`、`view`、`save`、`submit`

## 5. 页面消费范围

### 表单页

会消费：

- `fields.visible`
- `fields.readonly`
- `fields.required`
- `fields.disabled`
- `detailTables.*`
- `actions.visible`
- `actions.disabled`
- `actions.displayName`

### 列表页

会消费：

- `fields.visible`
- `fields.label`
- `fields.sortable`
- `actions.visible`
- `actions.disabled`
- `actions.displayName`

## 6. 推荐写法

### 6.1 表单加载前控制

```python
return ok({
    "business": {
        "ui": {
            "fields": {
                "email": {"readonly": True},
                "address": {"visible": False}
            },
            "actions": {
                "save": {"disabled": True}
            }
        }
    }
})
```

### 6.2 列表动作执行后控制

```python
return ok({
    "business": {
        "ui": {
            "fields": {
                "status": {"label": "供应商状态"},
                "email": {"visible": False}
            },
            "actions": {
                "view": {"displayName": "查看详情"},
                "delete": {"disabled": True}
            }
        }
    }
})
```

## 7. 不建议的写法

- 返回前端组件结构
- 返回 CSS 片段
- 返回与实体字段名不一致的 key
- 同时混用驼峰和下划线字段名
- 依赖前端内部变量名或未公开状态结构

## 8. 与其他协议的关系

- 业务消息：放在 `business.message`
- 主表数据回填：放在 `business.model`
- 明细数据回填：放在 `business.detailRows`
- UI 控制：放在 `business.ui`

这三者可以同时返回。
