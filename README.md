# AsapFlow Plugin Dev Kit

AsapFlow Plugin Dev Kit 是提供给插件开发人员使用的独立开发套件，用于开发、验证、构建和发布 AsapFlow 插件。平台只接收构建后的 `.afplugin` 制品。

## 第一次使用

1. 阅读 [`docs/README.md`](docs/README.md)，按任务选择文档，不需要通读全部内容。
2. 配置当前系统的 CLI：将 `bin/<platform>/asapflow/config.example.json` 复制为 `config.json`，填写平台地址和服务令牌。
3. 使用 CLI 创建插件，或从 `plugin-workspace/examples/` 复制示例。
4. 在 `plugin-workspace/` 中开发，通过根目录脚本构建和发布。
5. 已配置的开发环境运行 `python verify-dev-kit.py`；发布 Dev Kit 新版本前运行 `python verify-dev-kit.py --mode delivery`，额外检查敏感文件和生成目录。

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
已有正式插件源码不会因查找示例而被修改。

构建结果写入 `dist/<plugin-code>-<version>.afplugin`。

## 顶层目录

| 目录 | 用途 | 日常开发是否需要 |
| --- | --- | --- |
| `bin/` | macOS、Windows 的 AsapFlow CLI 及配置模板 | 是 |
| `docs/` | 统一文档入口，按插件开发、CLI、AI、迁移分类 | 是 |
| `plugin-workspace/` | 插件开发人员维护的前后端插件源码、示例和 Runtime SDK | 是 |
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

- AI 或开发人员都必须通过 CLI/发布脚本发布，不通过 SSH、Git 或服务器文件目录直接部署插件。
- 前后端 manifest 的 `pluginCode` 和 `version` 必须一致。
- 后端插件不得执行原生 SQL，应使用平台 SDK 能力。
- 不要把真实令牌、`config.json`、`node_modules` 或 Python 虚拟环境放进发布包。
- Windows 可执行文件必须从当前版本源码构建。执行 `.\scripts\build-windows-tools.ps1` 后，CLI 写入 `bin\windows\asapflow\asapflow.exe`，迁移工具写入 `optional\migration\legacy-export\bin\windows\legacy-export.exe`。
- Windows CLI 的 Cargo 构建产物是 `tooling\cli\target\release\asapflow.exe`。不要使用 `tooling\cli\target\x86_64-pc-windows-gnu\release\asapflow.exe`；构建脚本会清理该旧目录。

## 文档维护

- `docs/README.md` 是统一文档入口。
- CLI 参数、目录结构、脚本、SDK 或示例发生变化时，必须在同一提交中更新对应文档。
- 发布新版本前必须完成 [`docs/ai/RELEASE_CHECKLIST.md`](docs/ai/RELEASE_CHECKLIST.md) 中的检查，避免代码、示例和文档版本不一致。
