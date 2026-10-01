# 插件目录总览

本文面向维护 `plugins/` 独立仓库的客户团队。

## 1. 运行时会读取哪些目录

- `plugins/backend/<plugin-code>/`
- `plugins/frontend/`
- `plugins/locales/`

其中：
- 后端 Runtime 会从 `plugins/backend/<plugin-code>/manifest.json` 和 `main.py` 加载插件。
- 前端插件工程会从 `plugins/frontend/src/pages/<plugin-code>/manifest.json` 收集页面并统一构建。
- 后端 runtime 会直接读取 `plugins/locales/*.json` 作为插件语言资源。
- 宿主前端会把 `plugins/locales/*.json` merge 到现有 `vue-i18n`，作为客户可维护的覆盖层。

## 2. 哪些目录只是文档和示例

- `plugins/docs/`
- `plugins/examples/`

这些目录不会被宿主自动加载，适合放：
- 客户开发说明
- API 参考
- 可复制的 demo
- 交付约定

`plugins/locales/` 用于维护插件统一语言包：
- `zh-CN.json`
- `en-US.json`

这里是客户统一维护的平台/插件语言资源目录，建议按命名空间区分：

- `platform.*`
- `<pluginCode>.*`

## 3. 推荐开发流程

1. 先在 `examples/` 里挑一个最接近的模板，例如 `supplier_guard` 或 `supplier_portal`。
2. 复制到正式目录：
   - 后端插件复制到 `plugins/backend/<plugin-code>/`
   - 前端页面复制到 `plugins/frontend/src/pages/<plugin-code>/`
3. 修改 `pluginCode`、页面路由、权限、manifest 元数据。
4. 本地联调。
5. 提交到客户自己的插件 Git 仓库。

## 4. 文档导航

- [backend-plugin-guide.md](backend-plugin-guide.md)
- [frontend-plugin-guide.md](frontend-plugin-guide.md)
- [platform-capabilities-guide.md](platform-capabilities-guide.md)
- [core-sdk-reference.md](core-sdk-reference.md)
- [plugin-i18n-guide.md](plugin-i18n-guide.md)
- [plugin-ui-protocol.md](plugin-ui-protocol.md)
- [plugin-e2e-examples.md](plugin-e2e-examples.md)
