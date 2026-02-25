import re

file_path = "src/components/views/space_graph_explorer_view.rs"
with open(file_path, "r") as f:
    content = f.read()

replacements = [
    (r'DpubWorkbenchService::get_lens_summary\(\)', r'DpubWorkbenchService::get_lens_summary("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_edition_trends\(([^,]+),([^)]+)\)', r'DpubWorkbenchService::get_edition_trends("nostra-governance-v0", \1, \2)'),
    (r'DpubWorkbenchService::evaluate_lenses\(([^)]+)\)', r'DpubWorkbenchService::evaluate_lenses("nostra-governance-v0", \1)'),
    (r'DpubWorkbenchService::get_violations_by_node\(\)', r'DpubWorkbenchService::get_violations_by_node("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_run\(([^)]+)\)', r'DpubWorkbenchService::get_run("nostra-governance-v0", \1)'),
    (r'DpubWorkbenchService::query\(([^)]+)\)', r'DpubWorkbenchService::query("nostra-governance-v0", \1)'),
    (r'DpubWorkbenchService::get_edition_diff\(([^,]+),([^)]+)\)', r'DpubWorkbenchService::get_edition_diff("nostra-governance-v0", \1, \2)'),
    (r'DpubWorkbenchService::export_steward_packet\(', r'DpubWorkbenchService::export_steward_packet("nostra-governance-v0", '),
    (r'DpubWorkbenchService::run_pipeline\(([^,]+),([^,]+),([^)]+)\)', r'DpubWorkbenchService::run_pipeline("nostra-governance-v0", \1, \2, \3)'),
    (r'DpubWorkbenchService::get_overview\(\)', r'DpubWorkbenchService::get_overview("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_graph\(\)', r'DpubWorkbenchService::get_graph("nostra-governance-v0", "Topology")'),
    (r'DpubWorkbenchService::get_path_assessment\(\)', r'DpubWorkbenchService::get_path_assessment("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_doctor\(\)', r'DpubWorkbenchService::get_doctor("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_simulations\(\)', r'DpubWorkbenchService::get_simulations("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_editions\(\)', r'DpubWorkbenchService::get_editions("nostra-governance-v0")'),
    (r'DpubWorkbenchService::get_runs\(([^)]+)\)', r'DpubWorkbenchService::get_runs("nostra-governance-v0", \1)'),
    (r'DpubWorkbenchService::get_blast_radius\(([^)]+)\)', r'DpubWorkbenchService::get_blast_radius("nostra-governance-v0", \1)')
]

for pattern, repl in replacements:
    content = re.sub(pattern, repl, content)

with open(file_path, "w") as f:
    f.write(content)

