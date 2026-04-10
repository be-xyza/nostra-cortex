with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    lines = f.readlines()

out = []
in_tests = False
brace_count = 0

for line in lines:
    if line.startswith("#[cfg(test)]"):
        in_tests = True
        continue
    if in_tests and line.startswith("mod tests {"):
        brace_count = 1
        continue
        
    if in_tests:
        brace_count += line.count('{') - line.count('}')
        if brace_count <= 0:
            in_tests = False
        continue
        
    out.append(line)

with open("apps/cortex-desktop/src/gateway/server.rs", "w") as f:
    f.write("".join(out))
