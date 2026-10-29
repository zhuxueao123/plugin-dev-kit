# AI 插件开发操作指南

本文件是 AI 的统一入口。先识别任务类型，只读取对应材料，避免把插件开发和旧系统迁移混在一起。

## 普通插件开发

依次读取：

1. `docs/README.md`
2. `docs/plugin-development/overview.md`
3. `docs/plugin-development/backend-plugin-guide.md` 或 `frontend-plugin-guide.md`
4. `docs/plugin-development/release.md`
5. `plugin-workspace/examples/` 中最接近的示例

允许在 `plugin-workspace/backend/`、`plugin-workspace/frontend/` 和 `plugin-workspace/locales/` 内生成代码。使用 `plugin validate`、`plugin pack`、`plugin publish` 完成校验与发布。

## 自定义登录页

优先读取 `docs/plugin-development/login-page-guide.md` 和 `release.md`。从 `plugin-workspace/examples/frontend/customer-brand/` 复制到正式前端工程，再按用户设计要求修改。使用 `purpose: "login"` 与 `loginContext`，保留当前 CLI 所需的兼容 route；不注册业务菜单、不重写认证、不直接导入宿主源码。更新构建器与文档必须一起获取，完成 `plugin-build.*` 验证与制品检查后报告结果；只有用户明确要求发布时才运行 `plugin-publish.*`。

## 平台配置任务

读取 `docs/cli/README.md`、`docs/cli/usage.md`、`docs/cli/capabilities.md` 及相关 JSON 示例。只通过 `asapflow` CLI 调用平台能力，不直接操作数据库。

## 旧系统迁移任务

只有明确要求迁移旧系统时，才额外读取：

1. `docs/migration/MIGRATION_RULES.md`
2. `optional/migration/legacy-export/README.md`
3. `optional/migration/legacy-export/docs/EXPORT_SCHEMA.md`
4. `optional/migration/samples/`

## 强制规则

- 不通过 SSH、Git 同步或服务器文件目录直接发布插件。
- 不将真实令牌写入源码或发布包。
- 不直接连数据库手写迁移 SQL 代替 CLI 或 Core SDK。
- 需要查看或分批提取旧平台业务记录时，使用 `legacy-export query-data`，显式传入 `pageIndex/pageSize`；不要把业务记录混入功能 handoff。
- 功能编码、实体编码和字段名默认保持原值，不擅自转换大小写。
- 后端插件优先调用 `context.core.*`，不照搬旧 SQL。
- 保存前校验与业务动作是不同绑定点，不得混用。
- 修改现有对象前先读取完整定义，避免覆盖未涉及的配置。
