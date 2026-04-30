import type { InternetIdentityDelegationProof } from "../../contracts.ts";

const DEFAULT_IDENTITY_PROVIDER = "https://id.ai/authorize";
const OPERATOR_AUTH_FLAG = "VITE_II_OPERATOR_AUTH_ENABLED";
const IDENTITY_PROVIDER_FLAG = "VITE_II_PROVIDER_URL";

function env(): Record<string, string | undefined> {
  return ((import.meta as unknown as { env?: Record<string, string | undefined> }).env ?? {});
}

export function isInternetIdentityOperatorLoginEnabled(): boolean {
  const configured = env()[OPERATOR_AUTH_FLAG]?.trim().toLowerCase();
  if (configured) {
    return ["1", "true", "yes", "on"].includes(configured);
  }
  return false;
}

export function internetIdentityProviderUrl(): string {
  return env()[IDENTITY_PROVIDER_FLAG]?.trim() || DEFAULT_IDENTITY_PROVIDER;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

export async function createInternetIdentityDelegationProof(): Promise<InternetIdentityDelegationProof> {
  const { AuthClient } = await import("@icp-sdk/auth/client");
  const provider = internetIdentityProviderUrl();
  const authClient = new AuthClient({ identityProvider: provider });

  const identity = authClient.isAuthenticated()
    ? await authClient.getIdentity()
    : await authClient.signIn({
        maxTimeToLive: BigInt(8 * 60 * 60 * 1_000_000_000),
      });
  const delegationIdentity = identity as {
    getDelegation?: () => unknown;
    getPublicKey?: () => { toDer?: () => ArrayBuffer | Uint8Array };
  };
  const publicKeyDer = delegationIdentity.getPublicKey?.().toDer?.();
  if (!delegationIdentity.getDelegation || !publicKeyDer) {
    throw new Error("Internet Identity did not return a delegation proof.");
  }

  return {
    principal: identity.getPrincipal().toText(),
    identityProvider: provider,
    publicKeyDer: bytesToBase64(new Uint8Array(publicKeyDer)),
    delegation: delegationIdentity.getDelegation(),
    signedAt: new Date().toISOString(),
  };
}
