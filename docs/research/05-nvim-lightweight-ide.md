# Lightweight Neovim IDE - Research (nvim-leuwi-panjang)

## Overview

Replace CoC.nvim (heavy, Node.js-based) with Neovim's native LSP + modern Lua plugins for a lightweight IDE experience.

## Why Native LSP Beats CoC

| Aspect | CoC.nvim | Native LSP |
|--------|----------|-----------|
| Runtime | Node.js process (~80-150 MB) | Built into Neovim (0 MB extra) |
| Startup | +200-500ms (Node.js boot) | Near-zero |
| Communication | Neovim -> RPC -> Node.js -> LSP server | Neovim -> LSP server (direct) |
| Completion | Built-in JS engine | nvim-cmp (Lua, lazy-loadable) |
| Config Language | JSON (vimscript) | Lua (10-30x faster than vimscript) |
| Extensibility | VSCode-style JS extensions | Modular Lua plugins |

**Expected improvement:**
- Startup: 30-60ms (vs 300-800ms with CoC)
- Memory: ~30-50 MB base (vs 150-300 MB with CoC+Node.js)

## Core Plugin Stack

### Plugin Manager: lazy.nvim (`folke/lazy.nvim`)
- Automatic lazy loading by event/command/filetype/keymap
- Lockfile for reproducible installs
- Built-in profiler

### Directory Structure
```
~/.config/nvim-leuwi-panjang/
├── init.lua
└── lua/
    ├── plugins/
    │   ├── lsp.lua          -- LSP, mason, nvim-cmp
    │   ├── treesitter.lua   -- Syntax, textobjects
    │   ├── telescope.lua    -- Fuzzy finder
    │   ├── editor.lua       -- File explorer, git, which-key
    │   ├── ui.lua           -- Statusline, colorscheme
    │   ├── dap.lua          -- Debug adapters
    │   ├── ai.lua           -- AI integrations
    │   └── terminal.lua     -- Terminal management
    └── config/
        ├── keymaps.lua
        ├── options.lua
        └── autocmds.lua
```

## LSP Servers per Language

| Language | LSP Server | Mason Name | Special Plugin |
|----------|-----------|------------|----------------|
| Java | Eclipse JDTLS | `jdtls` | `mfussenegger/nvim-jdtls` |
| Go | gopls | `gopls` | (gopls alone is excellent) |
| Rust | rust-analyzer | `rust_analyzer` | `mrcjkb/rustaceanvim` |
| Python | Pyright | `pyright` | |
| JavaScript/TypeScript | ts_ls | `ts_ls` | `pmizio/typescript-tools.nvim` (faster) |
| Kotlin | kotlin-language-server | `kotlin_language_server` | |
| Shell/Bash | bash-language-server | `bashls` | + shellcheck |
| HTML | vscode-html-ls | `html` | |
| CSS | vscode-css-ls | `cssls` | |
| PHP | Intelephense | `intelephense` | |

## DAP (Debug) Adapters per Language

| Language | Debug Adapter | Special Plugin |
|----------|--------------|----------------|
| Java | java-debug-adapter + java-test | via nvim-jdtls |
| Go | delve | `leoluz/nvim-dap-go` |
| Rust | codelldb | via rustaceanvim |
| Python | debugpy | `mfussenegger/nvim-dap-python` |
| JS/TS | js-debug-adapter | |
| Kotlin | kotlin-debug-adapter | |
| Shell | bash-debug-adapter | |
| PHP | php-debug-adapter | (requires Xdebug) |

## IDE Features Mapping

| Feature | Implementation |
|---------|---------------|
| Go to definition | `vim.lsp.buf.definition()` |
| Find references | Telescope lsp_references |
| Find implementations | Telescope lsp_implementations |
| Type hierarchy | `vim.lsp.buf.type_definition()` |
| Rename symbol | `vim.lsp.buf.rename()` |
| Extract function/variable | `ThePrimeagen/refactoring.nvim` + LSP code actions |
| Code actions | `vim.lsp.buf.code_action()` |
| Document symbols | Telescope lsp_document_symbols |
| Workspace symbols | Telescope lsp_workspace_symbols |
| File search | Telescope find_files / git_files |
| Text search (grep) | Telescope live_grep (ripgrep) |
| Git branches | Telescope git_branches + vim-fugitive |
| Git status/diff | gitsigns.nvim + diffview.nvim |
| File explorer | neo-tree.nvim |
| Terminal | toggleterm.nvim |
| Keybinding help | which-key.nvim |
| Debugging | nvim-dap + nvim-dap-ui |

## AI Integration

| Plugin | Description |
|--------|-------------|
| `greggh/claude-code.nvim` | Claude Code CLI integration with side panel |
| `yetone/avante.nvim` | Cursor-like AI panel (Claude, GPT, Gemini backends) |
| `olimorris/codecompanion.nvim` | Chat + inline AI, multiple providers |
| Custom toggleterm | Any CLI AI tool in floating terminal |

## Complete Plugin List

| Category | Plugin | Load Event |
|----------|--------|------------|
| Plugin Manager | `folke/lazy.nvim` | Bootstrap |
| LSP | `neovim/nvim-lspconfig` | BufReadPre |
| LSP Installer | `williamboman/mason.nvim` + mason-lspconfig | BufReadPre |
| Completion | `hrsh7th/nvim-cmp` + sources | InsertEnter |
| Snippets | `L3MON4D3/LuaSnip` + friendly-snippets | InsertEnter |
| Fuzzy Finder | `nvim-telescope/telescope.nvim` + fzf-native | Keys/Cmd |
| Syntax | `nvim-treesitter/nvim-treesitter` + textobjects | BufReadPost |
| File Explorer | `nvim-neo-tree/neo-tree.nvim` | Keys/Cmd |
| Git Signs | `lewis6991/gitsigns.nvim` | BufReadPre |
| Git Operations | `tpope/vim-fugitive` | Cmd |
| Git Diff | `sindrets/diffview.nvim` | Cmd |
| Debug | `mfussenegger/nvim-dap` + dap-ui | Keys |
| Terminal | `akinsho/toggleterm.nvim` | Keys |
| Keybindings | `folke/which-key.nvim` | VeryLazy |
| Refactoring | `ThePrimeagen/refactoring.nvim` | Keys |
| Java | `mfussenegger/nvim-jdtls` | ft=java |
| Rust | `mrcjkb/rustaceanvim` | ft=rust |
| AI (Claude) | `greggh/claude-code.nvim` | Keys |
| AI (General) | `yetone/avante.nvim` | Keys |

## Performance Optimization Tips

1. **Lazy load everything** - target startup under 50ms
2. **Disable unused built-ins** - gzip, matchit, netrw, tar, zip, etc.
3. **LSP**: `update_in_insert = false`, `diagnosticMode = "openFilesOnly"`
4. **Treesitter**: disable for files > 100KB
5. **Telescope**: use fzf-native (C-based, 10x faster sorting)
6. **Profile with**: `:Lazy profile`, `nvim --startuptime /tmp/startup.log`
