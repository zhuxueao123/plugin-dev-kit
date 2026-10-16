# 预构建 CLI

执行 `git pull --ff-only` 后即可使用，无需安装 Rust 或编译。

| 系统 | 程序路径 |
| --- | --- |
| Windows x64 | `bin/windows/asapflow/asapflow.exe` |
| macOS Apple 芯片 | `bin/macos/asapflow/asapflow` |
| Linux x64 | `bin/linux-x86_64/asapflow/asapflow` |

macOS 和 Linux 可在工具包根目录执行 `./asapflow.sh --version`，Windows 执行
`.\bin\windows\asapflow\asapflow.exe --version`。`plugin-build.sh` 和
`plugin-publish.sh` 自动选择当前系统的程序；Windows 继续使用 PowerShell 脚本。
Linux 构建基于 Ubuntu 22.04，需要 glibc 2.35+、OpenSSL 3 和系统 CA 证书。
macOS 为未公证的命令行程序，首次运行可能需要依照公司安全策略允许执行。

CLI 版本、构建源码版本及各平台 SHA256 记录在根目录 `CLI_VERSION.json`。
macOS 可运行 `shasum -a 256 -c CLI_SHA256SUMS`，Linux 可运行
`sha256sum -c CLI_SHA256SUMS`。更新覆盖固定路径，不创建历史版本目录；真实
`config.json` 不会被更新。请勿本地修改受 Git 管理的程序文件，以免拉取产生冲突。

三端构建全部通过后，才会统一发布。SDK、插件示例和平台镜像由各自的发布流程更新。
