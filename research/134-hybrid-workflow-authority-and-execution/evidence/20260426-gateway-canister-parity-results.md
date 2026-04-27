# Gateway Canister Parity Test Results

**Date**: 2026-04-26
**Gateway**: http://127.0.0.1:3555
**Canister**: workflow_engine (t63gs-up777-77776-aaaba-cai)
**IC Network**: http://127.0.0.1:8000

## Supported Definition Test

### Canister Path

- Definition ID: workflow_def_gateway_supported_canister_1
- Instance ID: workflow_instance_gateway_canister_supported_1
- Adapter: workflow_engine_canister_v1
- Start: POST /api/cortex/workflow-instances -> accepted=true, status=waiting_checkpoint
- Read: GET /api/cortex/workflow-instances/{id} -> OK
- Trace: GET /trace -> workflow_started, checkpoint_created
- Checkpoints: GET /checkpoints -> pending human checkpoint gateway_supported_review
- Signal: POST /signals with approve -> checkpoint status=resolved
- Post-signal instance: status=completed
- Post-signal outcome: status=completed

### Local Path

- Definition ID: workflow_def_gateway_supported_canister_1
- Instance ID: workflow_instance_gateway_local_supported_2
- Adapter: local_durable_worker_v1
- Start: POST /api/cortex/workflow-instances -> status=waiting_checkpoint
- Trace: workflow_started, checkpoint_created
- Checkpoints: pending human checkpoint gateway_supported_review
- Signal: approve -> checkpoint status=resolved
- Post-signal instance: status=completed
- Post-signal outcome: status=completed

## Cancel Path Test (Canister)

- Instance ID: workflow_instance_gateway_canister_cancel_1
- Cancel: POST /cancel -> checkpoint status=cancelled
- Post-cancel instance: status=cancelled
- Post-cancel outcome: status=cancelled

## Unsupported Definition Fail-Fast

- Active fixture with parallel/evaluation_gate nodes
- Start via workflow_engine_canister_v1 -> failed before instance creation
- Error: unsupported node kinds

## Parity Verdict

Both adapters reach identical semantic state for supported motifs:

- workflow_started event
- checkpoint_created event
- signal_received event
- resolved checkpoint
- completed instance
- completed outcome
