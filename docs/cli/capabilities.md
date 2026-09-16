# AsapFlow CLI 能力清单

## 1. 认证与环境

- `auth whoami`
  用途：确认当前 token 是否有效、当前身份是谁
- `auth health-check`
  用途：确认 CLI 是否能访问平台 API
- `auth use-token`
  用途：写入本地配置文件
- `auth login`
  用途：用用户名密码换取临时 JWT，仅作为辅助能力

## 2. 系统结构能力

- 术语说明：
  - `feature` = 功能，是一个可被菜单、场景和动作消费的业务功能单元
  - `scenario` = 场景，是功能下的页面/视图配置，例如 `list`、`detail`
  - `action` = 动作，是场景中的按钮或操作，不等于场景本身

- `system create-entity`
  用途：创建实体与初始字段
- `system list-entities`
  用途：列出实体定义；可用 `--limit <数量>` 限制返回条数
- `system add-entity-fields`
  用途：为已有实体补充字段
- `system get-entity`
  用途：查询实体定义
- `system update-entity`
  用途：更新实体定义
- `system delete-entity`
  用途：按 `--entity-id` 或 `--code` 删除实体定义
- `system update-entity-field`
  用途：更新实体字段定义
- `system delete-entity-field`
  用途：按 `--entity-code` 与 `--field-id`/`--field-code` 删除实体字段；会同步修改物理表，使用前必须确认目标
- `system create-feature`
  用途：创建功能并初始化默认场景
- `system list-features`
  用途：列出功能定义；可用 `--limit <数量>` 限制返回条数
- `system get-feature`
  用途：查询功能定义
- `system add-feature-fields`
  用途：为已有功能增量补充字段（按 `fieldKey` 自动新增或更新）
- `system update-feature`
  用途：更新功能定义；当输入包含 `fields` 时会同步 feature 字段（按 `fieldKey` 自动新增或更新）
- `system delete-feature`
  用途：按 `--feature-id` 或 `--code` 删除功能定义
- `system create-scenario`
  用途：为功能创建额外场景
- `system update-scenario`
  用途：整包更新指定场景
- `system delete-scenario`
  用途：删除指定功能下的场景
- `system create-action`
  用途：为功能创建动作
- `system list-actions`
  用途：列出动作，可按 feature 过滤
- `system update-action`
  用途：更新动作定义
- `system delete-action`
  用途：删除动作定义
- `system create-menu`
  用途：创建菜单
- `system list-menus`
  用途：列出菜单树
- `system update-menu`
  用途：更新菜单
- `system delete-menu`
  用途：删除菜单
- `system create-action-menu`
  用途：一次性创建动作与菜单
- `workflow create-definition`
  用途：创建流程定义草稿
- `workflow update-definition`
  用途：更新流程定义草稿
- `workflow get-definition`
  用途：读取流程定义与已发布版本
- `workflow publish-definition`
  用途：发布流程定义版本
- `workflow list-workbench`
  用途：查询待我处理、我发起、我已处理、抄送我的、已结束等流程视图
- `workflow start-instance`
  用途：在某个业务对象上发起流程实例，支持显式抄送 `ccUserIds`
- `workflow get-instance`
  用途：读取流程实例审批上下文、可执行动作、字段权限与历史
- `workflow execute-task-action`
  用途：执行当前任务动作，例如同意、驳回、确认发货
- `number-rule create`
  用途：创建编号规则
- `number-rule update`
  用途：更新编号规则
- `number-rule get`
  用途：按 `id` 或 `code` 读取编号规则
- `number-rule list`
  用途：查询全部编号规则
- `number-rule preview`
  用途：预览编号生成结果，不占用流水号
- `number-rule bind-field`
  用途：把编号规则绑定到实体字段 metadata.generator
- `workflow list-definitions`
  用途：列出流程定义
- `workflow delete-definition`
  用途：删除未产生实例的流程定义

## 3. 身份与权限能力

- `identity list-permissions`
  用途：列出平台权限
- `identity list-roles`
  用途：列出角色
