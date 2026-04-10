export const idlFactory = ({ IDL }) => {
  const ChatMessage = IDL.Record({
    msg_type: IDL.Text,
    content: IDL.Text,
    conversation_id: IDL.Opt(IDL.Text),
  });
  const WsClientKey = IDL.Record({
    client_principal: IDL.Principal,
    client_nonce: IDL.Nat64,
  });
  const WsResult = IDL.Variant({ Ok: IDL.Null, Err: IDL.Text });
  const WsMessageRecord = IDL.Record({
    client_key: WsClientKey,
    sequence_num: IDL.Nat64,
    timestamp: IDL.Nat64,
    is_service_message: IDL.Bool,
    content: IDL.Vec(IDL.Nat8),
  });

  return IDL.Service({
    ws_open: IDL.Func(
      [IDL.Record({ client_nonce: IDL.Nat64, gateway_principal: IDL.Principal })],
      [WsResult],
      []
    ),
    ws_close: IDL.Func([IDL.Record({ client_key: WsClientKey })], [WsResult], []),
    ws_message: IDL.Func(
      [WsMessageRecord, IDL.Opt(ChatMessage)],
      [WsResult],
      []
    ),
    ws_get_messages: IDL.Func(
      [IDL.Record({ nonce: IDL.Nat64 })],
      [
        IDL.Record({
          messages: IDL.Vec(WsMessageRecord),
          cert: IDL.Vec(IDL.Nat8),
          tree: IDL.Vec(IDL.Nat8),
        }),
      ],
      ["query"]
    ),
  });
};

export const init = ({ IDL }) => [];
