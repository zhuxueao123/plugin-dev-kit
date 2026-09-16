# 前端插件开发指南

## 1. 目录结构

前端插件页面统一放在：

```text
plugins/frontend/src/pages/<plugin-code>/
  manifest.json
  <page>.vue
```

## 2. manifest 示例

```json
{
  "pluginCode": "supplier_portal",
  "pluginName": "供应商门户页面插件",
  "version": "0.1.0",
  "pages": [
    {
      "code": "supplier-dashboard",
      "displayName": "供应商总览",
      "route": {
        "name": "plugin-supplier-dashboard",
        "path": "/plugins/supplier/dashboard"
      },
      "entry": "./supplier-dashboard.vue",
      "bundle": "supplier-dashboard.es.js",
      "entryExport": "default",
      "propsSchema": {
        "type": "object"
      },
      "permissions": [],
      "hostFeatures": [],
      "layout": {
        "fullWidth": true
      },
      "metadata": {
        "icon": "dashboard"
      }
    }
  ]
}
```

## 3. 开发流程

1. 在 `plugins/frontend/src/pages/<plugin-code>/` 下创建页面和 `manifest.json`。
2. 在 `plugins/frontend/` 下执行 `npm install`。
3. 执行 `npm run build`。
4. 宿主会读取构建后的 bundle 和 manifest。

## 4. 参考示例

- 正式示例：`plugins/frontend/src/pages/supplier_portal/`
- 可复制模板：`plugins/examples/frontend/supplier_portal/`

## 5. 国际化约定

前端插件与后端插件统一复用：

```text
plugins/locales/<locale>.json
```

并按 `pluginCode` 做命名空间。
宿主前端会把 `plugins/locales/*.json` merge 到现有 `vue-i18n`，所以前端插件应直接复用这套语言资源。

同一份语言包还可以覆盖平台前端文案，建议命名空间约定为：

- 平台前端：`platform.*`
- 前端插件：`<pluginCode>.*`
- 后端插件：`<pluginCode>.*`

推荐写法：

```js
import { usePluginI18n } from 'src/composables/usePluginI18n'

const { t } = usePluginI18n('supplier_portal')
t('pages.dashboard_title')
```

不要在不同插件目录里各自维护一套零散语言包。

## 6. 进一步阅读

- 场景动作跳插件页面时，AI 需注意以下边界：
  - 插件页面路由名取自 `manifest.json -> pages[].route.name`，配置动作跳转时必须使用这个真实值。
  - 如果场景动作 `metadata.navigation.type` 配成 `scenario-form`，前端只会跳平台内置业务表单页，不会跳插件页面。
  - 要跳插件页面，应使用 `metadata.navigation.type = "custom-page"`。
  - 当前记录、来源功能、来源场景等动态参数应写在 `metadata.navigation.payload` 中，使用 `@row.id`、`@context.featureCode` 这类占位符；不要使用 `{{row.id}}`。
  - 当前前端只会解析 `payload` 中的 `@...` 占位符，再把结果合并到最终 URL query；如果把 `@row.id` 直接写在 `navigation.query`，它会原样出现在 URL 中。
  - 如果是修改“某个场景中的某个按钮跳转到插件页面”，应更新场景动作绑定，而不是只更新动作定义。CLI 上通常应走 `system update-scenario`，不是 `system update-action`。

## 7. 插件页面调用宿主 API 的约定

- 前端插件页面不要直接使用绝对地址调用宿主 API，例如 `http://host:port/api/...`。
  原因：这样很容易绕过宿主前端现有的登录态与请求拦截器，导致 `401 Unauthorized`。

- 前端插件页面不要自行猜测宿主接口路径，例如直接调用 `/api/data/get-record`。
  说明：应优先复用宿主当前已使用的数据接口语义和路径，不要发明新的别名路径。

