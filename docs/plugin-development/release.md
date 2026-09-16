# 插件构建与发布

## 开始之前

1. 将 `bin/<platform>/asapflow/config.example.json` 复制为同目录的 `config.json`。
2. 在 `config.json` 中填写平台地址和具有 `system.plugin.manage` 权限的服务令牌。
3. 前端插件首次构建需要 Node.js 18 或更高版本，并能访问 npm Registry。

## 日常开发

源码始终保留在本机的 `plugin-workspace/`：

```text
plugin-workspace/backend/<plugin-code>/
plugin-workspace/frontend/src/pages/<plugin-code>/
```

修改完成后执行：

```bash
./plugin-build.sh <plugin-code>
./plugin-publish.sh <plugin-code>
```

Windows PowerShell：

```powershell
.\plugin-build.ps1 <plugin-code>
.\plugin-publish.ps1 <plugin-code>
```

根目录脚本按插件编码优先使用正式工作区，找不到时自动使用
`plugin-workspace/examples`。Windows 也可传 `-Root <目录>`，例如：

```powershell
.\plugin-build.ps1 supplier_guard -Root .\plugin-workspace\examples
```

构建结果位于 `dist/<plugin-code>-<version>.afplugin`。发布脚本会重新构建、上传并等待平台完成校验和切换。

## 发布安全

- 平台不接收 Vue 源码、node_modules 或 Python 虚拟环境。
- 新后端版本会先完成导入和能力校验，再原子替换活动 handler。
- 已经开始的业务请求继续执行旧 handler，新请求使用新版本。
- 新版本加载失败时平台恢复文件，旧 handler 保持可用。
- 已成功发布的版本可使用 `asapflow plugin rollback` 回滚。

## 高级命令

```bash
asapflow plugin pack --root . --code <plugin-code>
asapflow plugin publish --file dist/<file>.afplugin
asapflow plugin releases --plugin-code <plugin-code>
asapflow plugin rollback --plugin-code <plugin-code> --version <version>
```
