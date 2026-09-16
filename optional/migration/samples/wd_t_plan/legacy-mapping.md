# WD_T_PLAN Legacy Mapping

## 1. 旧系统功能

- `WD_T_PLAN`
  - 名称：`1. 配送计划`
  - 父菜单：`WD_TRAN`
- 主块：
  - `BMAINLIST`
  - `BMAINBLOCK`
- 明细块：
  - `BPLAND`
  - 对应表：`WD_T_PLAND`

## 2. 旧系统表

### `WD_T_PLAN`

- 中文名：`配送计划`
- 主键：`ID_PLAN`
- 主要字段：
  - `DESC_PLAN`
  - `F_DEFAULT`
  - `DATE_EFF`
  - `DATE_EXP`
  - `F_ENABLED`
  - `TIME_COST`
  - `TIME_START`
  - `ROUND`
  - `BREAK_START`
  - `BREAK_END`

### `WD_T_PLAND`

- 中文名：`配送计划明细`
- 组合主键：
  - `ID_PLAN`
  - `LINE_PLAND`
- 主要字段：
  - `CODE_SITE`
  - `DESC_SITE`
  - `CODE_LINE`
  - `DESC_LINE`
  - `TIME_START`
  - `TIME_END`
  - `ROUND`

## 3. 旧系统按钮

来自 `raw/buttons.json`：

- `BTNCALC`
  - 中文名：`计算时间`
  - 所在块：`BMAINBLOCK`
- `BTNSELECT`
  - 中文名：`选择站点`
  - 所在块：`BPLAND`

建议的新系统动作映射：

- `calc_time`
  - 对应 `BTNCALC`
  - 倾向实现为详情页动作或表单内服务动作
- `select_site`
  - 对应 `BTNSELECT`
  - 倾向实现为明细行辅助动作或弹窗选择逻辑

## 4. 当前缺口

旧系统导出器当前能把 `raw` 导出来，但下面这些对象还没有正确标准化：

- `featureCode`
- `mainEntityCode`
- `scenario code / scenario name`
- `feature fields`
- `action code`

因此这条样板迁移当前仍需要人工补充：

1. 新系统 feature code
2. list/detail 场景的显示字段
3. 明细表在新系统的承载方式
4. `BTNCALC` 和 `BTNSELECT` 的动作执行逻辑

## 5. 下一步建议

1. 先用 `wd_t_plan.entity.json` 和 `wd_t_plan.feature.json` 在新系统创建主对象
2. 再补 `wd_t_pland`，决定是作为子实体还是明细表配置
3. 再把 `计算时间`、`选择站点` 重写为新系统动作/插件
4. 最后补菜单、编号规则和权限
