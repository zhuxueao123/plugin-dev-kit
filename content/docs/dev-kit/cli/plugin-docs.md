# AI 生成插件时应补充的资料

如果要让 AI 使用 CLI 之外的插件开发能力，还应补充以下资料：

1. [plugins.md](/Users/mac4/Workspace/AsapFlow/docs/plugins.md)
2. [backend-plugin-guide.md](/Users/mac4/Workspace/AsapFlow/plugins/docs/backend-plugin-guide.md)
3. [core-sdk-reference.md](/Users/mac4/Workspace/AsapFlow/plugins/docs/core-sdk-reference.md)
4. [frontend-plugin-guide.md](/Users/mac4/Workspace/AsapFlow/plugins/docs/frontend-plugin-guide.md)
5. [plugin-i18n-guide.md](/Users/mac4/Workspace/AsapFlow/plugins/docs/plugin-i18n-guide.md)

## 使用原则

1. 插件用于业务扩展，不用于系统结构建模
2. 创建实体、功能、动作、菜单应走 CLI 的系统结构能力
3. 插件代码应生成在本地工作目录
4. 生产服务器只做受控发布、工作区包下载/上传和 reload，不直接让 AI 在线编辑线上源码
5. 多语言统一维护在 `plugins/locales/*.json`
6. 平台前端内置语言包只是 fallback，客户主要维护的是 `plugins/locales`

## 发布模式

插件源码只保留在客户本地工作区。CLI 在本地构建并生成 `.afplugin` 制品，平台通过发布接口完成校验、备份、激活和回滚，不再提供 Git 同步。

推荐命令：

1. `plugin validate --code <code>`
2. `plugin pack --code <code>`
3. `plugin publish --code <code>`
4. `plugin releases --plugin-code <code>`
5. `plugin rollback --plugin-code <code> --version <version>`

详细设计见 [PLUGIN_WORKSPACE_TRANSPORT_DESIGN.md](/Users/mac4/Workspace/AsapFlow/docs/PLUGIN_WORKSPACE_TRANSPORT_DESIGN.md)。
