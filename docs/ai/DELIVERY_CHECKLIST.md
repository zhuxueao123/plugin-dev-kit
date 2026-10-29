# 开发包交付检查清单

## 必需内容

- 当前操作系统的 `bin/<platform>/asapflow/asapflow(.exe)` 可执行。
- Windows CLI 可通过 `.\scripts\build_windows_cli.ps1` 构建；唯一 Cargo 源产物为 `cli\target\release\asapflow.exe`，检查通过后复制到 `dist\windows\asapflow.exe`。
- 不得从 `cli\target\x86_64-pc-windows-gnu\` 或其它显式 target 目录选择 Windows CLI；构建前应清除这类遗留产物。
- Windows `legacy-export` 可通过 `.\scripts\build_windows_legacy_export.ps1` 构建，输出位于 `dist\windows\legacy-export.exe`。
- Linux x86_64 `legacy-export` 可通过 `./scripts/build_linux_legacy_export.sh` 构建，输出位于 `dist/linux-x86_64/legacy-export`。
- Windows EXE 的源码提交与开发包文档/示例必须来自同一版本，禁止回退使用 `tools/plugin-dev-kit-binaries` 历史缓存。
- `docs/README.md` 能作为唯一文档入口，链接均指向包内文件。
- `plugin-workspace/backend/`、`frontend/`、`locales/`、`examples/`、`runtime-sdk/` 完整。
- 前端工程声明在线依赖 `@trusteem/asapflow-plugin-sdk`，并能通过 npm 安装。
- 根目录构建、发布脚本可执行。
- 本地使用执行 `python verify-dev-kit.py`；正式交付前必须执行 `python verify-dev-kit.py --mode delivery`，且 Runtime SDK 可独立导入。
- `plugin-workspace/examples/` 下存在 `supplier_guard`、`supplier_runtime_rules` 与 `supplier_portal` 完整示例。
- `runtime-sdk/asap_runtime`、`runtime-sdk/plugins_sdk` 均可独立导入，`runtime-sdk/tests/__init__.py` 存在。

## 安全检查

- 包内不含真实 `config.json`、服务令牌、`node_modules` 或 Python 虚拟环境。
- 插件发布只使用 `.afplugin` 上传接口，不使用 Git 同步或直接编辑服务器目录。
- 前后端 manifest 的插件编码和版本一致。

## 可选迁移内容

- 仅在客户需要旧系统迁移时检查 `optional/migration/`。
- 对应平台的 `legacy-export(.exe)` 可执行，开发包应同时携带 macOS 与 Linux x86_64 版本。
- 至少保留一个主从表样板和一个业务动作插件样板。

## 登录页插件交付

- 根目录 `AGENTS.md`、`docs/plugin-development/login-page-guide.md` 与 AI 入口齐全。
- `plugin-workspace/examples/frontend/customer-brand/` 包含页面和带 `purpose: "login"` 的清单，并保留 CLI 0.2.8 兼容 route。
- 工作区构建器保留 purpose 并共享宿主 Vue；复制示例到正式前端工程后运行 `plugin-build.* customer-brand`，检查 `.afplugin` 内清单、页面 JS、CSS 与共享依赖。
- 不默认发布示例或更改平台登录页；连接测试平台的验收需用户明确授权。
