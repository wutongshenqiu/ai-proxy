import { describe, it, expect, beforeEach, vi } from 'vitest';
import axios from 'axios';

const mockInstance = {
  interceptors: {
    request: { use: vi.fn() },
    response: { use: vi.fn() },
  },
  get: vi.fn(),
  post: vi.fn(),
  patch: vi.fn(),
  delete: vi.fn(),
};

vi.mock('axios', () => ({
  default: {
    create: vi.fn(() => mockInstance),
    post: vi.fn(),
  },
}));

describe('API service', () => {
  beforeEach(() => {
    vi.resetModules();
    localStorage.clear();
  });

  it('creates axios instance with correct base URL', async () => {
    await import('../../services/api');
    expect(axios.create).toHaveBeenCalledWith(
      expect.objectContaining({
        baseURL: '/api/dashboard',
      })
    );
  });

  it('registers request and response interceptors', async () => {
    await import('../../services/api');
    const instance = vi.mocked(axios.create).mock.results[0]?.value;
    expect(instance.interceptors.request.use).toHaveBeenCalled();
    expect(instance.interceptors.response.use).toHaveBeenCalled();
  });
});

describe('routingApi', () => {
  it('sends default-profile to backend', async () => {
    const mod = await import('../../services/api');
    const instance = vi.mocked(axios.create).mock.results[0]?.value;
    vi.mocked(instance.patch).mockResolvedValueOnce({ data: {} });

    await mod.routingApi.update({ 'default-profile': 'balanced' });

    expect(instance.patch).toHaveBeenCalledWith(
      '/routing',
      expect.objectContaining({ 'default-profile': 'balanced' })
    );
  });

  it('sends profile update', async () => {
    const mod = await import('../../services/api');
    const instance = vi.mocked(axios.create).mock.results[0]?.value;
    vi.mocked(instance.patch).mockResolvedValueOnce({ data: {} });

    await mod.routingApi.update({ 'default-profile': 'lowest-latency' });

    expect(instance.patch).toHaveBeenCalledWith(
      '/routing',
      expect.objectContaining({ 'default-profile': 'lowest-latency' })
    );
  });
});

describe('providersApi', () => {
  it('passes format field in create request', async () => {
    const mod = await import('../../services/api');
    const instance = vi.mocked(axios.create).mock.results[0]?.value;
    vi.mocked(instance.post).mockResolvedValueOnce({ data: {} });

    await mod.providersApi.create({
      name: 'deepseek',
      format: 'openai',
      api_key: 'key',
      disabled: false,
      models: ['model-1'],
    });

    expect(instance.post).toHaveBeenCalledWith(
      '/providers',
      expect.objectContaining({ name: 'deepseek', format: 'openai' })
    );
  });
});
