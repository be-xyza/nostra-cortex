import re

with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    text = f.read()

funcs = set(re.findall(r"fn ([a-zA-Z0-9_]+)\(", text))

with open("/tmp/server_old.rs", "r") as f:
    orig = f.read()

match = re.search(r"(impl GatewayService \{\n\s*pub async fn start\(port: u16\) \{(?:.*?)\n\})", orig, re.DOTALL)
if not match:
    print("Cannot find orig start block")
    exit(1)
start_block = match.group(1)

def replacer(m):
    handler = m.group(1)
    if handler in funcs or handler == 'ws_handler' or handler == 'ws_collab_handler' or handler == 'list_canisters' or handler == 'runtime_gateway_dispatch_middleware':
        return m.group(0)
    return ""

new_start = re.sub(r'\.route\(\s*"[^"]*",\s*(?:get|post|put|delete|patch|options|head|trace)\(([a-zA-Z0-9_]+)\)\s*,?\s*\)', replacer, start_block)

text_no_start = re.sub(r"impl GatewayService \{\n\s*pub async fn start\(port: u16\) \{.*", "", text, flags=re.DOTALL)

with open("apps/cortex-desktop/src/gateway/server.rs", "w") as f:
    f.write(text_no_start + new_start)
