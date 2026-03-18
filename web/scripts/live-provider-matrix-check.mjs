import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const apiBaseUrl = process.env.PRISM_API_BASE_URL ?? 'http://127.0.0.1:8327';
const baseUrl = process.env.PRISM_WEB_BASE_URL ?? 'http://127.0.0.1:3100';
const dashboardUsername = process.env.PRISM_DASHBOARD_USERNAME ?? 'admin';
const dashboardPassword = process.env.PRISM_DASHBOARD_PASSWORD ?? 'admin';
const providerSuffix = Date.now().toString().slice(-6);
const runtimeAuthKeyName = `e2e-runtime-key-${providerSuffix}`;
const dashScopeProviderName = `e2e-dashscope-${providerSuffix}`;
const dashScopeTenantId = 'e2e-dashscope';
const codexProviderName = 'ChatGPT Pro';
const codexModel = 'gpt-5';
const dashScopeDefaultModel = 'qwen3-coder-plus';

const scriptPath = fileURLToPath(import.meta.url);
const scriptDir = path.dirname(scriptPath);
const repoRoot = path.resolve(scriptDir, '..', '..');
const artifactRoot =
  process.env.PRISM_PROVIDER_ARTIFACTS_DIR ??
  path.join(repoRoot, 'artifacts', 'runtime', 'provider-live-check');
const runId = new Date().toISOString().replace(/\.\d{3}Z$/, 'Z').replaceAll(':', '-');
const runDir = path.join(artifactRoot, 'runs', runId);
const latestDir = path.join(artifactRoot, 'latest');

await fs.mkdir(runDir, { recursive: true });

function logStep(message) {
  console.error(`STEP ${message}`);
}

function toWsUrl(httpUrl) {
  return httpUrl.replace(/^http/, 'ws');
}

async function launchBrowser() {
  try {
    return await chromium.launch({ channel: 'chrome', headless: true });
  } catch {
    return chromium.launch({ headless: true });
  }
}

