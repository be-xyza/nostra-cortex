# Canister Deployment Receipt

**Date**: 2026-04-26
**Canister Name**: workflow_engine
**Canister ID**: t63gs-up777-77776-aaaba-cai
**Network**: Local managed ICP network (http://127.0.0.1:8000)
**ICP CLI Version**: @icp-sdk/icp-cli 0.2.1
**Module Hash**: 0x670a45da6cb292be4180e1fefc0e8a35603b0d8fbf363e7ea9f6782a5fc568ca

## Deployed Methods

- compile_workflow_v1(definition_json: text, binding_json: text) -> (record { ok: opt text; err: opt text })
- start_workflow_v1(definition_json: text, binding_json: text) -> (record { ok: opt text; err: opt text })
- signal_workflow_v1(instance_id: text, signal_json: text) -> (record { ok: opt text; err: opt text })
- snapshot_workflow_v1(instance_id: text) -> (record { ok: opt text; err: opt text })
- cancel_workflow_v1(instance_id: text, reason: text) -> (record { ok: opt text; err: opt text })

## Deployment Command

```bash
icp deploy workflow_engine --project-root-override /Users/xaoj/ICP -y
```

## Candid Metadata

- candid-extractor version: 0.1.6
- Service metadata embedded via ic-wasm during icp build
- Inferred-type warnings resolved after metadata extraction
