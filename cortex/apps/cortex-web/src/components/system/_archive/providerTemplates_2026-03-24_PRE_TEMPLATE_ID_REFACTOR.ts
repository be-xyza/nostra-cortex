import type { LlmProviderType, ProviderType } from "../../contracts.ts";

export type ProviderTemplateId = "openrouter" | "openai" | "ollama" | "manual";

export interface ProviderTemplate {
  id: ProviderTemplateId;
  label: string;
  description: string;
  providerType: ProviderType;
  providerKind?: LlmProviderType;
  defaultEndpoint: string;
  endpointLabel: string;
  endpointHint: string;
  defaultModelHint: string;
  validateKey: boolean;
  validateChat: boolean;
  validateEmbeddings: boolean;
  isOpenAiCompatible: boolean;
}

export const providerTemplates: ProviderTemplate[] = [
  {
    id: "openrouter",
    label: "OpenRouter",
    description: "OpenAI-compatible router with a shared model catalog and a key status endpoint.",
    providerType: "Llm",
    providerKind: "OpenRouter",
    defaultEndpoint: "https://openrouter.ai/api/v1",
    endpointLabel: "OpenRouter base URL",
    endpointHint: "https://openrouter.ai/api/v1",
    defaultModelHint: "Choose from the discovered catalog after validation",
    validateKey: true,
    validateChat: true,
    validateEmbeddings: true,
    isOpenAiCompatible: true,
  },
  {
    id: "openai",
    label: "OpenAI",
    description: "Hosted OpenAI-compatible endpoint with direct model discovery.",
    providerType: "Llm",
    providerKind: "OpenAI",
    defaultEndpoint: "https://api.openai.com/v1",
    endpointLabel: "OpenAI base URL",
    endpointHint: "https://api.openai.com/v1",
    defaultModelHint: "Choose from the discovered catalog after validation",
    validateKey: false,
    validateChat: true,
    validateEmbeddings: true,
    isOpenAiCompatible: true,
  },
  {
    id: "ollama",
    label: "Local OpenAI-compatible",
    description: "Local or tunneled runtime that speaks the OpenAI chat and models surface.",
    providerType: "Llm",
    providerKind: "Ollama",
    defaultEndpoint: "http://127.0.0.1:11434/v1",
    endpointLabel: "Runtime base URL",
    endpointHint: "http://127.0.0.1:11434/v1",
    defaultModelHint: "Choose from the discovered catalog after validation",
    validateKey: false,
    validateChat: true,
    validateEmbeddings: true,
    isOpenAiCompatible: true,
  },
  {
    id: "manual",
    label: "Custom provider",
    description: "Operator-owned provider record with custom routing and metadata.",
    providerType: "Llm",
    defaultEndpoint: "",
    endpointLabel: "Endpoint",
    endpointHint: "https://example.com/v1",
    defaultModelHint: "Enter a custom model name",
    validateKey: false,
    validateChat: false,
    validateEmbeddings: false,
    isOpenAiCompatible: false,
  },
];

export function slugifyProviderName(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

export function resolveProviderTemplate(
  providerKind?: string | null,
  endpoint?: string | null,
): ProviderTemplate {
  const normalizedKind = providerKind?.trim().toLowerCase();
  if (normalizedKind) {
    const byKind = providerTemplates.find((template) => template.providerKind?.toLowerCase() === normalizedKind);
    if (byKind) {
      return byKind;
    }
  }

  const normalizedEndpoint = endpoint?.trim().toLowerCase() ?? "";
  if (normalizedEndpoint.includes("openrouter")) {
    return providerTemplates[0];
  }
  if (normalizedEndpoint.includes("api.openai.com")) {
    return providerTemplates[1];
  }
  if (normalizedEndpoint.includes("localhost") || normalizedEndpoint.includes("127.0.0.1")) {
    return providerTemplates[2];
  }

  return providerTemplates[3];
}

export function providerTemplateById(templateId: ProviderTemplateId): ProviderTemplate {
  return providerTemplates.find((template) => template.id === templateId) ?? providerTemplates[3];
}
