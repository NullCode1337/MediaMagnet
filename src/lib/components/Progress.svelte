<script>
  import {
    isDownloading,
    downloadProgress,
    expandStatus,
    statusMessages,
  } from "$lib/stores/store";

  import { fade, slide } from "svelte/transition";
  import "@fortawesome/fontawesome-free/css/all.min.css";

  const radius = 60;
  const centerX = 64;
  const centerY = 64;

  // @ts-ignore
  const createSmoothProgressStore = (initialValue) => {
    let value = initialValue;
    let targetValue = initialValue;
    let subscribers = new Set();
    // @ts-ignore
    let animationFrameId = null;
    const smoothingFactor = 0.1; 

    // @ts-ignore
    function set(newValue) {
      targetValue = newValue;
      // @ts-ignore
      if (!animationFrameId) {
        update();
      }
    }

    function update() {
      const diff = targetValue - value;
      
      if (Math.abs(diff) > 0.1) { 
        value += diff * smoothingFactor;
        subscribers.forEach((sub) => sub(value));
        animationFrameId = requestAnimationFrame(update);
      } else {
        if (value !== targetValue) {
          value = targetValue;
          subscribers.forEach((sub) => sub(value));
        }
        animationFrameId = null;
      }
    }

    return {
      // @ts-ignore
      subscribe(run) {
        subscribers.add(run);
        run(value);
        return () => subscribers.delete(run);
      },
      set,
    };
  };

  const smoothProgress = createSmoothProgressStore(0);
  
  $: smoothProgress.set($downloadProgress);
  
  // @ts-ignore
  function getSquiggleArc(startAngle, endAngle, amplitude, frequency, currentRadius) {  
    let path = "";
    const segments = 100;
    const deltaAngle = (endAngle - startAngle) / segments;

    for (let i = 0; i <= segments; i++) {
      const angle = startAngle + deltaAngle * i;
      const radialOffset = Math.sin(angle * frequency) * amplitude;
      const r = currentRadius + radialOffset;
      const x = centerX + r * Math.cos(angle);
      const y = centerY + r * Math.sin(angle);

      if (i === 0) {
        path += `M ${x} ${y}`;
      } else {
        path += ` L ${x} ${y}`;
      }
    }
    return path;
  }

  $: arcD = getSquiggleArc(
    -Math.PI / 2,
    -Math.PI / 2 + (2 * Math.PI * $smoothProgress) / 100,
    3, 10, 
    radius
  );
</script>

<div class="progress">
  {#if $isDownloading}
    <div class="progress-container">
      <progress 
        class="progress-bar"
        value={$smoothProgress} 
        max="100"
      >
      </progress>
      <span class="progress-text">{$downloadProgress}%</span>
      <button
        class="expand-btn {$expandStatus ? 'expanded' : ''}"
        on:click={() => ($expandStatus = !$expandStatus)}
        title="Show download progress"
        aria-label="Button to expand status bar"
      >
        <i class="fas fa-chevron-down"></i>
      </button>
    </div>

    <div class="circular-progress">
      <svg viewBox="0 0 128 128" class="progress-ring">
        <circle class="progress-ring-bg" cx="64" cy="64" r="62" />
        <path class="progress-ring-arc" d={arcD} /> 
      </svg>
    </div>
  {/if}

  {#if $expandStatus}
    <div class="status-container" transition:slide|local={{ duration: 500 }}>
      {#each $statusMessages as message, index (index)}
        <p
          class="status-message {index === $statusMessages.length - 1 ? 'latest' : ''}"
          in:fade={{ delay: index * 100, duration: 300 }}
        >
          {message}
        </p>
      {/each}
    </div>
  {/if}
</div>

<style>
  /* Rotation Animation Definition */
  @keyframes rotate {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .progress {
    width: 1:0%;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .progress-container {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 20px;
    width: 100%;
  }
  .progress-bar {
    flex: 1;
    height: 8px;
    width: 50vw;
    border-radius: 4px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.2);
  }
  .progress-bar::-webkit-progress-bar {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }
  .progress-bar::-webkit-progress-value {
    background: #6e8efb;
    border-radius: 4px;
    transition: width 0.3s ease;
  }
  .progress-text {
    color: white;
    font-family: "Poppins-bold", sans-serif;
    min-width: 40px;
    text-align: right;
  }
  .expand-btn {
    background: transparent;
    border: none;
    color: white;
    cursor: pointer;
    padding: 5px;
    border-radius: 4px;
    transition: transform 0.3s ease;
  }
  .expand-btn.expanded {
    transform: rotate(180deg);
  }
  .status-container {
    width: 100%;
    margin-top: 10px;
    padding: 15px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    max-height: 200px;
    overflow-y: auto;
  }
  .status-message {
    color: white;
    font-family: "noto-sans-semibold", Cambria, sans-serif;
    margin: 5px 0;
    font-size: 14px;
    line-height: 1.4;
  }
  .status-message.latest {
    font-weight: bold;
    color: #6e8efb;
  }
  .status-container::-webkit-scrollbar {
    width: 4px;
  }
  .status-container::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
  }
  .status-container::-webkit-scrollbar-thumb {
    background: #670f6a;
    border-radius: 6px;
  }
  .circular-progress {
    display: none;
    width: 128px;
    height: 128px;
    z-index: 980;
    position: absolute;
    top: 0;
    left: 0;
  }
  .progress-ring {
    width: 100%;
    height: 100%;
    animation: rotate 1s linear infinite; 
  }
  .progress-ring-bg {
    fill: none;
    stroke: rgba(255, 255, 255, 0.0);
    stroke-width: 4px;
  }
  .progress-ring-arc {
    fill: none;
    stroke: #6e8efb;
    stroke-width: 4px;
    stroke-linecap: round;
    transition: d 0.1s ease;
  }
  @media (max-width: 360px) {
    .progress-container {
      display: none;
    }
    .circular-progress {
      display: block;
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
    }
    .status-container {
      display: none;
    }
  }
</style>