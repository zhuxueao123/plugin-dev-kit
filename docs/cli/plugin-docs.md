# CLI 插件开发说明

CLI 负责创建插件骨架、校验、打包、上传、查询发布状态和回滚。插件协议与 SDK 细节统一查阅相邻的 `../plugin-development/`，不要在 CLI 文档中维护第二份插件规范。

## 推荐流程

```bash
asapflow plugin init --root . --code <plugin-code> --kind full
asapflow plugin validate --root . --code <plugin-code>
asapflow plugin pack --root . --code <plugin-code>
asapflow plugin publish --root . --code <plugin-code> --wait
```

也可以直接使用开发包根目录的 `plugin-build.*` 和 `plugin-publish.*` 脚本。

## 文档入口

- [`../plugin-development/overview.md`](../plugin-development/overview.md)
- [`../plugin-development/backend-plugin-guide.md`](../plugin-development/backend-plugin-guide.md)
- [`../plugin-development/frontend-plugin-guide.md`](../plugin-development/frontend-plugin-guide.md)
- [`../plugin-development/release.md`](../plugin-development/release.md)
- [`../plugin-development/core-sdk-reference.md`](../plugin-development/core-sdk-reference.md)