function parseJson(text) {
  if (!text) {
    return null;
  }
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function snippet(value, max = 240) {
  if (value === null || value === undefined) {
    return '';
  }
  const text = String(value).trim().replace(/\s+/g, ' ');
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

function capabilityStatus(result, capability) {
  return result?.checks?.find((check) => check.capability === capability)?.status ?? null;
}

function extractResponsesText(payload) {
  if (!payload || typeof payload !== 'object') {
    return '';
  }
  if (typeof payload.output_text === 'string') {
    return payload.output_text;
  }
  if (!Array.isArray(payload.output)) {
    return '';
  }
  return payload.output
    .flatMap((item) => item?.content ?? [])
    .filter((item) => item?.type === 'output_text' && typeof item.text === 'string')
    .map((item) => item.text)
    .join(' ')
    .trim();
}

function extractChatContent(payload) {
  return payload?.choices?.[0]?.message?.content?.trim?.() ?? '';
}

async function refreshLatestArtifacts() {
  await fs.rm(latestDir, { recursive: true, force: true });
  await fs.mkdir(latestDir, { recursive: true });
  const entries = await fs.readdir(runDir, { withFileTypes: true });
  await Promise.all(
    entries
      .filter((entry) => entry.isFile())
      .map((entry) => fs.copyFile(path.join(runDir, entry.name), path.join(latestDir, entry.name))),
  );
}

async function captureProviderAtlasScreenshots() {
  const browser = await launchBrowser();
  try {
    const context = await browser.newContext({
      viewport: { width: 1600, height: 1200 },
    });
    const page = await context.newPage();

    const waitForStable = async () => {
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(350);
    };

    const capture = async (name) => {
      await page.screenshot({
        path: path.join(runDir, `${name}.png`),
        fullPage: true,
      });
    };

    const closeWorkbench = async () => {
      const closeButton = page.getByRole('button', { name: 'Close workbench' });
      if (await closeButton.isVisible()) {
        await closeButton.click();
        await page.waitForTimeout(250);
      }
    };

    await page.goto(`${baseUrl}/login`, { waitUntil: 'domcontentloaded' });
    await page.getByRole('heading', { name: 'Enter the control plane', exact: true }).waitFor({ timeout: 10_000 });
    await page.getByLabel('Username').fill(dashboardUsername);
    await page.getByLabel('Password').fill(dashboardPassword);
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await page.getByRole('heading', { name: 'Operate from runtime posture, not page sprawl', exact: true }).waitFor({ timeout: 10_000 });
    await waitForStable();

    await page.getByRole('link', { name: /Provider Atlas/i }).click();
    await page.getByRole('heading', { name: 'Runtime entities with identity and auth posture', exact: true }).waitFor({ timeout: 10_000 });
    await waitForStable();

    await page.locator('.table-grid--providers .table-grid__cell--strong')
      .filter({ hasText: new RegExp(`^${codexProviderName}$`) })
      .first()
      .click();
    await waitForStable();
    await page.getByRole('button', { name: 'Open provider editor', exact: true }).click();
    await page.getByRole('heading', { name: 'Provider editor', exact: true }).waitFor({ timeout: 10_000 });
    await capture('provider-atlas-codex-live');
    await closeWorkbench();

    if (dashScopeProviderCreated) {
      await page.locator('.table-grid--providers .table-grid__cell--strong')
        .filter({ hasText: new RegExp(`^${dashScopeProviderName}$`) })
        .first()
        .click();
      await waitForStable();
      await page.getByRole('button', { name: 'Open provider editor', exact: true }).click();
      await page.getByRole('heading', { name: 'Provider editor', exact: true }).waitFor({ timeout: 10_000 });
      await page.locator('.sheet__actions').getByRole('button', { name: 'Run health probe', exact: true }).click();
      await page.getByText(/Health probe completed with status/i).waitFor({ timeout: 20_000 });
      await capture('provider-atlas-dashscope-live');
      await closeWorkbench();
    }
  } finally {
    await browser.close();
  }
}

let dashboardCookieHeader = '';
let runtimeAuthKeyId = null;
let dashScopeProviderCreated = false;

async function ensureDashboardSession() {
  if (dashboardCookieHeader) {
    return dashboardCookieHeader;
  }
  const response = await fetch(`${apiBaseUrl}/api/dashboard/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username: dashboardUsername,
      password: dashboardPassword,
    }),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`dashboard login failed: ${response.status} ${text}`);
  }
  const cookie = response.headers.get('set-cookie');
  if (!cookie) {
    throw new Error('dashboard login did not return a session cookie');
  }
  dashboardCookieHeader = cookie.split(';')[0];
  return dashboardCookieHeader;
}

async function dashboardRequest(pathname, init = {}) {
  const cookie = await ensureDashboardSession();
  const response = await fetch(`${apiBaseUrl}${pathname}`, {
    ...init,
    headers: {
      cookie,
      ...(init.body ? { 'Content-Type': 'application/json' } : {}),
      ...(init.headers ?? {}),
    },
  });
  const text = await response.text();
  return {
    ok: response.ok,
    status: response.status,
    text,
    data: parseJson(text),
  };
}

async function dashboardJson(pathname, init = {}) {
  const response = await dashboardRequest(pathname, init);
  if (!response.ok) {
    throw new Error(`dashboard request failed: ${response.status} ${response.text}`);
  }
  return response.data;
}

async function cleanupRuntimeAuthKey() {
  if (runtimeAuthKeyId === null) {
    return;
  }
  await dashboardRequest(`/api/dashboard/auth-keys/${runtimeAuthKeyId}`, { method: 'DELETE' });
  runtimeAuthKeyId = null;
}

async function createRuntimeAuthKey() {
  await cleanupRuntimeAuthKey();
  await dashboardJson('/api/dashboard/auth-keys', {
    method: 'POST',
    body: JSON.stringify({
      name: runtimeAuthKeyName,
      tenant_id: dashScopeTenantId,
    }),
  });
  const listResponse = await dashboardJson('/api/dashboard/auth-keys');
  const created = (listResponse.auth_keys ?? []).find((key) => key.name === runtimeAuthKeyName);
  if (!created) {
    throw new Error('runtime auth key was not created');
  }
  runtimeAuthKeyId = created.id;
  const revealed = await dashboardJson(`/api/dashboard/auth-keys/${created.id}/reveal`, {
    method: 'POST',
  });
  return revealed.key;
}

async function cleanupDashScopeProvider() {
  if (!dashScopeProviderCreated) {
    return;
  }
  await dashboardRequest(`/api/dashboard/providers/${encodeURIComponent(dashScopeProviderName)}`, {
    method: 'DELETE',
  });
  dashScopeProviderCreated = false;
}

async function resolveDashScopeFixture() {
  if (process.env.PRISM_DASHSCOPE_API_KEY?.trim()) {
    return {
      source: 'env',
      apiKey: process.env.PRISM_DASHSCOPE_API_KEY.trim(),
      baseUrl: (process.env.PRISM_DASHSCOPE_BASE_URL ?? 'https://coding.dashscope.aliyuncs.com').trim(),
      model: (process.env.PRISM_DASHSCOPE_MODEL ?? dashScopeDefaultModel).trim(),
    };
  }

  const fixturePath = path.join(repoRoot, 'config.test.yaml');
  const text = await fs.readFile(fixturePath, 'utf8');
  const blockMatch = text.match(/openai-compatibility:\s*\n([\s\S]*?)(?:\n[a-zA-Z0-9_-]+:|\n$)/);
  if (!blockMatch) {
    return null;
  }
  const block = blockMatch[1];
  const apiKey = block.match(/api-key:\s*([^\n]+)/)?.[1]?.trim();
  const baseUrl = block.match(/base-url:\s*([^\n]+)/)?.[1]?.trim();
  const model = block.match(/-\s+id:\s*([^\n]+)/)?.[1]?.trim();
  if (!apiKey || !baseUrl || !model) {
    return null;
  }
  return {
    source: 'config.test.yaml',
    apiKey,
    baseUrl,
    model,
  };
}

async function sendCodexResponsesRequest(authKey) {
  const response = await fetch(`${apiBaseUrl}/api/provider/${encodeURIComponent(codexProviderName)}/v1/responses`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${authKey}`,
    },
    body: JSON.stringify({
      model: codexModel,
      input: 'Reply with the single word ok.',
    }),
  });
  const text = await response.text();
  const data = parseJson(text);
  if (!response.ok) {
    throw new Error(`codex responses request failed: ${response.status} ${text}`);
  }
  return {
    status: response.status,
    responseId: data?.id ?? null,
    outputText: snippet(extractResponsesText(data)),
  };
}

