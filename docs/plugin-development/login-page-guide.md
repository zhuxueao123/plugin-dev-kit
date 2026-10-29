# 使用智能体创建自定义登录页

本指南面向独立开发工具包。页面由前端插件提供，平台负责认证、会话与跳转，开发者可以自由设计布局、图片、颜色和文案。无需修改平台前端源码。

## 1. 交给智能体的任务

在开发工具包目录打开智能体，使用以下任务说明，并补充品牌名称、配色、Logo 和期望布局：

> 先阅读 AGENTS.md、docs/ai/AI_OPERATOR_GUIDE.md、docs/plugin-development/login-page-guide.md 和 docs/plugin-development/release.md。基于 plugin-workspace/examples/frontend/customer-brand 创建项目的自定义登录页，复制到 plugin-workspace/frontend/src/pages/customer-brand 后修改。使用 purpose: "login" 与 loginContext；保留兼容 CLI 的唯一 route，不覆盖平台 /login。界面应适配桌面和手机，包含账号密码、提交状态、错误提示与自动填充。通过 plugin-build 脚本验证并生成 .afplugin，报告产物路径和验证结果。只有我明确要求发布时才上传到配置的平台，不通过 SSH 或服务器文件目录部署。

此任务默认只创建和打包。需要发布时，再明确要求智能体运行发布脚本；真实服务令牌保存在本机配置中，不要粘贴到页面源码或对话生成的示例文件里。

## 2. 前提与工作目录

- 平台前后端需支持登录页插件（包含平台源码提交 `95005cb` 或后续版本）。开发服务器 `http://172.16.1.12:18080` 已于 2026-10-29 部署支持版本；其他环境由部署方确认，更新工具包本身不会升级服务器。
- 使用本工具包随附的新版 `plugin-workspace/frontend/vite.config.mjs`：构建器必须保留 `purpose`，并为登录页共享宿主 Vue 运行时。不要从旧项目覆盖该文件，也不要只拉取某一份文档。
- Node.js 18+、npm；发布还需本机 CLI 配置及 `system.plugin.manage` 权限。详见 [发布指南](release.md)。
- 原始示例在 `plugin-workspace/examples/frontend/customer-brand/`。复制到 `plugin-workspace/frontend/src/pages/customer-brand/`，不要直接在 examples 中构建登录页，因为它不是完整的前端工程。
- 智能体可自定插件编码，但目录名、`pluginCode`、构建和发布命令中的编码必须一致；路由名、路径也需唯一。

macOS / Linux 在工具包根目录执行：

```bash
cp -R plugin-workspace/examples/frontend/customer-brand plugin-workspace/frontend/src/pages/customer-brand
./plugin-build.sh customer-brand
```

Windows PowerShell：

```powershell
Copy-Item -Recurse plugin-workspace/examples/frontend/customer-brand plugin-workspace/frontend/src/pages/customer-brand
.\plugin-build.ps1 customer-brand
```

复制前检查目标是否已存在；已有页面先读取后修改，不覆盖同事的代码。macOS / Linux 工具包使用根目录 `asapflow.sh` 自动选择 CLI；不要写死某个平台的可执行文件路径。

## 3. 清单与组件约定

示例清单：

```json
{
  "pluginCode": "customer-brand",
  "pluginName": "项目登录页",
  "version": "1.0.0",
  "pages": [{
    "code": "login",
    "purpose": "login",
    "entry": "./LoginPage.vue",
    "route": {
      "name": "plugin-customer-brand-login",
      "path": "/plugins/customer-brand/login"
    }
  }]
}
```

`purpose: "login"` 是识别依据。当前 CLI 0.2.8 校验仍要求 `route`，因此必须保留上面的兼容字段；宿主不会将该页面注册到业务菜单或主布局，也不会让该路径取代 `/login`。不需要新增菜单、后端插件、实体、场景或 Python handler。

入口 Vue 组件默认导出，通过属性接收 `loginContext`：

| 能力 | 用法 |
| --- | --- |
| 提交登录 | `await loginContext.signIn({ userName, password })`，返回成功与否 |
| 禁止重复提交 | `loginContext.state.submitting`，用于禁用按钮或显示加载状态 |
| 显示错误 | `loginContext.state.error`，建议放到 `role="alert"` 的元素中 |

平台统一处理初始化跳转、强制改密、token 保存、返回地址校验和成功跳转。插件不直接保存 token、不重复调用认证接口、不自行实现这些跳转，不导入 `src/stores/auth` 等宿主内部模块。

把页面和子组件放在入口所在目录中，使用标准 `import ... from 'vue'`；构建器让这个目录复用宿主运行时。不要额外引入 Vue CDN 或另一份运行时。使用 scoped CSS，设置合理的标题字号，避免平台全局标题样式影响手机排版。Logo、背景等通过相对 import 或 CSS url 打包，不能依赖登录后才能读取的资源或业务接口。

## 4. 校验、构建与发布

`plugin-build.* customer-brand` 会调用工具包现有 CLI 校验和打包，自动构建前端。输出为 `dist/customer-brand-1.0.0.afplugin`，其中应包含 `release.json`、`frontend.zip`；内层包含 `manifests/customer-brand.json`、页面 JS、CSS 以及需要的共享文件。构建清单中必须保留 `purpose: "login"`，并有 `/plugins/frontend/dist/` 下的 `bundlePath`。

不要编辑生成后的 dist 清单来补 purpose；应更新工作区构建器，然后重新构建。不要把 node_modules、config.json、Vue 源码或真实令牌打进制品。

用户明确要求发布后，在工具包根目录执行：

```bash
./plugin-publish.sh customer-brand
```

Windows 使用 `.\plugin-publish.ps1 customer-brand`。脚本会重新构建、上传并等待平台完成切换。通过 CLI 查询发布状态或回滚，不通过 SSH、Git 拉取或手工覆盖服务器目录发布。只修改登录页插件时不需要重新构建平台或 Runtime Reload。

## 5. 如何启用与验收

- 平台只有一个登录页候选时自动采用；多个候选时不会随意选择，而是回退默认页。
- 多候选由部署管理员设置 `LoginPage__PluginCode=customer-brand` 和 `LoginPage__PageCode=login`，然后重启后端；这不是页面设计所需配置，也不要猜测存在一个能修改该部署参数的 CLI 接口。
- 匿名 `GET /api/public/login-page` 返回所选入口；204 表示未选出有效页面，404 通常说明宿主尚未升级。指定的插件或页面找不到时回退默认页。
- 使用无痕窗口打开 `/#/login`（hash 路由）或 `/login`（history 路由），验证 Logo、图片、桌面及手机布局；错误账号应显示错误，提交中应禁用按钮。成功登录、强制改密和原页面返回需使用测试账号验证。
- 模块加载失败、超时、缺少导出或组件渲染异常会回退默认登录页。恢复入口为 `/#/login?builtin=1` 或 `/login?builtin=1`；不会绕过认证。
- 若样式、提交状态或错误提示不更新，先确认使用新版构建器、共享宿主 Vue、子组件在入口目录下，再检查浏览器控制台与资源响应。

登录页插件与平台前端拥有同样的执行权限，能读取密码输入，只能使用项目维护者信任的代码。任意脚本死循环或破坏全局环境无法保证自动恢复，必要时刷新并使用默认入口。
