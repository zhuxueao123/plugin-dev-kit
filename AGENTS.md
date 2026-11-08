# Asapment 插件开发工具包

先读取 `docs/ai/AI_OPERATOR_GUIDE.md`，按任务选择文档。

- 自定义登录页：读取 `docs/plugin-development/login-page-guide.md`，从 `project-template/examples/frontend/customer-brand/` 复制到正式前端工程，使用 `purpose: "login"` 和 `loginContext`，保留当前 CLI 需要的唯一 route。不要修改宿主源码或创建业务菜单。
- 普通前后端插件：读取 `docs/plugin-development/` 中对应指南。
- 只在用户明确要求迁移时读取 `docs/migration/` 与 `optional/migration/`。
- 通过工具包的构建、发布脚本和 CLI 操作，不使用 SSH 或直接编辑服务器目录。不将真实令牌写入源码或发布包。
- 用户只要求设计或创建页面时，完成本地校验和打包；发布需要用户明确要求。不要覆盖已有项目文件。

业务插件必须在工具包目录外的独立项目开发。通过 plugin init-project 初始化；Git 由开发者自行管理。文档更新限定 docs/dev-kit/，不要覆盖业务源码。
