# 插件开发包验收矩阵

## 本地自动检查

在开发包根目录运行：

```bash
python verify-dev-kit.py
python verify-dev-kit.py --mode delivery
python -m unittest discover plugin-workspace/runtime-sdk/tests
cd plugin-workspace/runtime-sdk && python -m unittest tests.test_sdk_smoke
```

| 声明 | 自动检查 | 通过标准 |
| --- | --- | --- |
| 后端 SDK 可在干净环境导入 | `verify-dev-kit.py`、SDK smoke tests | `asap_runtime` 与 `plugins_sdk` 均可导入，工厂与结果构造器可用 |
| 测试目录可作为 Python 包导入 | `python -m unittest tests.test_sdk_smoke` | 测试模块成功加载并通过 |
| 完整供应商示例包含在仓库中 | `verify-dev-kit.py` | `plugin-workspace/examples/` 下两个后端插件和前端页面入口全部存在 |
| 已安装工作区结构可用 | `verify-dev-kit.py` | 允许本地 `config.json` 和依赖目录，SDK 与示例结构检查通过 |
| 发布版本不包含敏感或临时文件 | `verify-dev-kit.py --mode delivery` | 不含 `config.json`、`.DS_Store`、虚拟环境和依赖缓存 |
| 前端示例可以构建 | `cd plugin-workspace/frontend && npm install && npm run build` | Vite 构建成功并生成两个页面入口 |
| 插件可以校验和打包 | `plugin-build.* supplier_portal` | CLI 校验成功并生成 `.afplugin` |

## 必须连接平台的人工验收

以下能力不能仅凭离线开发包证明，发布前应在测试平台留存命令输出：

| 声明 | 验收方式 | 证据 |
| --- | --- | --- |
| 前后端版本原子发布 | 发布同版本前后端插件，并构造一次加载失败 | 发布结果、平台日志、失败后现有版本 |
| Runtime 热重载 | 发布后执行 `plugin reload` 并调用对应能力 | CLI 输出、Runtime 日志、调用结果 |
| 回滚恢复旧版本 | 成功发布 v2 后回滚到 v1 | 回滚输出和 v1 行为截图/响应 |
| 页面通过宿主鉴权访问 | 登录后访问两个 `supplier_portal` 页面 | 页面截图及无 401 的网络记录 |
