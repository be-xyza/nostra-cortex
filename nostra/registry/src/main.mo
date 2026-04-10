import Principal "mo:base/Principal";
import Result "mo:base/Result";
import Array "mo:base/Array";
import Time "mo:base/Time";
import Text "mo:base/Text";
import Int "mo:base/Int";
import Nat "mo:base/Nat";

persistent actor Registry {

    // -------------------------------------------------------------------------
    // Types
    // -------------------------------------------------------------------------

    public type SpaceId = Text;
    public type Timestamp = Int;

    public type SpaceRecord = {
        id: SpaceId;
        name: Text;
        owner: Principal;
        dataCanisterId: ?Principal;
        createdAt: Timestamp;
        metadata: [(Text, Text)];
    };

    // -------------------------------------------------------------------------
    // State
    // -------------------------------------------------------------------------

    // Use a simple list for MVP. In production, use a Map or Trie.
    var spaces : [(SpaceId, SpaceRecord)] = [];

    // -------------------------------------------------------------------------
    // Private Helpers
    // -------------------------------------------------------------------------

    private func generateId() : Text {
        // Simple distinct ID for MVP
        let time = Time.now();
        let size = spaces.size();
        "space-" # Int.toText(time) # "-" # Nat.toText(size)
    };

    private func findSpaceIndex(id: SpaceId) : ?Nat {
        var i : Nat = 0;
        for ((k, _) in spaces.vals()) {
            if (k == id) {
                return ?i;
            };
            i += 1;
        };
        null
    };

    // -------------------------------------------------------------------------
    // Public Query API
    // -------------------------------------------------------------------------

    public query func getSpace(id: SpaceId) : async ?SpaceRecord {
        for ((k, v) in spaces.vals()) {
            if (k == id) {
                return ?v;
            };
        };
        null
    };

    public query func listSpaces(ownerFilter: ?Principal) : async [SpaceRecord] {
        var result : [SpaceRecord] = [];
        for ((_, v) in spaces.vals()) {
            switch(ownerFilter) {
                case (null) {
                    result := Array.append(result, [v]);
                };
                case (?owner) {
                    if (v.owner == owner) {
                        result := Array.append(result, [v]);
                    };
                };
            };
        };
        result
    };

    // -------------------------------------------------------------------------
    // Public Update API
    // -------------------------------------------------------------------------

    public shared({ caller }) func registerSpace(name: Text, metadata: [(Text, Text)]) : async Result.Result<SpaceId, Text> {
        let newId = generateId();
        let record : SpaceRecord = {
            id = newId;
            name = name;
            owner = caller;
            dataCanisterId = null;
            createdAt = Time.now();
            metadata = metadata;
        };

        // Correct append syntax
        spaces := Array.append(spaces, [(newId, record)]);
        #ok(newId)
    };

    public shared({ caller }) func updateSpace(id: SpaceId, updates: SpaceRecord) : async Result.Result<(), Text> {
        // Check existence
        switch(findSpaceIndex(id)) {
            case (null) {
                return #err("Space not found");
            };
            case (?index) {
                let current = spaces[index].1;
                
                // Authorization check
                if (current.owner != caller) {
                    return #err("Unauthorized");
                };

                // Apply update - NOTE: In a real implementation, 'updates' might be a partial type
                // Here we replace the record but preserve immutable fields if needed
                // For MVP, we trust the 'updates' object but ensure ID matches
                if (updates.id != id) {
                    return #err("ID mismatch");
                };

                // Reconstruct to ensure owner doesn't change implicitly (unless intended)
                // For now, allow full overwrite if auth passes
                var newSpaces = Array.thaw<(SpaceId, SpaceRecord)>(spaces);
                newSpaces[index] := (id, updates);
                spaces := Array.freeze(newSpaces);
                
                #ok(())
            };
        };
    };

    public shared({ caller }) func linkDataCanister(spaceId: SpaceId, canisterId: Principal) : async Result.Result<(), Text> {
        switch(findSpaceIndex(spaceId)) {
            case (null) {
                return #err("Space not found");
            };
            case (?index) {
                let current = spaces[index].1;
                if (current.owner != caller) {
                    return #err("Unauthorized");
                };

                let updatedRecord = {
                    id = current.id;
                    name = current.name;
                    owner = current.owner;
                    dataCanisterId = ?canisterId; // Link!
                    createdAt = current.createdAt;
                    metadata = current.metadata;
                };

                var newSpaces = Array.thaw<(SpaceId, SpaceRecord)>(spaces);
                newSpaces[index] := (spaceId, updatedRecord);
                spaces := Array.freeze(newSpaces);

                #ok(())
            };
        };
    };
};
