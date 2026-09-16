# AsapFlow 建模配置参考

这份文档面向通过 CLI 或 AI 进行建模的使用者，说明实体、功能、场景相关请求字段的含义、默认规则与常见误区。

适用范围：

- `system create-entity`
- `system add-entity-fields`
- `system create-feature`
- `workflow create-definition`
- `workflow update-definition`
- `number-rule create`
- `number-rule update`
- 后台功能/场景配置页保存时对应的请求结构

## 1. 术语

- `entity`：实体，表示真实业务数据结构，通常会对应业务库中的表。
- `feature`：功能，是一个业务功能单元，负责承载字段、场景、动作、菜单。
- `scenario`：场景，是功能下的页面或视图配置，例如 `list`、`detail`。
- `action`：动作，是场景中的按钮或操作，例如“新建”“编辑”“删除”“查看”。
- `workflow`：流程定义，用于描述节点、动作、参与人和流转规则。
- `number rule`：编号规则，用于描述编号如何拼接、何时重置、如何生成流水号。

规则：

- `action` 不等于 `scenario`
- `feature` 可以绑定某个 `entity`
- `scenario` 隶属于 `feature`
- `workflow` 不等于 `scenario`
- `number rule` 不等于某一个具体业务字段

## 2. 数据类型与表单控件

AI 创建功能和场景时，不能按“`dataType = string` 就用 `q-input`”这种简单规则建模。`dataType` 只描述存储层逻辑类型；场景表单中的控件由字段语义和场景字段配置里的 `component` / `componentType` 决定。

推荐按这个顺序判断：

1. 先判断字段是不是自由输入，还是有限选项、关联对象、附件、图片、富文本。
2. 再判断选项来源是 `static`、`dictionary`、`relation` 还是 `plugin`。
3. 最后再决定 `component` / `componentType`，不要只按 `dataType` 选控件。

### 2.1 平台支持的常用逻辑数据类型

常见 `dataType`：

- `string`
- `text`
- `int`
- `bigint`
- `smallint`
- `decimal`
- `float`
- `double`
- `boolean`
- `date`
- `time`
- `datetime`
- `timestamp`
- `uuid`
- `json`
- `binary`

说明：

- `dataType` 面向实体字段建模，决定业务数据如何落库。
- `component` / `componentType` 面向场景表单渲染，决定用户如何录入或查看数据。
- 同一个 `dataType = string`，可能对应文本框、下拉框、关联选择器、上传组件，不应默认视为普通文本。

### 2.2 Agent 建模决策表

| `dataType` | 字段语义 | 推荐组件 | 选项来源/补充说明 |
| --- | --- | --- | --- |
| `string` | 名称、编码、手机号、短标题 | `q-input` | 仅在确认为自由文本时使用 |
| `string` / `text` | 备注、说明、原因、地址 | `q-textarea` | 长文本不要退化成单行输入 |
| `string` | 状态、类型、等级、分类、审批结论 | `q-select` / `radio-group` | 优先判断为有限选项；来源用 `dictionary` 或 `static` |
| `string` | 标签、适用范围、可选服务 | `checkbox-group` | 适合多选有限集合；来源用 `dictionary` / `static` / `plugin` |
| `string` / `uuid` | 客户、供应商、物料、人员、部门、仓库 | `relation-picker-field` | 关联对象优先用 `relation`，小规模选项也可 `q-select + relation` |
| `int` / `bigint` / `smallint` | 数量、次数、排序号 | `q-number` | 纯数字输入 |
| `decimal` / `float` / `double` | 金额、单价、比例、重量、汇率 | `q-decimal` | 需要保留小数语义时优先用 `q-decimal` |
| `boolean` | 是否启用、是否默认、是否通过 | `q-toggle` / `q-checkbox` | 布尔开关或单勾选 |
| `date` | 业务日期、计划日期、生效日期 | `q-date` | 仅日期，不含时间 |
| `time` | 开始时间、结束时间、班次时间 | `q-time` | 仅时间 |
| `datetime` / `timestamp` | 创建时间、审批时间、发货时间 | `q-datetime` | 日期时间组合 |
| `text` / `string` | 公告正文、邮件模板、长格式内容 | `q-editor` | 需要富文本时使用 |
| `binary` / `string` | 现场照片、产品图片、签名图片 | `image-upload-field` | 图片不要建模成普通文本链接输入 |
| `binary` / `string` | 合同、报价单、检验报告、附件包 | `attachment-upload-field` | 普通附件上传 |
| `json` | 结构化配置、扩展参数 | 视场景而定 | 第一版通常不直接暴露通用编辑器，除非明确要求 |

强规则：

- `dataType` 不能直接推出组件；必须先判断字段语义。
- 只要字段是“有限候选集合”，就不应默认用 `q-input`。
- 只要字段语义是“关联业务对象”，就不应默认用自由文本框。
- 图片、附件、富文本都不应退化成普通文本输入。

### 2.3 表单控件类型

AI 创建功能和场景时，不应只根据数据类型使用 `q-input` 或 `q-number`。字段在场景表单中的控件由场景字段配置里的 `component` / `componentType` 决定；如果业务语义明确，应显式配置合适组件。

