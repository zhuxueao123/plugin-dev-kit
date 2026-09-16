# 后端插件开发指南

## 1. 目录结构

后端插件放在：

```text
plugin-workspace/backend/<plugin-code>/
  manifest.json
  main.py
```

最小示例：

```json
{
  "pluginCode": "order_guard",
  "pluginName": "订单校验插件",
  "version": "0.1.0",
  "capabilities": [
    {
      "code": "pre_submit",
      "displayName": "提交前校验",
      "slot": "scenario.submit.pre",
      "description": "示例能力",
      "inputSchema": {
        "type": "object",
        "properties": {
          "maxAmount": { "type": "number" }
        }
      },
      "defaultParams": {
        "maxAmount": 100000
      },
      "permissions": [],
      "timeoutSeconds": 3,
      "auditLevel": "basic"
    }
  ]
}
```

## 2. 插件入口

```python
from plugins_sdk import PluginApp, PluginContext
from plugins_sdk.result import ok, reject

app = PluginApp()

@app.capability("pre_submit")
def pre_submit(context: PluginContext, payload: bytes):
    model = context.model
    if model.amount and model.amount > 100000:
        return reject("amount too large")
    return ok(
        {
            "business": {
                "message": "校验通过",
                "model": model.to_dict()
            }
        }
    )

PLUGIN_APP = app
```

## 3. 可以使用的上下文

- `context.request_id`
- `context.params`
- `context.data.root`
- `context.model`
- `context.user`
- `context.feature`
- `context.scenario`
- `context.action`
- `context.selection`
- `context.core`
- `context.t(...)`

说明：
- 普通调用和流式调用都保证 `context.data.root` 可用。
- `context.model` 是 `context.data.root` 的点访问包装。
- `context.model` / `context.data.root` 中的字段名统一使用实体原始字段名，例如 `contact_person`，不要假设会自动转成 `contactPerson`。
- 列表场景动作里可直接使用：
  - `context.selection.row`
  - `context.selection.rows`
  - `context.selection.ids`
  - `context.selection.count`
- 多语言文案统一使用 `context.t(...)`，语言资源放在 `plugin-workspace/locales/*.json`，不要在插件里手写 `if locale == ...`

## 4. 宿主能力访问

插件通过 `context.core` 访问平台受控能力。

常用方法：

- `context.core.query_records(...)`
- `context.core.get_record(...)`
- `context.core.create_record(...)`
- `context.core.update_record(...)`
- `context.core.execute_action(...)`
- `context.core.get_scenario(feature_code, scenario_code)`
- `context.core.get_scenario_config(feature_code, scenario_code)`
- `context.core.get_action(code, feature_code=None)`
- `context.core.get_feature_config(feature_code)`
- `context.core.exists(entity_code, filters=None)`
- `context.core.count_records(entity_code, filters=None)`
- `context.core.aggregate(entity_code, aggregate, field=None, filters=None, group_by=None)`
- `context.t(key, params=None, default=None)`

例如：

```python
if context.core.exists("supplier", filters={"name": model.name}).payload["exists"]:
    return reject("供应商名称已存在")

totals = context.core.aggregate(
    "purchase_order",
    aggregate="sum",
    field="amount",
    group_by=["status"],
).payload
```

典型用法：

```python
scenario_config = context.core.get_scenario_config(
    context.feature.code,
    context.scenario.code,
).payload

save_action = context.core.get_action("save", feature_code=context.feature.code).payload
```

插件国际化示例：

```python
return reject(context.t("messages.name_required"))
```

完整规范见 [plugin-i18n-guide.md](plugin-i18n-guide.md)

## 5. manifest 常用字段

- `pluginCode`：插件编码，目录名建议与之保持一致。
- `capabilities[].code`：能力编码。
- `capabilities[].slot`：宿主侧绑定位置。
- `capabilities[].defaultParams`：默认参数，会自动合并到 `context.params`。
- `capabilities[].permissions`：调用该能力所需权限，宿主会校验。
- `capabilities[].timeoutSeconds`：调用超时秒数，宿主会下发到 Runtime 调用链路。
- `capabilities[].auditLevel`：审计级别，会进入执行元数据并写审计入口。

## 6. 数据库访问约束

- 不支持执行原生 SQL。
- 如需数据库侧逻辑，请调用：
  - `context.core.execute_procedure("schema.proc_name", {...})`
- 过程名与参数名会做格式校验。
- 参数值统一参数化传递。

## 7. 返回消息结构

插件返回的业务 JSON 统一约定为：

```json
{
  "business": {
    "message": "给前端展示的成功消息，可选",
    "model": {},
    "detailRows": {},
    "data": {}
  },
  "error": {
    "code": "可选错误码",
    "message": "失败消息",
    "details": {}
  }
}
```

约定：
- 成功时优先使用 `ok({...})`，业务消息放在 `business.message`。
- 如果需要回填主表数据，放在 `business.model`。
- 如果需要回填明细表数据，放在 `business.detailRows`。
- 如果需要控制表单字段或列表列行为，放在 `business.ui.fields`。
- 失败时使用 `reject("message", payload)` 或 `error("message", payload)`。
- `system` 由 runtime 自动注入，插件不要自己拼装。
- 没有特殊需要时，不要返回顶层 `message`、`model`、`detailRows`，统一放到 `business` / `error` 内。

字段 / 列 UI 控制示例：

```python
return ok({
    "business": {
        "ui": {
            "fields": {
                "discount_rate": {"visible": False},
                "approved_by": {"readonly": True},
                "supplier_code": {"required": True},
                "status": {"label": "状态", "sortable": False}
            }
        }
    }
})
```

更完整的 UI 控制示例：

```python
return ok({
    "business": {
        "ui": {
            "fields": {
                "discount_rate": {"visible": False},
                "approved_by": {"readonly": True},
                "supplier_code": {"required": True}
            },
            "detailTables": {
                "items": {
                    "readonly": True,
                    "allowAdd": False,
                    "columns": {
                        "price": {"readonly": True},
                        "remark": {"visible": False}
                    }
                }
            },
            "actions": {
                "save": {"disabled": True},
                "submit": {"displayName": "重新提交"}
            }
        }
    }
})
```

说明：
- `business.ui.fields`
  - 表单页会消费：`visible / readonly / required / disabled`
  - 列表页会消费：`visible / label / sortable`
- `business.ui.detailTables`
  - 仅表单页消费，用于控制明细表及其列
- `business.ui.actions`
  - 表单页和列表页都会消费
  - 支持：`visible / disabled / displayName`

正式协议见 [plugin-ui-protocol.md](plugin-ui-protocol.md)

## 8. 参考示例

- 正式插件：`plugin-workspace/backend/<plugin-code>/`
- 可复制模板：`plugin-workspace/examples/backend/supplier_guard/`、`plugin-workspace/examples/backend/supplier_runtime_rules/`
- 完整联调链路见 [plugin-e2e-examples.md](plugin-e2e-examples.md)
