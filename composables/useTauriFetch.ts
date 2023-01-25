import { fetch as tauriFetch, FetchOptions } from "@tauri-apps/api/http";

// CORS workaround because nuxt serves as SPA here
export const useTauriFetch = () => {
  const fetch = async (path: string, options?: FetchOptions) => {
    try {
      let resp = await tauriFetch(`http://localhost:8000/api${path}`, options);

      return { ...resp };
    } catch (error) {
      console.error(`Could not reach api: ${error}`);

      return {
        data: "",
        ok: false,
        status: error,
      };
    }
  };
  return fetch;
};
