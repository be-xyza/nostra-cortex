import re

with open("/tmp/server_old.rs", "r") as f:
    orig = f.read()

funcs_to_restore = [
    "get_cortex_heap_blocks",
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

with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    current = f.read()

restored = ""
for func in funcs_to_restore:
    # only restore if not already there
    if f"fn {func}(" not in current:
        # find in orig
        # it might be async fn or pub fn or fn
        # match until the next fn or end of block? No, regex for matching braces is hard, let's use a simple brace counting or AWK-like approach per function
        lines = orig.splitlines()
        in_func = False
        brace_count = 0
        func_lines = []
        for line in lines:
            if not in_func:
                if line.lstrip().startswith(f"pub async fn {func}(") or line.lstrip().startswith(f"async fn {func}(") or line.lstrip().startswith(f"fn {func}(") or line.lstrip().startswith(f"pub fn {func}("):
                    in_func = True
                    brace_count = line.count('{') - line.count('}')
                    func_lines.append(line)
            else:
                brace_count += line.count('{') - line.count('}')
                func_lines.append(line)
                if brace_count == 0:
                    in_func = False
                    restored += "\n" + "\n".join(func_lines) + "\n"
                    break

with open("apps/cortex-desktop/src/gateway/server.rs", "a") as f:
    f.write(restored)
print("Restored functions")
