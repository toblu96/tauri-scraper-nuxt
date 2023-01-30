import { fetch } from "@tauri-apps/api/http";

export default defineNuxtRouteMiddleware(async (to, from) => {
  // check if backend is still online, otherwise navigate to setup page
  try {
    // use Tauri fetch because here we can specify timeout
    const response = await fetch("http://localhost:8000/api/info", {
      method: "GET",
      timeout: {
        secs: 0,
        nanos: 500000000, // 500ms
      },
    });
  } catch (error) {
    if (to.path != "/setup") return navigateTo("/setup");
  }
});
