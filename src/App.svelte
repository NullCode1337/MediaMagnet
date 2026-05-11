<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Download, Library, Globe } from '@lucide/svelte';
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

  let appWindow: any = null;
  let url = "";
  let isProcessing = false;
  let downloads: any[] = []; 

  onMount(async () => {
    try {
      const { window } = await import('@tauri-apps/api');
      appWindow = window.getCurrent();
    } catch (e) { console.warn("Web mode active"); }
  });
</script>

<div class="relative flex h-screen w-full flex-col overflow-hidden bg-[#050505] text-zinc-300 antialiased">
  <div class="absolute inset-0 z-0 overflow-hidden">
    <div class="blob blob-1 absolute -top-[10%] -left-[10%] h-[500px] w-[500px] rounded-full bg-emerald-500/10 blur-[120px]"></div>
    <div class="blob blob-2 absolute top-[20%] -right-[5%] h-[400px] w-[400px] rounded-full bg-blue-500/10 blur-[100px]"></div>
    <div class="blob blob-3 absolute -bottom-[10%] left-[20%] h-[600px] w-[600px] rounded-full bg-purple-500/10 blur-[150px]"></div>
  </div>

  <div class="noise-overlay pointer-events-none absolute inset-0 z-1 opacity-[0.02]"></div>

  <div class="relative z-10 flex flex-1 overflow-hidden p-2 md:p-4">
    <aside 
      class="flex w-20 flex-col items-center border border-white/10 bg-white/[0.03] backdrop-blur-3xl md:w-64 md:rounded-3xl shadow-[0_8px_32px_0_rgba(0,0,0,0.8)]"
    >
      <div class="flex h-full w-full flex-col p-6">
        <div class="mb-10 flex gap-2" data-tauri-drag-region>
          <button on:click={() => appWindow?.close()} class="h-3 w-3 rounded-full bg-red-500/20 hover:bg-red-500 transition-all shadow-[0_0_10px_rgba(239,68,68,0.2)]" />
          <button class="h-3 w-3 rounded-full bg-white/10 hover:bg-white/20 transition-all" />
        </div>

        <nav class="flex-1 space-y-4">
          <div class="space-y-1">
             <Button variant="ghost" class="w-full justify-start gap-4 rounded-xl bg-white/5 px-4 py-6 text-white shadow-[inset_0_1px_1px_rgba(255,255,255,0.1)] hover:bg-white/10">
              <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-500/20 text-emerald-400 shadow-inner">
                <Plus size={18} />
              </div>
              <span class="hidden font-semibold md:block">New Task</span>
            </Button>
          </div>

          <div class="flex flex-col gap-1">
            <Button variant="ghost" class="w-full justify-start gap-4 text-zinc-400 hover:text-white">
              <Download size={20} /><span class="hidden md:block">Active</span>
            </Button>
            <Button variant="ghost" class="w-full justify-start gap-4 text-zinc-400 hover:text-white">
              <Library size={20} /><span class="hidden md:block">Library</span>
            </Button>
          </div>
        </nav>
    </aside>

    <main class="flex flex-1 flex-col px-4 md:px-8">
      <header class="flex h-24 items-center" data-tauri-drag-region>
        <div class="relative flex flex-1 items-center gap-4 rounded-2xl border border-white/10 bg-white/[0.03] p-2 pr-3 backdrop-blur-2xl shadow-2xl">
          <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-white/5 text-emerald-400">
            <Globe size={18} />
          </div>
          <Input 
            bind:value={url}
            placeholder="Drop a link here..." 
            class="h-12 border-none bg-transparent text-lg font-medium placeholder:text-zinc-600 focus-visible:ring-0"
          />
          <Button 
            disabled={!url || isProcessing}
            class="h-12 rounded-xl bg-emerald-500 px-8 font-black tracking-tight text-black transition-all hover:scale-[1.02] hover:bg-emerald-400 active:scale-95 shadow-[0_0_20px_rgba(16,185,129,0.3)]"
          >
            {isProcessing ? 'WORKING...' : 'PROCESS'}
          </Button>
        </div>
      </header>

      <ScrollArea class="flex-1">
        {#if downloads.length === 0}
          <div class="flex h-full flex-col items-center justify-center pt-20">
             <div class="relative mb-6">
                <div class="absolute inset-0 animate-pulse rounded-full bg-emerald-500/20 blur-2xl"></div>
                <div class="relative flex h-24 w-24 items-center justify-center rounded-full border border-white/10 bg-white/5 backdrop-blur-xl">
                  <Download size={32} class="text-emerald-500/50" />
                </div>
             </div>
             <h2 class="text-2xl font-light tracking-tight text-white/90">MediaMagnet</h2>
             <p class="mt-2 text-zinc-500">Paste a URL (or multiple URLs)</p>
          </div>
        {/if}
      </ScrollArea>
    </main>
  </div>
</div>

<style lang="postcss">
  :global(body) {
    background-color: #050505;
    margin: 0;
  }

  .blob {
    transition: all 10s ease-in-out;
    animation: move 25s infinite alternate;
  }
  .blob-2 { animation-duration: 30s; animation-delay: -5s; }
  .blob-3 { animation-duration: 35s; animation-delay: -10s; }

  @keyframes move {
    from { transform: translate(0, 0) scale(1); }
    to { transform: translate(100px, 50px) scale(1.1); }
  }

  .noise-overlay {
    background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
  }

  :global([data-radix-scroll-area-viewport]) {
    mask-image: linear-gradient(to bottom, transparent, black 10%, black 90%, transparent);
  }
</style>