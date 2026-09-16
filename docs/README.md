# 文档导航

所有开发包文档统一放在本目录。根据当前任务阅读对应分组，不需要从头通读。

文档是 Dev Kit 的组成部分。CLI 参数、目录结构、构建脚本、SDK、示例或平台能力发生变化时，必须在同一提交中更新相关文档。版本发布前按 [`ai/RELEASE_CHECKLIST.md`](ai/RELEASE_CHECKLIST.md) 验证文档、代码和示例的一致性。

## 开发插件

建议按以下顺序阅读：

1. [`plugin-development/overview.md`](plugin-development/overview.md)：插件目录、运行方式和开发流程。
2. [`plugin-development/backend-plugin-guide.md`](plugin-development/backend-plugin-guide.md) 或 [`plugin-development/frontend-plugin-guide.md`](plugin-development/frontend-plugin-guide.md)：选择后端或前端开发指南。
3. [`plugin-development/release.md`](plugin-development/release.md)：本地构建、上传、状态查询和回滚。
4. [`plugin-development/plugin-e2e-examples.md`](plugin-development/plugin-e2e-examples.md)：端到端示例。

其他参考资料：

- `plugin-development/core-sdk-reference.md`：后端 Core SDK。
- `plugin-development/platform-capabilities-guide.md`：平台能力边界。
- `plugin-development/plugin-ui-protocol.md`：前端宿主协议。
- `plugin-development/plugin-i18n-guide.md`：多语言约定。

## 使用 CLI

进入 [`cli/README.md`](cli/README.md)。`cli/usage.md` 说明配置与通用参数，`cli/capabilities.md` 是命令能力索引，`cli/examples/` 提供 JSON 示例。

## 让 AI 开发

先让 AI 阅读 [`ai/AI_OPERATOR_GUIDE.md`](ai/AI_OPERATOR_GUIDE.md)，再根据任务提供 `plugin-development/` 或 `cli/` 中的相关文档。不要一次性要求 AI 读取迁移材料。

## 迁移旧系统

迁移不是普通插件开发的必经步骤。只有涉及旧系统元数据迁移时，才阅读 `migration/`，并使用开发包根目录下的 `optional/migration/` 工具和样板。