- 宿主前端当前会在统一 `api` 实例上自动追加 `Authorization: Bearer <token>`。
  如果插件页面没有复用宿主的鉴权请求通道，而是自己 `fetch(...)` 或自己创建 `axios` 实例，请求通常不会自动携带 token。

- 读取单条业务记录时，应对齐宿主当前数据接口：
  - 单条读取：`GET /api/data/{entityCode}/{recordId}`
  - 列表查询：`POST /api/data/{entityCode}/query`
  - 新增记录：`POST /api/data/{entityCode}`
  - 更新记录：`PUT /api/data/{entityCode}/{recordId}`

- 如果插件页面需要根据场景动作传入的 URL 参数读取记录，应先从路由 query 中取 `id` 或 `recordId`，再按宿主数据接口读取该记录。

- 如无明确 SDK 能力说明，不要假设 `@trusteem/asapflow-plugin-sdk` 中的 `useApi()` 已经可用。
- 当前仓库已在宿主前端通过 `frontend/src/boot/plugin-sdk.js` 注入桥接层，插件页面应优先通过 `@trusteem/asapflow-plugin-sdk` 获取能力，不要绕过宿主直接造请求。
- `@trusteem/asapflow-plugin-sdk` 当前已提供：
  - `useApi()`：宿主统一 axios 实例，自动带 `Authorization` 与 `X-Locale`
  - `useAuthContext()`：当前用户、claims、roles、permissions、`isAuthenticated`、`refresh()`
  - `useRouteContext()` / `usePluginContext()`：当前路由、query、插件页元数据、来源场景参数
  - `useHostDataApi()`：面向 `/api/data/{entityCode}` 的标准 CRUD 包装
- 若插件页面需要当前登录人信息，先用 `useAuthContext()`，不要自行维护 token 或重复实现 `/identity/whoami` 调用。
  AI 需要先核对当前宿主是否已接入 `frontend/src/boot/plugin-sdk.js`，再决定这些能力在运行时是否可用。

### 7.1 AI 调用 SDK 的固定规则

- 默认安装方式是 `npm install @trusteem/asapflow-plugin-sdk`。
- 只有平台维护人员在完整源码仓库内联调 SDK 本身时，才临时改用本地文件依赖。
- npm 不可达时，应由平台方提供版本匹配的 tarball，不要假设开发包旁边存在 SDK 源码目录。

- 先检查宿主是否已接入桥接：
  - `frontend/quasar.config.js` 的 `boot` 中应包含 `plugin-sdk`
  - `frontend/src/boot/plugin-sdk.js` 应存在
  - 若不存在，不要假设 SDK 在运行时一定可用，应先补宿主桥接

- 再检查前端插件工程里 SDK 包是否真实可解析：
  - 查看 `node_modules/@trusteem/asapflow-plugin-sdk/package.json` 是否存在
  - 查看其 `exports` 是否导出 `./index.js`
  - 若使用 npm 安装，优先确认 `package.json` 中已声明 `@trusteem/asapflow-plugin-sdk`
  - 如果这里只是坏软链、坏 junction、缺失目录或未安装依赖，先修复包，再改业务代码

- 插件页读取当前用户时，优先这样写：

```js
import { useAuthContext } from '@trusteem/asapflow-plugin-sdk'

const { user, claims, roles, permissions, isAuthenticated, refresh } = useAuthContext()
```

- 插件页读取场景动作传入参数时，优先这样写：

```js
import { usePluginContext } from '@trusteem/asapflow-plugin-sdk'

const { recordId, mode, featureCode, scenarioCode, query } = usePluginContext()
```

- 插件页读取宿主业务数据时，优先这样写：

```js
import { useHostDataApi, usePluginContext } from '@trusteem/asapflow-plugin-sdk'

const { recordId } = usePluginContext()
const { getRecord, queryRecords, createRecord, updateRecord, deleteRecord } = useHostDataApi()

const record = await getRecord('electronic_cost_card', recordId.value)
```

- 如果只是想调用宿主统一请求实例，也可以这样写：

