import { getApiErrorMessage } from '../services/errors';
import type { RouteProfile, RouteRule, RoutingConfig } from '../types/backend';

export function cloneRoutingConfig(config: RoutingConfig): RoutingConfig {
  return JSON.parse(JSON.stringify(config)) as RoutingConfig;
}

export function prettyJson(value: unknown) {
  return JSON.stringify(value, null, 2);
}

export function parseCsv(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

export function parseHeaders(value: string): Record<string, string[]> | undefined {
  const lines = value
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean);

  if (lines.length === 0) {
    return undefined;
  }

  return Object.fromEntries(
    lines.map((line) => {
      const [name, rawValues = ''] = line.split(':');
      return [
        name.trim(),
        rawValues
          .split(',')
          .map((item) => item.trim())
          .filter(Boolean),
      ];
    }),
  );
}

export function headersToDraft(headers?: Record<string, string[]>) {
  if (!headers || Object.keys(headers).length === 0) {
    return '';
  }

  return Object.entries(headers)
    .map(([name, values]) => `${name}: ${values.join(', ')}`)
    .join('\n');
}

export function buildNewRule(profileName: string, index: number): RouteRule {
  return {
    name: `draft-rule-${index + 1}`,
    priority: index + 1,
    match: {
      models: [],
      tenants: [],
      endpoints: [],
      regions: [],
      headers: {},
    },
    'use-profile': profileName,
  };
}

export function parseJsonDraft<T>(label: string, raw: string): T {
  try {
    return JSON.parse(raw) as T;
  } catch (error) {
    throw new Error(`${label} JSON is invalid: ${error instanceof Error ? error.message : 'parse failed'}`);
  }
}

export function extractValidationMessage(error: unknown, fallback: string) {
  if (
    error &&
    typeof error === 'object' &&
    'response' in error &&
    error.response &&
    typeof error.response === 'object' &&
    'data' in error.response &&
    error.response.data &&
    typeof error.response.data === 'object' &&
    'details' in error.response.data &&
    Array.isArray(error.response.data.details)
  ) {
    return error.response.data.details.join('; ');
  }

  return getApiErrorMessage(error, fallback);
}

export function parseProfileJson(raw: string) {
  return parseJsonDraft<RouteProfile>('Profile policy', raw);
}

export function parseModelResolutionJson(raw: string) {
  return parseJsonDraft<RoutingConfig['model-resolution']>('Model resolution', raw);
}
