import type {
  AuthKeyCreateRequest,
  AuthKeySummary,
  BudgetConfig,
  KeyRateLimitConfig,
} from '../types/backend';

export interface AccessPolicyFormState {
  name: string;
  tenantId: string;
  allowedModels: string;
  allowedCredentials: string;
  rpm: string;
  tpm: string;
  costPerDayUsd: string;
  budgetEnabled: boolean;
  budgetTotalUsd: string;
  budgetPeriod: 'daily' | 'monthly';
  expiresAt: string;
}

export const emptyAccessForm: AccessPolicyFormState = {
  name: 'e2e-temp-key',
  tenantId: '',
  allowedModels: '',
  allowedCredentials: '',
  rpm: '',
  tpm: '',
  costPerDayUsd: '',
  budgetEnabled: false,
  budgetTotalUsd: '',
  budgetPeriod: 'daily',
  expiresAt: '',
};

export function parseListField(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

export function formFromAuthKey(key: AuthKeySummary): AccessPolicyFormState {
  return {
    name: key.name ?? '',
    tenantId: key.tenant_id ?? '',
    allowedModels: key.allowed_models.join(', '),
    allowedCredentials: key.allowed_credentials.join(', '),
    rpm: key.rate_limit?.rpm?.toString() ?? '',
    tpm: key.rate_limit?.tpm?.toString() ?? '',
    costPerDayUsd: key.rate_limit?.cost_per_day_usd?.toString() ?? '',
    budgetEnabled: key.budget != null,
    budgetTotalUsd: key.budget?.total_usd?.toString() ?? '',
    budgetPeriod: key.budget?.period ?? 'daily',
    expiresAt: key.expires_at ? key.expires_at.slice(0, 16) : '',
  };
}

export function buildRateLimit(form: AccessPolicyFormState): KeyRateLimitConfig | undefined {
  const rpm = form.rpm ? Number(form.rpm) : undefined;
  const tpm = form.tpm ? Number(form.tpm) : undefined;
  const cost = form.costPerDayUsd ? Number(form.costPerDayUsd) : undefined;
  if (rpm === undefined && tpm === undefined && cost === undefined) {
    return undefined;
  }
  return {
    rpm,
    tpm,
    cost_per_day_usd: cost,
  };
}

export function buildBudget(form: AccessPolicyFormState): BudgetConfig | undefined {
  if (!form.budgetEnabled || !form.budgetTotalUsd) {
    return undefined;
  }
  return {
    total_usd: Number(form.budgetTotalUsd),
    period: form.budgetPeriod,
  };
}

export function buildAuthKeyCreateRequest(form: AccessPolicyFormState): AuthKeyCreateRequest {
  return {
    name: form.name.trim() || undefined,
    tenant_id: form.tenantId.trim() || undefined,
    allowed_models: parseListField(form.allowedModels),
    allowed_credentials: parseListField(form.allowedCredentials),
    rate_limit: buildRateLimit(form),
    budget: buildBudget(form),
    expires_at: form.expiresAt ? new Date(form.expiresAt).toISOString() : undefined,
  };
}
