# 插件源码工作区

此目录只保存插件源码、示例和本地开发依赖，不保存开发文档。文档统一位于 Dev Kit 根目录的 `docs/`。

## 目录用途

- `backend/`：正式后端插件，路径为 `backend/<plugin-code>/`。
- `frontend/`：正式前端插件工程，页面源码位于 `frontend/src/pages/<plugin-code>/`。
- `locales/`：插件开发人员维护的中英文语言资源。
- `examples/`：可复制的前后端示例，不会被平台自动发布。
- `runtime-sdk/`：后端插件本地开发所需的 Python SDK、依赖清单和模块白名单。

前端 SDK 不放在工作区内，由 `frontend/package.json` 从 npm 安装 `@trusteem/asapflow-plugin-sdk`。
后端 SDK 可在干净虚拟环境中通过 `pip install -e runtime-sdk` 安装，并可运行
`python -m unittest discover runtime-sdk/tests` 做本地烟雾测试。`runtime-sdk/tests`
包含 `__init__.py`；进入 `runtime-sdk/` 后也可执行
`python -m unittest tests.test_sdk_smoke` 验证其可导入性。

完整示例位于 `examples/backend/supplier_guard`、
`examples/backend/supplier_runtime_rules` 和 `examples/frontend/supplier_portal`。

## 常用操作

在 Dev Kit 根目录执行：

```bash
bin/macos/asapflow/asapflow plugin init --root . --code <plugin-code> --kind full
./plugin-build.sh <plugin-code>
./plugin-publish.sh <plugin-code>
```

Windows 将命令中的可执行文件和脚本替换为 `bin\windows\asapflow\asapflow.exe`、`plugin-build.ps1` 和 `plugin-publish.ps1`。
根目录脚本会按插件编码自动在正式目录和 `examples/` 中定位源码；CLI 的
`plugin validate/pack/publish --root plugin-workspace` 也会在正式目录找不到时回退到
`plugin-workspace/examples`。

开发前从 [`../docs/README.md`](../docs/README.md) 进入对应指南。