当前推荐控件：

| 组件值 | 用途 | 常见字段 |
| --- | --- | --- |
| `q-input` | 单行文本 | 名称、编码、手机号、备注摘要 |
| `q-textarea` | 多行文本 | 备注、说明、处理意见、地址 |
| `q-number` | 整数或普通数字 | 数量、次数、排序号 |
| `q-decimal` | 精确小数 | 金额、单价、重量、比例 |
| `q-toggle` | 布尔开关 | 是否启用、是否默认、是否通过 |
| `q-checkbox` | 单个布尔勾选项 | 同意条款、是否加急 |
| `q-date` | 日期 | 业务日期、计划日期、出生日期 |
| `q-time` | 时间 | 开始时间、结束时间、班次时间 |
| `q-datetime` | 日期时间 | 创建时间、审批时间、发货时间 |
| `q-select` | 下拉单选或可搜索选择 | 状态、类型、等级、客户、物料 |
| `radio-group` | 少量互斥选项 | 性别、审批结论、优先级 |
| `checkbox-group` | 多选项 | 标签、适用范围、可选服务 |
| `relation-picker-field` | 关联对象选择，适合弹窗/复杂搜索 | 客户、供应商、物料、人员、部门 |
| `q-editor` | 富文本 | 公告正文、邮件模板、长格式说明 |
| `image-upload-field` | 图片上传与只读预览 | 现场照片、产品图片、签名图片 |
| `attachment-upload-field` | 附件上传 | 合同、报价单、检验报告 |

运行时实现说明：`q-textarea` 由表单渲染器转换为 `QInput` 并设置 `type = textarea`；不要尝试直接动态加载不存在的 `QTextarea` 组件。

选项来源规则：

- `q-select`、`radio-group`、`checkbox-group` 支持 `optionSourceType = static | dictionary | plugin`。
- `relation-picker-field` 使用 `optionSourceType = relation`。
- 静态选项写入 `props.options` 或 `metadata.options`，每项建议包含 `label` 和 `value`。
- 系统字典字段优先使用 `q-select + optionSourceType = dictionary`，不要把字典值硬编码在静态文本框里。
- 关联实体字段优先使用 `relation-picker-field`；如果选项规模小，也可用 `q-select + relation`。
- `relation-picker-field` 必须同时具有可访问的目标实体和完整关系配置；只保留旧平台表名或数据源字符串不能形成可用选择器。
- 迁移依赖尚未落地时，先降级为 `q-input`，在 `extraMetadata` 中保留旧数据源和 `suggestedComponent`，依赖完成后再升级控件。

AI 选型规则：

- 先看业务描述判断字段语义，再决定控件；字段名只能作为弱提示，不能作为唯一依据。
- 需求表达为“状态/类型/等级/结论/分类/优先级/阶段”等有限候选集合时，优先考虑 `q-select` 或 `radio-group`。
- 需求表达为“客户/供应商/物料/人员/部门/组织/仓库”等业务对象选择时，优先使用 `relation-picker-field`。
- 需求表达为“备注/说明/原因/地址/处理意见”等长文本时，优先使用 `q-textarea`。
- 需求表达为“金额/单价/成本/比例/重量”等数值且带小数语义时，优先使用 `q-decimal`。
- 需求表达为“业务日期/计划日期”时，优先使用 `q-date`；表达为“创建时间/审批时间/发货时间”时，优先使用 `q-datetime` 或 `q-time`。
- 附件、图片、照片、合同、报告等字段不要用普通文本框，使用上传组件。

场景字段配置示例：

```json
{
  "fieldKey": "order_status",
  "form": {
    "component": "q-select",
    "optionSourceType": "dictionary",
    "optionSource": {
      "dictionaryCode": "order_status"
    },
    "span": 6,
    "required": true
  }
}
```

## 5. 编号规则配置

### 5.1 `number-rule create`

核心字段：

- `code`
  含义：规则编码。
  必填：是。
  建议：英文小写加下划线，例如 `sales_order_no`。

- `name`
  含义：规则名称。
  必填：是。

- `status`
  含义：规则状态。
  必填：否。
  推荐值：
  - `draft`
  - `active`

- `description`
  含义：规则说明。
  必填：否。

- `segments`
  含义：编号段列表。
  必填：是。
  说明：编号最终由多个段按顺序拼接。

- `sequencePolicy`
  含义：流水号策略。
  必填：否，但包含 `sequence` 段时建议显式传。

- `overridePolicy`
  含义：覆盖策略。
  必填：否。
  说明：通常用于表达是否允许手工覆盖自动编号。

### 5.2 `segment`

- `type`
  含义：段类型。
  必填：是。
  第一版支持：
  - `text`
  - `date`
  - `sequence`
  - `field`
  - `context`

- `value`
  含义：固定文本值。
  适用：`type = text`。

- `format`
  含义：日期格式。
  适用：`type = date`。
  示例：`yyyyMMdd`、`yyyyMM`。

- `length`
  含义：流水号长度。
  适用：`type = sequence`。

- `padChar`
  含义：流水号左侧补位字符。
  适用：`type = sequence`。

