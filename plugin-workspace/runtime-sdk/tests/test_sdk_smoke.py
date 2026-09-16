"""Smoke tests shipped with the standalone backend plugin SDK."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

SDK_ROOT = Path(__file__).resolve().parents[1]
if str(SDK_ROOT) not in sys.path:
    sys.path.insert(0, str(SDK_ROOT))

import asap_runtime
from asap_runtime import PluginContext, wrap_model
from plugins_sdk import ExecutionResultBuilder, PluginContextFactory


class PluginSdkSmokeTests(unittest.TestCase):
    def test_asap_runtime_public_imports(self) -> None:
        self.assertIs(asap_runtime.PluginContext, PluginContext)
        self.assertTrue(wrap_model({"ready": True}).ready)

    def test_context_factory_and_model_wrapper(self) -> None:
        context = PluginContextFactory.from_fixture(
            datasets={"order": {"amount": 100}}
        )
        self.assertEqual(context.model.amount, 100)
        context.model.amount = 120
        self.assertEqual(context.data.root["amount"], 120)

    def test_result_builder(self) -> None:
        result = (
            ExecutionResultBuilder()
            .status("ok")
            .payload({"checked": True})
            .build()
        )
        self.assertEqual(result.status, "ok")
        self.assertEqual(result.payload, {"checked": True})


if __name__ == "__main__":
    unittest.main()
