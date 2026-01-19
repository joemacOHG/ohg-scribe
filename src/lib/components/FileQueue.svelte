<script lang="ts">
  import type { FileJob } from "../types";
  import FileItem from "./FileItem.svelte";

  interface Props {
    jobs: FileJob[];
    onOpen?: (path: string) => void;
    onRetry?: (id: string) => void;
    onViewTranscript?: (job: FileJob) => void;
  }

  let { jobs, onOpen, onRetry, onViewTranscript }: Props = $props();
</script>

{#if jobs.length > 0}
  <div class="file-queue">
    {#each jobs as job (job.id)}
      <FileItem {job} {onOpen} {onRetry} {onViewTranscript} />
    {/each}
  </div>
{/if}

<style>
  .file-queue {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: 300px;
    overflow-y: auto;
    padding: 4px;
  }
</style>