- `key`
  含义：字段名或上下文键。
  适用：`type = field` 或 `context`。

### 5.3 `sequencePolicy`

- `resetPolicy`
  含义：重置周期。
  必填：否。
  推荐值：
  - `none`
  - `daily`
  - `monthly`
  - `yearly`

- `startValue`
  含义：起始序号。
  必填：否。
  默认：`1`。

- `step`
  含义：步长。
  必填：否。
  默认：`1`。

- `scope`
  含义：编号分组作用域。
  必填：否。
  说明：可按 `field` 或 `context` 维度独立计数。

### 5.4 `scope`

- `type`
  含义：作用域来源。
  必填：是。
  推荐值：
  - `field`
  - `context`

- `key`
  含义：字段名或上下文键名。
  必填：是。

### 5.5 `number-rule preview`

常用字段：

- `ruleCode`
  含义：规则编码。
  必填：是。

- `businessData`
  含义：业务数据上下文。
  必填：否。

- `contextData`
  含义：系统上下文。
  必填：否。

- `now`
  含义：预览时使用的时间。
  必填：否。

说明：

- `preview` 不占用流水号
- `sequence` 段会返回补位占位值，例如 `0000`

### 5.6 实体字段绑定编号规则

如果希望在创建记录时自动生成编号，应在实体字段 `metadata` 中配置：

```json
{
  "generator": {
    "type": "number_rule",
    "ruleCode": "sales_order_no",
    "trigger": "create",
    "allowManualOverride": false
  }
}
```

规则：

- 自动编号在“创建保存时”生成，不在“打开新建表单时”占号
- `allowManualOverride = true` 时，用户已输入值则保留
- `allowManualOverride = false` 时，保存时会强制按规则生成

## 4. 流程配置

### 4.1 `workflow create-definition`

核心字段：

- `code`
  含义：流程编码。
  必填：是。
  建议：英文小写加下划线，例如 `order_approval`。

- `name`
  含义：流程名称。
  必填：是。

- `businessObjectType`
  含义：流程绑定的业务对象类型。
  必填：是。
  示例：`order`、`purchase_request`、`invoice`。

- `description`
  含义：流程说明。
  必填：否。

- `metadata.authorization`
  含义：流程运行期授权配置。
  必填：否。
  说明：可配置 `startPermissionCode` 和 `visiblePrincipals`，用于控制谁能发起、谁能看见实例。

- `draftDefinition`
  含义：流程草稿定义。
  必填：是。
  说明：这是流程真正的结构化定义，平台发布后按它执行。

### 4.2 `draftDefinition`

最小结构：

- `startNodeId`
  含义：起始节点 ID。
  必填：建议显式传。

- `nodes`
  含义：节点列表。
  必填：是。

- `edges`
  含义：默认连线列表。
  必填：是。

### 4.3 `node`

- `id`
  含义：节点 ID。
  必填：是。

- `name`
  含义：节点名称。
  必填：是。

- `type`
  含义：节点类型。
  必填：是。
  第一版推荐值：
  - `start`
  - `approval`
  - `task`
  - `service`
  - `end`

- `participant`
  含义：参与人规则。
  必填：`approval` / `task` 节点通常应提供。

- `actions`
  含义：节点动作。
  必填：`approval` / `task` 节点通常应提供。

- `fieldPermissions`
  含义：节点字段权限覆盖。
  必填：否。
  说明：用于控制业务表单中字段的只读、隐藏、必填、禁用。

- `approvalVisibleFieldKeys`
  含义：审批上下文允许返回的字段编码白名单。
  必填：否。
  说明：未配置时返回实体字段全集，但会排除 `fieldPermissions.visible = false` 的字段和系统敏感字段。`fieldPermissions` 只控制只读、隐藏、必填、禁用等表单行为，不作为默认展示白名单。

- `position`
  含义：设计器画布位置。
  必填：建议提供。
  说明：这是设计器布局信息，不影响执行语义。

### 4.4 `participant`

- `type`
  含义：参与人类型。
  必填：是。
  第一版推荐值：
  - `initiator`
  - `user`
  - `role`

- `userId`
  含义：指定用户 ID。
  适用：`type = user`。

- `roleCode`
  含义：角色编码。
  适用：`type = role`。

### 4.4.1 `metadata.authorization`

- `startPermissionCode`
  含义：发起该流程要求具备的权限码。
  必填：否。

- `visiblePrincipals`
  含义：流程实例可见主体集合。
  必填：否。
  当前推荐值：
  - `initiator`
  - `active_assignee`
  - `active_candidate_role_member`
  - `history_actor`
  - `cc_user`
  - `workflow_instance_admin`

### 4.5 `action`

- `code`
  含义：动作编码。
  必填：是。
  示例：`approve`、`reject`、`confirm_ship`。

- `label`
  含义：动作显示名称。
  必填：是。

- `nextNodeId`
  含义：动作对应的下一节点。
  必填：通常是。

- `completeProcess`
  含义：执行动作后是否直接结束流程。
  必填：否。
  默认：`false`。

### 4.6 `fieldPermission`

- `fieldKey`
  含义：业务表单字段键。
  必填：是。

