<template>
  <div class="flex h-full flex-col">
    <!-- Top nav-->
    <header class="relative flex h-16 flex-shrink-0 items-center bg-white">
      <!-- Logo area -->
      <div class="absolute inset-y-0 left-0 md:static md:flex-shrink-0">
        <NuxtLink
          to="/"
          class="flex h-16 w-16 items-center justify-center bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-600 md:w-20"
        >
          <img
            class="h-8 w-auto"
            src="https://tailwindui.com/img/logos/mark.svg?color=white"
            alt="Your Company"
          />
        </NuxtLink>
      </div>

      <!-- Picker area -->
      <div class="mx-auto md:hidden">
        <div class="relative">
          <label for="inbox-select" class="sr-only">Choose inbox</label>
          <select
            @change="navigateTo($event)"
            id="inbox-select"
            class="rounded-md border-0 bg-none pl-3 pr-8 text-base font-medium text-gray-900 focus:ring-2 focus:ring-indigo-600"
          >
            <option
              v-for="item in sidebarNavigation"
              :key="item.name"
              :selected="item.current"
            >
              {{ item.name }}
            </option>
          </select>
          <div
            class="pointer-events-none absolute inset-y-0 right-0 flex items-center justify-center pr-2"
          >
            <ChevronDownIcon class="h-5 w-5 text-gray-500" aria-hidden="true" />
          </div>
        </div>
      </div>

      <!-- Desktop title area -->
      <div
        class="hidden md:flex md:min-w-0 md:flex-1 md:items-center md:justify-between"
      >
        <h2
          class="ml-6 text-xl font-semibold leading-7 text-gray-900 sm:truncate sm:tracking-tight"
        >
          {{
            sidebarNavigation[
              sidebarNavigation.findIndex((nav) => nav.href == $route.path)
            ].name
          }}
        </h2>
      </div>
    </header>

    <!-- Bottom section -->
    <div class="flex min-h-0 flex-1 overflow-hidden">
      <!-- Narrow sidebar-->
      <nav
        aria-label="Sidebar"
        class="hidden md:block md:flex-shrink-0 md:overflow-y-auto md:bg-gray-800"
      >
        <div class="relative flex w-20 flex-col space-y-3 p-3">
          <NuxtLink
            v-for="item in sidebarNavigation"
            :key="item.name"
            :to="item.href"
            :class="[
              item.current
                ? 'bg-gray-900 text-white'
                : 'text-gray-400 hover:bg-gray-700',
              'inline-flex h-14 w-14 flex-shrink-0 items-center justify-center rounded-lg',
            ]"
          >
            <span class="sr-only">{{ item.name }}</span>
            <component :is="item.icon" class="h-6 w-6" aria-hidden="true" />
          </NuxtLink>
        </div>
      </nav>

      <!-- Main area -->
      <main
        class="min-w-0 flex-1 overflow-auto border-t border-gray-200 lg:flex"
      >
        <slot />
      </main>
    </div>
  </div>
</template>

<script setup>
import { ChevronDownIcon } from "@heroicons/vue/20/solid";
import {
  AdjustmentsHorizontalIcon,
  DocumentTextIcon,
  RectangleGroupIcon,
} from "@heroicons/vue/24/outline";

const sidebarNavigation = ref([
  { name: "Dashboard", href: "/", icon: RectangleGroupIcon, current: true },
  {
    name: "Scrapers",
    href: "/scrapers",
    icon: DocumentTextIcon,
    current: false,
  },
  {
    name: "Settings",
    href: "/settings",
    icon: AdjustmentsHorizontalIcon,
    current: false,
  },
]);

// set current page
useRouter().beforeEach((to, from) => {
  setActiveNav(to.path);
});
onMounted(() => {
  setActiveNav(useRoute().path);
});
const setActiveNav = (path) => {
  sidebarNavigation.value.forEach((nav, idx, navs) => {
    if (nav.href === path) {
      nav.current = true;
    } else {
      nav.current = false;
    }
  });
};

const navigateTo = (event) => {
  useRouter().push(
    sidebarNavigation.value.filter((nav) => nav.name === event.target.value)[0]
      .href
  );
};
</script>

<style>
html {
  @apply h-full bg-gray-100;
}

body {
  @apply h-full;
}

#__nuxt {
  @apply h-full overflow-hidden;
}
</style>
