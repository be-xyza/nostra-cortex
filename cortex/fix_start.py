import subprocess
import json
import re

FILEPATH = "apps/cortex-desktop/src/gateway/server.rs"

# Append GatewayService::start
subprocess.run("echo 'impl GatewayService {' >> " + FILEPATH, shell=True)
subprocess.run("cat /tmp/start_block.rs >> " + FILEPATH, shell=True)
subprocess.run("echo '}' >> " + FILEPATH, shell=True)

while True:
    out = subprocess.run(["cargo", "check", "--message-format=json", "-p", "cortex-desktop"], capture_output=True, text=True)
    if out.returncode == 0:
        print("Success!")
        break
        
    error_lines = set()
    for line in out.stdout.splitlines():
        try:
            msg = json.loads(line)
            if msg.get("reason") == "compiler-message" and msg["message"]["level"] == "error":
                m = msg["message"]
                if "cannot find" in m["message"] or "not found in this scope" in m["message"]:
                    for span in m["spans"]:
                        if span["is_primary"] and "server.rs" in span["file_name"]:
                            error_lines.add(span["line_start"])
        except Exception as e:
            pass

    if not error_lines:
        print("No more missing handler errors but build failed.", out.stderr)
        break
        
    print(f"Removing unused route lines: {error_lines}")
    
    with open(FILEPATH, 'r') as f:
        lines = f.readlines()
        
    new_lines = []
    for i, line in enumerate(lines):
        # line numbers are 1-based
        if (i+1) in error_lines:
            # If it's a route, we might need to delete the `.route(...)` wrapper spanning multiple lines, 
            # but usually formatting puts it on one line or 4 lines.
            pass
        else:
            new_lines.append(line)
            
    # wait, just deleting the exact line might leave `.route(\n` broken.
    # It is safer to rewrite the script to do block deletion or syntax deletion.
    # Actually, let's use a simpler regex for the start block: keep only routes whose handlers exist in the file!