async function sendCodexResponsesWs(authKey) {
  const wsUrl = `${toWsUrl(apiBaseUrl)}/api/provider/${encodeURIComponent(codexProviderName)}/v1/responses/ws?key=${encodeURIComponent(authKey)}`;
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(wsUrl);
    const eventTypes = [];
    let resolved = false;

    const finish = (fn, value) => {
      if (resolved) {
        return;
      }
      resolved = true;
      clearTimeout(timeout);
      try {
        socket.close();
      } catch {
        // ignore close errors on best-effort cleanup
      }
      fn(value);
    };

    const timeout = setTimeout(() => {
      finish(reject, new Error('codex websocket timed out before response.completed'));
    }, 60_000);

    socket.addEventListener('open', () => {
      socket.send(JSON.stringify({
        type: 'response.create',
        model: codexModel,
        input: [
          {
            role: 'user',
            content: [
              {
                type: 'input_text',
                text: 'Reply with the single word ok.',
              },
            ],
          },
        ],
      }));
    });

    socket.addEventListener('message', (event) => {
      const raw = typeof event.data === 'string'
        ? event.data
        : Buffer.from(event.data).toString('utf8');
      const data = parseJson(raw);
      if (!data || typeof data !== 'object') {
        return;
      }
      const type = typeof data.type === 'string' ? data.type : 'unknown';
      eventTypes.push(type);
      if (type === 'error') {
        finish(reject, new Error(data.error?.message ?? raw));
        return;
      }
      if (type === 'response.completed') {
        finish(resolve, {
          responseId: data.response?.id ?? null,
          eventCount: eventTypes.length,
          terminalEvent: type,
          eventTypes: Array.from(new Set(eventTypes)),
          outputText: snippet(extractResponsesText(data.response)),
        });
      }
    });

    socket.addEventListener('error', () => {
      finish(reject, new Error('codex websocket connection failed'));
    });

    socket.addEventListener('close', (event) => {
      if (!resolved) {
        finish(reject, new Error(`codex websocket closed before completion: ${event.code}`));
      }
    });
  });
}

