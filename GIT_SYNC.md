# Git 同步说明

本目录是独立的 Plugin Dev Kit Git 仓库。插件源码、开发工具、文档和本地构建脚本可以通过普通 `git pull` / `git push` 在多台电脑间同步，不再需要反复解压 ZIP。

## Windows 首次使用

```powershell
git clone <仓库地址> plugin-dev-kit
cd plugin-dev-kit
.\scripts\build-windows-tools.ps1
Copy-Item .\bin\windows\asapflow\config.example.json .\bin\windows\asapflow\config.json
```

填写本机 `config.json` 后执行：

```powershell
python .\verify-dev-kit.py
.\plugin-build.ps1 -PluginCode supplier_guard
```

`config.json`、`node_modules`、虚拟环境、Rust `target` 和插件构建产物均被 Git 忽略，不会提交令牌或机器缓存。

## 日常同步

开始工作前：

```powershell
git pull --rebase
```

完成插件修改后：

```powershell
git status
git add plugin-workspace
git commit -m "feat(plugin): update business plugin"
git push
```

Git 不会删除仓库中独有的插件目录。若两台电脑修改了同一个已跟踪文件，Git 会要求合并冲突，而不是静默覆盖。

## 工具更新

仓库内保留了两套最小工具源码：

- `tooling/cli/`
- `tooling/legacy-export/`

拉取到新版本后运行：

```powershell
.\scripts\build-windows-tools.ps1
```

脚本会测试并构建当前提交对应的 Windows EXE，然后更新：

```text
bin\windows\asapflow\asapflow.exe
optional\migration\legacy-export\bin\windows\legacy-export.exe
```

正式发布仓库快照前运行：

```powershell
python .\verify-dev-kit.py --mode delivery
```

本地已配置环境日常检查只运行 `python .\verify-dev-kit.py`。
