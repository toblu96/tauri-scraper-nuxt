export default defineNuxtRouteMiddleware(async (to, from) => {
  // check if backend is still online, otherwise navigate to setup page
  if (from.path != "/setup") {
    const { error } = await useFetch(`http://localhost:8000/api/info`);
    if (error.value && to.path != "/setup") {
      return navigateTo("/setup");
    }
  }
});
