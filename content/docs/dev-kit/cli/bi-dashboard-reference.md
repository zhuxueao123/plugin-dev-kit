# BI 看板 CLI 配置参考

## 使用范围

`bi` 命令组管理 BI 看板草稿、发布版本、查询测试和登录后个人主页绑定。网页设计器适合常用配置，CLI 可直接维护完整 `definition` JSON。

当前支持三类 BI 查询数据源：`sql`、`plugin`、`static`。数据库连接必须先在平台现有数据源中配置；`dataSourceCode` 为空表示默认业务库，填写时必须引用已启用的数据源编码。

## 看板保存对象

```json
{
  "code": "sales_overview",
  "name": "销售概览",
  "description": "销售聚合指标",
  "definition": {
    "schemaVersion": 1,
    "layout": { "columns": 12, "rowHeight": 72, "gap": 12 },
    "filters": [],
    "queries": [],
    "components": []
  }
}
```

`update` 是完整草稿更新。修改已有看板前先执行：

```bash
asapflow bi get --dashboard-id <dashboard-id>
```

保留原有 `definition.queries` 和 `definition.components` 中不需要修改的内容，再提交完整对象。

## 查询定义

### SQL 查询

```json
{
  "id": "monthlySales",
  "name": "月度销售额",
  "sourceType": "sql",
  "dataSourceCode": "erp",
  "sql": "SELECT order_month, SUM(amount) AS value FROM sales_order WHERE order_date >= @start_date GROUP BY order_month ORDER BY order_month",
  "parameters": [
    {
      "name": "start_date",
      "type": "date",
      "required": true,
      "defaultValue": "2026-01-01",
      "label": "开始日期"
    }
  ],
  "securityMode": "public",
  "timeoutSeconds": 15,
  "maxRows": 1000
}
```

约束：

- `id` 在看板内唯一。
- `sourceType` 支持 `sql`、`plugin`、`static`。
- SQL 只能是单条 `SELECT` 或只读 CTE。
- 支持 PostgreSQL、SQL Server、MySQL；后端按实际数据源类型校验。
- 参数必须使用 `@name` 绑定，不允许通过字符串拼接传值。
- 参数类型为 `string`、`number`、`date`、`datetime`、`boolean`。
- 查询最多 50 个参数，最长 100,000 字符。
- `timeoutSeconds` 实际限制为 1–120 秒，`maxRows` 为 1–10,000 行。
- 生产数据源必须使用数据库只读账号和最小权限。

`securityMode = current_user` 时，SQL 必须引用 `@current_user_id`：

```sql
SELECT status, COUNT(*) AS value
FROM sales_order
WHERE owner_user_id = @current_user_id
GROUP BY status
```

该参数由服务器根据登录用户强制写入，不能在 `parameters` 中声明或由调用方覆盖。当前模式不会自动套用功能场景的数据范围。

服务端保留参数：

- `@current_user_id`：当前登录用户 ID。只能在 `securityMode = current_user` 的 SQL 中使用。
- `@current_user_name`：当前登录账号。只能在 `securityMode = current_user` 的 SQL 中使用。
- `@current_user_display_name`：当前用户显示名。只能在 `securityMode = current_user` 的 SQL 中使用。
- `@current_department_id`：当前用户主部门 ID。只能在 `securityMode = current_user` 的 SQL 中使用。
- `@current_primary_role_id`：当前用户主角色 ID。只能在 `securityMode = current_user` 的 SQL 中使用。
`current_` 前缀是服务端保留前缀，不能在 `parameters` 中声明，也不能由调用方传入覆盖。

### 插件接口查询

```json
{
  "id": "customerCount",
  "name": "当前用户客户数量",
  "sourceType": "plugin",
  "pluginCode": "crm",
  "capabilityCode": "customer_count",
  "parameters": [],
  "securityMode": "current_user",
  "timeoutSeconds": 15,
  "maxRows": 1000
}
```

平台会校验插件能力是否存在，并校验插件能力声明的 `permissions` 是否被当前用户满足。执行时插件会收到 `payload.parameters`、`payload.query`、`context.user`、`context.bi` 和 `context.params`。

插件推荐返回：

```json
{
  "columns": [{ "name": "customer_count", "type": "number" }],
  "rows": [{ "customer_count": 12 }],
  "truncated": false
}
```

也可以直接返回对象或数组，平台会自动推断列：

