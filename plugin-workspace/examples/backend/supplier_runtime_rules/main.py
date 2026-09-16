from plugins_sdk import PluginApp, PluginContext
from plugins_sdk.result import ok

app = PluginApp()


def _status_value(context: PluginContext) -> str:
    return str(getattr(context.model, "status", "") or "").strip().lower()


@app.capability("decorate_supplier_form")
def decorate_supplier_form(context: PluginContext, payload: bytes):
    status = _status_value(context)
    message = context.t("messages.form_applied")
    ui_fields = {
        "name": {"required": True},
        "status": {"required": True},
    }
    ui_actions = {}

    if status == "inactive":
        ui_fields["email"] = {"readonly": True}
        ui_fields["phone"] = {"readonly": True}
        ui_actions["save"] = {"disabled": True}
        message = context.t("messages.inactive_readonly")
    elif status == "draft":
        ui_fields["address"] = {"visible": False}
        message = context.t("messages.draft_hide_address")

    return ok(
        {
            "business": {
                "message": message,
                "ui": {"fields": ui_fields, "actions": ui_actions},
            }
        }
    )


@app.capability("decorate_supplier_list")
def decorate_supplier_list(context: PluginContext, payload: bytes):
    return ok(
        {
            "business": {
                "message": context.t("messages.list_applied"),
                "ui": {
                    "fields": {"email": {"visible": False}},
                    "actions": {"delete": {"disabled": True}},
                },
            }
        }
    )


PLUGIN_APP = app
