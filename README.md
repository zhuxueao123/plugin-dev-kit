# Asapment Plugin Dev Kit

工具包与项目插件使用独立 Git 仓库。工具包目录只存放工具、文档和初始化示例；不要在其中开发业务插件。每台电脑共用一套工具，将对应平台的 asapflow 可执行文件加入 PATH。

## 创建独立项目

在工具包目录以外创建项目：

```sh
asapflow plugin init-project --root ../customer-a-plugins --source ./content --offline
cd ../customer-a-plugins
asapflow plugin init --root . --code my_plugin --name MyPlugin --kind full
```

开发者自行决定何时执行 git init、添加远端、提交和同步。CLI 不执行任何项目 Git 操作。

前端在 frontend/ 执行 npm install。后端按开发文档安装 SDK。构建发布使用全局 CLI 的 plugin validate、pack、publish，显式传入 --root . 和 --code。

## 文档升级

```sh
asapflow plugin init-project --root ../customer-a-plugins --check
asapflow plugin init-project --root ../customer-a-plugins --update
```

内容源可以是本地 content 目录或提供 manifest.json 的 HTTPS 目录；通过 --source 更换来源。--offline 禁止网络。无网络时已有项目仍可继续开发。

文档版本独立记录在 CONTENT_VERSION；修改文档后递增该版本，用 scripts/build_dev_kit_content.py 生成内容，不需要重建 CLI。托管文档限定在 docs/dev-kit/，每个文件均校验 SHA-256。项目记录版本和校验值；本地改动发生冲突时拒绝覆盖。项目文档放 docs/project/。已删除的旧版文档保留，不自动删除。

初始化仅创建缺失的项目文件，保留已有 AGENTS.md、.gitignore、前端配置和业务源码；更新只处理托管文档。已有 .gitignore 请自行补充依赖、产物和密钥忽略规则。

旧开发包用户：先备份并将 plugin-workspace 移到工具包外作为独立项目，再对该目录初始化。不要把项目作为工具仓库的子模块或嵌套仓库。

project-template/ 仅供参考和复制示例，content/ 是版本化文档内容。不要把 CLI 二进制、SDK 源码副本或真实令牌提交到项目仓库。
