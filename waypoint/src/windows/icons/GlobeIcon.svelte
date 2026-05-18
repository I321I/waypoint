<script lang="ts">
  export let size: number = 14;
</script>

<svg
  data-globe-icon
  xmlns="http://www.w3.org/2000/svg"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  style="width: {size}px; height: {size}px; flex: none; min-width: {size}px; min-height: {size}px;"
>
  <circle cx="12" cy="12" r="9" />
  <path d="M3 12h18" />
  <path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z" />
</svg>

<style>
  svg { color: var(--text-secondary); flex-shrink: 0; vertical-align: middle; }
</style>
