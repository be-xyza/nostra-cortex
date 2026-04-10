import Array "mo:base/Array";
import Text "mo:base/Text";
import Graph "graph";
import Commons "commons";
import Institution "institution";
import SiqsTypes "siqs_types";
import SiqsEngine "siqs_engine";

module {

    public type RulesetLookup = (Text) -> ?SiqsTypes.CommonsRuleset;

    public type CommonsEvaluationContext = {
        commonsIds : [Text];
        ruleCount : Nat;
        graph : SiqsTypes.EvaluationGraph;
    };

    /// Build the evaluation graph from backend entities plus projected institutions.
    /// Space filtering keeps global entities (scopeId = null) and includes space-local entities.
    public func projectGraph(
        spaceId : Text,
        entities : [Graph.Entity],
        relationships : [Graph.Relationship],
        institutions : [Institution.Institution],
    ) : SiqsTypes.EvaluationGraph {
        var projectedEntities : [Graph.Entity] = [];
        var includedIds : [Text] = [];

        for (entity in entities.vals()) {
            if (inSpaceOrGlobal(spaceId, entity)) {
                projectedEntities := Array.append(projectedEntities, [entity]);
                includedIds := Array.append(includedIds, [entity.id]);
            };
        };

        for (inst in institutions.vals()) {
            if (inst.spaceId == spaceId or Commons.isCommons(inst)) {
                let projected = institutionAsEntity(inst);
                projectedEntities := Array.append(projectedEntities, [projected]);
                includedIds := Array.append(includedIds, [projected.id]);
            };
        };

        var projectedRelationships : [Graph.Relationship] = [];
        for (rel in relationships.vals()) {
            if (contains(includedIds, rel.from) or contains(includedIds, rel.to)) {
                projectedRelationships := Array.append(projectedRelationships, [rel]);
            };
        };

        {
            entities = projectedEntities;
            relationships = projectedRelationships;
        };
    };

    public func lookupCommonsForSpace(
        spaceId : Text,
        relationships : [Graph.Relationship],
        institutions : [Institution.Institution],
    ) : [Institution.Institution] {
        let adoptions = Commons.getAdoptionsForSpace(
            spaceId,
            relationships,
            func(id : Text) : ?Institution.Institution {
                for (inst in institutions.vals()) {
                    if (inst.id == id) {
                        return ?inst;
                    };
                };
                null;
            },
        );
        var result : [Institution.Institution] = [];
        for (adoption in adoptions.vals()) {
            for (inst in institutions.vals()) {
                if (inst.id == adoption.commonsId and Commons.isCommons(inst)) {
                    if (not containsInstitution(result, inst.id)) {
                        result := Array.append(result, [inst]);
                    };
                };
            };
        };
        Array.sort<Institution.Institution>(
            result,
            func(a, b) = Text.compare(a.id, b.id),
        );
    };

    public func aggregateRules(
        commonsInstitutions : [Institution.Institution],
        rulesetLookup : RulesetLookup,
    ) : [SiqsTypes.IntegrityRule] {
        let orderedCommons = Array.sort<Institution.Institution>(
            commonsInstitutions,
            func(a, b) = Text.compare(a.id, b.id),
        );

        var aggregated : [SiqsTypes.IntegrityRule] = [];
        for (commonsInst in orderedCommons.vals()) {
            switch (rulesetLookup(commonsInst.id)) {
                case (?ruleset) {
                    let orderedRules = Array.sort<SiqsTypes.IntegrityRule>(
                        ruleset.rules,
                        func(a, b) = Text.compare(a.id, b.id),
                    );
                    for (rule in orderedRules.vals()) {
                        aggregated := Array.append(aggregated, [{
                            id = commonsInst.id # "::" # rule.id;
                            name = rule.name;
                            description = rule.description;
                            scope = rule.scope;
                            predicate = rule.predicate;
                            severity = rule.severity;
                            remediationHint = rule.remediationHint;
                        }]);
                    };
                };
                case null {};
            };
        };
        aggregated;
    };

    public func evaluateForSpace(
        spaceId : Text,
        mode : SiqsTypes.CommonsEnforcementMode,
        entities : [Graph.Entity],
        relationships : [Graph.Relationship],
        institutions : [Institution.Institution],
        rulesetLookup : RulesetLookup,
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : (SiqsTypes.CommonsEnforcementOutcome, CommonsEvaluationContext) {
        let commonsInstitutions = lookupCommonsForSpace(spaceId, relationships, institutions);
        let aggregatedRules = aggregateRules(commonsInstitutions, rulesetLookup);
        let projectedGraph = projectGraph(spaceId, entities, relationships, institutions);

        let outcome : SiqsTypes.CommonsEnforcementOutcome = if (aggregatedRules.size() == 0) {
            {
                mode = mode;
                shouldBlock = false;
                shouldWarn = false;
                violations = [];
            };
        } else {
            let syntheticRuleset : SiqsTypes.CommonsRuleset = {
                commonsId = "aggregate:" # spaceId;
                commonsVersion = "synthetic";
                rules = aggregatedRules;
            };
            SiqsEngine.evaluateCommonsRuleset(
                syntheticRuleset,
                mode,
                projectedGraph.entities,
                projectedGraph.relationships,
                entityTypeToText,
            );
        };

        let commonsIds = Array.map<Institution.Institution, Text>(
            commonsInstitutions,
            func(inst : Institution.Institution) : Text { inst.id },
        );

        (
            outcome,
            {
                commonsIds = commonsIds;
                ruleCount = aggregatedRules.size();
                graph = projectedGraph;
            },
        );
    };

    private func inSpaceOrGlobal(spaceId : Text, entity : Graph.Entity) : Bool {
        switch (entity.scopeId) {
            case (?scope) scope == spaceId;
            case null {
                switch (getAttr(entity.attributes, "space_id")) {
                    case (?attrSpace) attrSpace == spaceId;
                    case null true;
                };
            };
        };
    };

    private func institutionAsEntity(inst : Institution.Institution) : Graph.Entity {
        let commonsTag : [Text] = if (Commons.isCommons(inst)) {
            ["institution", "commons"];
        } else {
            ["institution"];
        };
        {
            id = inst.id;
            name = inst.title;
            description = inst.summary;
            entityType = #institution;
            tags = commonsTag;
            creatorAddress = null;
            creatorActorId = null;
            timestamp = inst.updatedAt;
            libraryId = null;
            logRefs = null;
            attributes = [("space_id", inst.spaceId), ("scope", inst.scope)];
            scopeId = ?inst.spaceId;
        };
    };

    private func contains(values : [Text], target : Text) : Bool {
        for (value in values.vals()) {
            if (value == target) return true;
        };
        false;
    };

    private func containsInstitution(
        values : [Institution.Institution],
        targetId : Text,
    ) : Bool {
        for (value in values.vals()) {
            if (value.id == targetId) return true;
        };
        false;
    };

    private func getAttr(attrs : [(Text, Text)], key : Text) : ?Text {
        for ((k, v) in attrs.vals()) {
            if (k == key) return ?v;
        };
        null;
    };
};
