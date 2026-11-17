# Como compilar o Lwow e gerar o instalador (Windows)

## Pré-requisitos

1. **Rust (MSVC)** — instale via [rustup](https://rustup.rs/):
   ```powershell
   winget install Rustlang.Rustup
   ```
2. **Visual Studio Build Tools 2022** — workload "Desktop development with C++":
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```
3. **WiX Toolset** — para gerar o instalador:
   ```powershell
   winget install WiXToolset.WiXToolset
   ```
4. **WebView2 Runtime** — já presente no Windows 10/11 atualizado. Se ausente,
   o instalador baixa o Evergreen oficial da Microsoft automaticamente.

## 1. Compilar o navegador

```powershell
git clone https://github.com/creaturesofsonaria616-creator/LWOW.git
cd LWOW
git checkout feature/project-setup
cargo build --release
```

Saída: `target\release\lwow.exe`

## 2. Gerar o instalador Lwow-Setup-x64.exe

Do diretório do projeto:

```powershell
cd installer
wix.exe build lwow.wxs -o Lwow-Setup-x64.exe -arch x64 -d "SourceDir=.."
```

(WiX v4/v5. Para WiX v3, use `candle.exe` + `light.exe` — comandos no comentário
do topo de `lwow.wxs`.)

### Antes de distribuir publicamente

1. Substitua os placeholders `PUT-GUID-HERE-1` e `PUT-GUID-HERE-2` em
   `installer/lwow.wxs` por GUIDs únicos gerados uma única vez
   (`[guid]::NewGuid()` no PowerShell). O `UpgradeCode` do produto **nunca**
   deve mudar depois da primeira versão publicada.
2. Gere o ícone `assets/lwow.ico` e coloque no repositório.
3. **Assine** o `lwow.exe` e o `Lwow-Setup-x64.exe` com um certificado real:
   ```powershell
   signtool sign /fd SHA256 /tr http://timestamp.digicert.com lwow.exe
   ```
   Sem assinatura, o Windows SmartScreen exibirá advertências — não desative
   as proteções do Windows; assine o aplicativo.
4. Gere o hash SHA-256 do instalador para publicar junto da Release:
   ```powershell
   Get-FileHash Lwow-Setup-x64.exe -Algorithm SHA256
   ```

## 3. Teste final (checklist)

- [ ] `lwow.exe` abre com a janela principal
- [ ] Navegação real em sites modernos (https)
- [ ] Abrir/fechar 20 abas sem instabilidade
- [ ] Suspensão de abas reduz o RAM no Gerenciador de Tarefas
- [ ] Instalador: instala sem admin (per-user), atalho no Menu Iniciar
- [ ] Desinstalador remove o app e preserva dados do usuário quando escolhido
- [ ] WebView2 detectado/instalado corretamente em máquina limpa (VM)

## Limitações conhecidas (v0.1)

- A integração WebView2 por aba ainda está em desenvolvimento: o shell Slint e
  a ponte HWND estão prontos; a montagem do child webview por aba é o próximo
  marco (ver `docs/ROADMAP.md`).
- Benchmarks de RAM/CPU só serão publicados após medições reais em hardware
  Windows — nenhum número é divulgado antes disso.
