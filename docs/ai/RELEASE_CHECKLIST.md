# Dev Kit 版本发布检查清单

本清单用于维护 Plugin Dev Kit 自身版本，不是插件发布步骤。普通插件的构建和发布请阅读 `docs/plugin-development/release.md`。

## 文档一致性

- `docs/README.md` 是统一文档入口，新增、删除或重命名文档后必须同步更新导航链接。
- CLI 命令、参数或默认行为变化时，同步更新 `docs/cli/`、命令示例和 AI 操作指南。
- 目录结构、根脚本或构建输出路径变化时，同步更新根 `README.md`、相关开发指南和目录说明。
- Runtime SDK、前端 SDK、manifest 或 UI 协议变化时，同步更新对应参考文档和示例。
- 迁移工具及其输出结构变化时，同步更新 `docs/migration/` 与 `optional/migration/`。
- 文档中的命令必须以当前仓库实际存在的脚本和路径为准。

## 代码与工具

- 当前操作系统的 `bin/<platform>/asapflow/asapflow(.exe)` 可执行。
- Windows 工具通过 `.\scripts\build-windows-tools.ps1` 构建。
- Windows CLI 的 Cargo 构建产物为 `tooling\cli\target\release\asapflow.exe`，脚本将其写入 `bin\windows\asapflow\asapflow.exe`。
- Windows `legacy-export` 的 Cargo 构建产物为 `tooling\legacy-export\target\release\legacy-export.exe`，脚本将其写入 `optional\migration\legacy-export\bin\windows\legacy-export.exe`。
- 不使用 `tooling\cli\target\x86_64-pc-windows-gnu\` 等遗留路径中的产物。
- Windows EXE、源码、文档和示例来自同一个 Dev Kit 版本。
- 根目录构建、发布脚本可执行。

## 工作区与示例

- `plugin-workspace/backend/`、`frontend/`、`locales/`、`examples/`、`runtime-sdk/` 结构完整。
- 前端工程声明在线依赖 `@trusteem/asapflow-plugin-sdk`，并能通过 npm 安装。
- `plugin-workspace/examples/` 包含 `supplier_guard`、`supplier_runtime_rules` 与 `supplier_portal` 完整示例。
- `runtime-sdk/asap_runtime`、`runtime-sdk/plugins_sdk` 均可独立导入，`runtime-sdk/tests/__init__.py` 存在。

## 发布前验证

```bash
python verify-dev-kit.py
python verify-dev-kit.py --mode delivery
python -m unittest discover plugin-workspace/runtime-sdk/tests
```

- 检查结果必须全部通过。
- 仓库不得包含真实 `config.json`、服务令牌、`node_modules`、虚拟环境或生成缓存。
- 前后端 manifest 的 `pluginCode` 和 `version` 必须一致。
- 插件只能通过 `.afplugin` 上传接口发布，不得通过 Git、SSH 或服务器文件目录直接部署。
