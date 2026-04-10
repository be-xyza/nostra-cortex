import Types "../schema/types";

module {

    // Schema 0: Nostra Contribution
    public let contributionSchema : Types.CreateSchemaInput = {
        id = "nostra.contribution";
        name = "Contribution";
        description = "A discrete unit of value added to a project.";
        fields = [
            {
                name = "title";
                dataType = #Text;
                required = true;
                description = ?"Summary of contribution";
            },
            {
                name = "body";
                dataType = #Text;
                required = true;
                description = ?"Detailed description";
            },
            {
                name = "space_id";
                dataType = #Reference "nostra.space";
                required = true;
                description = ?"Owning Space";
            },
            {
                name = "type";
                dataType = #Text;
                required = true;
                description = ?"Contribution kind";
            },
            {
                name = "status";
                dataType = #Text;
                required = true;
                description = ?"Lifecycle state";
            },
            {
                name = "tags";
                dataType = #Tags;
                required = false;
                description = ?"Structured tag list";
            },
            {
                name = "contributors";
                dataType = #Text;
                required = false;
                description = ?"Contributor principal IDs serialized as JSON";
            },
            {
                name = "version";
                dataType = #Number;
                required = false;
                description = ?"Version number";
            },
            {
                name = "previous_version_id";
                dataType = #Text;
                required = false;
                description = ?"Prior contribution version ID";
            },
            {
                name = "previous_version_checksum";
                dataType = #Text;
                required = false;
                description = ?"Prior contribution checksum";
            },
            {
                name = "confidence";
                dataType = #Number;
                required = false;
                description = ?"Confidence score";
            },
            {
                name = "phase";
                dataType = #Text;
                required = false;
                description = ?"Exploratory, Deliberative, Decisive, Executable, or Archival";
            },
            {
                name = "created_at";
                dataType = #Date;
                required = true;
                description = ?"Creation timestamp";
            },
            {
                name = "updated_at";
                dataType = #Date;
                required = true;
                description = ?"Last update timestamp";
            },
            {
                name = "visibility";
                dataType = #Text;
                required = false;
                description = ?"Visibility snapshot inherited from the owning Space";
            },
            {
                name = "proof";
                dataType = #Blob;
                required = false;
                description = ?"Cryptographic or link proof";
            },
            {
                name = "impact_score";
                dataType = #Number;
                required = false;
                description = ?"Estimated impact";
            },
            // Constitutional Fields (Constitution Article II)
            {
                name = "parent_id";
                dataType = #Text;
                required = false;
                description = ?"Lineage: ID of prompt or parent contribution";
            },
            {
                name = "scope_id";
                dataType = #Reference "nostra.scope";
                required = false;
                description = ?"Jurisdiction of this contribution";
            },
            {
                name = "locus_id";
                dataType = #Reference "nostra.locus";
                required = false;
                description = ?"Spatial origin (Contextual Sovereignty)";
            },
            {
                name = "lifecycle";
                dataType = #Text;
                required = true;
                description = ?"Lifecycle: emergent, provisional, operational, dormant, archived";
            },
        ];
        constraints = ?{
            invariants = [
                { field = "id"; rule = #Immutable }, // ID cannot change
                { field = "created_at"; rule = #Immutable } // History cannot change
            ];
            mutability = null; // Default mutable for other fields
            accessControl = ?[{ field = "proof"; role = #Contributor }];
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 1: Nostra Space (Project)
    public let spaceSchema : Types.CreateSchemaInput = {
        id = "nostra.space";
        name = "Space";
        description = "A collaborative context for contributions.";
        fields = [
            {
                name = "name";
                dataType = #Text;
                required = true;
                description = ?"Space name";
            },
            {
                name = "mission";
                dataType = #Text;
                required = true;
                description = ?"Mission statement";
            },
            {
                name = "governance_canister";
                dataType = #Text;
                required = false;
                description = ?"SNS/DAO ID";
            },
            // Constitutional Fields (Spaces #3)
            {
                name = "mode";
                dataType = #Text;
                required = true;
                description = ?"Framing: 'Exploratory' vs 'Operational'";
            },
            {
                name = "visibility";
                dataType = #Text;
                required = true;
                description = ?"Openness: 'Public', 'Restricted', 'Private'";
            },
            // Hierarchy Fields (Strategy 034)
            {
                name = "parent_space_id";
                dataType = #Reference "nostra.space";
                required = false;
                description = ?"Parent context (Graph Hierarchy)";
            },
            {
                name = "locus_id";
                dataType = #Reference "nostra.locus";
                required = false;
                description = ?"Physical grounding of this Space";
            },
            {
                name = "constitution_id";
                dataType = #Reference "nostra.constitution";
                required = true;
                description = ?"Governing rules entity";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 2: Nostra Constitution (Governance)
    public let constitutionSchema : Types.CreateSchemaInput = {
        id = "nostra.constitution";
        name = "Constitution";
        description = "Defines the governance rules, parameters, and amendment process for a Space.";
        fields = [
            {
                name = "parent_constitution_id";
                dataType = #Reference "nostra.constitution";
                required = false;
                description = ?"Inheritance source";
            },
            {
                name = "param_overrides";
                dataType = #Blob;
                required = false;
                description = ?"Encoded map of diffs/overrides";
            },
            {
                name = "amendment_process";
                dataType = #Text;
                required = true;
                description = ?"Human-readable amendment policy";
            },
        ];
        constraints = ?{
            invariants = [
                { field = "id"; rule = #Immutable },
                { field = "parent_constitution_id"; rule = #Immutable } // Lineage is sacred
            ];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 3: Assumption (Epistemic Governance)
    public let assumptionSchema : Types.CreateSchemaInput = {
        id = "nostra.assumption";
        name = "Assumption";
        description = "A condition taken as true without direct validation.";
        fields = [
            {
                name = "statement";
                dataType = #Text;
                required = true;
                description = ?"The condition assumed true";
            },
            {
                name = "risk_if_false";
                dataType = #Text;
                required = true;
                description = ?"Impact analysis";
            },
            {
                name = "confidence";
                dataType = #Number;
                required = true;
                description = ?"Subjective confidence (0.0 - 1.0)";
            },
            {
                name = "status";
                dataType = #Text;
                required = true;
                description = ?"Active, Challenged, Invalidated";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 4: Theory (Epistemic Governance)
    public let theorySchema : Types.CreateSchemaInput = {
        id = "nostra.theory";
        name = "Operating Theory";
        description = "A coherent explanatory model or worldview.";
        fields = [
            {
                name = "core_principles";
                dataType = #Text;
                required = true;
                description = ?"The fundamental beliefs of this theory";
            },
            {
                name = "assumptions";
                dataType = #Reference "nostra.assumption";
                required = false;
                description = ?"Deep dependencies";
            }, // Should be List ideally, using single Ref for MVP schema
            {
                name = "confidence";
                dataType = #Number;
                required = true;
                description = ?"Aggregate confidence";
            },
            {
                name = "status";
                dataType = #Text;
                required = true;
                description = ?"Emergent, Accepted, Contested, Deprecated";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 5: Hypothesis (Epistemic Governance)
    public let hypothesisSchema : Types.CreateSchemaInput = {
        id = "nostra.hypothesis";
        name = "Decision Hypothesis";
        description = "A localized, testable prediction.";
        fields = [
            {
                name = "claim";
                dataType = #Text;
                required = true;
                description = ?"If [Action] then [Outcome] by [Time]";
            },
            {
                name = "parent_theory_id";
                dataType = #Reference "nostra.theory";
                required = false;
                description = ?"The worldview this tests";
            },
            {
                name = "success_criteria";
                dataType = #Text;
                required = true;
                description = ?"Observable metrics";
            },
            {
                name = "evaluation_window";
                dataType = #Date;
                required = true;
                description = ?"When to review";
            },
            {
                name = "status";
                dataType = #Text;
                required = true;
                description = ?"Proposed, Active, Supported, Falsified";
            },
        ];
        constraints = ?{
            invariants = [
                { field = "claim"; rule = #Immutable } // You can't change the bet after placing it
            ];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 6: Argument (Deliberation)
    public let argumentSchema : Types.CreateSchemaInput = {
        id = "nostra.argument";
        name = "Argument";
        description = "The atomic unit of reasoning - a structured claim supported by evidence.";
        fields = [
            {
                name = "claim";
                dataType = #Text;
                required = true;
                description = ?"The assertion being made";
            },
            {
                name = "premises";
                dataType = #Text;
                required = false;
                description = ?"References to assumptions or evidence";
            },
            {
                name = "conclusion";
                dataType = #Text;
                required = true;
                description = ?"The logical outcome";
            },
            {
                name = "confidence";
                dataType = #Number;
                required = true;
                description = ?"Strength of argument (0.0 - 1.0)";
            },
            {
                name = "stance";
                dataType = #Text;
                required = true;
                description = ?"support, oppose, alternative";
            },
            {
                name = "scope";
                dataType = #Text;
                required = true;
                description = ?"idea, proposal, decision, theory";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 7: Evaluation (Deliberation - Lightweight Feedback)
    public let evaluationSchema : Types.CreateSchemaInput = {
        id = "nostra.evaluation";
        name = "Evaluation";
        description = "Lightweight feedback on an artifact - replaces Review and Quick Critique.";
        fields = [
            {
                name = "target";
                dataType = #Reference "nostra.contribution";
                required = true;
                description = ?"The artifact being evaluated";
            },
            {
                name = "stance";
                dataType = #Text;
                required = true;
                description = ?"supportive, skeptical, neutral";
            },
            {
                name = "summary";
                dataType = #Text;
                required = true;
                description = ?"Brief feedback text";
            },
            {
                name = "score";
                dataType = #Number;
                required = false;
                description = ?"Optional rating (0-10)";
            },
            {
                name = "status";
                dataType = #Text;
                required = true;
                description = ?"submitted, acknowledged";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 8: Formal Critique (Deliberation - dPub Required)
    public let critiqueSchema : Types.CreateSchemaInput = {
        id = "nostra.critique";
        name = "Formal Critique";
        description = "A rigorous, dPub-backed challenge. Reserved for substantial/critical challenges.";
        fields = [
            {
                name = "target";
                dataType = #Reference "nostra.contribution";
                required = true;
                description = ?"The artifact being challenged";
            },
            {
                name = "mode";
                dataType = #Text;
                required = true;
                description = ?"methodological, logical, empirical, ethical, systemic";
            },
            {
                name = "thesis";
                dataType = #Text;
                required = true;
                description = ?"What the critique claims is flawed";
            },
            {
                name = "severity";
                dataType = #Text;
                required = true;
                description = ?"substantial, critical (no minor - use Evaluation)";
            },
            {
                name = "status";
                dataType = #Text;
                required = true;
                description = ?"submitted, acknowledged, addressed, unresolved";
            },
            {
                name = "body_ref";
                dataType = #Reference "nostra.dpub";
                required = true;
                description = ?"Required dPub document";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 9: Claim (Constitution II.2)
    public let claimSchema : Types.CreateSchemaInput = {
        id = "nostra.claim";
        name = "Claim";
        description = "An assertion about reality made by an actor.";
        fields = [
            {
                name = "subject";
                dataType = #Text;
                required = true;
                description = ?"What is being claimed";
            },
            {
                name = "claimant";
                dataType = #Text;
                required = true;
                description = ?"Who asserts it";
            },
            {
                name = "basis";
                dataType = #Reference "nostra.contribution";
                required = false;
                description = ?"Evidence or reference";
            },
            {
                name = "scope_id";
                dataType = #Reference "nostra.scope";
                required = false;
                description = ?"Where it applies";
            },
            {
                name = "confidence";
                dataType = #Number;
                required = true;
                description = ?"Asserted confidence level";
            },
        ];
        constraints = ?{
            invariants = [{ field = "id"; rule = #Immutable }];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 10: Attestation (Constitution II.3)
    public let attestationSchema : Types.CreateSchemaInput = {
        id = "nostra.attestation";
        name = "Attestation";
        description = "A contribution that affirms, disputes, or evaluates a Claim.";
        fields = [
            {
                name = "target_id";
                dataType = #Reference "nostra.claim";
                required = true;
                description = ?"The claim being attested";
            },
            {
                name = "attestor";
                dataType = #Text;
                required = true;
                description = ?"Who is attesting";
            },
            {
                name = "type";
                dataType = #Text;
                required = true;
                description = ?"affirm, dispute, contextualize, evaluate";
            },
            {
                name = "confidence";
                dataType = #Number;
                required = true;
                description = ?"Confidence in this attestation";
            },
            {
                name = "rationale";
                dataType = #Text;
                required = false;
                description = ?"Explanation of the stance";
            },
        ];
        constraints = ?{
            invariants = [{ field = "id"; rule = #Immutable }];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 11: Scope (Constitution II.4)
    public let scopeSchema : Types.CreateSchemaInput = {
        id = "nostra.scope";
        name = "Scope";
        description = "Defines the contextual and jurisdictional boundaries of contributions.";
        fields = [
            {
                name = "applies_to";
                dataType = #Text;
                required = true;
                description = ?"Entity type or ID this scope governs";
            },
            {
                name = "bounded_by";
                dataType = #Text;
                required = false;
                description = ?"Constraint: time, membership, geography, etc.";
            },
            {
                name = "parent_scope_id";
                dataType = #Reference "nostra.scope";
                required = false;
                description = ?"Hierarchical parent scope";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 12: Institution (Constitution II.7)
    public let institutionSchema : Types.CreateSchemaInput = {
        id = "nostra.institution";
        name = "Institution";
        description = "A named and anchored coordination pattern.";
        fields = [
            {
                name = "mission";
                dataType = #Text;
                required = true;
                description = ?"Primary intent";
            },
            {
                name = "charter_ref";
                dataType = #Reference "nostra.dpub";
                required = false;
                description = ?"Constitutional documentation";
            },
            {
                name = "lifecycle";
                dataType = #Text;
                required = true;
                description = ?"Current state of the institution";
            },
        ];
        constraints = ?{
            invariants = [{ field = "id"; rule = #Immutable }];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 13: Asset Reference (Constitution II.8)
    public let assetReferenceSchema : Types.CreateSchemaInput = {
        id = "nostra.asset_reference";
        name = "Asset Reference";
        description = "A symbolic reference to an external or custodial asset.";
        fields = [
            {
                name = "provider";
                dataType = #Text;
                required = true;
                description = ?"External system (e.g. Ledger, Registry)";
            },
            {
                name = "external_id";
                dataType = #Text;
                required = true;
                description = ?"Unique ID in provider system";
            },
            {
                name = "claimed_owner";
                dataType = #Text;
                required = true;
                description = ?"Who claims ownership";
            },
        ];
        constraints = ?{
            invariants = [];
            mutability = null;
            accessControl = null;
        };
        isPublic = true;
        initialStatus = null;
    };

    // Schema 14: Locus (Core 5 Primitive - Privacy & Compliance)
    public let locusSchema : Types.CreateSchemaInput = {
        id = "nostra.locus";
        name = "Locus";
        description = "A physical point, boundary, or jurisdiction (Core 5 Primitive).";
        fields = [
            {
                name = "coordinates";
                dataType = #Text; // e.g., GeoJSON string. Kept as Text for MVP.
                required = false;
                description = ?"Lat/Long or GeoJSON boundary";
            },
            {
                name = "geohash";
                dataType = #Text;
                required = false;
                description = ?"Spatial index (e.g. geohash-6 for Neighborhood)";
            },
            {
                name = "jurisdiction";
                dataType = #Reference "nostra.scope";
                required = false;
                description = ?"Governing compliance region (e.g. EU)";
            },
            {
                name = "confidence";
                dataType = #Number;
                required = true;
                description = ?"0-1 indicating source verification strength";
            },
            {
                name = "source";
                dataType = #Text;
                required = true;
                description = ?"GPS, IP, User-Declared, TEE-Verified";
            },
        ];
        constraints = ?{
            invariants = [
                { field = "id"; rule = #Immutable }
            ];
            mutability = null; // Coordinates must remain mutable for "Right to Erasure"
            accessControl = ?[
                { field = "coordinates"; role = #Owner } // Privacy by default
            ];
        };
        isPublic = true;
        initialStatus = null;
    };
};
