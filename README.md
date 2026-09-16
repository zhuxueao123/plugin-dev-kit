# AsapFlow Plugin Dev Kit

本目录同时是可独立克隆和同步的 Git 仓库。跨电脑使用、Windows 工具构建和提交插件源码的流程见 [`GIT_SYNC.md`](GIT_SYNC.md)。

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
| `bin/` | macOS、Windows 的 AsapFlow CLI 及配置模板 | 是 |
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
- Windows 可执行文件必须从同一源码版本构建。先执行 `.\scripts\build_windows_cli.ps1` 和 `.\scripts\build_windows_legacy_export.ps1`，产物统一写入 `dist\windows\`；生成开发包时只允许从该目录取件，不再回退到历史二进制缓存。
- Windows CLI 唯一认可的 Cargo 源产物是 `cli\target\release\asapflow.exe`。不要使用 `cli\target\x86_64-pc-windows-gnu\release\asapflow.exe`；构建脚本会删除该旧目录，防止误打包。
