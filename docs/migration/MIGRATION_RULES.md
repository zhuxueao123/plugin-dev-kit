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
- 带子表的功能必须通过 `update-scenario` 或 `patch-scenario` 写入场景明细表配置。
- 首次挂接完整场景可使用 `update-scenario`；后续只修改过滤、布局、字段组或动作时应使用 `patch-scenario`，避免覆盖未提交节点。
- 每次场景更新前平台只保存一份最近快照；误操作后使用 `restore-scenario-backup`，不维护多版本历史。
- `detailTables` 必须显式配置 `entityCode`、关联键和列
- 旧平台主维护场景即使属于 `list`，其中的 `formLayout/detailTables` 也要同步到新平台实际承载新建/编辑/查看的默认 `detail` 场景；优先使用导出物 `handoff/default_detail_scenario_patch_seed.json`。

## 6. 插件

- 保存前校验插件和业务动作插件必须分开处理
- 业务动作优先挂到旧系统对应列表或业务场景
- 后端插件应优先使用 `context.core.*`

## 7. 场景过滤

- 旧平台 block 的 `GFILTER/BFILTER` 必须随对应场景迁移，不能只迁字段级 `filterable`。
- 可转换条件同时写入 `ScenarioRequest.dataBinding.defaultFilter` 和 `metadata.search.defaultFilter`；原始表达式保留在 `dataBinding.legacyFilter` 和 `metadata.legacyFilters.<blockCode>.gfilter/bfilter`。
- 新平台运行时不执行旧 SQL 片段。无法安全结构化的表达式必须标记为待人工处理，不能静默丢失或直接执行。
- 可用 `data query-records` 请求体中的 `featureCode`、`scenarioCode` 验证场景过滤；`pageIndex` 从 `0` 开始。

## 8. 业务数据与依赖

- 迁移样本或正式业务数据时，必须盘点字段 `optionSource` 指向的字典和关联实体；只迁主表会导致下拉/参照无法显示。
- 关联显示字段、快照字段应从对应主数据回填，不能仅导入编码。
- 旧数据源 `DSTEXT/textFields` 表示关联源表的查询/展示字段；`DSVALUE/valueFields` 表示本地回填目标，不保证是源表字段。创建最小关联实体时只能采用源表字段目录中真实存在的字段。
- 需要保留旧编号时，使用 `data create-record --preserve-number-values`，避免已绑定的编号规则覆盖输入值。
- `data create-record` 的输入是扁平字段对象，不需要 `{ "data": ... }` 包装；`--preserve-number-values` 只控制本次编号规则应用。
- `legacy-export export-system` 和功能 handoff 不自动混入业务记录；需要旧平台业务数据时使用 `legacy-export query-data` 按 `pageIndex/pageSize` 走独立、可审计的只读提取流程。

## 9. 唯一性与物理索引

- `isUnique` 或索引元数据不是数据库约束本身；只有业务库中实际存在的唯一约束/唯一索引才能阻止重复写入。
- 新迁移会把旧平台 `Field.xml` 的 `UNIQUE` 标记转换为实体唯一索引。复合唯一索引必须根据旧平台索引定义显式配置，不能仅凭单字段标记推断。
- 迁移或发布后必须执行索引对账：

  ```bash
  asapflow system reconcile-entity-indexes --entity-code <实体编码>
  ```

- 默认命令只检查，不修改元数据或数据库。确认报告后使用 `--apply` 同步结构：缺少物理索引时补建；物理索引已存在但缺少元数据时只回填元数据，不重复执行 DDL：

  ```bash
  asapflow system reconcile-entity-indexes --entity-code <实体编码> --apply
  ```

- `metadata_missing` 表示物理索引已存在但平台元数据缺失，`--apply` 后应变为 `metadata_imported`。
- `failed` 会返回数据库原始错误。它可能表示重复数据，也可能是 `schemaName/physicalName/dataSourceCode` 指向了错误对象；不要仅凭通用失败提示判断为重复键。
- `mismatch` 表示同名物理索引的字段、顺序或唯一性与元数据不同；平台不会自动覆盖。
- `SchemaManaged=false` 的实体由数据库管理员维护物理结构。平台会拒绝在线新增索引，不能把“元数据保存成功”当作索引创建成功。
- `add-entity-index` 用于新增索引；迁移后接管数据库既有索引必须使用 `reconcile-entity-indexes --apply`。

## 场景动作规则

- `list` 场景通常配置 `create`、`edit`、`delete`、`view`。
- `form/detail` 场景如果需要提交保存，必须显式配置 `save` 动作。
- 不要因为场景配置页里默认勾选了 `save` 就认为它已经生效；只有保存场景后，动作才会真正落库。