- `identity create-role`
  用途：创建角色并绑定权限
- `identity update-role`
  用途：更新角色及权限
- `identity delete-role`
  用途：删除角色
- `identity list-users`
  用途：列出用户
- `identity create-user`
  用途：创建用户
- `identity assign-user-roles`
  用途：为用户分配角色
- `identity get-user-permissions`
  用途：查看某个用户的最终权限
- `identity get-password-policy`
  用途：读取当前密码强度和失效期限策略
- `identity set-password-policy`
  用途：保存密码强度和失效期限策略

## 4. 系统使用能力

- `data query-records`
  用途：分页查询实体数据
- `data get-record`
  用途：读取单条记录
- `data create-record`
  用途：新增记录
- `data update-record`
  用途：更新记录
- `data delete-record`
  用途：删除记录
- `data execute-action`
  用途：执行某个实体动作

## 5. 配置与运维能力

- `configuration list`
  用途：列出系统配置，可通过 `--prefix` 过滤
- `configuration set`
  用途：保存系统配置项
- `configuration delete`
  用途：删除系统配置项
- `configuration get-file-storage`
  用途：读取文件存储配置
- `configuration set-file-storage`
  用途：保存文件存储配置
- `data-source list`
  用途：列出业务数据源
- `data-source get`
  用途：读取业务数据源
- `data-source validate`
  用途：校验数据源连接
- `data-source create`
  用途：创建业务数据源
- `data-source update`
  用途：更新业务数据源
- `data-source delete`
  用途：删除业务数据源
- `dictionary list`
  用途：列出系统字典
- `dictionary get`
  用途：读取系统字典
- `dictionary create`
  用途：创建系统字典
- `dictionary update`
  用途：更新系统字典
- `dictionary delete`
  用途：删除系统字典
- `scheduled-job list`
  用途：列出计划任务
- `scheduled-job get`
  用途：读取计划任务详情
- `scheduled-job create`
  用途：创建计划任务，任务目标为插件能力
- `scheduled-job update`
  用途：更新计划任务
- `scheduled-job delete`
  用途：删除计划任务
- `scheduled-job run`
  用途：立即执行一次计划任务
- `scheduled-job runs`
  用途：查询计划任务运行记录
- `import-export config`
  用途：读取功能场景的导入导出配置
- `import-export template`
  用途：下载导入模板
- `import-export create-import-task`
  用途：上传文件并创建导入任务
- `import-export create-export-task`
  用途：创建导出任务
- `import-export list-tasks`
  用途：查询当前用户的导入导出任务
- `import-export download`
  用途：下载导入导出任务结果文件
- `feature-package export`
  用途：导出功能包 ZIP
- `feature-package import-preview`
  用途：预览功能包导入影响
- `feature-package import`
  用途：导入功能包
- `feature-package legacy-preview`
  用途：预览旧平台包迁移结果
- `feature-package legacy-convert`
  用途：把旧平台包转换为新平台功能包 ZIP
- `feature-package legacy-import`
  用途：转换并导入旧平台包

## 6. 流程能力输入原则

1. 流程定义使用结构化 JSON，不再使用 Visio 导入
2. `businessObjectType` 必须稳定，例如 `order`、`purchase_request`
3. 流程定义与场景解耦，不为每个节点单独创建场景
4. 如果流程最终需要跳转到业务页面，应显式传 `featureCode` 与 `scenarioCode`
5. 节点字段权限统一写在 `fieldPermissions`
6. AI 生成流程时，第一版优先使用：
   - `start`
   - `approval`
   - `task`
   - `service`
   - `end`
7. AI 修改已有流程前，应先读取当前定义，再做增量更新

## 7. 插件能力

### 本地工程能力

- `plugin init`
  用途：初始化插件骨架
- `plugin validate`
  用途：校验插件目录与 manifest
- `plugin manifest-check`
  用途：校验单个 manifest 文件
- `plugin build-frontend`
  用途：构建前端插件。默认使用 `plugin-workspace/frontend`，也可用 `--working-dir` 明确指定前端工程目录

