# AD_T_WPLAN Migration Sample

这个目录用于把旧系统 `安灯 -> 作业 -> 计划导入与维护` 迁成新系统中的第一条“带动作插件”的真实样板。

旧系统信息：

- 功能：`AD_T_WPLAN`
- 主表：`AD_T_WPLAN`
- 明细表：`AD_T_WPLAND`
- 草稿场景按钮：`BTNRELEASE`
- 旧系统事件入口：`TK.AD_T_WPLAN.Release`

## 这条样板验证的重点

- 主表 + 明细表迁移
- 多状态列表场景迁移
- 旧系统 `下达` 按钮迁成新系统自定义动作
- 自定义动作绑定后端插件能力

## 文件说明

- `ad_t_wplan.entity.json`
  - 主表实体
- `ad_t_wpland.entity.json`
  - 明细表实体
- `ad_t_wplan.feature.json`
  - 功能定义
- `ad_t_wplan.detail-scenario.update.json`
  - 把 `AD_T_WPLAND` 挂到 detail 场景
- `draft_list_scenario.json`
  - 草稿列表场景，默认过滤 `status = NDRF`
- `release.action.json`
  - `下达` 自定义动作定义
- `release-scenario.update.json`
  - 把 `release` 动作挂到 `draft` 列表场景
- `ad_t_wplan.create-sample.json`
  - 创建草稿计划样例
- `release.execute-sample.json`
  - 执行 `release` 动作的样例入参

## 当前迁移策略

旧系统 exporter 在这个功能上依然无法产出可直接编排的 `normalized/handoff`，因此仍采用系统级元数据回填：

1. 用 `sys_table / sys_tablefield` 提取主从表
2. 用 `v_sys_groupblock` 识别多状态列表和明细块
3. 用 `raw/events.json` 与 `v_sys_groupbutton.json` 识别 `BTNRELEASE -> TK.AD_T_WPLAN.Release`
4. 在新系统中改写成 `action + plugin`

## 已验证结果

- `detail` 场景已挂接 `AD_T_WPLAND`
- `draft` 列表场景已挂接 `release` 动作
- `ad_t_wplan.create-sample.json` 已验证主表 + 2 条明细可一起保存
- `ad_wplan_release.release_plan` 已实际执行成功
- 草稿记录 `NDRF -> DREL` 的状态切换已验证通过
- 故意构造坏明细后，主表不会残留，事务回滚已验证通过

## 迁移规则

- `下达` 这类旧系统业务按钮，不要挂到 `detail` 场景
- 当前平台会在 `create/edit` 时执行 `scenarioMappings` 指向场景中的 `pluginBinding`
- 如果把 `release` 挂到 `detail`，普通保存也会误触发该插件
- 因此这类动作应优先挂到旧系统对应的业务场景，这里是 `draft` 列表场景

## 当前状态

- 当前已验证的是主从保存 + `draft` 场景执行 `release` 插件两条链路
- 旧系统多状态场景还只迁了 `draft` 一个业务场景，`released/completed/closed` 还没继续补
