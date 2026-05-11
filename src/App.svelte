<script lang="ts">
  import { Download, Link2, Music, Video, Loader2 } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Progress } from "$lib/components/ui/progress";
  import { Label } from "$lib/components/ui/label";

  let url = "";
  let isDownloading = false;
  let progress = 0;
  let selectedFormat = "video";

  async function handleDownload() {
    if (!url) return;
    
    isDownloading = true;
    progress = 0;

    // Simulate download progress
    const interval = setInterval(() => {
      progress += 10;
      if (progress >= 100) {
        clearInterval(interval);
        isDownloading = false;
        url = "";
      }
    }, 400);
  }
</script>

<div class="flex items-center justify-center min-h-screen bg-muted/40 p-4">
  <Card.Root class="w-full max-w-md shadow-lg">
    <Card.Header class="space-y-1">
      <Card.Title class="text-2xl font-bold">Media Downloader</Card.Title>
      <Card.Description>
        Paste a URL from YouTube, Instagram, or TikTok to begin.
      </Card.Description>
    </Card.Header>
    
    <Card.Content class="grid gap-6">
      <!-- URL Input -->
      <div class="grid gap-2">
        <Label for="url">Content URL</Label>
        <div class="relative">
          <Link2 class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
          <Input 
            id="url"
            placeholder="https://example.com/video/..." 
            bind:value={url}
            class="pl-9"
          />
        </div>
      </div>

      <!-- Format Selection -->
      <Tabs.Root bind:value={selectedFormat} class="w-full">
        <Tabs.List class="grid w-full grid-cols-2">
          <Tabs.Trigger value="video" class="flex items-center gap-2">
            <Video class="h-4 w-4" /> Video
          </Tabs.Trigger>
          <Tabs.Trigger value="audio" class="flex items-center gap-2">
            <Music class="h-4 w-4" /> Audio
          </Tabs.Trigger>
        </Tabs.List>
      </Tabs.Root>

      <!-- Progress Section -->
      {#if isDownloading}
        <div class="space-y-2">
          <div class="flex justify-between text-sm font-medium">
            <span>Downloading...</span>
            <span>{progress}%</span>
          </div>
          <Progress value={progress} max={100} class="h-2" />
        </div>
      {/if}
    </Card.Content>

    <Card.Footer>
      <Button 
        class="w-full" 
        disabled={!url || isDownloading} 
        onclick={handleDownload}
      >
        {#if isDownloading}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
          Processing
        {:else}
          <Download class="mr-2 h-4 w-4" />
          Download {selectedFormat === 'video' ? 'Video' : 'MP3'}
        {/if}
      </Button>
    </Card.Footer>
  </Card.Root>
</div>