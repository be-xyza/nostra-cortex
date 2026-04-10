// Polyfill process for browser compatibility
window.process = { env: { NODE_ENV: 'production' } };

import { IcWebSocket, generateRandomIdentity } from "ic-websocket-js";
import { Actor, HttpAgent } from "@dfinity/agent";

// Minimal IDL for WebSocket
const idlFactory = ({ IDL }) => {
    const ChatMessage = IDL.Record({
        'content': IDL.Text,
        'msg_type': IDL.Text,
        'conversation_id': IDL.Opt(IDL.Text),
    });

    return IDL.Service({
        'ws_close': IDL.Func(
            [
                IDL.Record({
                    'client_key': IDL.Record({
                        'client_principal': IDL.Principal,
                        'client_nonce': IDL.Nat64,
                    }),
                }),
            ],
            [IDL.Variant({ 'Ok': IDL.Null, 'Err': IDL.Text })],
            [],
        ),
        'ws_get_messages': IDL.Func(
            [IDL.Record({ 'nonce': IDL.Nat64 })],
            [
                IDL.Record({
                    'cert': IDL.Vec(IDL.Nat8),
                    'tree': IDL.Vec(IDL.Nat8),
                    'messages': IDL.Vec(
                        IDL.Record({
                            'content': IDL.Vec(IDL.Nat8),
                            'client_key': IDL.Record({
                                'client_principal': IDL.Principal,
                                'client_nonce': IDL.Nat64,
                            }),
                            'is_service_message': IDL.Bool,
                            'timestamp': IDL.Nat64,
                            'sequence_num': IDL.Nat64,
                        })
                    ),
                }),
            ],
            ['query'],
        ),
        'ws_message': IDL.Func(
            [
                IDL.Record({
                    'content': IDL.Vec(IDL.Nat8),
                    'client_key': IDL.Record({
                        'client_principal': IDL.Principal,
                        'client_nonce': IDL.Nat64,
                    }),
                    'is_service_message': IDL.Bool,
                    'timestamp': IDL.Nat64,
                    'sequence_num': IDL.Nat64,
                }),
                IDL.Opt(ChatMessage),
            ],
            [IDL.Variant({ 'Ok': IDL.Null, 'Err': IDL.Text })],
            [],
        ),
        'ws_open': IDL.Func(
            [
                IDL.Record({
                    'gateway_principal': IDL.Principal,
                    'client_nonce': IDL.Nat64,
                }),
            ],
            [IDL.Variant({ 'Ok': IDL.Null, 'Err': IDL.Text })],
            [],
        ),
    });
};

let ws = null;
let identity = null;

let connectionAttempted = false;
let streamingAvailable = false;

// Helper function to establish the actual IC WebSocket connection
function establishConnection(gatewayUrl, canisterId, actor, icUrl) {
    const isLocal = icUrl.includes("127.0.0.1") || icUrl.includes("localhost");
    ws = new IcWebSocket(gatewayUrl, undefined, {
        canisterId: canisterId,
        canisterActor: actor,
        networkUrl: icUrl,
        identity: identity,
        localTest: isLocal,
    });

    ws.onopen = () => {
        console.log("[NostraStreaming] Connected!");
        streamingAvailable = true;
        if (window.onNostraStateChange) window.onNostraStateChange("CONNECTED");
    };

    ws.onmessage = (event) => {
        const msg = event.data;
        console.log("[NostraStreaming] Message:", msg);
        if (window.onNostraMessage) {
            window.onNostraMessage(msg);
        }
    };

    ws.onclose = () => {
        console.log("[NostraStreaming] Closed");
        streamingAvailable = false;
        if (window.onNostraStateChange) window.onNostraStateChange("DISCONNECTED");
    };

    ws.onerror = (err) => {
        console.error("[NostraStreaming] Error:", err);
        streamingAvailable = false;
        if (window.onNostraStateChange) window.onNostraStateChange("ERROR: " + err);
    };
}

window.NostraStreaming = {
    isConnected: () => streamingAvailable,

    connect: async (gatewayUrl, canisterId, icUrl) => {
        // Prevent multiple connection attempts
        if (connectionAttempted) {
            console.log("[NostraStreaming] Connection already attempted, skipping...");
            return;
        }
        connectionAttempted = true;

        console.log("[NostraStreaming] Initializing connection...");

        // Generate identity if not exists (in prod, use auth client)
        if (!identity) {
            identity = generateRandomIdentity();
            console.log("[NostraStreaming] Generated temporary identity:", identity.getPrincipal().toText());
        }

        const httpAgent = new HttpAgent({ host: icUrl, identity });
        if (icUrl.includes("127.0.0.1") || icUrl.includes("localhost")) {
            await httpAgent.fetchRootKey();
        }

        const actor = Actor.createActor(idlFactory, {
            agent: httpAgent,
            canisterId: canisterId,
        });

        console.log("[NostraStreaming] Connecting to Gateway:", gatewayUrl);

        // Check if gateway is reachable first (with timeout)
        try {
            const wsCheck = new WebSocket(gatewayUrl);
            const connectionTimeout = setTimeout(() => {
                wsCheck.close();
                console.warn("[NostraStreaming] Gateway not available at", gatewayUrl, "- streaming disabled. Start the IC WebSocket Gateway to enable real-time features.");
                if (window.onNostraStateChange) window.onNostraStateChange("UNAVAILABLE");
            }, 3000);

            wsCheck.onopen = () => {
                clearTimeout(connectionTimeout);
                wsCheck.close();
                // Gateway is reachable, now establish the real connection
                establishConnection(gatewayUrl, canisterId, actor, icUrl);
            };

            wsCheck.onerror = () => {
                clearTimeout(connectionTimeout);
                console.warn("[NostraStreaming] Gateway not available at", gatewayUrl, "- streaming disabled. Chat will use HTTP fallback.");
                if (window.onNostraStateChange) window.onNostraStateChange("UNAVAILABLE");
            };
        } catch (e) {
            console.warn("[NostraStreaming] Failed to check gateway availability:", e.message || e);
            if (window.onNostraStateChange) window.onNostraStateChange("UNAVAILABLE");
        }
    },

    send: (content, conversationId) => {
        if (!ws || !streamingAvailable) {
            console.warn("[NostraStreaming] Streaming not available - message not sent. Use HTTP fallback for chat.");
            return false;
        }

        // Message structure matching backend ChatMessage: { msg_type, content, conversation_id }
        const msg = {
            msg_type: "user_message",
            content: content,
            conversation_id: conversationId ? [conversationId] : []
        };

        try {
            ws.send(msg);
            return true;
        } catch (e) {
            console.error("[NostraStreaming] Send failed:", e);
            return false;
        }
    }
};
