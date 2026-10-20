import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [svelte({ compilerOptions: { runes: true } }), tailwindcss()],
	resolve: {
		// `$lib` is the import prefix used inside @mediamagnet/ui. The shell keeps
		// no components of its own, so the alias points at the UI package.
		alias: {
			$lib: path.resolve('../../packages/ui/src/lib')
		},
		dedupe: ['svelte']
	},
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
		watch: {
			ignored: ['**/src-tauri/**']
		}
	},
	envPrefix: ['VITE_', 'TAURI_ENV_*'],
	build: {
		target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
		minify: !process.env.TAURI_ENV_DEBUG ? 'oxc' : false,
		sourcemap: !!process.env.TAURI_ENV_DEBUG
	}
});
