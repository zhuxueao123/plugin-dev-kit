# 插件目录总览

本文面向使用 Plugin Dev Kit 开发和维护插件的开发人员。

## 1. 运行时会读取哪些目录

- `plugin-workspace/backend/<plugin-code>/`
- `plugin-workspace/frontend/`
- `plugin-workspace/locales/`

其中：
- 后端插件源码位于 `plugin-workspace/backend/<plugin-code>/manifest.json` 和 `main.py`。
- 前端插件工程从 `plugin-workspace/frontend/src/pages/<plugin-code>/manifest.json` 收集页面并统一构建。
- 统一语言资源位于 `plugin-workspace/locales/*.json`。
- 发布后，后端 Runtime 和宿主前端分别加载包内对应的插件代码与语言资源。

## 2. 哪些目录只是文档和示例

- `docs/`
- `plugin-workspace/examples/`

这些目录不会被宿主自动加载，适合放：
- 插件开发说明
- API 参考
- 可复制的 demo
- 构建与发布约定

`plugin-workspace/locales/` 用于维护插件统一语言包：
- `zh-CN.json`
- `en-US.json`

这里是插件开发人员统一维护的平台/插件语言资源目录，建议按命名空间区分：

- `platform.*`
- `<pluginCode>.*`

## 3. 推荐开发流程

1. 先在 `plugin-workspace/examples/` 里挑一个最接近的模板，例如 `supplier_guard` 或 `supplier_portal`。
2. 复制到正式目录：
   - 后端插件复制到 `plugin-workspace/backend/<plugin-code>/`
   - 前端页面复制到 `plugin-workspace/frontend/src/pages/<plugin-code>/`
3. 修改 `pluginCode`、页面路由、权限、manifest 元数据。
4. 本地联调。
5. 使用根目录脚本构建、验证并发布插件。

## 4. 文档导航

- [backend-plugin-guide.md](backend-plugin-guide.md)
- [frontend-plugin-guide.md](frontend-plugin-guide.md)
- [platform-capabilities-guide.md](platform-capabilities-guide.md)
- [core-sdk-reference.md](core-sdk-reference.md)
- [plugin-i18n-guide.md](plugin-i18n-guide.md)
- [plugin-ui-protocol.md](plugin-ui-protocol.md)
- [plugin-e2e-examples.md](plugin-e2e-examples.md)
