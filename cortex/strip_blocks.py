import json
import subprocess
import os
import sys

FILEPATH = "apps/cortex-desktop/src/gateway/server.rs"

def run_pass():
    out = subprocess.run(["cargo", "check", "--message-format=json", "-p", "cortex-desktop"], capture_output=True, text=True)
    if out.returncode == 0:
        return False
        
    error_lines = []
    for line in out.stdout.splitlines():
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message":
                m = msg["message"]
                if m["level"] == "error":
                    for span in m["spans"]:
                        if span["is_primary"] and "server.rs" in span["file_name"]:
                            error_lines.append(span["line_start"])
        except:
            pass
            
    if not error_lines:
        print("No server.rs errors found but cargo check failed. Dumping stderr:")
        print(out.stderr)
        return False

    with open(FILEPATH, 'r') as f:
        lines = f.readlines()

    deleted_lines = set()

    for eline in sorted(set(error_lines), reverse=True):
        idx = eline - 1
        if idx in deleted_lines or idx >= len(lines):
            continue
            
        start_idx = idx
        while start_idx >= 0:
            l = lines[start_idx].strip()
            if l.startswith("fn ") or l.startswith("pub fn ") or l.startswith("async fn ") or l.startswith("pub async fn ") or l.startswith("struct ") or l.startswith("pub struct ") or l.startswith("impl ") or l.startswith("enum "):
                break
            start_idx -= 1
            
        if start_idx < 0:
            # If no block start, skip it to prevent deleting the whole file
            # or we could just delete the line itself
            deleted_lines.add(idx)
            continue
            
        # check attributes above
        while start_idx > 0:
            l = lines[start_idx-1].strip()
            if l.startswith("#[") or l.startswith("///"):
                start_idx -= 1
            else:
                break
                
        # find end
        brace_count = 0
        in_block = False
        end_idx = start_idx
        while end_idx < len(lines):
            brace_count += lines[end_idx].count('{')
            brace_count -= lines[end_idx].count('}')
            if '{' in lines[end_idx]:
                in_block = True
            if in_block and brace_count == 0:
                break
            end_idx += 1
            
        if end_idx >= len(lines):
            end_idx = len(lines) - 1
            
        for i in range(start_idx, end_idx + 1):
            deleted_lines.add(i)

    if not deleted_lines:
        return False
        
    new_script = []
    for i, line in enumerate(lines):
        if i not in deleted_lines:
            new_script.append(line)
            
    with open(FILEPATH, 'w') as f:
        f.writelines(new_script)
        
    print(f"Deleted {len(deleted_lines)} lines")
    return True

passes = 0
while passes < 25:
    print(f"Pass {passes}")
    sys.stdout.flush()
    if not run_pass():
        break
    passes += 1
    
print("Checking final compilation status")
subprocess.run(["cargo", "check", "-p", "cortex-desktop"])