- `visible`
  含义：是否显示。
  必填：否。

- `readonly`
  含义：是否只读。
  必填：否。

- `required`
  含义：是否必填。
  必填：否。

- `disabled`
  含义：是否禁用。
  必填：否。

### 4.7 `edge`

- `id`
  含义：连线 ID。
  必填：是。

- `sourceId`
  含义：起点节点 ID。
  必填：是。

- `targetId`
  含义：终点节点 ID。
  必填：是。

- `actionCode`
  含义：与哪个动作编码关联。
  必填：否。
  说明：如果不传，表示默认流转线。

- `order`
  含义：优先级顺序。
  必填：否。

### 4.8 流程实例发起

`workflow start-instance` 常用字段：

- `definitionCode`
  含义：流程定义编码。
  必填：是。

- `businessObjectType`
  含义：业务对象类型。
  必填：建议显式传。

- `businessId`
  含义：业务主键或单号。
  必填：是。
  说明：推荐业务页面自动发起时传记录 UUID；如果由 CLI、AI 或外部系统传业务单号，审批上下文会在实例可见性校验通过后按绑定实体的业务标识字段解析实时记录，普通业务详情接口不会因此获得额外权限。

- `businessTitle`
  含义：工作台展示标题。
  必填：是。

- `featureCode`
  含义：跳转业务页面时对应的功能编码。
  必填：否，但建议业务型流程显式传。

- `scenarioCode`
  含义：跳转业务页面时对应的场景编码。
  必填：否，但建议业务型流程显式传。

- `mode`
  含义：进入场景时的页面模式。
  必填：否。
  常见值：`view`、`edit`。

- `businessData`
  含义：实例启动时附带的业务上下文载荷。
  必填：否。
  说明：审批详情不以该字段作为业务数据源；详情展示通过审批上下文接口读取实时业务记录，避免提交后单据修改导致审批页与实际数据不一致。

- `ccUserIds`
  含义：显式抄送的系统用户 ID 列表。
  必填：否。
  说明：这些用户会在流程工作台的 `cc` 视图中看到该实例，但不会自动获得任务处理权。

### 4.9 流程建模建议

1. 第一版优先保持线性流程，不要一次生成过多复杂分支
2. 参与人优先用 `role` 或固定 `user`
3. 审批节点建议显式配置 `approvalVisibleFieldKeys`，不要默认把所有业务字段暴露给审批上下文
4. 需要影响业务表单时，再补 `fieldPermissions`
5. 发起实例前，尽量确保 `featureCode` 和 `scenarioCode` 已存在
6. 如果需要限制实例可见范围，优先配置 `metadata.authorization.visiblePrincipals`
7. 如果是修改流程，先读取当前定义再变更
8. 业务标识字段建议使用 `*_no`、`*_code` 或在字段 `metadata` 中标记 `businessIdentifier`，便于审批上下文从业务单号解析实时记录

## 2. 实体配置

### 2.1 `CreateEntityDefinitionRequest`

字段说明：

- `code`
  含义：实体编码，系统内唯一标识。
  必填：是。
  建议：使用英文小写加下划线，如 `supplier`。

- `name`
  含义：实体显示名称。
  必填：是。

- `module`
  含义：所属模块或业务域。
  必填：是。
  示例：`crm`、`supply`、`sales`。

- `storageType`
  含义：实体存储类型。
  必填：是。
  默认：`table`。
  常见值：`table`。
  说明：当前常规建模场景应使用 `table`。

- `schemaName`
  含义：目标 schema 名称。
  必填：是。
  默认：`public`。

- `physicalName`
  含义：业务库中的物理表名。
  必填：是。
  建议：通常与 `code` 保持一致。

- `dataSourceCode`
  含义：业务数据源标识。
  必填：否。
  默认规则：如果未提供，服务端按默认业务数据源规则处理。
  说明：除非客户明确要求建到指定业务库，否则不要显式传这个字段。

- `writeEnabled`
  含义：是否允许通过平台写入该实体。
  必填：否。
  默认：由 CLI 简化输入默认补成 `true`。

- `description`
  含义：实体说明。
  必填：否。

- `metadata`
  含义：扩展元数据。
  必填：否。
  默认：`{}`。
  说明：当前更多用于备注和后续扩展，不应用来承载核心业务逻辑。

- `fields`
  含义：实体字段列表。
  必填：否，但创建可用实体时通常应提供至少一个业务字段。

### 2.2 `EntityFieldRequest`

- `code`
  含义：字段编码。
  必填：是。

- `name`
  含义：字段显示名。
  必填：是。

- `dataType`
  含义：字段类型。
  必填：是。
  常见值：`string`、`int`、`decimal`、`bool`、`date`、`datetime`、`uuid`。

- `length`
  含义：字符串长度。
  必填：否。
  建议：`string` 字段一般显式传。

- `precision` / `scale`
  含义：数值精度与小数位。
  必填：否。
  说明：仅在 `decimal` 等数值类型下使用。

- `isNullable`
  含义：是否可空。
  必填：否。
  默认：`true`。

