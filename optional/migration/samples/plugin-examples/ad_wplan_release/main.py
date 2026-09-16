from plugins_sdk import PluginApp, PluginContext
from plugins_sdk.result import ok, reject

app = PluginApp()


@app.capability("release_plan")
def release_plan(context: PluginContext, payload: bytes):
    model = context.model

    record_id = str(getattr(model, "id", "") or "").strip()
    current_status = str(getattr(model, "status", "") or "").strip()
    draft_status = str(context.params.get("draftStatus") or "NDRF").strip()
    released_status = str(context.params.get("releasedStatus") or "DREL").strip()

    if not record_id:
      return reject("缺少记录 id，无法执行下达。")

    if current_status and current_status != draft_status:
      return reject(f"当前状态为 {current_status}，只有草稿状态才能下达。")

    context.core.update_record(
        "ad_t_wplan",
        record_id,
        {
            "status": released_status
        },
    )

    return ok(
        {
            "business": {
                "message": "下达成功",
                "data": {
                    "recordId": record_id,
                    "status": released_status
                }
            }
        }
    )


PLUGIN_APP = app
