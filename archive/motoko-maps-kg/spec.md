# Motoko Maps Knowledge Graph Demo with AI Integration and Chat Interface

## Overview
A demonstration application showcasing Motoko Map data structures for building entity-based knowledge bases and relationship graphs with integrated AI agent capabilities and conversational chat interface. The application allows users to create entities with metadata and establish connections between them, forming a comprehensive knowledge graph. The application comes pre-loaded with a complete Internet Computer Protocol (ICP) knowledge base as the default dataset and includes secure AI integration for enhanced knowledge management. Users can interact with the knowledge base through a conversational chat interface powered by AI agents.

## Backend Functionality

### Data Storage
The backend stores five primary data structures in stable memory:
- **Entity Map**: Stores entities with unique identifiers, names, descriptions, entity types, tags, timestamps, and creator principals
- **Relationship Map**: Stores directional relationships between entities as adjacency lists with creator principals
- **Encrypted API Keys Map**: Stores encrypted external API keys per principal using canister-generated encryption keys in a `Map<Principal, Map<Text, Text>>` structure
- **Chat Conversations Map**: Stores chat conversations per principal with message history, timestamps, and context persistence
- **Access Control State**: Stores admin and user principals with authentication and authorization data
- **Initialization Flag**: Stores a boolean flag in stable memory to track whether the canister has been initialized

### Core Operations
- **Add Entity**: Create new entities with id, name, description, entity type, tags, timestamp, and caller principal as creator
- **Add Relationship**: Establish directional connections between entities with caller principal as creator
- **Get Entity**: Retrieve entity details by ID
- **Get Relations**: Fetch all entities connected to a given entity
- **Query by Tag**: Find all entities that contain a specific tag
- **Is Initialized**: Query function to confirm when the canister's knowledge graph and admin setup are complete

### Enhanced Initialization Process
The backend implements a comprehensive initialization system that automatically sets up the canister upon first deployment:

#### Automatic Deployment Detection and Admin Setup
- **Deploying Principal Detection**: Automatically detect the deploying principal upon first deployment
- **Admin Registration**: Register the deploying principal as both admin and user via the AccessControl system
- **Access Control Initialization**: Call `_initializeAccessControlWithSecret` or similar initialization route with a generated secret token if not yet initialized
- **Single Execution Guarantee**: Store an initialization flag in stable memory to prevent reinitialization on subsequent upgrades

#### Knowledge Graph Seeding with Admin Principal
- **Dataset Creation**: After assigning the admin principal, call `createEntitiesAndRelationships(caller)` to seed the full ICP knowledge graph dataset
- **Principal Assignment**: All seeded entities and relationships are created with the deploying principal attached as `creatorPrincipal`
- **Core Entity Verification**: Verify that the initialization process successfully creates core entities (`icp_protocol`, `nns`, `canisters`, `cycles`, `internet_identity`)

#### Stable Memory Persistence and Validation
- **Automatic Execution**: The initialization runs at deployment automatically without manual intervention
- **Persistent Storage**: All mappings (`entityMap`, `relationshipMap`, `accessControlState`) persist properly to stable memory to prevent reset on canister redeployments
- **Initialization Validation**: The `isInitialized` query function confirms when the canister's knowledge graph and admin setup are complete
- **Upgrade Safety**: The initialization flag prevents duplicate setup during canister upgrades while maintaining all existing data

### Chat Management Functions
The backend provides secure chat conversation management with authentication and context persistence:

#### Chat Operations
- **Save Chat Message**: Store user messages and AI responses with timestamps and conversation context
- **Get Chat History**: Retrieve complete conversation history for authenticated users
- **Process AI Query**: Handle user queries through AI agents using stored API keys and knowledge graph data
- **Clear Chat History**: Allow users to clear their conversation history
- **Get Chat Context**: Retrieve conversation context for maintaining continuity across interactions

#### Chat Data Structure
- Chat conversations are stored per principal with message arrays containing user queries and AI responses
- Each message includes content, timestamp, message type (user/ai), and conversation context
- Context persistence maintains conversation flow and reference to previous interactions
- All chat data is linked to authenticated user principals for security and privacy

### Public API Key Management Functions
The backend provides secure API key management with authorization checks and encrypted storage through the following public functions in the main actor interface:

#### Public API Key Operations
- **`public func saveApiKey(serviceName: Text, apiKey: Text): async ()`**: Store or update an encrypted API key for a specific service, associated with the caller's principal identity
- **`public func getApiKey(serviceName: Text): async ?Text`**: Retrieve and decrypt an API key for a specific service for the authenticated caller
- **`public func deleteApiKey(serviceName: Text): async ()`**: Remove an API key for a specific service from the caller's stored keys
- **`public func listApiKeys(): async [Text]`**: Return a list of service names for which the caller has stored API keys (without exposing the actual keys)
- **`public func isApiKeyIntegrationReady(): async Bool`**: Returns true to confirm full backend integration is available and suppress "Backend Integration Required" messages
- **`public func isInitialized(): async Bool`**: Query function to confirm when the canister's knowledge graph and admin setup are complete

