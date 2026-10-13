# 后端插件标准能力参考

后端插件通过 `context.core` 调用宿主提供的受控能力。

说明：

- `context.core` 是当前推荐写法
- `context.core` 属于后端 Python Runtime；前端 npm 包 `@trusteem/asapflow-plugin-sdk` 不会替代它
- 旧文档中的 `context.core_sdk` 可视为已废弃写法，后续统一使用 `context.core`
- 插件上下文和 `context.core` 相关的实体数据统一使用实体原始字段名，例如 `contact_person`
- 插件文案不要手写语言分支，统一使用 `context.t(...)`，见 [plugin-i18n-guide.md](plugin-i18n-guide.md)

## 1. 上下文能力

- `get_context()`
- `get_user()`
- `get_config(keys=None)`
- `get_config_snapshot()`
- `get_scenario(feature_code, scenario_code)`
- `get_scenario_config(feature_code, scenario_code)`
- `get_action(code, feature_code=None)`
- `get_feature_config(feature_code)`

## 2. 数据查询与写入

- `list_entities(module=None)`
- `get_entity(code)`
- `get_entity_relationships(entity_code, direction="both")`
- `query_records(entity_code, filters=None, page_index=0, page_size=50, sort=None, fields=None)`
- `exists(entity_code, filters=None)`
- `count_records(entity_code, filters=None)`
- `aggregate(entity_code, aggregate, field=None, filters=None, group_by=None)`
- `get_record(entity_code, record_id)`
- `create_record(entity_code, payload)`
- `update_record(entity_code, record_id, payload)`
- `patch_record(entity_code, record_id, payload)`
- `delete_record(entity_code, record_id)`

`get_entity_relationships` 的 `direction` 支持 `both`、`outgoing`、`incoming`，返回源/目标实体编码、源/目标字段编码、关系类型、桥接实体和删除行为，可用于级联处理与关系校验。

过滤操作符支持 `$eq`、`$ne`、`$isnull`、`$in`、`$nin`、`$gt`、`$gte`、`$lt`、`$lte`、`$contains` 以及 `$and` / `$or`。不支持的操作符会直接报错，不会静默删除或返回空结果。

## 3. 数据库侧能力

- `execute_procedure(procedure_name, parameters=None, returns_rows=False)`

说明：
- 只允许调用存储过程或函数。
- 不开放原生 SQL。
- `procedure_name` 支持 `name` 或 `schema.name`。
- 参数名与过程名会做格式校验。
- 参数值统一参数化传递。

示例：

```python
result = context.core.execute_procedure(
    "sales.sync_order",
    {"order_id": "o-1001"},
    returns_rows=False,
)
```

如需读取结果集：

```python
result = context.core.execute_procedure(
    "sales.list_open_orders",
    {"customer_id": "c-1"},
    returns_rows=True,
)

rows = result.payload.get("rows", [])
```

聚合示例：

```python
count = core.count_records("Order", filters={"status": "open"}).payload["count"]
exists = core.exists("Order", filters={"customer_id": "c-1"}).payload["exists"]

summary = core.aggregate(
    "Order",
    aggregate="sum",
    field="amount",
    group_by=["status"],
).payload
```

## 4. 动作与日志

- `execute_action(action_code, payload=None, parameters=None)`
- `log(level, message, extra=None, source="plugin")`
- `audit(event, payload=None)`

## 5. 返回约定

所有能力调用统一返回：

```python
result.code
result.message
result.payload
result.success
```

后端插件自己的返回结果建议统一写成：

```python
ok({
    "business": {
        "message": "执行成功",
        "model": {...},
        "detailRows": {...},
        "data": {...}
    }
})
```

失败时：

```python
reject("业务校验失败", {
    "error": {
        "code": "VALIDATION_FAILED",
        "details": {...}
    }
})
```

说明：
- `business.message` 是给前端展示的成功消息。
- `business.model` / `business.detailRows` 用于回填表单。
- `business.ui.fields`
  - 在表单页控制主表字段的 `visible/readonly/required/disabled`
  - 在列表页控制列的 `visible/label/sortable`
- `business.ui.detailTables` 可控制明细表的显隐、只读、增删按钮和列级行为。
- `business.ui.actions` 可控制表单页和列表页动作的显隐、禁用和显示名称。
- `system` 字段由 runtime 自动补充，不需要插件自己返回。

## 6. 最小示例

```python
core = context.core

user = core.get_user().payload
config = core.get_config().payload
scenario = core.get_scenario("supplier_management", "list").payload
scenario_config = core.get_scenario_config("supplier_management", "list").payload
records = core.query_records("Order", page_size=20).payload
core.log("info", "plugin executed", {"user": user})
```

## 7. 连通性判断

`get_user()` 和 `get_context()` 读取的是 Runtime 已收到的调用上下文，不访问平台数据服务，因此它们成功不代表 Core SDK 网络连通。验证数据服务应调用 `count_records()`、`list_entities()` 等远程能力。

正式发布包由平台注入 Core SDK 地址，插件代码不应硬编码 URL。容器化部署默认访问同容器的 `http://127.0.0.1:8080`；只有自定义部署拓扑才需要由平台运维设置 `ASAPFLOW_CORE_BASE_URL`。若出现 `Connection refused`，应检查平台 API 是否已启动以及该环境变量，而不是修改插件的 `allowed_modules.yaml`。