async function createDashScopeProvider(fixture) {
  const response = await dashboardRequest('/api/dashboard/providers', {
    method: 'POST',
    body: JSON.stringify({
      name: dashScopeProviderName,
      format: 'openai',
      upstream: 'openai',
      api_key: fixture.apiKey,
      base_url: fixture.baseUrl,
      models: [fixture.model],
      disabled: false,
    }),
  });
  dashScopeProviderCreated = response.ok;
  return response;
}

async function runDashScopeHealthCheck() {
  return dashboardRequest(`/api/dashboard/providers/${encodeURIComponent(dashScopeProviderName)}/health`, {
    method: 'POST',
  });
}

async function fetchDashScopeModels(fixture) {
  return dashboardRequest('/api/dashboard/providers/fetch-models', {
    method: 'POST',
    body: JSON.stringify({
      format: 'openai',
      upstream: 'openai',
      api_key: fixture.apiKey,
      base_url: fixture.baseUrl,
    }),
  });
}

async function sendDashScopeChatCompletion(authKey, model) {
  const response = await fetch(`${apiBaseUrl}/api/provider/${encodeURIComponent(dashScopeProviderName)}/v1/chat/completions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${authKey}`,
    },
    body: JSON.stringify({
      model,
      stream: false,
      messages: [
        {
          role: 'user',
          content: 'Reply with the single word ok.',
        },
      ],
    }),
  });
  const text = await response.text();
  const data = parseJson(text);
  return {
    ok: response.ok,
    status: response.status,
    text,
    data,
  };
}

const report = {
  artifactRoot,
  latestDir,
  runDir,
  runId,
  checks: [],
  failures: [],
};

function recordCheck(name, status, details = {}) {
  const { name: _ignoredName, status: _ignoredStatus, ...rest } = details;
  const entry = {
    name,
    status,
    ...rest,
  };
  report.checks.push(entry);
  if (status !== 'passed') {
    report.failures.push(entry);
  }
}

let runtimeAuthKey = null;
let dashScopeFixture = null;

