import re

with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    text = f.read()

# 1. find existing functions
funcs = set(re.findall(r"fn ([a-zA-Z0-9_]+)\(", text))

# 2. find GatewayService::start block
start_idx = text.find('impl GatewayService {')
if start_idx == -1:
    print("Cannot find impl GatewayService")
    exit(1)

# we will just replace the `Router::new()` builder chain directly using Python regex
# We want to match: `.route("path", method(handler_name))`
def repl(m):
    full_route = m.group(0)
    handler = m.group(1)
    
    # allow axum::routing methods if we accidentally capture them? No, we just captured handler.
    if handler in funcs or handler == 'ws_handler' or handler == 'ws_collab_handler' or handler == 'list_canisters': # manually keep known ones just in case
        return full_route
    return ""

# we can match `.route([whitespace]*"...",[whitespace]*method(handler))`
# because axum format can span lines, we match more broadly
# `.route(\s*"[^"]*",\s*[a-z]+\(([a-zA-Z0-9_]+)\)\s*,?\s*)`
# Note `.route` could take multiple arguments or layers. Let's do a basic brace/paren balance algorithm.

lines = text.splitlines()
out_lines = []

in_route = False
route_buffer = []
for line in lines:
    if line.strip().startswith(".route("):
        if not in_route:
            in_route = True
            route_buffer = [line]
            # check if it closes on same line
            if line.endswith(")") or line.endswith("),"):
                in_route = False
                handler_search = re.search(r"(get|post|put|delete|patch)\(\s*([a-zA-Z0-9_]+)\s*\)", " ".join(route_buffer))
                if handler_search and handler_search.group(2) not in funcs:
                    pass # skip
                else:
                    out_lines.extend(route_buffer)
                route_buffer = []
        else:
            # wait, `.route` inside a route? no, but just append
            route_buffer.append(line)
    elif in_route:
        route_buffer.append(line)
        if line.strip().startswith(")") or line.strip().endswith("),") or line.strip().endswith(");"):
            in_route = False
            handler_search = re.search(r"(get|post|put|delete|patch)\(\s*([a-zA-Z0-9_]+)\s*\)", " ".join(route_buffer))
            if handler_search and handler_search.group(2) not in funcs:
                pass # strip
            else:
                out_lines.extend(route_buffer)
            route_buffer = []
    else:
        out_lines.append(line)
        
with open("apps/cortex-desktop/src/gateway/server.rs", "w") as f:
    f.write("\n".join(out_lines))