```json
[
  { "name": "华东", "value": 120 },
  { "name": "华南", "value": 90 }
]
```

### 静态 JSON 数据

```json
{
  "id": "salesTarget",
  "name": "销售目标",
  "sourceType": "static",
  "staticData": [
    { "month": "2026-01", "target": 100000 },
    { "month": "2026-02", "target": 120000 }
  ],
  "parameters": [],
  "maxRows": 1000
}
```

对象会转成一行数据；数组会转成多行数据。静态数据适合目标值、说明性小数据集和临时看板，不适合大量明细数据。

## 组件定义

通用结构：

```json
{
  "id": "monthlySalesChart",
  "type": "line",
  "title": "月度销售额",
  "queryRef": "monthlySales",
  "mapping": {
    "categoryField": "order_month",
    "valueField": "value"
  },
  "layout": { "x": 0, "y": 0, "w": 8, "h": 4 },
  "style": {
    "background": "#ffffff",
    "textColor": "#1f2937",
    "borderColor": "#e5e7eb",
    "borderRadius": 12,
    "titleFontSize": 14,
    "area": true
  }
}
```

当前组件类型：

- `indicator`：通过 `mapping.valueField` 读取首行指标；`format` 可为 `number`、`currency`、`percent`。
- `pie`：使用 `categoryField` 和 `valueField`；`style.donut` 控制环形样式。
- `line`：使用分类和值字段；`style.area` 控制面积填充。
- `bar`：使用分类和值字段。
- `table`：可设置 `pageSize`，也可通过 `columns` 指定字段、标题和对齐方式。
- `text`：使用 `content`，不需要查询。
- `dynamic-text`：使用 `template`，可引用查询首行字段和系统变量。

只有 `dynamic-text` 动态文本组件会解析变量；普通 `text` 文本组件按原文显示。动态文本变量包括：

- `{{ currentUser.id }}`
- `{{ currentUser.displayName }}`
- `{{ currentUser.userName }}`
- `{{ system.datetime }}`
- `{{ filters.<查询参数名> }}`
- `{{ <查询首行字段名> }}`

日期时间变量可以追加前端格式参数：

```text
{{ system.datetime:yyyy-MM-dd HH:mm }}
```

当前支持的格式 token 为 `yyyy`、`yy`、`MM`、`dd`、`HH`、`mm`、`ss`、`SSS`。格式化使用浏览器客户端当前时间，不依赖服务端。

动态文本按纯文本渲染，不支持 HTML。

## CLI 操作顺序

```bash
asapflow bi create --input examples/create_bi_dashboard.json
asapflow bi get --dashboard-id <dashboard-id>
asapflow bi execute-query --dashboard-id <dashboard-id> --input examples/execute_bi_query.json
asapflow bi publish --dashboard-id <dashboard-id> --input examples/publish_bi_dashboard.json
asapflow bi get-published --dashboard-id <dashboard-id>
asapflow bi save-homepage-binding --input examples/save_bi_homepage_binding.json
asapflow bi homepage
```

## 个人主页绑定

主页绑定只能指向已发布看板。解析顺序为用户、主部门、主角色、全局；命中后登录首页会打开对应 BI 看板，没有命中则回到默认工作流首页。

```json
{
  "scopeType": "global",
  "scopeId": null,
  "dashboardId": "00000000-0000-0000-0000-000000000000",
  "isActive": true
}
```

`scopeType` 支持：

- `global`：全局默认，`scopeId` 必须为空。
- `department`：按用户主部门匹配，`scopeId` 为部门 ID。
- `role`：按用户主角色匹配，`scopeId` 为角色 ID。
- `user`：按用户 ID 精确匹配。

CLI：

```bash
asapflow bi homepage
asapflow bi list-homepage-bindings
asapflow bi save-homepage-binding --input save_bi_homepage_binding.json
asapflow bi delete-homepage-binding --binding-id <binding-id>
```

## 尚未实现

- 多看板标签页和用户自定义标签排序。
- 部门层级继承规则。
- 用户个人看板自定义副本。
- 标准功能查询、存储过程等更多数据源类型。
- 平台数据范围自动注入 SQL、插件数据权限协议细化。
- 缓存、钻取、组件联动、定时刷新、历史版本回滚和完整 ECharts option。

CLI 和 API 目前只表达已经实现的能力；不要自行在 JSON 中增加标签页、部门继承或个人副本字段并假定服务端会处理。