try {
  logStep('dashboard-session');
  await ensureDashboardSession();
  runtimeAuthKey = await createRuntimeAuthKey();
  recordCheck('dashboard.runtime-auth-key', 'passed', {
    keyName: runtimeAuthKeyName,
    tenantId: dashScopeTenantId,
  });

  logStep('codex-http');
  try {
    const result = await sendCodexResponsesRequest(runtimeAuthKey);
    recordCheck('codex.responses.http', 'passed', result);
  } catch (error) {
    recordCheck('codex.responses.http', 'failed', { message: error instanceof Error ? error.message : String(error) });
  }

  logStep('codex-ws');
  try {
    const result = await sendCodexResponsesWs(runtimeAuthKey);
    recordCheck('codex.responses.ws', 'passed', result);
  } catch (error) {
    recordCheck('codex.responses.ws', 'failed', { message: error instanceof Error ? error.message : String(error) });
  }

  logStep('dashscope-fixture');
  dashScopeFixture = await resolveDashScopeFixture();
  if (!dashScopeFixture) {
    recordCheck('dashscope.fixture', 'failed', {
      message: 'No DashScope fixture found in env or config.test.yaml',
    });
  } else {
    recordCheck('dashscope.fixture', 'passed', {
      source: dashScopeFixture.source,
      baseUrl: dashScopeFixture.baseUrl,
      model: dashScopeFixture.model,
    });

    logStep('dashscope-fetch-models');
    const fetchModelsResponse = await fetchDashScopeModels(dashScopeFixture);
    if (fetchModelsResponse.ok) {
      recordCheck('dashscope.fetch-models', 'passed', {
        supported: fetchModelsResponse.data?.supported ?? true,
        count: Array.isArray(fetchModelsResponse.data?.models) ? fetchModelsResponse.data.models.length : 0,
        firstModel: fetchModelsResponse.data?.models?.[0] ?? null,
        message: fetchModelsResponse.data?.message ?? null,
      });
    } else {
      recordCheck('dashscope.fetch-models', 'failed', {
        httpStatus: fetchModelsResponse.status,
        message: snippet(fetchModelsResponse.text),
      });
    }

    logStep('dashscope-create-provider');
    const createProviderResponse = await createDashScopeProvider(dashScopeFixture);
    if (createProviderResponse.ok) {
      recordCheck('dashscope.provider-create', 'passed', {
        provider: dashScopeProviderName,
      });

      logStep('dashscope-health');
      const healthResponse = await runDashScopeHealthCheck();
      if (healthResponse.ok && healthResponse.data?.status !== 'error') {
        recordCheck('dashscope.provider-health', 'passed', {
          healthSummary: healthResponse.data?.status ?? null,
          textStatus: capabilityStatus(healthResponse.data, 'text'),
          authStatus: capabilityStatus(healthResponse.data, 'auth'),
        });
      } else {
        recordCheck('dashscope.provider-health', 'failed', {
          httpStatus: healthResponse.status,
          healthSummary: healthResponse.data?.status ?? null,
          textStatus: capabilityStatus(healthResponse.data, 'text'),
          authStatus: capabilityStatus(healthResponse.data, 'auth'),
          message: snippet(healthResponse.text),
        });
      }

      logStep('dashscope-chat');
      const chatResponse = await sendDashScopeChatCompletion(runtimeAuthKey, dashScopeFixture.model);
      if (chatResponse.ok) {
        recordCheck('dashscope.chat-completions', 'passed', {
          httpStatus: chatResponse.status,
          content: snippet(extractChatContent(chatResponse.data)),
        });
      } else {
        recordCheck('dashscope.chat-completions', 'failed', {
          httpStatus: chatResponse.status,
          message: snippet(chatResponse.text),
        });
      }
    } else {
      recordCheck('dashscope.provider-create', 'failed', {
        httpStatus: createProviderResponse.status,
        message: snippet(createProviderResponse.text),
      });
    }
  }

  logStep('browser-screenshots');
  try {
    await captureProviderAtlasScreenshots();
    recordCheck('browser.provider-atlas-screenshots', 'passed', {
      files: [
        'provider-atlas-codex-live.png',
        ...(dashScopeProviderCreated ? ['provider-atlas-dashscope-live.png'] : []),
      ],
    });
  } catch (error) {
    recordCheck('browser.provider-atlas-screenshots', 'failed', {
      message: error instanceof Error ? error.message : String(error),
    });
  }
} finally {
  await cleanupDashScopeProvider();
  await cleanupRuntimeAuthKey();
}

await fs.writeFile(
  path.join(runDir, 'report.json'),
  `${JSON.stringify(report, null, 2)}\n`,
  'utf8',
);
await refreshLatestArtifacts();

console.log(JSON.stringify(report, null, 2));

if (report.failures.length > 0) {
  process.exit(1);
}
