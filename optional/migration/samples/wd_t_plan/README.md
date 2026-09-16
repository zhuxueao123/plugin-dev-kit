# WD_T_PLAN Migration Sample

这个目录是基于旧系统 `站点配送` 模块中的 `WD_T_PLAN` / `WD_T_PLAND` 导出结果整理出的第一条新系统样板迁移输入。

当前目标不是一次性完全自动迁移，而是先把下面几类信息固化成可复用输入：

- 新系统实体草稿
- 新系统功能草稿
- 旧系统按钮到新系统动作的初步映射
- 旧系统导出器当前识别缺口

## 来源

- 旧系统功能包：
  - `WD_T_PLAN`
  - `WD_T_DO`
- 旧系统系统级元数据导出：
  - `sys_table`
  - `sys_tablefield`
  - `v_sys_groupblock`
  - `v_sys_groupblockfield`
  - `sys_function`

## 当前结论

`legacy-export export-function` 对这套旧库可以导出 `raw/`，但 `normalized/` 和 `handoff/` 里的 `featureCode / entityCode / scenarioCode` 基本没有被正确识别出来。

因此这条样板迁移暂时采用：

1. 用 `raw/` 识别功能结构
2. 用 `export-system` 的元数据识别实体、字段和子表
3. 手工补一层新系统 CLI 输入草稿

## 文件说明

- `wd_t_plan.entity.json`
  - 主表 `WD_T_PLAN` 的新系统实体草稿
- `wd_t_pland.entity.json`
  - 明细表 `WD_T_PLAND` 的新系统实体草稿
- `wd_t_plan.feature.json`
  - `配送计划` 功能草稿
- `wd_t_plan.menu.json`
  - `配送计划` 列表菜单草稿
- `calc_time.action.json`
  - 对应旧系统 `BTNCALC`
- `select_site.action.json`
  - 对应旧系统 `BTNSELECT`
- `legacy-mapping.md`
  - 旧系统字段、块、按钮和新系统对象的映射说明
- `wd_t_plan.detail-scenario.update.json`
  - 把 `WD_T_PLAND` 挂到 `wd_t_plan` 的 `detail` 场景
  - 这是这条样板里非常关键的一步，不能省略

## 这次样板暴露出的关键规则

第一次迁移时，`配送计划明细` 没出现在新系统表单页，不是因为子表实体没建成功，而是因为主功能的 `detail` 场景没有配置 `metadata.detailTables`。

对 AI 来说，这里很容易犯一个错误：

- 看到主表实体和子表实体都已经创建，就误以为表单页会自动出现明细区

实际上新系统不是这样工作的：

1. `create-feature` 只会初始化默认 `list` / `detail` 场景
2. 不会自动从实体关系里推断明细表
3. 主从表功能必须再执行一次 `system update-scenario`
4. 在 `detail` 场景的 `metadata.detailTables` 里显式写入：
   - 子表 `entityCode`
   - 主从关联 `parentKey / childKey`
   - 明细列 `columns`

这条规则后面迁移 `WD_T_DO` 或其他带子表的旧功能时都必须复用。

## 当前假设

- 新系统 `module` 暂定为 `wd`
- 功能编码暂定使用 `wd_t_plan`
- `TIME_START / TIME_END / BREAK_START / BREAK_END` 先按 `string` 迁移
  - 原因：旧系统字段长度为 `5`，更像 `HH:mm` 文本时间，而不是完整时间戳
- `F_DEFAULT / F_ENABLED` 先按 `boolean` 迁移
- `ROUND / TIME_COST / LINE_PLAND` 先按 `integer` 迁移

这些假设在开始真正导入前还需要你确认一次。