#### Security and Authorization
- **Principal-based Access Control**: Each function includes authorization checks to ensure only the authenticated caller can access or modify their own API keys and chat data
- **Encrypted Storage**: API keys are encrypted using a lightweight reversible algorithm before storage in stable memory
- **Secure Key Structure**: Keys are stored in a nested map structure `Map<Principal, Map<Text, Text>>` where the outer key is the principal and inner map contains service names mapped to encrypted API keys
- **Unique Caller Principal Encryption**: Encryption/decryption logic ensures key confidentiality is linked to the unique caller principal

#### Encryption and Storage
- API keys are encrypted using canister-generated encryption keys unique to each principal
- Encrypted keys are stored in stable memory to persist across canister upgrades
- The encryption algorithm is lightweight and reversible to allow proper decryption when keys are retrieved
- Storage structure maintains separation between different principals' API keys and chat data
- All API key management functions are exported as public functions in the main actor interface for frontend detection and integration

### AI Agent Integration with Chat Support
- **Query Interpretation**: Process natural language queries from chat interface to extract entities and relationships
- **Schema Optimization**: Analyze knowledge graph structure and suggest improvements through conversational responses
- **Knowledge Synchronization**: Sync data with external knowledge sources and report results in chat
- **Visualization Assistance**: Generate optimal graph layouts and visual representations based on chat requests
- **Conversational Responses**: Generate contextual AI responses based on knowledge graph data and user queries
- **Context Awareness**: Maintain conversation context across multiple interactions for coherent responses

### Stable Memory Persistence
All entity, relationship, encrypted API key, chat conversation, access control, and initialization flag data is stored in stable memory to ensure:
- Data persists across canister upgrades and redeployments
- Permanent storage of the comprehensive ICP knowledge base
- Creator principal information is maintained with proper assignment to deploying principal
- One-time initialization setup is preserved with verification
- Guaranteed persistence through atomic operations and verification checks
- Secure storage of encrypted API keys per principal in the nested map structure
- Chat conversation history persistence across sessions and canister upgrades
- Access control state and admin assignments persist across upgrades
- Initialization flag prevents duplicate setup while maintaining data integrity

### Pre-loaded Comprehensive ICP Knowledge Base
The backend includes an extensive base dataset representing the Internet Computer Protocol ecosystem, permanently stored during canister initialization with guaranteed deploying principal assignment:

#### Core Architecture Entities
- Internet Computer Protocol (ICP), Canisters, Subnets, Nodes, Network Nervous System (NNS), ICP Token, Cycles, Internet Identity, Neuron, DFX SDK, Motoko Language, Ledger Canister, Smart Contracts, Threshold ECDSA, Chain Key Cryptography, Boundary Nodes, Service Canisters

#### Extended ICP Ecosystem Entities
The knowledge base includes all extended ICP-related entities with guaranteed creation during initialization:
- Governance systems, protocols, crypto assets, development tools, infrastructure components, and other ICP ecosystem elements
- Each entity includes its proper type classification and comprehensive descriptions
- Entity descriptions incorporate detailed metadata and notes about functionality, purpose, and relationships within the ICP ecosystem
- All entities are tagged with appropriate categories for easy discovery and filtering
- Every extended entity is created with the deploying principal assignment and verified for persistence

#### Comprehensive Relationship Network
The system creates an extensive network of relationships with guaranteed persistence:
- All original and extended relationships are created during initialization
- Bidirectional and directional relationships with descriptive names are properly established
- All relationship links maintain proper naming, timestamps, and deploying principal assignment
- Relationships are verified for successful creation and storage in stable memory

## Frontend Features

### Enhanced Backend Initialization with Validation
- `useInitializeBackend` hook that calls the backend `initialize()` function and waits for completion
- Integration with the new `isInitialized()` query function to confirm canister setup completion
- Comprehensive verification that all entities and relationships are available after deployment
- Polling mechanism to confirm backend state reflects the complete dataset
- Error handling and retry logic for initialization failures
- Loading states that accurately reflect initialization progress
- Validation that extended ICP entities and relationships are properly loaded from stable memory
- **Persistence Detection**: The frontend properly detects and reports persisted dataset availability once initialization completes
- **Data Validation**: Confirms that entities created by `createEntitiesAndRelationships` are accessible and properly stored
- **Admin Setup Validation**: Confirms that access control and admin principal assignment are complete

### Entity Management
- Form to add new entities with name, description, entity type, and tags
- Form to create relationships between existing entities
- Display list of all entities in a table format showing entity types

