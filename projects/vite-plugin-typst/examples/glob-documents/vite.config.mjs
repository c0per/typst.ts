import { defineConfig } from 'vite';
import { TypstPlugin } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [TypstPlugin({ documents: ['content/**/*.typ'] })],
});
