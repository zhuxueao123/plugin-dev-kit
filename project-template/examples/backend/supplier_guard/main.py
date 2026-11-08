import json

from plugins_sdk import PluginApp, PluginContext
from plugins_sdk.result import ok, reject

app = PluginApp()


@app.capability("validate_before_save")
def validate_before_save(context: PluginContext, payload: bytes):
    model = context.model
    raw_payload = {}
    if payload:
        try:
            raw_payload = json.loads(payload.decode("utf-8"))
        except Exception:
            raw_payload = {}

    name = str(getattr(model, "name", "") or raw_payload.get("name") or "").strip()
    if not name:
        return reject("供应商名称不能为空")

    return ok(
        {
            "business": {
                "message": "示例校验通过",
                "data": {"checked": True, "name": name},
            }
        }
    )


PLUGIN_APP = app
