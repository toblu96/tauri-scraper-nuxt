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
      <div class="hidden md:flex lg:w-96">
        <VueDatePicker
          range
          utc
          auto-apply
          show-now-button
          enable-seconds
          text-input
          placeholder="Select Range"
          :clearable="false"
          :preset-ranges="presetDatePickerRanges"
          v-model="date"
        />
      </div>
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
          :class="pending && 'opacity-30'"
          class="ml-6 rounded-md border border-transparent bg-indigo-600 py-2 px-3 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
        >
          <ArrowPathIcon
            class="h-5 w-5"
            :class="pending && 'animate-spin'"
            aria-hidden="true"
          />
        </button>
      </div>
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
import VueDatePicker from "@vuepic/vue-datepicker";
import "@vuepic/vue-datepicker/dist/main.css";
import {
  endOfMonth,
  endOfYear,
  startOfMonth,
  startOfYear,
  subMonths,
} from "date-fns";

const searchString = ref("");
const logLevel = ref("ALL");
const date = ref("");

const logLevelText = ["ALL", "DEBUG", "TRACE", "INFO", "WARN", "ERROR"];

const presetDatePickerRanges = ref([
  {
    label: "Today",
    range: [
      new Date(new Date().setHours(0, 0, 0, 0)),
      new Date(new Date().setHours(23, 59, 59, 0)),
    ],
  },
  {
    label: "This month",
    range: [startOfMonth(new Date()), endOfMonth(new Date())],
  },
  {
    label: "Last month",
    range: [
      startOfMonth(subMonths(new Date(), 1)),
      endOfMonth(subMonths(new Date(), 1)),
    ],
  },
  {
    label: "This year",
    range: [startOfYear(new Date()), endOfYear(new Date())],
  },
]);

const emit = defineEmits<{
  (e: "refresh"): void;
  (
    e: "filterParamUpdate",
    value: {
      level: string;
      searchString: string;
      startDate: string;
      endDate: string;
    }
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
  async (searchString: string, level: string, startDate, endDate) => {
    emit("filterParamUpdate", {
      level,
      searchString,
      startDate,
      endDate,
    });
  },
  500
);
watchEffect(() => {
  const [startDate, endDate] = date.value;
  updateSearchParams(searchString.value, logLevel.value, startDate, endDate);
});
</script>
