import SiqsTypes "siqs_types";

module {

    public func listTemplates() : [SiqsTypes.CommonsRuleTemplate] {
        [
            {
                templateId = "structural.required_constitutional_reference";
                category = "structural";
                name = "Required Constitutional Reference";
                description = "Institution nodes must reference at least one constitutional basis edge.";
                rule = {
                    id = "required_constitutional_reference";
                    name = "Required Constitutional Reference";
                    description = "Institution nodes must reference constitutional basis.";
                    scope = #global;
                    predicate = {
                        target = { entityType = ?("Institution"); tags = null };
                        relation = ?{
                            relationType = "constitutional_basis";
                            direction = #outgoing;
                        };
                        constraint = #requiresConstitutionalReference;
                    };
                    severity = #critical;
                    remediationHint = ?("Link the institution to a constitutional basis contribution.");
                };
            },
            {
                templateId = "structural.no_conflicting_relations";
                category = "structural";
                name = "No Conflicting Relations";
                description = "Institution nodes must not contain contradicts edges.";
                rule = {
                    id = "no_conflicting_relations";
                    name = "No Conflicting Relations";
                    description = "Prevent contradictory structural relationships.";
                    scope = #global;
                    predicate = {
                        target = { entityType = ?("Institution"); tags = null };
                        relation = ?{
                            relationType = "contradicts";
                            direction = #outgoing;
                        };
                        constraint = #noConflicts;
                    };
                    severity = #violation;
                    remediationHint = ?("Resolve or remove contradicting edges.");
                };
            },
            {
                templateId = "lifecycle.no_cycles_for_dependency_graph";
                category = "lifecycle";
                name = "No Cycles for Dependency Graph";
                description = "Project dependency graph must remain acyclic.";
                rule = {
                    id = "no_cycles_for_dependency_graph";
                    name = "No Cycles for Dependency Graph";
                    description = "Prevents circular dependency chains.";
                    scope = #global;
                    predicate = {
                        target = { entityType = ?("Project"); tags = null };
                        relation = ?{
                            relationType = "depends_on";
                            direction = #outgoing;
                        };
                        constraint = #noCycles;
                    };
                    severity = #critical;
                    remediationHint = ?("Remove at least one edge in the dependency cycle.");
                };
            },
        ];
    };
};