- `isPrimary`
  含义：是否主键。
  必填：否。
  说明：常规业务字段不需要显式设置，系统保留 `id` 主键。

- `defaultValue`
  含义：默认值。
  必填：否。

- `orderIndex`
  含义：字段顺序。
  必填：否。
  说明：CLI 会尽量补默认顺序；手工精排时再显式传。

- `category`
  含义：字段分类。
  必填：是。
  默认：`business`。
  说明：普通业务字段使用 `business`。

- `metadata`
  含义：字段扩展元数据。
  必填：否。

### 2.3 实体建模默认规则

1. 客户未强调业务库时，不传 `dataSourceCode`。
2. `storageType` 默认使用 `table`。
3. `schemaName` 默认使用 `public`。
4. `physicalName` 通常与 `code` 一致。
5. 实体创建成功后，系统会自动补系统字段：`id`、`created_at`、`created_by`、`updated_at`、`updated_by`。

## 3. 功能配置

### 3.1 `CreateFeatureRequest`

- `code`
  含义：功能编码。
  必填：是。
  示例：`supplier_management`。

- `name`
  含义：功能显示名。
  必填：是。

- `module`
  含义：所属模块。
  必填：是。

- `dataSourceType`
  含义：功能数据来源类型。
  必填：是。
  默认：`table`。
  常见值：`table`。

- `dataSourceName`
  含义：功能绑定的实体或数据对象名称。
  必填：否，但如果功能对应某个实体，建议明确绑定。
  重要：对于实体型功能，这个字段本质上应指向实体编码，例如 `supplier`。

- `dataSourceCode`
  含义：业务数据源标识。
  必填：否。
  默认规则：不强调业务库时不传。

- `description`
  含义：功能说明。
  必填：否。

- `isActive`
  含义：是否启用。
  必填：否。
  默认：`true`。

- `metadata`
  含义：功能扩展元数据。
  必填：否。

- `fields`
  含义：功能字段池。
  必填：通常是。
  说明：场景字段配置依赖功能字段池。

- `scenarios`
  含义：功能下预置场景。
  必填：否。
  说明：通常不手写全部场景，而是通过默认场景初始化生成。

### 3.2 CLI `system create-feature` 的额外输入语义

CLI 在简化输入中额外支持：

- `entityCode`
  含义：该功能绑定的实体编码。
  作用：CLI 会自动把它映射到 `feature.dataSourceName`。
  建议：只要功能对应某个实体，就应传这个字段。

- `initializeDefaultScenarios`
  含义：是否初始化默认场景。
  默认：`true`。

- `includeDefaultListActions`
  含义：是否在默认 `list` 场景中附带系统动作。
  默认：`true`。
  默认动作：`create`、`edit`、`delete`、`view`。
  说明：如果客户明确要求“不要默认动作”，应传 `false`。

### 3.3 `FeatureFieldRequest`

- `fieldKey`
  含义：功能字段键。
  必填：是。
  约定：如果字段来源是实体，`fieldKey` 应直接使用实体原始字段名，例如 `contact_person`。
  CLI 兼容：对于 `sourceType = entity`，如果只提供 `sourceField`，CLI 会自动把它补成 `fieldKey`。

- `displayName`
  含义：字段显示名称。
  必填：是。

- `dataType`
  含义：字段类型。
  必填：是。

- `sourceType`
  含义：字段来源类型。
  必填：是。
  默认：`entity`。
  常见值：
  - `entity`：直接来自绑定实体
  - `related`：来自关联实体
  - `expression`：来自表达式
  - `procedure`：来自过程返回
  - `plugin`：来自插件输出

- `sourceConfig`
  含义：来源配置。
  必填：否。
  说明：`entity` 场景下通常由 CLI 自动生成最小配置。

- `sourceField`
  含义：CLI 简化输入中的实体原始字段名。
  必填：否。
  说明：推荐对实体字段显式传这个值，并与 `fieldKey` 保持一致。

- `isIdentifier`
  含义：是否标识字段。
  必填：否。
  说明：通常主名称字段设为 `true`，如客户名称、供应商名称。

- `defaultVisible`
  含义：默认是否可见。
  必填：否。
  默认：`true`。

- `defaultEditable`
  含义：默认是否可编辑。
  必填：否。
  默认：`true`。

- `validationRules`
  含义：校验规则。
  必填：否。

- `metadata`
  含义：扩展元数据。
  必填：否。

## 4. 场景配置

### 4.1 `ScenarioRequest`

- `code`
  含义：场景编码。
  必填：是。
  常见值：`list`、`detail`。

- `name`
  含义：场景名称。
  必填：是。

- `scenarioType`
  含义：场景类型。
  必填：是。
  常见值：`list`、`form`、`detail`、`report`。
  说明：当前默认初始化里使用 `list` 和 `form`/`detail` 语义。

- `description`
  含义：场景说明。
  必填：否。

- `dataBinding`
  含义：数据绑定配置。
  必填：否。
  说明：通常是 JSON 字符串，用于表达过滤、排序、关联加载等规则。

- `defaultView`
  含义：默认视图配置。
  必填：否。
  说明：通常是 JSON 字符串，用于保存列表/表单的默认视图偏好。

