/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module "tauri-plugin-m3" {
  export type ColorScheme = {
    primary?: string;
    onPrimary?: string;
    primaryContainer?: string;
    onPrimaryContainer?: string;
    inversePrimary?: string;
    secondary?: string;
    onSecondary?: string;
    secondaryContainer?: string;
    onSecondaryContainer?: string;
    tertiary?: string;
    onTertiary?: string;
    tertiaryContainer?: string;
    onTertiaryContainer?: string;
    background?: string;
    onBackground?: string;
    surface?: string;
    onSurface?: string;
    surfaceVariant?: string;
    onSurfaceVariant?: string;
    surfaceTint?: string;
    inverseSurface?: string;
    inverseOnSurface?: string;
    error?: string;
    onError?: string;
    errorContainer?: string;
    onErrorContainer?: string;
    outline?: string;
    outlineVariant?: string;
    scrim?: string;
    surfaceBright?: string;
    surfaceDim?: string;
    surfaceContainer?: string;
    surfaceContainerHigh?: string;
    surfaceContainerHighest?: string;
    surfaceContainerLow?: string;
    surfaceContainerLowest?: string;
    primaryFixed?: string;
    primaryFixedDim?: string;
    onPrimaryFixed?: string;
    onPrimaryFixedVariant?: string;
    secondaryFixed?: string;
    secondaryFixedDim?: string;
    onSecondaryFixed?: string;
    onSecondaryFixedVariant?: string;
    tertiaryFixed?: string;
    tertiaryFixedDim?: string;
    onTertiaryFixed?: string;
    onTertiaryFixedVariant?: string;
  };

  export type DeviceInsets = {
    adjustedInsetTop: number;
    adjustedInsetBottom: number;
    adjustedInsetLeft: number;
    adjustedInsetRight: number;
    rawInsetTop: number;
    rawInsetBottom: number;
    rawInsetLeft: number;
    rawInsetRight: number;
  };

  export const M3: {
    getColors: (theme?: "dark" | "light" | "system") => Promise<ColorScheme>;
    applyColors: (
      theme?: "dark" | "light" | "system",
    ) => Promise<boolean | string>;
    getInsets: () => Promise<DeviceInsets>;
    setBarColor: (
      theme?: "dark" | "light" | "system",
    ) => Promise<boolean | string>;
  };
}
