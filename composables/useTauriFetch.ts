import { fetch as tauriFetch, FetchOptions } from "@tauri-apps/api/http";
// console.log((await tauriFetch("http://localhost:8000/api/info")).data);

export const useTauriFetch = () => {
  const fetch = async (path: string, options?: FetchOptions) => {
    let resp = await tauriFetch(`http://localhost:8000/api${path}`, options);

    return { ...resp };
  };
  return fetch;
};
