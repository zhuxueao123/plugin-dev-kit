# AsapFlow Plugin Dev Kit

本开发包用于在客户电脑上开发、验证、打包和发布 AsapFlow 插件。插件源码保留在本机，平台只接收 `.afplugin` 制品。

## 第一次使用

1. 阅读 [`docs/README.md`](docs/README.md)，按任务选择文档，不需要通读全部内容。
2. 配置当前系统的 CLI：将 `bin/<platform>/asapflow/config.example.json` 复制为 `config.json`，填写平台地址和服务令牌。
3. 使用 CLI 创建插件，或从 `plugin-workspace/examples/` 复制示例。
4. 在 `plugin-workspace/` 中开发，通过根目录脚本构建和发布。
5. 已配置的本地工作区运行 `python verify-dev-kit.py`；制作对外交付包时运行 `python verify-dev-kit.py --mode delivery`，额外检查敏感文件和生成目录。

macOS：

```bash
./plugin-build.sh <plugin-code>
./plugin-publish.sh <plugin-code>
```

Windows PowerShell：

```powershell
.\plugin-build.ps1 <plugin-code>
.\plugin-publish.ps1 <plugin-code>
```

脚本会先查找 `plugin-workspace/<正式目录>`，找不到指定编码时再查找
`plugin-workspace/examples/`。也可通过 `-Root <目录>` 明确指定插件根目录；
已有正式插件源码不会因查找示例而被覆盖。

构建结果写入 `dist/<plugin-code>-<version>.afplugin`。

## 顶层目录

| 目录 | 用途 | 日常开发是否需要 |
| --- | --- | --- |
| `bin/` | Windows x64、macOS ARM64、Linux x64 的 AsapFlow CLI 及配置模板 | 是 |
| `docs/` | 唯一文档入口，按插件开发、CLI、AI、迁移分类 | 是 |
| `plugin-workspace/` | 客户维护的前后端插件源码、示例和 Runtime SDK | 是 |
| `optional/` | 旧系统导出和迁移材料，与普通插件开发无关 | 否 |
| `dist/` | 执行构建后生成的发布包，不应提交源码仓库 | 发布时使用 |

根目录的 `plugin-build.*` 和 `plugin-publish.*` 是快捷脚本。完整发布说明见 [`docs/plugin-development/release.md`](docs/plugin-development/release.md)。

## 插件源码位置

```text
plugin-workspace/backend/<plugin-code>/
plugin-workspace/frontend/src/pages/<plugin-code>/
plugin-workspace/locales/
```

- 前端插件在本机完成 npm 构建，平台只接收构建产物。
- 前端 SDK 通过 npm 安装：`@trusteem/asapflow-plugin-sdk`，开发包不携带 SDK 源码副本。
- 后端插件包包含 `manifest.json`、`main.py` 和所需模块。
- 前后端可以合并为同一个版本的 `.afplugin`。
- 平台发布失败时保留旧后端 handler，成功版本可以回滚。

## 重要约束

- AI 或开发人员都必须通过 CLI/发布脚本发布，不使用 SSH、Git 同步或直接编辑服务器目录。
- 前后端 manifest 的 `pluginCode` 和 `version` 必须一致。
- 后端插件不得执行原生 SQL，应使用平台 SDK 能力。
- 不要把真实令牌、`config.json`、`node_modules` 或 Python 虚拟环境放进发布包。
- 三端平台 CLI 由平台仓库 GitHub Actions 统一构建并自动同步，版本记录见 `CLI_VERSION.json`，无需在开发包中本地编译。

## 提交开发包改动

提交前先执行 `git pull --ff-only`，保留 GitHub 发布的三端程序。`.gitignore` 会忽略新增的本机构建文件；已跟踪的程序继续保留在仓库中，便于所有用户拉取。

维护此仓库时执行一次以下命令，启用二进制提交保护：

```bash
git config core.hooksPath .githooks
```

提交检查会阻止手工提交预构建程序及 CLI 版本、校验文件，避免旧程序覆盖自动构建版本。GitHub Actions 使用独立检出，不启用此本地 Hook。旧系统迁移工具的本地二进制也受保护，迁移工具的受控发布需单独处理。