- `visibilityRule`
  含义：可见性规则。
  必填：否。
  说明：通常是 JSON 或规则表达式字符串。

- `isDefault`
  含义：是否默认场景。
  必填：否。

- `metadata`
  含义：场景扩展元数据。
  必填：否。
  重要：搜索配置、页面行为、表单布局等常放在这里。

  表单类场景支持在 `metadata.formLayout` 中配置字段标签页。通过 CLI 传参时，`metadata` 仍是 JSON 字符串，内部结构示例：

  ```json
  {
    "formLayout": {
      "type": "tabs",
      "tabs": [
        { "key": "basic", "label": "基本信息", "fields": ["code", "name"] },
        { "key": "quality", "label": "质量信息", "fields": ["standard", "result"] }
      ]
    }
  }
  ```

  AI 迁移旧平台功能时，如果旧平台字段已有分组、块、页签、区域标题，应优先映射为这里的 `tabs`。字段很多但没有明确分组时，也应按业务语义拆分，例如基本信息、业务信息、财务信息、附件信息。省略该配置或使用 `type = "flat"` 时，表单页会按字段顺序平铺展示。

- `fieldGroups`
  含义：场景字段分组与视图配置。
  必填：建议是。
  说明：这是当前推荐使用的结构。

- `fields`
  含义：旧版场景字段结构。
  状态：兼容保留，不建议新调用使用。

- `actions`
  含义：场景动作列表。
  必填：否。
  说明：如果更新场景时传了这个数组，后端会按新数组整体重建场景动作。

### 4.2 `ScenarioFieldGroupRequest`

- `featureFieldKey`
  含义：引用的功能字段键。
  必填：是。

- `orderIndex`
  含义：字段在场景中的顺序。
  必填：否。

- `list`
  含义：列表视图配置。
  必填：否。

- `form`
  含义：表单视图配置。
  必填：否。

- `detail`
  含义：详情视图配置。
  必填：否。

### 4.3 `ScenarioFieldViewConfigRequest`

- `enabled`
  含义：该视图上下文是否启用该字段。
  默认：`true`。

- `visible`
  含义：是否显示。

- `readonly`
  含义：是否只读。

- `required`
  含义：是否必填。

- `filterable`
  含义：列表中是否可筛选。

- `columnWidth`
  含义：列表列宽。

- `align`
  含义：对齐方式。

- `componentType`
  含义：表单组件类型。

- `placeholder`
  含义：占位提示。

- `span`
  含义：表单布局跨度。

- `props`
  含义：组件属性扩展。

- `modes`
  含义：按 `create/edit/view` 模式覆盖的配置。

- `extraMetadata`
  含义：额外视图元数据。

### 4.3.1 `metadata.formLayout`

适用范围：

- `scenarioType = form`
- `scenarioType = detail`
- 代码为 `detail`、实际用于新建/编辑/查看的表单场景

支持字段：

- `type`
  含义：表单布局类型。
  推荐值：
  - `flat`：平铺表单。
  - `tabs`：按标签页分组展示。

- `tabs`
  含义：标签页数组，仅在 `type = tabs` 时生效。

- `tabs[].key`
  含义：标签页稳定键。
  建议：英文小写加下划线，例如 `basic`、`finance`。

- `tabs[].label`
  含义：标签页显示名。

- `tabs[].fields`
  含义：该标签页包含的字段键数组。
  说明：这里填写的是 `featureFieldKey` / 字段编码，不是字段显示名。

运行规则：

- 标签页和字段均按数组顺序展示。
- 重复字段只归入第一次出现的标签页。
- 未分组但在表单中可见的字段会自动归入“其他信息”。
- `fieldGroups` 仍然决定字段是否启用、是否可见、是否必填、控件类型和排序；`formLayout.tabs` 只决定这些可见字段如何分组。
- 更新场景时应保留已有 `metadata.search`、`metadata.detailTables` 等其它 metadata 配置，只增改 `formLayout` 相关部分。

AI 建模规则：

- 字段数量超过 12 个，建议主动配置标签页。
- 从旧平台迁移时，旧平台字段块、分组标题、页签、区域都应优先转为 `tabs`。
- 如果包含主从表，主表字段的标签页配置放在 `metadata.formLayout`；子表列表仍放在 `metadata.detailTables`。
- 不要用标签页隐藏字段；字段显隐应在 `fieldGroups[].form.visible` / `fieldGroups[].detail.visible` 中配置。

CLI 请求示例见 `examples/update_detail_scenario_tabs.json`。

### 4.4 `ScenarioActionRequest`

- `actionId`
  含义：引用的动作定义 ID。
  必填：是。

- `displayName`
  含义：场景中的动作显示名。
  必填：是。

- `orderIndex`
  含义：动作顺序。

- `layoutSlot`
  含义：动作展示位置。
  默认：`toolbar`。
  常见值：`toolbar`、`row`、`toolbar-primary`、`row-primary`。

- `visibilityRule`
  含义：显示规则。

- `confirmation`
  含义：二次确认配置。

- `metadata`
  含义：动作扩展元数据。
  重要：导航规则、选择范围、插件绑定通常都放这里。

