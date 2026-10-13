import type { APIRequestContext } from '@playwright/test';

/**
 * Headers for mutating API calls made directly through the `request` fixture.
 * The server's CSRF middleware requires the `csfr_` cookie value echoed back
 * as `X-Csrf-Token`, as the web client does.
 */
export const csrfHeaders = async (request: APIRequestContext): Promise<Record<string, string>> => {
  const csrf = (await request.storageState()).cookies.find(cookie => cookie.name === 'csfr_')?.value;
  return csrf ? { 'X-Csrf-Token': csrf } : {};
};
