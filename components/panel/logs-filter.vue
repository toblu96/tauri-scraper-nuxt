<template>
  <header
    class="flex items-center justify-between border-b border-gray-200 bg-white py-4"
  >
    <!-- search -->
    <div>
      <label for="search" class="sr-only">Search</label>
      <div class="relative mt-1 rounded-md shadow-sm">
        <div
          class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3"
          aria-hidden="true"
        >
          <MagnifyingGlassIcon
            class="mr-3 h-4 w-4 text-gray-400"
            aria-hidden="true"
          />
        </div>
        <input
          type="text"
          name="search"
          id="search"
          v-model="searchString"
          class="block w-full rounded-md border-gray-300 pl-9 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
          placeholder="Search"
        />
      </div>
    </div>

    <div class="flex items-center">
      <div>datetime range picker</div>
      <div class="hidden md:ml-4 md:flex md:items-center">
        <div>
          <select
            @change="onSelectChange($event)"
            v-model="logLevel"
            id="location"
            name="location"
            class="mt-1 block w-full rounded-md border-gray-300 py-2 pl-3 pr-10 text-base focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm"
          >
            <option v-for="text in logLevelText" :value="text">
              {{ text }}
            </option>
          </select>
        </div>
        <div class="ml-6 h-6 w-px bg-gray-300" />
        <button
          type="button"
          @click="$emit('refresh')"
          :disabled="pending"
          class="ml-6 rounded-md border border-transparent bg-indigo-600 py-2 px-3 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
        >
          <ArrowPathIcon
            class="h-5 w-5"
            :class="pending && 'animate-spin'"
            aria-hidden="true"
          />
        </button>
      </div>
      <Menu as="div" class="relative ml-6 md:hidden">
        <MenuButton
          class="-mx-2 flex items-center rounded-full border border-transparent p-2 text-gray-400 hover:text-gray-500"
        >
          <span class="sr-only">Open menu</span>
          <EllipsisHorizontalIcon class="h-5 w-5" aria-hidden="true" />
        </MenuButton>

        <transition
          enter-active-class="transition ease-out duration-100"
          enter-from-class="transform opacity-0 scale-95"
          enter-to-class="transform opacity-100 scale-100"
          leave-active-class="transition ease-in duration-75"
          leave-from-class="transform opacity-100 scale-100"
          leave-to-class="transform opacity-0 scale-95"
        >
          <MenuItems
            class="absolute right-0 z-10 mt-3 w-36 origin-top-right divide-y divide-gray-100 overflow-hidden rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
          >
            <div class="py-1">
              <MenuItem v-slot="{ active }">
                <a
                  href="#"
                  :class="[
                    active ? 'bg-gray-100 text-gray-900' : 'text-gray-700',
                    'block px-4 py-2 text-sm',
                  ]"
                  >Create event</a
                >
              </MenuItem>
            </div>
            <div class="py-1">
              <MenuItem v-slot="{ active }">
                <a
                  href="#"
                  :class="[
                    active ? 'bg-gray-100 text-gray-900' : 'text-gray-700',
                    'block px-4 py-2 text-sm',
                  ]"
                  >Go to today</a
                >
              </MenuItem>
            </div>
            <div class="py-1">
              <MenuItem v-slot="{ active }">
                <a
                  href="#"
                  :class="[
                    active ? 'bg-gray-100 text-gray-900' : 'text-gray-700',
                    'block px-4 py-2 text-sm',
                  ]"
                  >Day view</a
                >
              </MenuItem>
              <MenuItem v-slot="{ active }">
                <a
                  href="#"
                  :class="[
                    active ? 'bg-gray-100 text-gray-900' : 'text-gray-700',
                    'block px-4 py-2 text-sm',
                  ]"
                  >Week view</a
                >
              </MenuItem>
              <MenuItem v-slot="{ active }">
                <a
                  href="#"
                  :class="[
                    active ? 'bg-gray-100 text-gray-900' : 'text-gray-700',
                    'block px-4 py-2 text-sm',
                  ]"
                  >Month view</a
                >
              </MenuItem>
              <MenuItem v-slot="{ active }">
                <a
                  href="#"
                  :class="[
                    active ? 'bg-gray-100 text-gray-900' : 'text-gray-700',
                    'block px-4 py-2 text-sm',
                  ]"
                  >Year view</a
                >
              </MenuItem>
            </div>
          </MenuItems>
        </transition>
      </Menu>
    </div>
  </header>
</template>

<script setup lang="ts">
import {
  EllipsisHorizontalIcon,
  ArrowPathIcon,
  MagnifyingGlassIcon,
} from "@heroicons/vue/20/solid";
import { Menu, MenuButton, MenuItem, MenuItems } from "@headlessui/vue";
import { debounce } from "ts-debounce";

const searchString = ref("");
const logLevel = ref("ALL");

const logLevelText = ["ALL", "DEBUG", "TRACE", "INFO", "WARN", "ERROR"];

// const emit = defineEmits(["refresh"]);
const emit = defineEmits<{
  (e: "refresh"): void;
  (
    e: "filterParamUpdate",
    value: { level: string; searchString: string }
  ): void;
}>();
const props = defineProps({
  pending: {
    type: Boolean,
    required: true,
  },
});

// handle log levels
const onSelectChange = (e: any) => {
  logLevel.value = e.target.value;
};

// debounce log filter changes
const updateSearchParams = debounce(
  async (searchString: string, level: string) => {
    emit("filterParamUpdate", {
      level,
      searchString,
    });
  },
  800
);
watchEffect(() => {
  updateSearchParams(searchString.value, logLevel.value);
});
</script>