常见 metadata 内容：

- `placement`
  含义：动作放在工具条还是行内。
- `selectionScope`
  含义：需要的选择范围，如 `none`、`row`、`single`、`multi`。
- `navigation`
  含义：点击动作后的跳转规则。

场景动作导航补充约定：

- `navigation.type = "scenario-form"`
  含义：跳平台内置业务表单页。
  结果：前端会拼成 `/business/<featureCode>/<scenarioCode>/form/<mode>/<recordId?>` 这一类宿主路由，而不是插件页面。

- `navigation.type = "custom-page"`
  含义：跳前端插件页面。
  必填：`navigation.routeName`。
  说明：这里必须填写插件 `manifest.json` 中声明的真实路由名，例如 `plugin-supplier-dashboard`，不能写 `scenarioCode` 代替。

- 如果目标是“列表行内按钮跳插件自定义页面”，不要使用 `scenario-form`。
  推荐：把动作定义的 `actionType` 设为非标准类型（例如 `execute`），再在场景动作 `metadata.navigation` 中配置 `custom-page`。
  原因：`create/update/view/delete` 会被列表运行时识别为标准动作，容易继续走默认表单逻辑。

- 动态参数占位符写法使用 `@...`，例如 `@row.id`、`@context.featureCode`，不要使用 `{{row.id}}`。

- 动态参数应放在 `navigation.payload` 中，不应只写在 `navigation.query` 中。
  说明：当前前端会先解析 `navigation.payload` 里的 `@row.id` 占位符，再把解析结果和 `navigation.query` 合并成最终 URL query。
  结果：`navigation.query` 中的 `@row.id` 不会被展开，会原样进入 URL。

示例：行内按钮跳插件页面并传当前记录 ID

```json
{
  "actionId": "replace-with-action-id",
  "displayName": "编辑",
  "orderIndex": 20,
  "layoutSlot": "row",
  "metadata": "{\"selectionScope\":\"row\",\"navigation\":{\"type\":\"custom-page\",\"routeName\":\"plugin-cost-card-create\",\"openMode\":\"router\",\"payload\":{\"id\":\"@row.id\",\"recordId\":\"@row.id\",\"mode\":\"edit\",\"sourceFeatureCode\":\"@context.featureCode\",\"sourceScenarioCode\":\"@context.scenarioCode\"}},\"ui\":{\"icon\":\"edit\",\"color\":\"primary\",\"variant\":\"flat\",\"size\":\"sm\"}}"
}
```

## 5. 更新行为说明

当前后端的更新粒度如下：

1. 实体有单独的新增字段、更新字段、删除字段接口。
2. 功能字段也有单独的新增、更新、删除接口。
3. 场景有单独的新增、更新接口，但场景内部大多数配置项不是细粒度 patch。
4. 更新场景时，`fieldGroups` 和 `actions` 更接近“整包覆盖”语义。

这意味着：

- 如果 AI 修改场景配置，应基于当前完整场景结构再更新，不要只凭想象发一小段残缺 JSON。
- 特别是 `actions`，传入的新数组会替换原场景动作集合。
- 如果目标是修改某个场景里某一个按钮的导航配置，应优先走 `system update-scenario`，而不是 `system update-action`。
  说明：`update-action` 更新的是动作定义；列表里某个按钮的 `metadata/navigation` 实际挂在 `ScenarioAction` 绑定上，属于场景配置的一部分。
- 使用 CLI 更新场景时，应按 `ScenarioRequest` 结构发送整包，推荐使用 `fieldGroups + actions`。
  不要直接把 `get-feature` 返回的场景 DTO 原样回写，因为返回结构与更新请求结构并不完全相同。

请求与回读字段对应关系：

| 更新请求 | `get-feature` 回读 | 说明 |
| --- | --- | --- |
| `fieldGroups[].featureFieldKey` | `scenario.fields[].featureFieldId` | 回读先用功能字段列表把 ID 映射回 `fieldKey` |
| `fieldGroups[].form/detail/list` | `scenario.fields[].metadata.contexts.*` | 视图配置在持久化后进入上下文 metadata |
| `metadata.formLayout` | `scenario.metadata.formLayout` | 标签页布局 |
| `metadata.detailTables` | `scenario.metadata.detailTables` | 明细表布局和列配置 |

回读验证不得直接在 `scenario.fields[]` 中查找 `fieldKey`；应先建立 `featureFieldId -> fieldKey` 映射，否则会误判更新没有保存。

## 6. 最重要的默认规则

1. 客户没有强调业务库时，不传 `dataSourceCode`。
2. 功能如果对应实体，应显式传 `entityCode`。
3. 默认情况下，创建功能会初始化 `list` 和 `detail` 场景。
4. 默认情况下，`list` 场景会附带 `create/edit/delete/view` 四个系统动作。
5. 如果客户明确说不要默认动作，传 `includeDefaultListActions = false`。
6. 已存在实体的数据查询和写入，不需要再传业务库，由服务端根据元数据自动解析。

## 7. 动作配置

### 7.1 `CreateActionRequest`

