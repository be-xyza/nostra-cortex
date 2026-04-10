import re

with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    current = f.read()
    
# strip everything after async fn post_cortex_heap_emit
idx = current.find("async fn post_cortex_heap_emit(")
if idx != -1:
    current = current[:idx]

with open("/tmp/server_old.rs", "r") as f:
    orig = f.read()

funcs_to_restore = [
    "post_cortex_heap_emit",
    "post_cortex_heap_block_pin",
    "post_cortex_heap_block_delete",
    "post_cortex_heap_blocks_context",
    "get_cortex_heap_block_export",
    "get_cortex_heap_block_history",
    "read_heap_projection_store",
    "write_heap_projection_store",
    "derive_surface_json",
    "parse_heap_cursor",
    "heap_cursor_key"
]

for func in funcs_to_restore:
    # use regex to carefully extract from pub async fn down to the closing brace!
    # A bit tricky. We can find the start index of the function, and then brace count.
    start_match = re.search(r"((?:pub )?(?:async )?fn\s+" + func + r"\s*\([^)]*\)(?:\s*->\s*[^{]+)?\s*\{)", orig)
    if not start_match:
        # try without matching closing paren, parameters might be multiline
        start_match = re.search(r"((?:pub )?(?:async )?fn\s+" + func + r"\s*\()", orig)
        if not start_match:
            print("Cannot find", func)
            continue
            
    start_idx = orig.find(start_match.group(1))
    brace_count = 0
    in_block = False
    
    func_text = ""
    for char in orig[start_idx:]:
        func_text += char
        if char == '{':
            in_block = True
            brace_count += 1
        elif char == '}':
            brace_count -= 1
            if in_block and brace_count == 0:
                break
                
    current += "\n" + func_text + "\n"

with open("apps/cortex-desktop/src/gateway/server.rs", "w") as f:
    f.write(current)
    
print("Restored cleanly!")
