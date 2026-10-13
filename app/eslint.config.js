import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import reactHooks from 'eslint-plugin-react-hooks';
import tseslint from 'typescript-eslint';

const TAURI_RESTRICTION = {
  patterns: [
    {
      group: ['@tauri-apps/*'],
      message: 'Use the typed wrappers in src/ipc',
    },
  ],
};

const NO_POP_LAYOUT = {
  selector: "JSXAttribute[name.name='mode'][value.value='popLayout']",
  message: 'AnimatePresence mode="popLayout" injects a style element that the CSP blocks',
};

export default defineConfig(
  { ignores: ['dist', 'src-tauri', 'src/ipc/generated'] },
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      js.configs.recommended,
      tseslint.configs.strictTypeChecked,
      tseslint.configs.stylisticTypeChecked,
      reactHooks.configs.flat.recommended,
      jsxA11y.flatConfigs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      'no-restricted-imports': ['error', TAURI_RESTRICTION],
      'no-restricted-syntax': ['error', NO_POP_LAYOUT],
    },
  },
  { files: ['src/ipc/**'], rules: { 'no-restricted-imports': 'off' } },
  { files: ['**/*.js'], extends: [js.configs.recommended] },
);
