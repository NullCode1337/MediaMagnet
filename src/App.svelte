<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { Progress } from "$lib/components/ui/progress";
  import { Download, Settings, HardDrive, Plus, Menu, X, File, Folder } from "@lucide/svelte";

  // State Runes
  let isCollapsed = $state(false);
  let innerWidth = $state(0);
  let files = $state<File[]>([]);

  // Effect Rune: Replaces the $: reactive statement
  $effect(() => {
    isCollapsed = innerWidth < 768;
  });

  function handleFileSelect(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files) {
      files = [...files, ...Array.from(target.files)];
    }
  }

  function removeFile(index: number) {
    files = files.filter((_, i) => i !== index);
  }
</script>

<svelte:window bind:innerWidth />

<div class="flex h-screen bg-background text-foreground overflow-hidden">
  <aside 
    class="flex flex-col border-r bg-muted/20 transition-all duration-300 {isCollapsed ? 'w-20' : 'w-64'}"
  >
    <div class="p-4 flex items-center justify-between">
      {#if !isCollapsed}<span class="font-bold tracking-tight px-2 text-sm">LOADER</span>{/if}
      <Button variant="ghost" size="icon" onclick={() => (isCollapsed = !isCollapsed)} class="mx-auto">
        <Menu size={20} />
      </Button>
    </div>

    <nav class="flex-1 p-3 space-y-2">
      <label class="flex items-center justify-center w-full cursor-pointer">
        <input type="file" class="hidden" onchange={handleFileSelect} multiple />
        <div class="flex h-10 w-full items-center justify-center rounded-md bg-primary text-primary-foreground hover:opacity-90 transition-opacity">
          <Plus size={20} />
          {#if !isCollapsed}<span class="ml-2 text-sm font-medium">New Task</span>{/if}
        </div>
      </label>

      <Button variant="ghost" class="w-full {isCollapsed ? 'justify-center' : 'justify-start gap-3'}">
        <Download size={20} />
        {#if !isCollapsed}<span>Downloads</span>{/if}
      </Button>
      <Button variant="ghost" class="w-full {isCollapsed ? 'justify-center' : 'justify-start gap-3'}">
        <Folder size={20} />
        {#if !isCollapsed}<span>Local Files</span>{/if}
      </Button>
    </nav>

    <div class="p-4 border-t space-y-4">
      <div class="bg-muted/50 rounded-lg p-3">
        <div class="flex items-center gap-3 {isCollapsed ? 'justify-center' : ''}">
          <HardDrive size={18} class="text-muted-foreground" />
          {#if !isCollapsed}
            <div class="flex-1">
              <p class="text-[10px] uppercase font-bold text-muted-foreground">Disk Space</p>
              <Progress value={72} class="h-1.5 mt-1" />
            </div>
          {/if}
        </div>
      </div>
      <Button variant="ghost" class="w-full {isCollapsed ? 'justify-center' : 'justify-start gap-3'}">
        <Settings size={20} />
        {#if !isCollapsed}<span>Settings</span>{/if}
      </Button>
    </div>
  </aside>

  <main class="flex-1 flex flex-col min-w-0">
    <header class="h-14 border-b flex items-center px-6 bg-background justify-between">
      <h1 class="font-semibold text-sm">Queue ({files.length})</h1>
      <div class="text-[10px] text-muted-foreground opacity-50 hidden sm:block">V0.4.2</div>
    </header>

    <div class="flex-1 p-4 sm:p-8 overflow-y-auto">
      {#if files.length === 0}
        <div class="h-full flex flex-col items-center justify-center text-muted-foreground border-2 border-dashed rounded-xl opacity-50">
          <Download size={40} class="mb-4" />
          <p class="text-sm font-medium">No active tasks</p>
        </div>
      {:else}
      <div class="grid gap-3">
        {#each files as file, i (file.name + file.size + i)}
          <Card.Root>
            <Card.Content class="p-4 flex items-center gap-4">
              <div class="bg-primary/10 p-2 rounded">
                <File size={20} class="text-primary" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex justify-between items-start mb-1">
                  <p class="font-medium text-xs truncate uppercase tracking-tight">
                    {file.name}
                  </p>
                  <button onclick={() => removeFile(i)}>
                    <X size={14} class="text-muted-foreground hover:text-destructive" />
                  </button>
                </div>
                <Progress value={0} class="h-1" />
              </div>
            </Card.Content>
          </Card.Root>
        {/each}
      </div>
      {/if}
    </div>
  </main>
</div>