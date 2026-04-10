import Time "mo:base/Time";
import Auth "auth";

module {

    // -- Types --

    public type Space = {
        id : Text;
        name : Text;
        description : Text;
        visibility : Visibility;
        owner : Auth.ActorID;
        roles : [Role];
        members : [Member];
        createdAt : Int;
        source : ?ForkSource; // Lineage Preservation (§7)
        locusId : ?Text; // Spatial Origin
    };

    public type ForkSource = {
        spaceId : Text;
        blockHeight : Int; // Logical time of fork
        actorId : Auth.ActorID; // Who performed the fork
    };

    public type Visibility = { #public_; #private_; #member_only };

    public type Role = {
        id : Text;
        name : Text;
        permissions : [Permission];
    };

    public type Permission = {
        #manage_space; // Update settings, delete space
        #manage_members; // Add/remove members, assign roles
        #manage_workflow; // Create/Edit/Delete workflow definitions
        #trigger_step; // Trigger workflow transitions
        #view_private; // View private contributions
        #create_contribution; // Post new items (ideas, feedback)
    };

    public type Member = {
        actorId : Auth.ActorID;
        roleIds : [Text];
        joinedAt : Int;
    };

    // -- Default Roles --

    public let OWNER_ROLE_ID = "owner";
    public let ADMIN_ROLE_ID = "admin";
    public let MEMBER_ROLE_ID = "member";

    public func defaultRoles() : [Role] {
        return [
            {
                id = OWNER_ROLE_ID;
                name = "Owner";
                permissions = [#manage_space, #manage_members, #manage_workflow, #trigger_step, #view_private, #create_contribution];
            },
            {
                id = ADMIN_ROLE_ID;
                name = "Admin";
                permissions = [#manage_members, #manage_workflow, #trigger_step, #view_private, #create_contribution];
            },
            {
                id = MEMBER_ROLE_ID;
                name = "Member";
                permissions = [#create_contribution, #view_private];
            },
        ];
    };

    // -- Logic --

    public func create(name : Text, description : Text, visibility : Visibility, owner : Auth.ActorID) : Space {
        let members : [Member] = [{
            actorId = owner;
            roleIds = [OWNER_ROLE_ID];
            joinedAt = Time.now();
        }];

        return {
            id = "space_" # owner; // Simple ID generation for MVP (ActorID is already Text)
            name = name;
            description = description;
            visibility = visibility;
            owner = owner;
            roles = defaultRoles();
            members = members;
            createdAt = Time.now();
            source = null;
            locusId = null;
        };
    };

    public func hasPermission(space : Space, caller : Auth.ActorID, requiredPerm : Permission) : Bool {
        // 1. Find member
        var memberRoles : [Text] = [];
        label findMember {
            for (m in space.members.vals()) {
                if (m.actorId == caller) {
                    memberRoles := m.roleIds;
                    break findMember;
                };
            };
        };

        // 2. Check roles for permission
        for (roleId in memberRoles.vals()) {
            for (role in space.roles.vals()) {
                if (role.id == roleId) {
                    for (perm in role.permissions.vals()) {
                        if (perm == requiredPerm) {
                            return true;
                        };
                    };
                };
            };
        };

        return false;
    };

};
