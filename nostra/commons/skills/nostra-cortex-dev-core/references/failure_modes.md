# Failure Modes Reference

## FM-009 governance-bypass-hardcode

Detection classes:
- `hardcoded_canister_id`
- `hardcoded_workspace_root`
- `hardcoded_gateway_url`
- `hardcoded_authority_mode`
- `hardcoded_policy_override`
- `bypassed_config_service`

Remediation:
1. Replace literal with environment/config/governance-backed source.
2. Update contract and checks where needed.
3. Add regression test coverage.
4. Record escalation decision for recurrence.
