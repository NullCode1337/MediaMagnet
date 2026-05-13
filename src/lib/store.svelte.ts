// store.svelte.ts
export const uiState = $state({
  innerWidth: 0,
  innerHeight: 0,
  isMaximized: false,
  
  get headless() {
    return (
      !this.isMaximized && 
      this.innerWidth > 0 && 
      (this.innerWidth < 250 || this.innerHeight < 250)
    );
  }
});