### 服务端运行能力

- `plugin list`
  用途：查看当前平台识别到的插件能力
- `plugin reload`
  用途：通知平台重载插件
- `plugin pages`
  用途：查看当前平台识别到的插件页面
- `plugin invoke`
  用途：手动调用某个插件能力
- `plugin pack`
  用途：在本地构建并生成 `.afplugin` 发布制品
- `plugin publish`
  用途：上传制品，并可等待后台发布完成
- `plugin release-status`
  用途：查询指定发布任务的状态和日志
- `plugin releases`
  用途：查询插件发布历史
- `plugin rollback`
  用途：回滚到平台保留的成功版本

## 8. BI 看板能力

- `bi list`
  用途：列出可管理的 BI 看板
- `bi get`
  用途：读取看板草稿完整定义和发布版本摘要
- `bi create`
  用途：通过完整 JSON 创建看板草稿
- `bi update`
  用途：整包更新看板草稿
- `bi delete`
  用途：删除看板及其历史版本
- `bi publish`
  用途：校验全部 SQL 并发布不可变版本
- `bi execute-query`
  用途：执行草稿查询；增加 `--published` 后执行已发布查询
- `bi list-published`
  用途：列出当前用户可查看的已发布看板
- `bi get-published`
  用途：读取已发布且已移除 SQL 的运行时定义
- `bi homepage`
  用途：解析当前登录用户的 BI 个人主页
- `bi list-homepage-bindings`
  用途：列出用户、部门、角色和全局主页绑定
- `bi save-homepage-binding`
  用途：创建或覆盖一个主页绑定，输入为完整 JSON
- `bi delete-homepage-binding`
  用途：删除主页绑定

当前 BI 查询支持 SQL、插件接口和静态 JSON 数据；SQL 复用平台现有数据源。个人主页已支持用户、主部门、主角色和全局默认的单看板解析；多看板标签页、部门继承和个人自定义副本尚未实现。详细协议见 `bi-dashboard-reference.md`。

## 9. AI 调用原则

1. 优先使用最贴近目标的能力，不自行拆底层 API
2. 已有实体的数据操作只使用 `entityCode`
3. 没有必要时，不显式传 `dataSourceCode`
4. `system create-feature` 在包含字段时默认会初始化 `list` 和 `detail` 两个基础场景；`fields=[]` 时默认跳过初始化
5. 如果未显式说明，默认列表场景会附带四个系统动作：`create`、`edit`、`delete`、`view`
6. 如果客户明确要求“不带默认系统动作”，应在创建 feature 时传 `includeDefaultListActions = false`
7. 插件相关的源码生成只发生在本地工作目录
8. AI 直接调用 `auth/system/data/plugin` 分组命令，不再使用额外的 `run` 别名层
9. 流程创建优先调用 `workflow create-definition`，不要绕过成手工调用后端 REST
10. 流程发布后，如果需要测试，应优先调用 `workflow start-instance`
11. 编号规则优先调用 `number-rule create` 创建，再通过实体字段 `metadata.generator` 绑定到业务字段
12. 迁移已有对象时，优先使用 `list/get/update` 组合；如果只需要跳过已存在实体或功能，可对 `system create-entity` / `system create-feature` 使用 `--if-exists skip`
13. 生成表单场景时应显式选择合适控件，控件清单见 [modeling-reference.md](modeling-reference.md)；不要默认把所有字段都配置成 `q-input` 或 `q-number`
14. 生成表单场景时如果字段较多，或旧平台存在字段块/分组，应配置 `metadata.formLayout.type = "tabs"`，让运行页按标签页展示字段
15. 修改 BI 看板前先执行 `bi get`；`bi update` 是完整草稿更新，必须保留未修改的查询和组件
16. BI SQL 必须参数化，CLI 不在本地执行 SQL，数据库识别与安全校验由后端完成
17. BI 主页绑定使用 `bi save-homepage-binding`，不要写入看板 `definition` JSON
