// =============================================================================
// KIP Module: Knowledge Interaction Protocol Implementation
// =============================================================================
// This module provides KIP-compliant query and mutation capabilities for the
// Nostra knowledge graph. It implements a subset of KQL/KML/META commands.
// =============================================================================

import Text "mo:base/Text";
import Array "mo:base/Array";
import Time "mo:base/Time";
import Int "mo:base/Int";
import Iter "mo:base/Iter";
import Nat "mo:base/Nat";
import Principal "mo:base/Principal";
import Graph "graph";
import Logs "logs";
import Result "mo:base/Result";
import Economics "economics";
import Capabilities "capabilities";
import Auth "auth";

module {

    // --- KIP Types ---

    public type KipResult = {
        #ok : (Text, ?Graph.Entity); // JSON response + Optional Entity (for internal use/hooks)
        #err : Text; // Error message
    };

    public type KipCommand = {
        #find : FindCommand;
        #upsert : UpsertCommand;
        #delete : DeleteCommand;
        #describe : DescribeCommand;
        #search : SearchCommand;
        #unsupported : Text;
    };

    public type FindCommand = {
        projection : [Text]; // Fields to return
        typeFilter : ?Text; // Type to filter by
        nameFilter : ?Text; // Name to filter by
        domainFilter : ?Text; // Domain to filter by
        attributes : [(Text, Text)]; // Inline attribute filters
        limit : ?Nat;
    };

    public type UpsertCommand = {
        conceptType : Text;
        conceptName : Text;
        attributes : [(Text, Text)];
        tags : [Text];
        propositions : [(Text, Text)]; // (RelationshipType, TargetConceptName)
        metadata : [(Text, Text)];
    };

    public type DeleteCommand = {
        conceptType : ?Text;
        conceptName : ?Text;
        detach : Bool;
    };

    public type DescribeCommand = {
        target : Text; // "CONCEPT TYPES", "DOMAINS", "PRIMER", etc.
    };

    public type SearchCommand = {
        searchType : Text; // "CONCEPT", "PROPOSITION"
        searchQuery : Text;
    };

    // --- KIP Query Results ---

    public type ConceptNode = {
        id : Text;
        conceptType : Text;
        name : Text;
        attributes : [(Text, Text)];
        metadata : EntityMetadata;
    };

    public type EntityMetadata = {
        source : ?Text;
        author : ?Text;
        confidence : ?Float;
        createdAt : Int;
        updatedAt : Int;
        status : Text;
    };

    // Helper: Convert EntityType to Text
    public func entityTypeToText(t : Graph.EntityType) : Text {
        switch (t) {
            case (#protocol) { "Protocol" };
            case (#governanceSystem) { "GovernanceSystem" };
            case (#cryptoAsset) { "CryptoAsset" };
            case (#developmentTool) { "DevelopmentTool" };
            case (#infrastructure) { "Infrastructure" };
            case (#component) { "Component" };
            case (#cryptography) { "Cryptography" };
            case (#security) { "Security" };
            case (#economy) { "Economy" };
            case (#feature) { "Feature" };
            case (#model) { "Model" };
            case (#observation) { "Observation" };
            case (#book) { "Book" };
            case (#idea) { "Idea" };
            case (#question) { "Question" };
            case (#comment) { "Comment" };
            case (#project) { "Project" };
            case (#issue) { "Issue" };
            case (#proposal) { "Proposal" };
            case (#decision) { "Decision" };
            case (#bounty) { "Bounty" };
            case (#milestone) { "Milestone" };
            case (#deliverable) { "Deliverable" };
            case (#discussion) { "Discussion" };
            case (#initiative) { "Initiative" };
            case (#reflection) { "Reflection" };
            case (#artifact) { "Artifact" };
            case (#poll) { "Poll" };
            case (#essay) { "Essay" };
            case (#post) { "Post" };
            case (#mediaEssay) { "MediaEssay" };
            case (#review) { "Review" };
            case (#report) { "Report" };
            case (#pledge) { "Pledge" };
            case (#service) { "Service" };
            case (#event) { "Event" };
            case (#person) { "Person" };
            case (#organization) { "Organization" };
            case (#library) { "Library" };
            case (#institution) { "Institution" };
            case (#assetReference) { "AssetReference" };
            case (#credentialReference) { "CredentialReference" };
            case (#dpub) { "DPub" };
            case (#chapter) { "Chapter" };
        };
    };

    // Helper: Convert Text to EntityType
    public func textToEntityType(t : Text) : Graph.EntityType {
        switch (t) {
            case ("Protocol") #protocol;
            case ("GovernanceSystem") #governanceSystem;
            case ("CryptoAsset") #cryptoAsset;
            case ("DevelopmentTool") #developmentTool;
            case ("Infrastructure") #infrastructure;
            case ("Component") #component;
            case ("Cryptography") #cryptography;
            case ("Security") #security;
            case ("Economy") #economy;
            case ("Feature") #feature;
            case ("Model") #model;
            case ("$ConceptType") #model; // Alias for Schema
            case ("$PropositionType") #model; // Alias for Schema
            case ("Observation") #observation;
            case ("Book") #book;
            case ("Idea") #idea;
            case ("Question") #question;
            case ("Comment") #comment;
            case ("Project") #project;
            case ("Issue") #issue;
            case ("Proposal") #proposal;
            case ("Decision") #decision;
            case ("Bounty") #bounty;
            case ("Milestone") #milestone;
            case ("Deliverable") #deliverable;
            case ("Discussion") #discussion;
            case ("Initiative") #initiative;
            case ("Reflection") #reflection;
            case ("Artifact") #artifact;
            case ("Poll") #poll;
            case ("Essay") #essay;
            case ("Post") #post;
            case ("MediaEssay") #mediaEssay;
            case ("Review") #review;
            case ("Report") #report;
            case ("Pledge") #pledge;
            case ("Service") #service;
            case ("Event") #event;
            case ("Person") #person;
            case ("Organization") #organization;
            case ("Institution") #institution;
            case ("AssetReference") #assetReference;
            case ("CredentialReference") #credentialReference;
            case ("Library") #library;
            case ("DPub") #dpub;
            case ("Chapter") #chapter;
            case (_) #component; // Default
        };
    };

    // --- Command Parser (Simplified) ---

    public func parseCommand(command : Text) : KipCommand {
        let trimmed = Text.trim(command, #char ' ');
        let upper = Text.toUppercase(trimmed);

        // Check command type by prefix
        if (Text.startsWith(upper, #text "FIND")) {
            return parseFindCommand(trimmed);
        } else if (Text.startsWith(upper, #text "UPSERT")) {
            return parseUpsertCommand(trimmed);
        } else if (Text.startsWith(upper, #text "DELETE")) {
            return parseDeleteCommand(trimmed);
        } else if (Text.startsWith(upper, #text "DESCRIBE")) {
            return parseDescribeCommand(trimmed);
        } else if (Text.startsWith(upper, #text "SEARCH")) {
            return parseSearchCommand(trimmed);
        } else {
            return #unsupported("Unknown command: " # command);
        };
    };

    func parseFindCommand(command : Text) : KipCommand {
        // Simple parser: extract type filter if present
        // Pattern: FIND(?var) WHERE { ?var {type: "TypeName"} }

        var typeFilter : ?Text = null;
        var nameFilter : ?Text = null;
        var limit : ?Nat = null;

        // Look for type filter
        if (Text.contains(command, #text "type:")) {
            let parts = Text.split(command, #text "type:");
            for (part in parts) {
                if (Text.contains(part, #char '\"')) {
                    let quoteParts = Text.split(part, #char '\"');
                    var foundQuote = false;
                    for (qp in quoteParts) {
                        if (foundQuote) {
                            switch (typeFilter) {
                                case (null) { typeFilter := ?qp };
                                case (_) {};
                            };
                        };
                        foundQuote := true;
                    };
                };
            };
        };

        // Look for name filter
        if (Text.contains(command, #text "name:")) {
            let parts = Text.split(command, #text "name:");
            for (part in parts) {
                if (Text.contains(part, #char '\"')) {
                    let quoteParts = Text.split(part, #char '\"');
                    var foundQuote = false;
                    for (qp in quoteParts) {
                        if (foundQuote) {
                            switch (nameFilter) {
                                case (null) { nameFilter := ?qp };
                                case (_) {};
                            };
                        };
                        foundQuote := true;
                    };
                };
            };
        };

        let attributes = parseInlineProps(command);

        // Look for LIMIT
        if (Text.contains(command, #text "LIMIT")) {
            // Simple extraction - would need proper parsing for production
            limit := ?10;
        };

        #find({
            projection = [];
            typeFilter = typeFilter;
            nameFilter = nameFilter;
            domainFilter = null;
            attributes = attributes;
            limit = limit;
        });
    };

    func parseUpsertCommand(command : Text) : KipCommand {
        // Extract type and name from command using improved parsing

        var conceptType = "Unknown";
        var conceptName = "Unknown";
        var attributes : [(Text, Text)] = [];
        var tags : [Text] = [];
        var metadata : [(Text, Text)] = [("debug_version", "v4")];

        // Extract type: "value" - find the actual quoted value
        conceptType := extractQuotedValue(command, "type:");

        // Extract name: "value"
        conceptName := extractQuotedValue(command, "name:");

        // Extract description if present
        let desc = extractQuotedValue(command, "description:");
        if (desc != "") {
            attributes := Array.append(attributes, [("description", desc)]);
        };

        let status = extractQuotedValue(command, "status:");
        if (status != "") {
            attributes := Array.append(attributes, [("status", status)]);
        };

        let wakeAt = extractQuotedValue(command, "wake_at:");
        if (wakeAt != "") {
            attributes := Array.append(attributes, [("wake_at", wakeAt)]);
        };

        let taskType = extractQuotedValue(command, "task_type:");
        if (taskType != "") {
            attributes := Array.append(attributes, [("task_type", taskType)]);
        };

        // Extract tags: tags: ["a", "b"]
        // Simplified parsing: Look for tags: [ ... ]
        if (Text.contains(command, #text "tags:")) {
            let tagBlock = extractSquareBracketBlock(command, "tags:");
            if (tagBlock != "") {
                // Split by comma and clean quotes
                let rawTags = Text.split(tagBlock, #char ',');
                for (rt in rawTags) {
                    let docleaned = Text.trim(rt, #char ' ');
                    let t = extractFirstQuoted(docleaned);
                    if (t != "") {
                        tags := Array.append(tags, [t]);
                    };
                };
            };
        };

        // Extract properties (for Schema Definition): properties: ["name:Type:req"]
        if (Text.contains(command, #text "properties:")) {
            let propBlock = extractSquareBracketBlock(command, "properties:");
            if (propBlock != "") {
                let rawProps = Text.split(propBlock, #char ',');
                for (rp in rawProps) {
                    let cleaned = Text.trim(rp, #char ' ');
                    let pVal = extractFirstQuoted(cleaned);
                    // Format: "name:Type:req" -> Attribute ("prop:name", "Type,req")
                    if (pVal != "") {
                        let parts = Text.split(pVal, #text ":");
                        var idx = 0;
                        var pName = "";
                        var pRule = "";
                        for (part in parts) {
                            if (idx == 0) pName := part;
                            if (idx > 0) {
                                if (idx > 1) pRule #= ",";
                                pRule #= part;
                            };
                            idx += 1;
                        };
                        if (pName != "") {
                            attributes := Array.append(attributes, [("prop:" # pName, pRule)]);
                        };
                    };
                };
            };
        };

        // Extract attributes: either attributes: [("k","v"), ...] or attributes: { "k": "v", ... }
        // Note: This parser is intentionally simple; it extracts quoted strings in-order and pairs them.
        if (Text.contains(command, #text "attributes:")) {
            let block = extractSquareBracketBlock(command, "attributes:");
            let fallback = if (block != "") { block } else { extractCurlyBraceBlock(command, "attributes:") };
            if (fallback != "") {
                let quoted = extractQuotedStrings(fallback);
                attributes := appendPairsUnique(attributes, quoted);
            };
        };

        let inlineProps = parseInlineProps(command);
        if (inlineProps.size() > 0) {
            attributes := Array.append(attributes, inlineProps);
        };

        // Extract metadata: metadata: [("k","v"), ...] or metadata: { "k": "v", ... }
        if (Text.contains(command, #text "metadata:")) {
            let block = extractSquareBracketBlock(command, "metadata:");
            let fallback = if (block != "") { block } else { extractCurlyBraceBlock(command, "metadata:") };
            if (fallback != "") {
                let quoted = extractQuotedStrings(fallback);
                metadata := appendPairsUnique(metadata, quoted);
            };
        };

        // Extract propositions via simplified syntax (@ "pred" "target")
        // Example: @ "extends" "Entity"
        var propositions : [(Text, Text)] = [];
        // Split by @ to find proposition entries
        let parts = Text.split(command, #char '@');
        var idx = 0;
        for (part in parts) {
            if (idx > 0) {
                // Determine if this part has "pred" "target" (2 quoted strings)
                let quoteParts = Text.split(part, #char '\"');
                // parts[1] should be predicate, parts[3] should be target
                var pred = "";
                var target = "";
                var k = 0;
                for (qp in quoteParts) {
                    if (k == 1) pred := qp;
                    if (k == 3) target := qp;
                    k += 1;
                };

                if (pred != "" and target != "") {
                    propositions := Array.append(propositions, [(pred, target)]);
                };
            };
            idx += 1;
        };

        #upsert({
            conceptType = conceptType;
            conceptName = conceptName;
            attributes = attributes;
            tags = tags;
            propositions = propositions;
            metadata = metadata;
        });
    };

    // Helper: Extract content between [ and ] after a key
    func extractSquareBracketBlock(text : Text, key : Text) : Text {
        if (not Text.contains(text, #text key)) return "";
        let parts = Text.split(text, #text key);
        var afterKey = "";
        var found = false;
        for (p in parts) {
            if (found) { afterKey := p; found := false } else { found := true };
        };

        var result = "";
        var inBlock = false;
        for (c in afterKey.chars()) {
            if (c == ']') return result;
            if (inBlock) result #= Text.fromChar(c);
            if (c == '[') inBlock := true;
        };
        "";
    };

    // Helper: Extract content between { and } after a key
    func extractCurlyBraceBlock(text : Text, key : Text) : Text {
        if (not Text.contains(text, #text key)) return "";
        let parts = Text.split(text, #text key);
        var afterKey = "";
        var found = false;
        for (p in parts) {
            if (found) { afterKey := p; found := false } else { found := true };
        };

        var result = "";
        var inBlock = false;
        for (c in afterKey.chars()) {
            if (c == '}') return result;
            if (inBlock) result #= Text.fromChar(c);
            if (c == '{') inBlock := true;
        };
        "";
    };

    func extractQuotedStrings(text : Text) : [Text] {
        var out : [Text] = [];
        var inQuote = false;
        var current = "";
        for (c in text.chars()) {
            if (c == '\"') {
                if (inQuote) {
                    out := Array.append(out, [current]);
                    current := "";
                    inQuote := false;
                } else {
                    inQuote := true;
                };
            } else if (inQuote) {
                current #= Text.fromChar(c);
            };
        };
        out;
    };

    func hasKey(pairs : [(Text, Text)], key : Text) : Bool {
        for ((k, _v) in pairs.vals()) {
            if (k == key) return true;
        };
        false;
    };

    func appendPairsUnique(existing : [(Text, Text)], quoted : [Text]) : [(Text, Text)] {
        var out = existing;
        var idx : Nat = 0;
        label L loop {
            if (idx + 1 >= quoted.size()) break L;
            let k = quoted[idx];
            let v = quoted[idx + 1];
            if (k != "" and not hasKey(out, k)) {
                out := Array.append(out, [(k, v)]);
            };
            idx += 2;
        };
        out;
    };

    func parseInlineProps(command : Text) : [(Text, Text)] {
        var props : [(Text, Text)] = [];
        let parts = Text.split(command, #text "prop:");
        var idx : Nat = 0;
        for (part in parts) {
            if (idx > 0) {
                let cleaned = Text.trimStart(part, #char ' ');
                let key = extractInlinePropKey(cleaned);
                let value = extractFirstQuoted(cleaned);
                if (key != "" and value != "") {
                    props := Array.append(props, [("prop:" # key, value)]);
                };
            };
            idx += 1;
        };
        props;
    };

    func extractInlinePropKey(fragment : Text) : Text {
        let parts = Text.split(fragment, #char ':');
        var key = "";
        var first = true;
        for (part in parts) {
            if (first) {
                key := Text.trim(part, #char ' ');
                first := false;
            };
        };
        key;
    };

    // Helper: Extract valid block content between { and }
    // func extractBlock... removed to avoid errors

    func extractFirstQuoted(text : Text) : Text {
        var inQuote = false;
        var res = "";
        for (c in text.chars()) {
            if (c == '\"') {
                if (inQuote) return res;
                inQuote := true;
            } else if (inQuote) {
                res #= Text.fromChar(c);
            };
        };
        "";
    };

    // Helper: Extract the quoted value after a key
    public func extractQuotedValue(text : Text, key : Text) : Text {
        // Find position of key
        if (not Text.contains(text, #text key)) {
            return "";
        };

        // Split on key and get the part after it
        let parts = Text.split(text, #text key);
        var afterKey = "";
        var foundKey = false;

        for (part in parts) {
            if (foundKey) {
                afterKey := part;
                // Only take first match
                foundKey := false;
            } else {
                foundKey := true;
            };
        };

        if (afterKey == "") {
            return "";
        };

        // Now extract the first quoted string from afterKey
        // Look for pattern: whitespace* " content "
        var inQuotes = false;
        var result = "";

        for (char in afterKey.chars()) {
            if (char == '\"') {
                if (inQuotes) {
                    // End of quoted string
                    return result;
                } else {
                    // Start of quoted string
                    inQuotes := true;
                };
            } else if (inQuotes) {
                result := result # Text.fromChar(char);
            };
        };

        result;
    };

    func parseDeleteCommand(command : Text) : KipCommand {
        // Extract name: DELETE "Name" or DELETE Name
        // Simple strategy: Remove "DELETE" and trim, if quoted extract.
        let trimmed = Text.trim(command, #char ' ');
        var name : ?Text = null;

        // Check if quoted
        if (Text.contains(trimmed, #text "\"")) {
            // Extract quoted value (hacky: use extractQuotedValue with empty key or just substring)
            // Using extractQuotedValue with "DELETE" might work if space exists?
            // Let's manually parse: find quote, find end quote.
            var inQuote = false;
            var extracted = "";
            for (c in trimmed.chars()) {
                if (c == '\"') {
                    if (inQuote) { inQuote := false; name := ?extracted } else {
                        inQuote := true;
                    };
                } else if (inQuote) {
                    extracted #= Text.fromChar(c);
                };
            };
        } else {
            // Assume DELETE Name
            let parts = Text.split(trimmed, #char ' ');
            // First part is DELETE. Second is Name.
            var idx = 0;
            for (p in parts) {
                if (idx == 1 and p != "") name := ?p;
                if (p != "") idx += 1;
            };
        };

        #delete({
            conceptType = null;
            conceptName = name;
            detach = Text.contains(command, #text "DETACH");
        });
    };

    func parseDescribeCommand(command : Text) : KipCommand {
        let upper = Text.toUppercase(command);

        var target = "PRIMER";
        if (Text.contains(upper, #text "CONCEPT TYPES")) {
            target := "CONCEPT TYPES";
        } else if (Text.contains(upper, #text "DOMAINS")) {
            target := "DOMAINS";
        } else if (Text.contains(upper, #text "PROPOSITION TYPES")) {
            target := "PROPOSITION TYPES";
        };

        #describe({ target = target });
    };

    func parseSearchCommand(command : Text) : KipCommand {
        let upper = Text.toUppercase(command);

        var searchType = "CONCEPT";
        if (Text.contains(upper, #text "PROPOSITION")) {
            searchType := "PROPOSITION";
        };

        // Extract search query (text in quotes after SEARCH)
        var queryText = "";
        let quoteParts = Text.split(command, #char '\"');
        var idx = 0;
        for (qp in quoteParts) {
            if (idx == 1) {
                queryText := qp;
            };
            idx += 1;
        };

        #search({ searchType = searchType; searchQuery = queryText });
    };

    // --- Capsule Parser ---

    public func parseCapsule(content : Text) : [Text] {
        var commands : [Text] = [];
        // Split by "UPSERT" keyword
        let parts = Text.split(content, #text "UPSERT");
        var idx = 0;
        for (part in parts) {
            if (idx > 0) {
                // Skip first part (usually header comments)
                // Check if it really starts with { (ignoring whitespace)
                let trimmed = Text.trimStart(part, #char ' ');
                let trimmed2 = Text.trimStart(trimmed, #char '\n');
                if (Text.startsWith(trimmed2, #text "{")) {
                    // It's a command. Re-attach UPSERT.
                    commands := Array.append(commands, ["UPSERT" # part]);
                };
            };
            idx += 1;
        };
        commands;
    };

    // --- Command Executor ---

    // Chronicle logging callback type
    public type ChronicleCallback = (Logs.LogEntry) -> ();

    // Scanner Types
    public type ScanCriteria = {
        typeFilter : ?Text;
        attrFilters : [(Text, Text)];
    };
    public type Scanner = (ScanCriteria) -> [Graph.Entity];

    // Validator Type
    public type Validator = (Graph.Entity) -> Result.Result<(), Text>;

    // Searcher Type (Story 4-2)
    public type Searcher = (Text) -> [Text]; // Query -> IDs

    // Relationship Event Callback (Phase 2)
    public type RelationshipCallback = (Graph.Relationship) -> ();

    // Economic Event Callback (Story 4-2)
    public type EconomicEventCallback = (Economics.EconomicEvent) -> ();

    public class KipExecutor(
        capabilityExecutor : Capabilities.CapabilityExecutor,
        scanner : Scanner,
        _validator : Validator,
        searcher : Searcher,
        actorId : Auth.ActorID, // Canonical Identity
        defaultScopeId : ?Text,
        economicEventCallback : ?EconomicEventCallback,
        relationshipCallback : ?RelationshipCallback,
    ) {

        // Helper: Log to Chronicle (Audit Log) - Delegated to Capabilities for mutations, but might be used logging warnings
        private func _chronicle(_level : Logs.LogLevel, _message : Text, _context : ?[(Text, Text)]) {
            // For now, we can just print debug or duplicate log if needed.
            // Capabilities handles the meaningful audit trail.
            // We'll leave this empty or point to a logger if passed (but we removed logToChronicle callback)
            // ideally we pass a logger.
        };

        public func execute(_cmd : Text) : KipResult {
            let parsed = parseCommand(_cmd);

            switch (parsed) {
                case (#find(cmd)) { executeFindCommand(cmd) };
                case (#upsert(cmd)) { executeUpsertCommand(cmd) };
                case (#delete(cmd)) { executeDeleteCommand(cmd) };
                case (#describe(cmd)) { executeDescribeCommand(cmd) };
                case (#search(cmd)) { executeSearchCommand(cmd) };
                case (#unsupported(msg)) { #err(msg) };
            };
        };

        func executeFindCommand(cmd : FindCommand) : KipResult {
            // 1. Construct Scan Criteria
            var attrFilters : [(Text, Text)] = [];

            // Map nameFilter to an attribute filter for now (or explicit field check)
            // Note: Scanner implementation will need to handle "name" specially if passed as attribute
            switch (cmd.nameFilter) {
                case (?n) {
                    attrFilters := Array.append(attrFilters, [("name", n)]);
                };
                case (null) {};
            };

            for ((key, value) in cmd.attributes.vals()) {
                attrFilters := Array.append(attrFilters, [(key, value)]);
            };

            let criteria : ScanCriteria = {
                typeFilter = cmd.typeFilter;
                attrFilters = attrFilters;
            };

            // 2. Scan
            let allResults = scanner(criteria);

            // 3. Paginate
            let total = allResults.size();
            var offset = 0;
            var limit = total; // Default to all

            // Apply strict limit if provided
            switch (cmd.limit) {
                case (?l) { limit := l };
                case (null) {};
            };

            // TODO: Extract offset from command if we add it to FindCommand type (currently not there)
            // Assuming offset 0 for now as per FindCommand definition

            var sliced : [Graph.Entity] = [];
            if (offset < total) {
                let available = Nat.sub(total, offset);
                let takeCount = if (limit < available) limit else available;
                sliced := Array.subArray(allResults, offset, takeCount);
            };

            // 4. Return JSON
            #ok(entitiesToJson(sliced, total), null);
        };

        func executeUpsertCommand(cmd : UpsertCommand) : KipResult {
            // 0. Adapter-Level Validation (Fast Fail)
            if (cmd.conceptName == "Unknown" or cmd.conceptName == "") {
                return #err("Validation error: 'name' field is required in UPSERT capsule");
            };
            if (cmd.conceptType == "Unknown" or cmd.conceptType == "") {
                return #err("Validation error: 'type' field is required in UPSERT capsule");
            };

            // 0.1 Validate the projected entity before commit.
            let candidateEntity : Graph.Entity = {
                id = "pending";
                name = cmd.conceptName;
                description = switch (findAttr(cmd.attributes, "description")) {
                    case (?desc) desc;
                    case null "";
                };
                entityType = textToEntityType(cmd.conceptType);
                tags = cmd.tags;
                creatorAddress = null;
                creatorActorId = ?actorId;
                timestamp = Time.now();
                libraryId = null;
                logRefs = null;
                attributes = cmd.attributes;
                scopeId = defaultScopeId;
            };
            switch (_validator(candidateEntity)) {
                case (#err(msg)) { return #err(msg) };
                case (#ok(())) {};
            };

            // 1. Construct MutationOp
            // We strip KIP-specifics and normalize to Capability primitives
            let op : Capabilities.MutationOp = #UpsertEntity({
                id = null; // Generated by Capabilities
                type_ = cmd.conceptType;
                name = cmd.conceptName;
                attributes = cmd.attributes;
                tags = cmd.tags;
                previousVersionId = null; // KIP syntax doesn't support version chaining yet
                previousChecksum = null;
            });

            // 2. Wrap in Envelope
            let envelope : Capabilities.CommandEnvelope = {
                caller = actorId;
                agentId = null; // Could derive from metadata if trusted
                timestamp = Time.now();
                scopeId = defaultScopeId;
                operation = op;
            };

            // 3. Execute via MCI
            let result = capabilityExecutor.executeMutation(envelope);

            switch (result) {
                case (#ok(id)) {
                    // 4. Post-Mutation Loops (Economic Event)
                    // We adapt the new ID into the legacy event system for now
                    switch (economicEventCallback) {
                        case (?callback) {
                            let econEvent = buildEconomicEvent(cmd, id);
                            switch (econEvent) {
                                case (?event) { callback(event) };
                                case (null) {};
                            };
                        };
                        case (null) {};
                    };

                    // 5. Phase 2: Relationships
                    // Note: Capabilities might handle this in future, but for now Adapter drives it
                    // using the returned ID.
                    for ((relType, targetName) in cmd.propositions.vals()) {
                        let targetCriteria : ScanCriteria = {
                            typeFilter = null;
                            attrFilters = [("name", targetName)];
                        };
                        let targets = scanner(targetCriteria);
                        // TODO: Use capabilityExecutor for relationships too!
                        // For MVP mix, we fallback to old callback or omit if critical invariant?
                        // The original code used a callback 'RelationshipCallback' which Constellation used.
                        // We should probably move relationship creation to MCI too.
                        // For now, we will construct simple Relationships manually *or*
                        // skip this if MCI doesn't support edges yet.
                        // The research plan said MCI handles "MutationOp".
                        // Assuming relationshipCallback is still passed in constructor, we use it.
                        // But wait, we removed 'relationshipCallback' from logic?
                        // No, it is in 'executeUpsertCommand' original logic.
                        // I will keep using 'relationshipCallback' here for backward compat
                        // until edges are full MCI citizens.
                        for (target in targets.vals()) {
                            let rel : Graph.Relationship = {
                                from = id;
                                to = target.id;
                                type_ = relType;
                                bidirectional = false;
                                creatorAddress = null; // Removed raw principal
                                creatorActorId = ?actorId;
                                timestamp = Time.now();
                                libraryId = null;
                                scopeId = null;
                            };
                            switch (relationshipCallback) {
                                case (?cb) cb(rel);
                                case (null) {};
                            };
                        };
                    };
                    // 8. Return Success
                    // To return the entity, we need to fetch it first.
                    // Assuming 'id' is the entityId and we need to fetch the new entity.
                    let newEntity = capabilityExecutor.getEntity(id);
                    #ok("{\"status\": \"success\", \"operation\": \"upsert\", \"id\": \"" # id # "\"}", newEntity);
                };
                case (#err(msg)) {
                    #err(msg);
                };
            };
        };

        private func findAttr(attrs : [(Text, Text)], key : Text) : ?Text {
            for ((k, v) in attrs.vals()) {
                if (k == key) return ?v;
            };
            null;
        };

        // Helper: Build EconomicEvent if applicable
        func buildEconomicEvent(cmd : UpsertCommand, entityId : Text) : ?Economics.EconomicEvent {
            // Extract economic context from attributes
            var valueType : Economics.EconomicValueType = #bounty;
            var unit : Economics.EconomicUnit = #credits;
            var amount : Float = 0.0;
            var conditions : ?Text = null;
            var escrowHint : ?Text = null;
            var payoutTrigger : ?Text = null;

            var mode : Economics.EconomicMode = #live; // Default to live if unspecified

            for ((k, v) in cmd.attributes.vals()) {
                if (k == "econ:value_type") {
                    valueType := switch (v) {
                        case ("bounty") #bounty;
                        case ("pledge") #pledge;
                        case ("service") #service;
                        case ("grant") #grant;
                        case (_) #bounty;
                    };
                };
                if (k == "econ:unit") {
                    unit := switch (v) {
                        case ("ICP") #icp;
                        case ("USD") #usd;
                        case ("credits") #credits;
                        case ("reputation") #reputation;
                        case ("cycles") #cycles;
                        case (_) #credits;
                    };
                };
                // Note: Float.fromText not available, using 0 as placeholder
                if (k == "econ:amount") { amount := 0.0 };
                if (k == "econ:conditions") { conditions := ?v };
                if (k == "econ:escrow_hint") { escrowHint := ?v };
                if (k == "econ:payout_trigger") { payoutTrigger := ?v };
                if (k == "econ:mode") {
                    mode := switch (v) {
                        case ("simulation") #simulation;
                        case ("live") #live;
                        case (_) #live;
                    };
                };
            };

            let context : Economics.EconomicContext = {
                valueType = valueType;
                unit = unit;
                amount = amount;
                conditions = conditions;
                escrowHint = escrowHint;
                payoutTrigger = payoutTrigger;
                mode = mode;
            };

            let creatorText = actorId;

            switch (cmd.conceptType) {
                case ("Bounty") {
                    ?#bountyPosted({
                        entityId = entityId;
                        context = context;
                        creator = creatorText;
                    });
                };
                case ("Pledge") {
                    ?#pledgeCommitted({
                        entityId = entityId;
                        context = context;
                        pledger = creatorText;
                    });
                };
                case ("Service") {
                    ?#serviceFulfilled({
                        serviceId = entityId;
                        consumer = creatorText;
                        usageMetric = 1.0;
                    });
                };
                case (_) { null };
            };
        };

        func executeDeleteCommand(_cmd : DeleteCommand) : KipResult {
            #ok("{\"status\": \"acknowledged\", \"operation\": \"delete\"}", null);
        };

        func executeDescribeCommand(cmd : DescribeCommand) : KipResult {
            switch (cmd.target) {
                case ("CONCEPT TYPES") {
                    let allTypes = [
                        "Protocol", "GovernanceSystem", "CryptoAsset",
                        "DevelopmentTool", "Infrastructure", "Component",
                        "Cryptography", "Security", "Economy", "Feature",
                        "Model", "Observation", "Book", "Idea", "Project",
                        "Question", "Comment", "Issue", "Proposal", "Decision",
                        "Bounty", "Milestone", "Deliverable", "Discussion",
                        "Initiative", "Reflection", "Artifact", "Poll",
                        "Essay", "Post", "MediaEssay", "Review", "Report",
                        "Pledge", "Service", "Event", "Person", "Organization",
                        "Library", "Institution", "AssetReference",
                        "CredentialReference", "DPub", "Chapter"
                    ];
                    var json = "{\"concept_types\": [";
                    var first = true;
                    for (t in allTypes.vals()) {
                        if (not first) json #= ", ";
                        json #= "\"" # t # "\"";
                        first := false;
                    };
                    json #= "]}";
                    #ok(json, null);
                };
                case ("DOMAINS") {
                    #ok("{\"domains\": [\"CoreSchema\", \"NostraCore\", \"ICPKnowledge\"]}", null);
                };
                case ("PROPOSITION TYPES") {
                    #ok("{\"proposition_types\": [\"relates_to\", \"spawns\", \"implements\", \"blocks\", \"resolves\", \"supersedes\", \"belongs_to_space\", \"authored_by\", \"part_of\", \"reviews\", \"rewards\"]}", null);
                };
                case (_) {
                    #ok("{\"primer\": \"KIP v1.0 - Knowledge Interaction Protocol. Available commands: FIND, UPSERT, DELETE, DESCRIBE, SEARCH\"}", null);
                };
            };
        };

        func executeSearchCommand(cmd : SearchCommand) : KipResult {
            // Execute search via callback
            let ids = searcher(cmd.searchQuery);

            // Fetch full entities
            var entities : [Graph.Entity] = [];
            for (id in ids.vals()) {
                switch (capabilityExecutor.getEntity(id)) {
                    case (?e) { entities := Array.append(entities, [e]) };
                    case (null) {};
                };
            };

            #ok(entitiesToJson(entities, entities.size()), null);
        };

        // Helper: Format entities as JSON
        func entitiesToJson(entities : [Graph.Entity], totalCount : Int) : Text {
            var json = "{\"results\": [";
            var first = true;

            for (e in entities.vals()) {
                if (not first) { json #= ", " };
                json #= "{";
                json #= "\"id\": \"" # e.id # "\", ";
                json #= "\"name\": \"" # e.name # "\", ";
                json #= "\"type\": \"" # entityTypeToText(e.entityType) # "\", ";
                json #= "\"description\": \"" # escapeJson(e.description) # "\", ";
                json #= "\"attributes\": " # attributesToJson(e.attributes);
                json #= "}";
                first := false;
            };

            // Add metadata
            json #= "], \"meta\": {";
            json #= "\"total\": " # Int.toText(totalCount);
            json #= ", \"count\": " # Int.toText(entities.size());
            json #= "}}";
            json;
        };

        // Helper: Convert attributes to JSON object
        func attributesToJson(attrs : [(Text, Text)]) : Text {
            var json = "{";
            var first = true;
            for ((k, v) in attrs.vals()) {
                if (not first) { json #= ", " };
                json #= "\"" # k # "\": \"" # escapeJson(v) # "\"";
                first := false;
            };
            json #= "}";
            json;
        };

        // Helper: Escape JSON special characters
        func escapeJson(s : Text) : Text {
            var result = "";
            for (c in s.chars()) {
                if (c == '\"') {
                    result #= "\\\"";
                } else if (c == '\\') {
                    result #= "\\\\";
                } else if (c == '\n') {
                    result #= "\\n";
                } else {
                    result #= Text.fromChar(c);
                };
            };
            result;
        };

        // Removed getters as they relied on array storage
    };

};
