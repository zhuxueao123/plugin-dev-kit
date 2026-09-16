# 插件开发关联的平台能力指南

本文面向在 `plugin-workspace/` 中开发插件的开发人员及其 AI 助手。

目的只有一个：当需求看起来像“要写插件”时，先判断哪些能力其实应直接使用平台标准能力，而不是在插件里重复实现。

## 1. 先判断是否真的需要插件

以下需求优先考虑平台标准能力，不要先写插件：

1. 有限枚举值下拉，例如状态、类型、分类、等级、优先级。
2. 自动生成业务编号，例如单据号、物料编码、客户编码。
3. 定时执行任务，例如每天同步、每小时轮询、夜间批处理。

以下需求才优先考虑插件：

1. 保存前复杂校验。
2. 提交后调用外部系统。
3. 特殊聚合计算或衍生逻辑。
4. 自定义页面、看板、监控界面。
5. 定时任务目标本身需要调用插件能力。

## 2. 系统字典

### 2.1 什么时候该用

字段是固定选项集时，优先使用系统字典：

1. `status`
2. `type`
3. `category`
4. `level`
5. `priority`

不要在插件前端长期硬编码这些选项。

### 2.2 字段配置写法

推荐形态：

```json
{
  "fieldKey": "status",
  "form": {
    "component": "q-select",
    "optionSourceType": "dictionary",
    "optionSource": {
      "dictionaryCode": "order_status"
    }
  }
}
```

### 2.3 推荐操作顺序

1. `dictionary get` / `dictionary list` 先确认是否已存在。
2. 不存在时执行 `dictionary create`。
3. 再更新 feature/scenario 字段配置引用该字典。

### 2.4 CLI 命令形态

```bash
asapflow dictionary list
asapflow dictionary get --code order_status
asapflow dictionary create --input create_dictionary.json
asapflow dictionary update --code order_status --input update_dictionary.json
asapflow dictionary delete --code order_status
```

## 3. 编号规则

### 3.1 什么时候该用

需要系统自动生成编号时，优先使用编号规则：

1. 采购单号
2. 报价单号
3. 成本卡编号
4. 客户编码
5. 供应商编码

不要在插件前端手工拼接编号。

### 3.2 推荐操作顺序

1. `number-rule get` / `number-rule list` 先确认规则是否已存在。
2. 不存在时执行 `number-rule create`。
3. 用 `number-rule preview` 检查格式。
4. 用 `number-rule bind-field`，或在实体字段 `metadata.generator.ruleCode` 中绑定。
5. 再确认功能页面是否展示该字段，但不要让页面承担编号生成职责。

### 3.3 CLI 命令形态

```bash
asapflow number-rule list
asapflow number-rule get --code sales_order_no
asapflow number-rule create --input create_number_rule.json
asapflow number-rule update --rule-id <rule-id> --input update_number_rule.json
asapflow number-rule preview --input preview_number_rule.json
asapflow number-rule bind-field --entity-code sales_order --field-code order_no --rule-code sales_order_no
```

### 3.4 边界提醒

1. 当前不要假设存在 `number-rule delete` CLI 命令。
2. 不要把编号规则理解成页面动作。
3. 不要把前缀、日期格式、补位规则硬编码进插件页面。

## 4. 计划任务

### 4.1 什么时候该用

需求里出现以下关键词时，优先考虑计划任务：

1. 每天同步
2. 每小时检查
3. 夜间执行
4. 定时回写
5. 批处理

不要先建议外部 cron，也不要先做前端轮询。

### 4.2 推荐操作顺序

1. `scheduled-job list` 看是否已有同类任务。
2. `scheduled-job get` 读取已有任务详情。
3. 新建或修改任务。
4. 用 `scheduled-job run` 立即执行一次验证。
5. 用 `scheduled-job runs` 查看运行记录。

### 4.3 CLI 命令形态

```bash
asapflow scheduled-job list
asapflow scheduled-job get --job-id <job-id>
asapflow scheduled-job create --input create_scheduled_job.json
asapflow scheduled-job update --job-id <job-id> --input update_scheduled_job.json
asapflow scheduled-job delete --job-id <job-id>
asapflow scheduled-job run --job-id <job-id>
asapflow scheduled-job runs --job-id <job-id>
```

### 4.4 边界提醒

1. 定时执行不是 workflow。
2. 创建后至少做一次手动运行或查看运行记录。
3. 如果任务目标是插件能力，先确认后端插件接口已经可调用。

## 5. 标准表单报错时的判断顺序

如果标准功能页面保存失败，不要先怀疑插件页跳转。优先检查：

1. 是否有必填字段未提交。
2. 是否把本应由编号规则生成的字段当成普通字段处理了。
3. 是否存在系统行号、排序号等字段未提供值且无默认值。
4. 是否有保存前插件钩子拦截了标准保存。

## 6. 与插件文档的关系

配合以下文档一起使用：

1. [frontend-plugin-guide.md](frontend-plugin-guide.md)
2. [backend-plugin-guide.md](backend-plugin-guide.md)
3. [core-sdk-reference.md](core-sdk-reference.md)
4. [plugin-e2e-examples.md](plugin-e2e-examples.md)
