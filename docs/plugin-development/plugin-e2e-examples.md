# 插件完整示例链路

本文给出一条可直接联调的完整示例链路，覆盖：

- CLI 建模
- 后端插件绑定
- 前端插件访问
- 多语言资源

示例基于当前供应商模块：

- 后端插件：`supplier_guard`
- 后端插件：`supplier_runtime_rules`
- 前端插件：`supplier_portal`

可复制示例位于 `plugin-workspace/examples/backend/` 和
`plugin-workspace/examples/frontend/`；启用后放入 `plugin-workspace/backend/`
和 `plugin-workspace/frontend/src/pages/`。

## 1. 前置条件

需要准备：

1. 平台 API 已启动
2. 前端宿主已启动
3. runtime 正常运行
4. CLI 已配置 `baseUrl` 与 token
5. `plugin-workspace/locales/*.json` 已存在

## 2. 创建实体

示例实体：`supplier`

最小字段建议：

- `name`
- `contact_person`
- `email`
- `phone`
- `address`
- `status`

CLI 示例：

```bash
asapflow system create-entity --input create_supplier_entity.json
```

## 3. 创建功能并绑定实体

示例功能：`supplier_management`

关键点：

- `feature` 代表功能
- 必须显式绑定 `entityCode = supplier`
- 默认会初始化 `list` / `detail` 场景

CLI 示例：

```bash
asapflow system create-feature --input create_supplier_feature.json
```

## 4. 绑定后端插件

### 4.1 保存前校验插件

插件：

- `pluginCode = supplier_guard`
- `capability = validate_before_save`
- `slot = scenario.submit.pre`

典型效果：

- `name` 为空时拦截
- `status = archived` 时拦截

### 4.2 表单规则插件

插件：

- `pluginCode = supplier_runtime_rules`
- `capability = decorate_supplier_form`
- `slot = scenario.load.post`

典型效果：

- `status = draft` 时隐藏 `address`
- `status = inactive` 时将 `email` / `phone` 设为只读
- 禁用 `save`

### 4.3 列表规则插件

插件：

- `pluginCode = supplier_runtime_rules`
- `capability = decorate_supplier_list`
- 建议绑定到列表场景动作

典型效果：

- 隐藏 `email` 列
- 将 `status` 列标题改为“供应商状态”
- 禁用 `delete`
- 将 `view` 改名为“查看详情”

## 5. 重载插件

后端插件更新后：

```bash
asapflow plugin reload --plugin-code supplier_guard
asapflow plugin reload --plugin-code supplier_runtime_rules
```

## 6. 构建前端插件

前端页面插件：

- `supplier_portal`

构建：

```bash
cd plugin-workspace/frontend
npm run build
```

页面示例：

- `/plugins/supplier/dashboard`
- `/plugins/supplier/status-board`

## 7. 验证点

### 保存前校验

1. 打开供应商表单
2. `name` 留空保存
3. 应被拦截
4. 将 `status` 设置为 `archived`
5. 应被拦截

### 表单 UI 控制

1. 打开 `status = draft` 的供应商
2. `address` 应隐藏
3. 打开 `status = inactive` 的供应商
4. `email` / `phone` 应只读
5. `save` 应禁用

### 列表 UI 控制

1. 打开供应商列表
2. 执行绑定了 `decorate_supplier_list` 的动作
3. `email` 列应隐藏
4. `status` 列标题应变化
5. `delete` 应禁用

### 前端插件页面

1. 打开 `supplier_portal` 页面
2. 确认页面文案跟随当前语言
3. 如果页面会读取宿主业务数据，不要直接请求绝对地址 `http://host/api/...`
4. 应确认页面通过宿主鉴权通道调用 API，并使用宿主当前实际接口路径
5. 若出现 `401 Unauthorized`，优先检查是否绕过了宿主统一 `api` 实例，或误用了自定义接口路径如 `/api/data/get-record`
6. 建议优先使用 `@trusteem/asapflow-plugin-sdk`：
   - `usePluginContext()` 取 `recordId`
   - `useHostDataApi().getRecord(entityCode, recordId)` 取单条记录
   - `useAuthContext()` 取当前用户与权限

## 8. 语言资源约定

统一语言资源目录：

```text
plugin-workspace/locales/
  zh-CN.json
  en-US.json
```

命名空间约定：

- 平台前端：`platform.*`
- 后端插件：`supplier_guard.*`、`supplier_runtime_rules.*`
- 前端插件：`supplier_portal.*`

## 9. 推荐提供给 AI 助手的文档

如果目标是让 AI 生成和联调插件，建议至少提供：

- [overview.md](overview.md)
- [backend-plugin-guide.md](backend-plugin-guide.md)
- [frontend-plugin-guide.md](frontend-plugin-guide.md)
- [core-sdk-reference.md](core-sdk-reference.md)
- [plugin-i18n-guide.md](plugin-i18n-guide.md)
- [plugin-ui-protocol.md](plugin-ui-protocol.md)
