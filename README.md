# Lwow

**Leve. Rápido. Seu.** — Um navegador premium para Windows, construído em Rust, focado em baixo consumo de RAM/CPU e personalização profunda.

## Status do projeto (honesto)

Lwow está em desenvolvimento ativo. Este repositório contém o **núcleo funcional** publicado até agora:

- ✅ `src/main.rs` — inicialização, criação do diretório de dados, arranque
- ✅ `src/store.rs` — SQLite (WAL): configurações, histórico, favoritos
- ✅ `src/app.rs` — estado global, Settings persistidas, event loop
- ✅ `src/tabs/mod.rs` — TabManager: abertura/fechamento, aba ativa, proteção de áudio, candidatos à suspensão
- ✅ `src/tabs/suspension.rs` — política de suspensão + 3 modos (Economia/Equilibrado/Desempenho)
- ✅ `src/themes.rs` — motor de temas com 4 presets (Graphite, Snow, Aurora, Amber) e persistência
- ✅ `src/webview.rs` — ponte HWND Slint → wry/WebView2

Ainda **não** publicados (próximos marcos): UI Slint completa, integração efetiva das WebViews por aba, instalador WiX, benchmarks.

## Arquitetura

- **Rust** — linguagem principal do núcleo
- **Slint** — interface nativa leve (chrome: abas, barra de endereço, menus)
- **WebView2 (Chromium)** — renderização de páginas via `wry` (child webviews)
- **SQLite** — histórico, favoritos, configurações, sessões

A integração Slint + WebView2 foi validada (17/11/2026): `wry` 0.57.2 suporta `WebViewBuilder::build_as_child` no Windows, e o Slint expõe o HWND via raw-window-handle 0.6, permitindo que o chrome Slint e as páginas WebView2 coexistam na mesma janela Win32.

## Requisitos

- Windows 10/11 (x64)
- [Rust](https://rustup.rs/) (MSVC toolchain)
- WebView2 Runtime (Evergreen) — presente por padrão no Windows 10/11 atualizados
- Visual Studio Build Tools (C++ workload) para linkar

## Compilar

```powershell
git clone https://github.com/creaturesofsonaria616-creator/LWOW.git
cd LWOW
git checkout feature/project-setup
cargo build --release
```

O executável sai em `target/release/lwow.exe`.

## Roadmap

1. ✅ Núcleo Rust (store, abas, temas, política de suspensão)
2. 🚧 UI Slint: janela do navegador, tab strip, omnibar, página inicial
3. 🚧 WebViews por aba (wry `build_as_child`) + suspensão TrySuspend
4. ⬜ Instalador WiX → `Lwow-Setup-x64.exe`
5. ⬜ Benchmarks reais em Windows (RAM/CPU, 1/5/10/20 abas)
6. ⬜ Lwow Command, Focus, Workspaces, Privacy, Session Recovery

## Licença

MIT (a definir formalmente antes da primeira Release pública).
