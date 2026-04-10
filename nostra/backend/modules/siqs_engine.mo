import Array "mo:base/Array";
import Nat "mo:base/Nat";
import Order "mo:base/Order";
import Text "mo:base/Text";
import Graph "graph";
import SiqsTypes "siqs_types";

module {

    public func evaluateAll(
        rules : [SiqsTypes.IntegrityRule],
        entities : [Graph.Entity],
        relationships : [Graph.Relationship],
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : [SiqsTypes.IntegrityViolation] {
        let orderedRules = Array.sort<SiqsTypes.IntegrityRule>(
            rules,
            func(a, b) = Text.compare(a.id, b.id),
        );

        var out : [SiqsTypes.IntegrityViolation] = [];
        for (rule in orderedRules.vals()) {
            out := Array.append(out, evaluateRule(rule, entities, relationships, entityTypeToText));
        };
        out;
    };

    public func evaluateCommonsRuleset(
        ruleset : SiqsTypes.CommonsRuleset,
        mode : SiqsTypes.CommonsEnforcementMode,
        entities : [Graph.Entity],
        relationships : [Graph.Relationship],
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : SiqsTypes.CommonsEnforcementOutcome {
        let violations = evaluateAll(ruleset.rules, entities, relationships, entityTypeToText);
        let shouldBlock = switch (mode) {
            case (#warnOrBlock) hasCritical(violations);
            case (#shadow) false;
        };
        {
            mode = mode;
            shouldBlock = shouldBlock;
            shouldWarn = violations.size() > 0;
            violations = violations;
        };
    };

    private func evaluateRule(
        rule : SiqsTypes.IntegrityRule,
        entities : [Graph.Entity],
        relationships : [Graph.Relationship],
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : [SiqsTypes.IntegrityViolation] {
        switch (rule.predicate.constraint) {
            case (#noCycles) {
                let relationType = switch (rule.predicate.relation) {
                    case (?rel) rel.relationType;
                    case null "depends_on";
                };
                let cycles = detectCycles(relationType, entities, relationships);
                var violations : [SiqsTypes.IntegrityViolation] = [];
                for (cycle in cycles.vals()) {
                    violations := Array.append(violations, [{
                        ruleId = rule.id;
                        affectedNodes = cycle;
                        severity = rule.severity;
                        explanation = "Cycle detected for rule '" # rule.name # "'";
                    }]);
                };
                violations;
            };
            case (_) {
                let candidates = selectCandidates(rule, entities, entityTypeToText);
                var violations : [SiqsTypes.IntegrityViolation] = [];
                for (entity in candidates.vals()) {
                    let relationCount = countRelations(entity.id, rule.predicate.relation, relationships);
                    let violated = switch (rule.predicate.constraint) {
                        case (#mustExist) relationCount == 0;
                        case (#mustNotExist) relationCount > 0;
                        case (#minCount(min)) relationCount < min;
                        case (#maxCount(max)) relationCount > max;
                        case (#noConflicts) {
                            countRelations(
                                entity.id,
                                ?{
                                    relationType = "contradicts";
                                    direction = #outgoing;
                                },
                                relationships,
                            ) > 0;
                        };
                        case (#requiresConstitutionalReference) {
                            countRelations(
                                entity.id,
                                ?{
                                    relationType = "constitutional_basis";
                                    direction = #outgoing;
                                },
                                relationships,
                            ) == 0;
                        };
                        case (#noCycles) false;
                    };
                    if (violated) {
                        violations := Array.append(violations, [{
                            ruleId = rule.id;
                            affectedNodes = [entity.id];
                            severity = rule.severity;
                            explanation = "Rule '" # rule.name # "' violated by node '" # entity.id #
                                "' with relation count " # Nat.toText(relationCount);
                        }]);
                    };
                };
                violations;
            };
        };
    };

    private func selectCandidates(
        rule : SiqsTypes.IntegrityRule,
        entities : [Graph.Entity],
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : [Graph.Entity] {
        let sorted = Array.sort<Graph.Entity>(entities, func(a, b) = Text.compare(a.id, b.id));
        var out : [Graph.Entity] = [];
        for (entity in sorted.vals()) {
            if (matchesScope(rule.scope, entity, entityTypeToText) and matchesSelector(rule.predicate.target, entity, entityTypeToText)) {
                out := Array.append(out, [entity]);
            };
        };
        out;
    };

    private func matchesScope(
        scope : SiqsTypes.IntegrityScope,
        entity : Graph.Entity,
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : Bool {
        switch (scope) {
            case (#global) true;
            case (#entityType(expected)) entityTypeToText(entity.entityType) == expected;
            case (#space(spaceId)) {
                switch (entity.scopeId) {
                    case (?sid) sid == spaceId;
                    case null {
                        switch (getAttr(entity.attributes, "space_id")) {
                            case (?sid) sid == spaceId;
                            case null false;
                        };
                    };
                };
            };
        };
    };

    private func matchesSelector(
        selector : SiqsTypes.NodeSelector,
        entity : Graph.Entity,
        entityTypeToText : (Graph.EntityType) -> Text,
    ) : Bool {
        switch (selector.entityType) {
            case (?expected) {
                if (entityTypeToText(entity.entityType) != expected) return false;
            };
            case null {};
        };

        switch (selector.tags) {
            case (?tags) {
                for (tag in tags.vals()) {
                    if (not containsText(entity.tags, tag)) return false;
                };
            };
            case null {};
        };
        true;
    };

    private func countRelations(
        entityId : Text,
        relation : ?SiqsTypes.EdgeSelector,
        relationships : [Graph.Relationship],
    ) : Nat {
        switch (relation) {
            case null 0;
            case (?rel) {
                var count : Nat = 0;
                for (edge in relationships.vals()) {
                    if (edge.type_ == rel.relationType) {
                        let matchesDirection = switch (rel.direction) {
                            case (#outgoing) edge.from == entityId;
                            case (#incoming) edge.to == entityId;
                        };
                        if (matchesDirection) {
                            count += 1;
                        };
                    };
                };
                count;
            };
        };
    };

    private func outgoingNodes(
        nodeId : Text,
        relationType : Text,
        relationships : [Graph.Relationship],
    ) : [Text] {
        var out : [Text] = [];
        for (edge in relationships.vals()) {
            if (edge.type_ == relationType and edge.from == nodeId) {
                if (not containsText(out, edge.to)) {
                    out := Array.append(out, [edge.to]);
                };
            };
        };
        Array.sort<Text>(out, Text.compare);
    };

    private func detectCycles(
        relationType : Text,
        entities : [Graph.Entity],
        relationships : [Graph.Relationship],
    ) : [[Text]] {
        let sortedEntities = Array.sort<Graph.Entity>(entities, func(a, b) = Text.compare(a.id, b.id));
        var cycles : [[Text]] = [];
        for (entity in sortedEntities.vals()) {
            switch (findCyclePath(entity.id, entity.id, [entity.id], relationType, relationships)) {
                case (?cyclePath) {
                    if (isSmallestNode(entity.id, cyclePath) and not containsCycle(cycles, cyclePath)) {
                        cycles := Array.append(cycles, [cyclePath]);
                    };
                };
                case null {};
            };
        };
        cycles;
    };

    private func findCyclePath(
        current : Text,
        target : Text,
        path : [Text],
        relationType : Text,
        relationships : [Graph.Relationship],
    ) : ?[Text] {
        let nextNodes = outgoingNodes(current, relationType, relationships);
        for (next in nextNodes.vals()) {
            if (next == target and path.size() > 1) {
                return ?Array.append(path, [target]);
            };
            if (not containsText(path, next)) {
                let nextPath = Array.append(path, [next]);
                switch (findCyclePath(next, target, nextPath, relationType, relationships)) {
                    case (?cycle) return ?cycle;
                    case null {};
                };
            };
        };
        null;
    };

    private func containsText(values : [Text], target : Text) : Bool {
        for (value in values.vals()) {
            if (value == target) return true;
        };
        false;
    };

    private func containsCycle(cycles : [[Text]], cycle : [Text]) : Bool {
        let key = Text.join("->", cycle.vals());
        for (existing in cycles.vals()) {
            if (Text.join("->", existing.vals()) == key) return true;
        };
        false;
    };

    private func isSmallestNode(start : Text, cycle : [Text]) : Bool {
        if (cycle.size() == 0) return true;
        var idx : Nat = 0;
        while (idx < cycle.size()) {
            let node = cycle[idx];
            if (node != start and Text.compare(node, start) == #less) {
                return false;
            };
            idx += 1;
        };
        true;
    };

    private func hasCritical(violations : [SiqsTypes.IntegrityViolation]) : Bool {
        for (violation in violations.vals()) {
            if (violation.severity == #critical) return true;
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