```js
import { useApi } from '@trusteem/asapflow-plugin-sdk'

const api = useApi()
const { data } = await api.get('/identity/whoami')
```

- 不要这样做：
  - 不要直接 `fetch('http://host:port/api/...')`
  - 不要自己新建 axios 实例去打宿主接口
  - 不要自行拼 `/api/data/get-record`
  - 不要把用户 token 存到插件自己的状态里

- 当场景动作把 `id` / `recordId` 通过路由 query 传进来时，插件页应从 `usePluginContext()` 读取，而不是自己解析 `window.location.href`

- 若插件页报 `401 Unauthorized`，AI 排查顺序应固定为：
  1. 是否用了 `useApi()` 或 `useHostDataApi()`
  2. 是否误调了不存在的宿主接口路径
  3. 是否在宿主页面内加载，而不是脱离宿主单独打开插件 bundle
  4. 宿主当前登录态是否已过期

### 7.2 `plugin-workspace` 场景的额外约束

- 客户开发包统一从 npm 安装 SDK，不携带 `plugins-sdk/` 源码副本。
- 客户侧推荐依赖写法：

```json
{
  "dependencies": {
    "@trusteem/asapflow-plugin-sdk": "^0.1.0"
  }
}
```

- 前端首次构建前需要访问 npm Registry；也可以显式执行：

```bash
npm install @trusteem/asapflow-plugin-sdk
```

- AI 在 `plugin-workspace` 中应先检查：
  1. `package.json` 是否已经声明 `@trusteem/asapflow-plugin-sdk`
  2. `node_modules/@trusteem/asapflow-plugin-sdk` 是否已成功安装且能读取包导出

- 如果这两个条件不成立，正确顺序是：
  1. 先执行 `npm install @trusteem/asapflow-plugin-sdk`
  2. 如果 npm 不可达，停止构建并提示需要平台方提供离线 tarball
  3. 再改插件页面代码
  4. 最后运行构建验证

## 8. 系统字典与编号规则的前端边界

- 系统字典和编号规则属于平台标准能力，AI 在做前端插件方案时，不要默认它们需要插件自己实现。

- 字段如果是有限枚举值，例如 `status`、`type`、`category`、`level`、`priority`，优先让平台字段配置引用系统字典：
  - `optionSourceType = "dictionary"`
  - `optionSource.dictionaryCode = "<dictionary-code>"`

- 前端插件不要把这类选项长期硬编码在页面里，除非是一次性临时演示数据。

- 如果插件页需要显示系统字典值，应优先复用宿主已有字段配置或宿主接口返回值；不要在插件里复制一套状态映射表并长期维护。

- 业务编号、单据号、编码规则应优先使用平台编号规则，不要在插件前端手工拼接。

- AI 设计或修改业务功能时，遇到“自动生成编号”的需求，固定顺序应为：
  1. 先检查是否已有可复用的 `number-rule`
  2. 没有则新增 `number-rule`
  3. 再把规则绑定到实体字段 `metadata.generator.ruleCode`
  4. 最后再确认表单是否展示该字段

- 插件页如果只是读取记录，不应自己重新生成编号。

- 插件页如果负责新增/编辑数据，也不应在前端补造业务编号后再提交，除非需求明确要求绕开平台编号引擎。

- 当标准表单保存返回类似 `Field '<field>' requires a value when no default is provided.` 时，AI 应优先排查：
  1. 是否有必填字段未提交
  2. 是否把本应由编号规则生成的字段当成普通字段处理了
  3. 是否把系统行号/排序号字段隐藏后又未提供默认值

- 若标准功能已经可以通过系统字典或编号规则配置解决，不要优先建议“再写一个插件页面来规避”。

- UI 控制协议见 [plugin-ui-protocol.md](plugin-ui-protocol.md)
- 完整联调链路见 [plugin-e2e-examples.md](plugin-e2e-examples.md)
