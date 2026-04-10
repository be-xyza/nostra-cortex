import re
import os

with open("/tmp/server_old.rs", "r") as f:
    orig = f.read()

match = re.search(r"impl GatewayService \{\n\s*pub async fn start\(port: u16\) \{(.*?)\n\}", orig, re.DOTALL)
if not match:
    # it might brace match differently, let's just grab everything after `impl GatewayService {` up to `axum::serve(listener, app)` block
    pass

orig_lines = orig.splitlines()
start_idx = -1
end_idx = -1
for i, line in enumerate(orig_lines):
    if line.startswith("impl GatewayService {"):
        start_idx = i
        break

if start_idx != -1:
    brace_count = 0
    in_block = False
    for i in range(start_idx, len(orig_lines)):
        brace_count += orig_lines[i].count('{') - orig_lines[i].count('}')
        if "{" in orig_lines[i] or "}" in orig_lines[i]:
            in_block = True
        if in_block and brace_count == 0:
            end_idx = i
            break

if start_idx != -1 and end_idx != -1:
    block = orig_lines[start_idx:end_idx+1]
    
    # filter the block
    new_block = []
    skip = False
    for i in range(len(block)):
        line = block[i]
        
        # if the line starts a route we don't want, we skip until the closing parenthesis
        # We don't want anything related to /api/cortex/layout, /api/cortex/viewspecs, /api/cortex/preferences, /api/cortex/runtime/closeout
        if ".route(" in line and ("/api/cortex/layout" in line or "/api/cortex/viewspecs" in line or "/api/cortex/preferences" in line or "/api/cortex/runtime/closeout" in line):
            if not line.rstrip().endswith(")"):
                skip = True
            continue
            
        if skip:
            if line.strip().startswith(")") or line.strip().endswith("),"):
                skip = False
            continue
            
        # also some are single line
        if ".route(" in line and ("/api/system/decision" in line):
            # actually decision routes should be kept unless they were part of UX?
            # Wait, did I delete decision methods? No.
            pass
            
        new_block.append(line)
        
    with open("apps/cortex-desktop/src/gateway/server.rs", "a") as f:
        f.write("\n" + "\n".join(new_block) + "\n")
    print("Injected start method.")
else:
    print("Failed to extract start block.")
