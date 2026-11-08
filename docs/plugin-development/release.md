# 插件构建与发布

## 开始之前

将工具包对应平台的 CLI 加入 PATH。在本机配置平台地址和具有 system.plugin.manage 权限的服务令牌，或使用 ASAPFLOW_BASE_URL、ASAPFLOW_TOKEN 环境变量。真实令牌不提交到项目仓库。

项目位于工具包之外，通过 plugin init-project 初始化。首次构建前端需要 Node.js 18+ 及 npm Registry 访问。

## 日常开发

在独立项目根目录操作，源码位于 backend/<plugin-code>/、frontend/src/pages/<plugin-code>/ 和 locales/。

```sh
asapflow plugin validate --root . --code <plugin-code>
asapflow plugin pack --root . --code <plugin-code>
```

pack 自动构建前端并生成 dist/<plugin-code>-<version>.afplugin。以上命令同样适用于 Windows PowerShell。项目不需要复制工具包快捷脚本。若使用工具包快捷脚本，须从项目目录调用，或明确指定外部项目根目录。

只有用户明确要求发布时执行：

```sh
asapflow plugin publish --root . --code <plugin-code>
```

该命令构建、上传并等待平台校验和切换。查询发布记录及回滚：

```sh
asapflow plugin releases --plugin-code <plugin-code>
asapflow plugin rollback --plugin-code <plugin-code> --version <version>
```

## 更新与源码保护

工具包仓库和项目仓库独立。项目 Git 由开发者管理。文档使用 plugin init-project --check 检查，--update 更新 docs/dev-kit/；源码、项目约定和前端构建配置不自动覆盖。

平台只接收 .afplugin 制品，不通过 Git 同步、SSH 或直接写服务器目录发布。不要把 node_modules、虚拟环境、真实配置和令牌放进制品。
