import re

with open("apps/cortex-desktop/src/gateway/server.rs", "r") as f:
    content = f.read()

# Remove UI dependencies imports at the top
content = re.sub(r'use crate::services::cortex_ux::.*?;', '', content, flags=re.DOTALL)
content = re.sub(r'use crate::services::cortex_ux_store::.*?;', '', content, flags=re.DOTALL)
content = re.sub(r'use crate::services::theme_policy::.*?;', '', content, flags=re.DOTALL)
content = re.sub(r'use crate::services::viewspec::.*?;', '', content, flags=re.DOTALL)
content = re.sub(r'use crate::services::viewspec_learning::.*?;', '', content, flags=re.DOTALL)
content = re.sub(r'use crate::services::viewspec_synthesis::.*?;', '', content, flags=re.DOTALL)

# Delete the route chain for cortex/layout, theme-policy, and viewspecs
# Finding where it starts: .route("/api/cortex/layout/spec"
start_str = '.route("/api/cortex/layout/spec"'
end_str = '.route("/api/cortex/studio/heap/emit"'

if start_str in content and end_str in content:
    start_idx = content.find(start_str)
    end_idx = content.find(end_str)
    content = content[:start_idx] + content[end_idx:]

with open("apps/cortex-desktop/src/gateway/server.rs", "w") as f:
    f.write(content)

