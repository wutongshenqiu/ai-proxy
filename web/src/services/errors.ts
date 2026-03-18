import axios from 'axios';

interface ApiErrorShape {
  message?: string;
  error?: string;
  errors?: string[];
}

export function isAbortLikeError(error: unknown, signal?: AbortSignal) {
  if (signal?.aborted) {
    return true;
  }

  return axios.isAxiosError(error) && error.code === 'ERR_CANCELED';
}

export function getApiErrorMessage(error: unknown, fallback: string) {
  if (axios.isAxiosError<ApiErrorShape>(error)) {
    const data = error.response?.data;
    if (Array.isArray(data?.errors) && data.errors.length > 0) {
      return data.errors.join(' · ');
    }
    if (typeof data?.message === 'string' && data.message.trim()) {
      return data.message;
    }
    if (typeof data?.error === 'string' && data.error.trim()) {
      return data.error;
    }
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  return fallback;
}
