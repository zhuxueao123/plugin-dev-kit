# AsapFlow CLI 使用指南

本文档面向插件开发人员及其 AI 助手。

目标：

- 帮助插件开发人员配置和使用 `asapflow` CLI
- 说明 CLI 可以调用的平台能力及安全边界
- 提供可直接参考的最小 JSON 示例
- 让 AI 助手依据同一套规则协助开发

本目录不包含内部实现细节，不包含数据库连接配置说明，也不要求 AI 理解平台内部架构。

## 目录说明

- `README.md`
  入口说明
- `capabilities.md`
  能力清单与调用原则
- `usage.md`
  CLI 使用方式、配置文件和输出约定
- `modeling-reference.md`
  实体、功能、场景配置项说明
- `bi-dashboard-reference.md`
  BI 看板、查询类型、通用组件和未实现能力说明
- `examples/`
  每个能力的最小 JSON 输入样例；先阅读 `examples/README.md` 确认前置依赖
- `plugin-docs.md`
  当需要 AI 生成插件时应补充给 AI 的说明

## CLI 使用原则

1. 只通过 `asapflow` CLI 调用系统能力。
2. 不假设数据库类型、连接字符串或业务库位置。
3. 已存在实体的数据读写只面向 `entityCode`。
4. CLI 使用的 Service Token 默认继承创建者当前权限与数据范围，AI 不应假设 token 拥有额外授权。
5. `feature` 是功能，`scenario` 是功能下的场景，`action` 是场景中的动作，不要混用这三个概念。
6. 创建功能时，如果对应已有实体，应显式传 `entityCode`。
7. 对于实体来源字段，统一使用实体原始字段名，例如 `contact_person`，不要再造 `contactPerson` 这类别名。
8. 创建功能时，默认会初始化 `list/detail` 场景；默认列表场景会附带 `create/edit/delete/view` 四个系统动作。
9. 如果客户明确要求不要默认系统动作，应在输入中传 `includeDefaultListActions = false`。
10. 插件开发发生在本地工作目录，不直接修改生产服务器源码。
11. 如果一个能力已有示例输入，优先复用示例结构。
12. 在 Windows PowerShell 中，如果 `asapflow.exe` 与当前工作目录相同，应使用 `.\asapflow.exe` 调用。
13. 如果 AI 工具无法稳定捕获 Windows `.exe` 的标准输出，优先调用 `.\asapflow.ps1`，不要默认把 `result.json` 写到插件工作区。
14. 如果同时提供 `asapflow.ps1` 或 `asapflow.sh`，AI 应优先调用包装脚本，让输出落到系统临时目录。
15. 临时请求数据优先使用 `--input -` 通过标准输入传入，避免在插件工作区创建一次性 JSON 文件。
16. 表单场景不是只要能从列表页“新建/编辑/查看”跳进去就算完整；如果需要真正保存数据，必须为该场景显式配置 `save` 动作。
17. 给已有 feature 补字段优先使用 `system add-feature-fields`；若使用 `system update-feature` 并包含 `fields`，CLI 也会按 `fieldKey` 同步新增/更新。
18. `workflow list-workbench` 支持 `todo`、`started`、`done`、`cc`、`finished` 五类视图。
19. `workflow start-instance` 支持在 JSON 中传 `ccUserIds`，也支持命令行多次传 `--cc-user-id`。
20. `workflow get-instance` 返回的是流程审批上下文，不应当把它当作绕过普通实体详情权限的通道。
21. 管理员可用 `identity get-password-policy` / `identity set-password-policy` 维护密码强度与失效期限。
22. 修改 BI 看板前先读取当前完整草稿；CLI 的 `bi update` 是整包更新。
23. BI 查询支持 SQL、插件接口和静态数据；SQL 必须参数化，且生产数据源必须使用数据库只读账号和最小权限。
24. BI 主页绑定使用 `bi save-homepage-binding`；多标签、部门继承和个人副本尚未实现，不要写进看板 `definition` JSON。
25. 创建或更新表单类场景时，如果字段很多，或旧平台字段已经按块/分组组织，应在场景 `metadata.formLayout` 中配置 `type = tabs`，不要让所有字段平铺在表单页。

## 建议先给 AI 的文件

如果目标是“让 AI 修改系统结构或使用系统数据”，建议先给 AI：

1. `README.md`
2. `usage.md`
3. `capabilities.md`
4. `modeling-reference.md`
5. `examples/` 目录下的相关 JSON
6. 如果目标包含 BI 看板，再补充 `bi-dashboard-reference.md`

如果目标包含“修改已有实体/功能/场景”，`modeling-reference.md` 里的“安全更新指南”也应一并提供给 AI。
同时建议补充：

- `examples/update_feature_reference.json`
- `examples/update_scenario_reference.json`
- `examples/update_detail_scenario_tabs.json`

如果目标是“让 AI 生成插件”，再额外给 AI：

1. `plugin-docs.md`
2. 仓库中的插件规范文档
3. `plugin-workspace/locales/*.json`，让 AI 了解当前统一语言资源目录的命名空间与 key

## 典型任务

1. 创建实体
2. 为实体补字段
3. 更新实体与实体字段
4. 创建功能并初始化默认场景
5. 更新功能定义
6. 创建或更新额外场景
7. 创建或更新动作
8. 创建或更新菜单
9. 创建角色、用户并分配权限
10. 创建编号规则并绑定到实体字段
11. 创建并发布流程定义
12. 发起与处理流程实例
13. 查询某个实体的数据
14. 新增、修改、删除记录
15. 初始化并校验插件骨架
16. 创建、测试并发布 BI 看板