- `featureId`
  含义：所属功能 ID。
  必填：否。
  说明：功能内动作通常会绑定到某个 feature。

- `code`
  含义：动作编码。
  必填：是。
  示例：`approve_order`、`export_supplier`。

- `name`
  含义：动作显示名称。
  必填：是。

- `actionType`
  含义：动作类型。
  必填：是。
  常见值：
  - `create`
  - `update`
  - `delete`
  - `view`
  - `custom`

- `executionMode`
  含义：执行模式。
  必填：是。
  常见值：
  - `sync`
  - `async`
  - `page`
  - `api`
  - `workflow`
  说明：当前 CLI 简化输入里自定义动作默认补成 `sync`。

- `permissionCode`
  含义：权限点编码。
  必填：是。
  说明：如果不需要单独权限控制，可由 CLI 或调用方补空字符串。

- `description`
  含义：动作说明。
  必填：否。

- `isSystem`
  含义：是否系统动作。
  必填：否。
  默认：`false`。
  说明：普通业务动作不要设为 `true`。

- `metadata`
  含义：动作扩展元数据。
  必填：否。
  说明：动作本身的视觉风格、按钮属性等可放这里。

### 7.2 动作使用建议

1. `create / update / delete / view` 这类通用动作优先复用系统动作。
2. 业务定制行为再创建 `custom` 动作。
3. 场景里最终展示给用户的是 `ScenarioAction`，不是裸 `ActionDefinition`。

## 8. 菜单配置

### 8.1 `MenuEntryRequest`

- `label`
  含义：菜单显示名称。
  必填：是。

- `code`
  含义：菜单编码。
  必填：否。
  说明：建议显式填写，便于后续查找和 AI 操作。

- `menuType`
  含义：菜单类型。
  必填：是。
  默认：`group`。
  常见值：
  - `group`
  - `scenario`
  - `custom`
  - `report`

- `parentId`
  含义：父菜单 ID。
  必填：否。
  说明：为空时表示顶级菜单。

- `orderIndex`
  含义：菜单排序。
  必填：否。

- `icon`
  含义：菜单图标。
  必填：否。

- `featureId`
  含义：关联功能 ID。
  必填：否。
  说明：`scenario` 类型菜单通常应关联功能。

- `scenarioId`
  含义：关联场景 ID。
  必填：否。
  说明：`scenario` 类型菜单通常应同时关联场景。

- `customRoute`
  含义：自定义路由。
  必填：否。
  说明：`custom` 类型菜单使用。

- `externalUrl`
  含义：外部链接地址。
  必填：否。

- `reportCode`
  含义：报表编码。
  必填：否。

- `description`
  含义：菜单说明。
  必填：否。

- `metadata`
  含义：菜单扩展元数据。
  必填：否。

- `isActive`
  含义：是否启用。
  必填：否。
  默认：`true`。

### 8.2 菜单使用建议

1. 普通业务入口优先使用 `scenario` 类型菜单。
2. `scenario` 菜单应尽量同时绑定 `featureId` 和 `scenarioId`。
3. `group` 只负责分组，不直接承载业务页面。

## 9. 安全更新指南

适用于 AI 或自动化脚本修改已有实体、功能、场景时的规则。

### 9.1 更新实体

建议：

1. 新增字段优先使用单独的 `add-entity-fields` 能力。
2. 修改已有字段时，先读取当前实体定义，再修改目标字段。
3. 不要随意改动系统字段。
4. 删除普通字段使用 `system delete-entity-field --entity-code <code> --field-code <field>`；该操作会同步删除物理列，不能用于系统内置字段。
5. `metadata` 可以在 CLI 输入中写成 JSON 对象，CLI 会转为接口需要的 JSON 字符串。`length` 仅适用于 `string/varchar`，`precision` 仅适用于 `decimal/float/double`，`scale` 仅在已指定 decimal precision 时保留。

风险：

- 改字段类型、长度、可空性可能触发物理表结构变化。
- 同名字段或保留字段会导致保存失败。

### 9.2 更新功能

建议：

1. 先读取当前 feature。
2. 明确它是否已绑定实体。
3. 如果是实体型功能，确保 `dataSourceName` 最终仍指向正确实体编码。

风险：

- 误改 `dataSourceName` 会导致列表/详情页找不到实体。
- 误改 `dataSourceType` 会导致运行时按错误方式解析数据源。

### 9.3 更新场景

建议：

1. 先读取当前场景完整配置。
2. 在原配置基础上调整，而不是只发局部猜测字段。
3. 修改动作数组前，先保留需要继续存在的动作。
4. 修改字段配置前，确认 `fieldGroups` 中引用的 `featureFieldKey` 仍然存在。

风险：

- `actions` 更接近整包覆盖，遗漏现有动作会被移除。
- `fieldGroups` 更接近整包覆盖，遗漏字段配置会被移除。
- 误改 `scenarioType` 可能导致前端页面行为与配置不匹配。

### 9.4 推荐的 AI 更新流程

1. 先读取当前对象。
2. 理解现状。
3. 只修改目标字段。
4. 保留未明确要求变更的配置。
5. 提交更新。
6. 如涉及场景，更新后验证运行页是否可打开。
