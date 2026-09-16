# 旧系统导出工具

此工具仅用于旧系统迁移，不参与普通插件的构建和发布。

## 可执行文件

- macOS：`bin/macos/legacy-export`
- Windows：在 Dev Kit 根目录执行 `.\scripts\build-windows-tools.ps1`。脚本会运行测试、构建并自检，输出到 `bin/windows/legacy-export.exe`。

从本目录的 `config.example.json` 创建本地 `config.json`，填写旧系统连接命令。不要提交或分发真实配置。

完整命令、配置和输出结构见 [`CLI_REFERENCE.md`](CLI_REFERENCE.md)，导出格式见 `docs/`，命令示例见 `examples/`。
