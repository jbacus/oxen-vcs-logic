import { AxiosError } from 'axios';

export interface ApiError {
  message: string;
  code?: string;
  details?: Record<string, unknown>;
}

export function isAxiosError(error: unknown): error is AxiosError {
  return (error as AxiosError).isAxiosError === true;
}

export function handleApiError(error: unknown): string {
  if (isAxiosError(error)) {
    const axiosError = error as AxiosError<ApiError>;

    if (axiosError.response) {
      const { status, data } = axiosError.response;

      // Handle specific status codes
      switch (status) {
        case 400:
          return data?.message || 'Invalid request. Please check your input.';
        case 401:
          return 'Please sign in to continue.';
        case 403:
          return "You don't have permission to perform this action.";
        case 404:
          return data?.message || 'The requested resource was not found.';
        case 409:
          return data?.message || 'Conflict - the resource already exists.';
        case 422:
          return data?.message || 'Validation failed. Please check your input.';
        case 429:
          return 'Too many requests. Please wait a moment and try again.';
        case 500:
          return 'Server error. Please try again later.';
        case 501:
          return 'This feature is not yet implemented.';
        case 502:
        case 503:
        case 504:
          return 'Service temporarily unavailable. Please try again later.';
        default:
          return data?.message || `Request failed with status ${status}`;
      }
    }

    // Network error
    if (axiosError.code === 'ECONNABORTED') {
      return 'Request timed out. Please try again.';
    }

    if (!axiosError.response) {
      return 'Unable to connect to the server. Please check your network connection.';
    }
  }

  // Generic error
  if (error instanceof Error) {
    return error.message;
  }

  return 'An unexpected error occurred. Please try again.';
}

export function getErrorMessage(error: unknown): string {
  return handleApiError(error);
}
