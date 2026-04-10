import Institution "institution";
import Graph "graph";
import Array "mo:base/Array";
import Text "mo:base/Text";
import Nat "mo:base/Nat";

/// Commons Module (119-nostra-commons, Phase 0)
///
/// Provides types and utilities for the Nostra Commons pattern.
/// A Commons is an Institution with scope containing "commons" marker,
/// enforced via convention, not type system.
module {

    // -- Constants --

    public let COMMONS_SCOPE_MARKER = "commons";
    public let COMMONS_CHARTER_TAG = "commons:ruleset";

    // -- Adoption Mode --

    public type AdoptionMode = {
        #adopted;  // Auto-upgrade to minor versions
        #pinned;   // Locked to specific version, manual upgrade
    };

    // -- Commons Metadata (stored in Relationship attributes) --

    public type CommonsAdoption = {
        commonsId : Text;
        spaceId : Text;
        mode : AdoptionMode;
        version : ?Text; // Pinned version (only for #pinned mode)
    };

    // -- Query Helpers --

    /// Check if an Institution is a Commons by convention (scope contains marker)
    public func isCommons(inst : Institution.Institution) : Bool {
        Text.contains(inst.scope, #text COMMONS_SCOPE_MARKER);
    };

    /// Filter institutions to only Commons
    public func filterCommons(institutions : [Institution.Institution]) : [Institution.Institution] {
        Array.filter<Institution.Institution>(institutions, isCommons);
    };

    /// Find all Commons governing a given Space by traversing `governs` edges.
    /// Returns the governing Commons institutions.
    public func getCommonsForSpace(
        spaceId : Text,
        relationships : [Graph.Relationship],
        getInstitution : (Text) -> ?Institution.Institution,
    ) : [Institution.Institution] {
        var result : [Institution.Institution] = [];
        for (rel in relationships.vals()) {
            if (
                rel.to == spaceId and (
                    rel.type_ == Institution.EdgeTypes.GOVERNS or
                    isAdoptionEdge(rel)
                )
            ) {
                switch (getInstitution(rel.from)) {
                    case (?inst) {
                        if (isCommons(inst)) {
                            var exists = false;
                            for (current in result.vals()) {
                                if (current.id == inst.id) {
                                    exists := true;
                                };
                            };
                            if (not exists) {
                                result := Array.append(result, [inst]);
                            };
                        };
                    };
                    case null {};
                };
            };
        };
        Array.sort<Institution.Institution>(
            result,
            func(a, b) = Text.compare(a.id, b.id),
        );
    };


    // -- Edge Builders --

    /// Build the `governs` edge for Commons adoption with mode conveyed
    /// via a composite type_ field: "governs:commons:adopted" or "governs:commons:pinned"
    public func buildAdoptionEdge(
        commonsId : Text,
        spaceId : Text,
        mode : AdoptionMode,
        actorId : Text,
        timestamp : Int,
    ) : Graph.Relationship {
        let modeText = switch (mode) {
            case (#adopted) "adopted";
            case (#pinned) "pinned";
        };
        {
            from = commonsId;
            to = spaceId;
            type_ = "governs:commons:" # modeText;
            bidirectional = false;
            creatorAddress = null;
            creatorActorId = ?actorId;
            timestamp = timestamp;
            libraryId = null;
            scopeId = null;
        };
    };

    /// Check if a relationship is a Commons adoption edge
    public func isAdoptionEdge(rel : Graph.Relationship) : Bool {
        Text.startsWith(rel.type_, #text "governs:commons:");
    };

    /// Extract adoption mode from a Commons adoption edge type_ field
    public func modeFromEdge(rel : Graph.Relationship) : ?AdoptionMode {
        if (Text.endsWith(rel.type_, #text ":adopted")) {
            ?#adopted;
        } else if (Text.endsWith(rel.type_, #text ":pinned")) {
            ?#pinned;
        } else {
            null;
        };
    };

    /// Find all Commons adoption edges for a Space
    public func getAdoptionEdgesForSpace(
        spaceId : Text,
        relationships : [Graph.Relationship],
    ) : [Graph.Relationship] {
        Array.filter<Graph.Relationship>(relationships, func(rel) {
            rel.to == spaceId and isAdoptionEdge(rel);
        });
    };

    /// Get adoption info for a Space from its adoption edges
    public func getAdoptionsForSpace(
        spaceId : Text,
        relationships : [Graph.Relationship],
        getInstitution : (Text) -> ?Institution.Institution,
    ) : [CommonsAdoption] {
        var result : [CommonsAdoption] = [];
        let edges = getAdoptionEdgesForSpace(spaceId, relationships);
        for (edge in edges.vals()) {
            switch (getInstitution(edge.from)) {
                case (?inst) {
                    if (isCommons(inst)) {
                        let mode = switch (modeFromEdge(edge)) {
                            case (?m) m;
                            case null #adopted;
                        };
                        let version : ?Text = switch (mode) {
                            case (#pinned) ?("v" # Nat.toText(inst.version));
                            case (#adopted) null;
                        };
                        result := Array.append(result, [{
                            commonsId = inst.id;
                            spaceId = spaceId;
                            mode = mode;
                            version = version;
                        }]);
                    };
                };
                case null {};
            };
        };
        result;
    };
};
