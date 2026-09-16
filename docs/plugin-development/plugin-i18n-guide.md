# 插件国际化指南

## 1. 总体原则

插件国际化统一采用：

- 语言来源：当前请求上下文中的 `context.locale`
- 语言资源：`plugin-workspace/locales/<locale>.json`
- 后端插件调用方式：`context.t(key, params=None, default=None)`
- 前端插件调用方式：宿主 `vue-i18n` + `usePluginI18n(pluginCode)`
- 平台内置语言包作为 fallback，插件文案统一在 `plugin-workspace/locales/*.json` 中维护

不要在插件代码里手写：

```python
if locale == "zh-CN":
    ...
else:
    ...
```

这种写法只适合临时调试，不适合作为正式插件实现。

## 2. 语言包目录

统一语言包目录：

```text
plugin-workspace/
  locales/
    zh-CN.json
    en-US.json
```

统一语言包由插件开发人员维护，建议同时承载三类文案：

- 平台前端覆盖文案：`platform.*`
- 后端插件文案：`<pluginCode>.*`
- 前端插件文案：`<pluginCode>.*`

语言包按命名空间组织，例如：

```json
{
  "platform": {
    "common": {
      "save": "保存"
    }
  },
  "supplier_guard": {
    "messages": {
      "name_required": "供应商名称不能为空",
      "archived_blocked": "状态为 {status} 的供应商不允许保存"
    }
  }
}
```

英文：

```json
{
  "supplier_guard": {
    "messages": {
      "name_required": "Supplier name is required.",
      "archived_blocked": "Suppliers with status {status} cannot be saved."
    }
  }
}
```

## 3. key 规则

插件代码里写：

```python
context.t("messages.name_required")
```

runtime 会自动按当前 `pluginCode` 补命名空间。
也就是说，`supplier_guard` 插件里：

```python
context.t("messages.name_required")
```

实际查的是：

```text
supplier_guard.messages.name_required
```

也支持显式全路径 key：

```python
context.t("supplier_guard.messages.name_required")
```

但通常不需要这样写。

## 4. 参数替换

统一使用命名参数：

```python
context.t("messages.archived_blocked", {"status": status})
```

模板：

```json
{
  "archived_blocked": "状态为 {status} 的供应商不允许保存"
}
```

## 5. fallback 规则

当前规则：

1. 先使用 `context.locale`
2. 如果找不到，对后端插件回退到 `zh-CN`
3. 仍找不到时，返回 `default`
4. 如果没有 `default`，返回 key 本身

## 6. 后端插件示例

```python
from plugins_sdk import PluginApp, PluginContext
from plugins_sdk.result import ok, reject

app = PluginApp()


@app.capability("validate_before_save")
def validate_before_save(context: PluginContext, payload: bytes):
    if not context.model.name:
        return reject(context.t("messages.name_required"))

    return ok({
        "business": {
            "message": context.t("messages.validation_passed")
        }
    })
```

## 7. 适用范围

这套机制当前已经正式适用于：

- 平台前端覆盖文案
- 后端 runtime 插件
- 前端插件页面
- `context.t(...)`
- `usePluginI18n(pluginCode)`
- `PluginContextFactory` 单元测试场景

其中：

- 后端插件和前端插件发布后都会使用 `plugin-workspace/locales/*.json` 中的语言资源
- 平台内置语言包仍作为系统默认 fallback

## 8. 开发维护原则

应当把 `plugin-workspace/locales/` 视为统一语言资源目录。

插件开发人员通常只需要维护：

- `plugin-workspace/locales/zh-CN.json`
- `plugin-workspace/locales/en-US.json`

不要修改平台内置语言资源。
