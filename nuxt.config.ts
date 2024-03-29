// https://v3.nuxtjs.org/api/configuration/nuxt.config
export default defineNuxtConfig({
  ssr: false,
  modules: ["@nuxtjs/tailwindcss"],
  alias: {
    // due to https://github.com/nuxt/framework/issues/6623
    // pinia: '/node_modules/@pinia/nuxt/node_modules/pinia/dist/pinia.mjs',
  },
  vite: {
    build: {
      target: "esnext",
    },
  },
});
