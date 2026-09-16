# WD_T_DO Migration Sample

这个目录是基于旧系统 `站点配送` 模块中的 `WD_T_DO` / `WD_T_DOD` 导出结果整理出的第二条新系统样板迁移输入。

这条样板的目标不是一次性还原所有状态场景和按钮，而是验证下面这条链路是否可以稳定复用：

- 主表实体
- 明细表实体
- 功能
- 默认 `list` / `detail` 场景
- `detail` 场景显式挂接明细表

## 来源

- 旧系统功能包：
  - `WD_T_DO`
- 旧系统系统级元数据导出：
  - `sys_table`
  - `sys_tablefield`
  - `v_sys_groupblock`
  - `v_sys_groupblockfield`
  - `v_sys_groupbutton`

## 当前结论

`legacy-export export-function` 对这套旧库在 `WD_T_DO` 上同样不能直接用于迁移编排：

- `normalized/feature.json` 为空
- `normalized/entities.json` 为空
- `handoff/summary.md` 基本不可用

因此这条样板继续采用系统级元数据回填：

1. 用 `sys_table` / `sys_tablefield` 提取 `WD_T_DO` 与 `WD_T_DOD`
2. 用 `v_sys_groupblock` 确认明细块 `BDOD`
3. 用 `PKLINK` 确认主从关联 `CODE_DO=CODE_DO`
4. 手工补齐新系统 CLI 输入

## 文件说明

- `wd_t_do.entity.json`
  - 主表 `WD_T_DO` 的新系统实体草稿
- `wd_t_dod.entity.json`
  - 明细表 `WD_T_DOD` 的新系统实体草稿
- `wd_t_do.feature.json`
  - `配送单` 功能草稿
- `wd_t_do.detail-scenario.update.json`
  - 把 `WD_T_DOD` 挂到 `wd_t_do` 的 `detail` 场景

## 这条样板验证的重点

- 主表标识从 `ID` 类字段切换到业务单号 `CODE_DO` 时，新系统能否正常承接
- 明细表关联键使用业务单号 `CODE_DO` 时，表单页能否正常展示子表
- `create-feature` 后再执行 `update-scenario` 挂接明细表，这个迁移模式是否能稳定复用
