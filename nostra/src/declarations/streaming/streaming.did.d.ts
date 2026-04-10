import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ChatMessage {
  msg_type: string;
  content: string;
  conversation_id: [] | [string];
}

export interface WsClientKey {
  client_principal: Principal;
  client_nonce: bigint;
}

export interface WsMessageRecord {
  client_key: WsClientKey;
  sequence_num: bigint;
  timestamp: bigint;
  is_service_message: boolean;
  content: Uint8Array | number[];
}

export type WsResult = { Ok: null } | { Err: string };

export interface WsGetMessagesResult {
  messages: Array<WsMessageRecord>;
  cert: Uint8Array | number[];
  tree: Uint8Array | number[];
}

export interface _SERVICE {
  ws_open: ActorMethod<[{ client_nonce: bigint; gateway_principal: Principal }], WsResult>;
  ws_close: ActorMethod<[{ client_key: WsClientKey }], WsResult>;
  ws_message: ActorMethod<[WsMessageRecord, [] | [ChatMessage]], WsResult>;
  ws_get_messages: ActorMethod<[{ nonce: bigint }], WsGetMessagesResult>;
}

export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
