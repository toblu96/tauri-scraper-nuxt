<script setup lang="ts">
async function handleToggle(id: string, state: boolean) {
  let res = await useFetch(`http://localhost:8000/api/files/${id}`, {
    method: "PATCH",
    body: {
      enabled: state,
    },
  });
  if (res.error.value) {
    console.error(res.error.value);
  }
}
</script>

<template>
  <div class="px-6 pt-8">
    <TableScrapers @toggleEnableState="handleToggle" />
  </div>
</template>
