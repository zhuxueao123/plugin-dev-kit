# 迁移硬规则

## 1. 迁移粒度

- 默认优先采用“按菜单目录迁移”
- 单功能迁移只适合样板验证
- 全量迁移适合最终整体切换

## 2. 目录迁移

- 目录迁移必须同时迁：
  - 父目录链
  - 目录下全部叶子功能
  - 每个叶子功能对应的场景菜单
- 不要只迁叶子菜单
- `menu not found` 通常表示菜单匹配策略问题，不应归因于网络

## 3. 导出策略

- 不要默认假设目录编码可以直接作为 `export-menu-subtree --menu-name`
- 优先传旧系统实际菜单名称
- 如果不稳定，先 `export-system` 再解析目录树和叶子功能

## 4. 命名保真

- 功能编码、实体编码、字段名必须与旧系统保持一致
- 默认不允许大小写转换
- 如果确实要映射，必须维护显式映射表并一并交付

## 5. 主从表

- `create-feature` 不会自动生成明细区
- 带子表的功能必须执行 `update-scenario`
- `detailTables` 必须显式配置 `entityCode`、关联键和列

## 6. 插件

- 保存前校验插件和业务动作插件必须分开处理
- 业务动作优先挂到旧系统对应列表或业务场景
- 后端插件应优先使用 `context.core.*`

## 7. 场景过滤

- 旧平台 block 的 `GFILTER/BFILTER` 必须随对应场景迁移，不能只迁字段级 `filterable`。
- 可转换条件写入 `ScenarioRequest.dataBinding.defaultFilter`；原始表达式同时保留在 `dataBinding.legacyFilter` 和 `metadata.legacyFilters.<blockCode>`。
- 新平台运行时不执行旧 SQL 片段。无法安全结构化的表达式必须标记为待人工处理，不能静默丢失或直接执行。
- 可用 `data query-records` 请求体中的 `featureCode`、`scenarioCode` 验证场景过滤；`pageIndex` 从 `0` 开始。

## 8. 业务数据与依赖

- 迁移样本或正式业务数据时，必须盘点字段 `optionSource` 指向的字典和关联实体；只迁主表会导致下拉/参照无法显示。
- 关联显示字段、快照字段应从对应主数据回填，不能仅导入编码。
- 需要保留旧编号时，使用 `data create-record --preserve-number-values`，避免已绑定的编号规则覆盖输入值。
- 当前 `legacy-export` 负责元数据和系统资料，不把业务记录导出混入功能 handoff；业务数据应走独立、可审计的只读提取与导入流程。
## 场景动作规则

- `list` 场景通常配置 `create`、`edit`、`delete`、`view`。
- `form/detail` 场景如果需要提交保存，必须显式配置 `save` 动作。
- 不要因为场景配置页里默认勾选了 `save` 就认为它已经生效；只有保存场景后，动作才会真正落库。
