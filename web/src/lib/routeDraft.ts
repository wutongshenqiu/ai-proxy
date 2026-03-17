import type { RouteExplanation } from '../types/backend';
import type { RouteScenarioRow } from '../types/controlPlane';

export const ROUTE_DRAFT_STORAGE_KEY = 'prism-control-plane:route-draft';

export interface RouteDraft {
  createdAt: string;
  scenario: RouteScenarioRow;
  explanation: RouteExplanation | null;
}

export function readRouteDraft() {
  const raw = localStorage.getItem(ROUTE_DRAFT_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as RouteDraft;
  } catch {
    localStorage.removeItem(ROUTE_DRAFT_STORAGE_KEY);
    return null;
  }
}

export function writeRouteDraft(draft: RouteDraft) {
  localStorage.setItem(ROUTE_DRAFT_STORAGE_KEY, JSON.stringify(draft));
}

export function clearRouteDraft() {
  localStorage.removeItem(ROUTE_DRAFT_STORAGE_KEY);
}
