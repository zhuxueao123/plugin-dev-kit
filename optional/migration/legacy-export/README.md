# 旧系统导出工具

此工具仅用于旧系统迁移，不参与普通插件的构建和发布。

## 可执行文件

- macOS：`bin/macos/legacy-export`
- Linux x86_64：`bin/linux-x86_64/legacy-export`
- Windows：`bin/windows/legacy-export.exe`

从本目录的 `config.example.json` 创建本地 `config.json`，填写旧系统连接命令。不要提交或分发真实配置。开发人员只需使用随附的可执行文件，无需构建迁移工具。

完整命令、配置和输出结构见 [`CLI_REFERENCE.md`](CLI_REFERENCE.md)，导出格式见 `docs/`，命令示例见 `examples/`。

业务数据抽样或分批提取使用 `query-data`。该命令要求显式传入 `pageIndex`、`pageSize`、表和字段，不会自动混入功能 handoff。