### Chat Interface
- Interactive chat interface for querying and interacting with the knowledge base
- Real-time message history display showing user queries and AI responses
- Input box for typing natural language queries and commands
- Conversational format with clear distinction between user messages and AI responses
- Message timestamps and conversation flow indicators
- Context persistence across chat sessions for authenticated users
- Loading indicators during AI processing
- Error handling for failed queries or AI service issues

### Chat Functionality
- Send queries to AI agents for knowledge graph analysis and responses
- Receive contextual responses based on entities, relationships, and knowledge base content
- Maintain conversation context across multiple interactions
- Clear chat history option for users
- Authentication integration ensuring chat sessions are linked to Internet Identity users
- Secure handling of AI responses without exposing API keys in frontend

### Visualization
- Dynamic display of entities and relationships loaded from persisted backend data
- Simple table view showing entities and their properties including entity types, timestamps and creators
- Basic relationship display showing connections between entities
- Tag-based filtering to find related entities
- Real-time loading of the comprehensive ICP knowledge graph from stable memory

### 3D Graph Visualization with Billboarding
- Interactive 3D graph visualization of entities and relationships
- Toggle option to enable/disable billboarding effect for entity name labels
- When billboarding is active, entity name text always faces the camera for optimal readability
- Smooth performance during camera movements and graph rotations
- Toggle button integrated into the GraphVisualization component interface
- Synchronization between billboarding state and camera updates

### AI Agent Management Interface
- Protected settings panel for API key management accessible only to authenticated users
- Add, view, and remove external API keys for different AI services through the AISettingsPanel
- Associate API keys with specific AI functions (query interpretation, schema optimization, knowledge synchronization, visualization assistance)
- Display configured AI agents, their statuses, and available functions
- Secure form handling that never exposes raw API keys in the frontend
- Agent configuration interface for enabling/disabling specific AI capabilities
- Integration with backend API key management functions for secure storage and retrieval
- Backend integration validation that removes "Backend Integration Required" message once API key endpoints are detected and functional

### Secure API Key Management with Frontend Integration
- Protected settings interface for managing external API keys through secure backend calls
- Add API keys for different services with descriptive labels using public `saveApiKey` function
- View list of configured API services without exposing actual keys using public `listApiKeys` function
- Remove API keys when no longer needed using public `deleteApiKey` function
- Retrieve API keys securely when needed using public `getApiKey` function
- Check backend integration status using public `isApiKeyIntegrationReady` function
- Check initialization status using public `isInitialized` function
- All API key operations are performed through secure backend calls with proper authorization
- Frontend never stores or displays raw API keys
- Integration with `useQueries.ts` hooks for seamless backend communication
- Automatic suppression of "Backend Integration Required" messages when `isApiKeyIntegrationReady` returns true

### Interactive Features
- Click on entity to view its details, type, and connections
- Search entities by tag or type
- Visual representation of the comprehensive knowledge graph structure
- Browse the expanded ICP ecosystem entities and relationships with creator information
- AI-powered query interpretation for natural language searches through chat interface
- AI-assisted schema optimization suggestions via conversational responses
- AI-enhanced visualization recommendations through chat interactions
- Context-aware conversations that reference previous queries and responses

## User Interface
The interface provides a clean, functional design focused on demonstrating the Map data structures with the comprehensive Internet Computer Protocol knowledge base as the primary example, enhanced with AI integration capabilities and conversational chat interface. The UI dynamically displays entities and relationships loaded from persisted backend data, ensuring that the expanded ICP knowledge network is properly retrieved from stable memory. Users can explore the comprehensive ICP ecosystem in both 2D table format and 3D graph visualization, add new entities with proper typing, create relationships, and extend the existing knowledge graph through simple interactions. The 3D graph includes a billboarding toggle that ensures entity labels remain readable from any viewing angle. The application includes a protected settings panel (AISettingsPanel) for secure API key management, allowing users to configure AI agents for enhanced knowledge graph operations through the secure backend API key management functions. The chat interface provides an intuitive conversational experience where users can query the knowledge base using natural language, receive AI-powered responses, and maintain context across interactions. Chat sessions are linked to authenticated users via Internet Identity and persist across browser sessions. The application automatically initializes the backend upon deployment with enhanced verification that confirms all extended entities and relationships are correctly persisted and displayed, including proper deploying principal assignment, access control setup, and data availability validation. The frontend properly detects when the persisted dataset and admin setup are complete using the `isInitialized` query function and reports successful initialization completion. All AI agent interactions are routed through the frontend to ensure secure API key handling without exposing sensitive credentials, utilizing the backend's encrypted storage and authorization system. The AI settings panel validates backend integration through the `isApiKeyIntegrationReady` endpoint and automatically removes "Backend Integration Required" messages once the public API key management endpoints are properly detected and functional. The frontend integrates seamlessly with the backend through `useQueries.ts` hooks that connect to all public API key management functions, chat operations, and initialization validation. All content is displayed in English.
