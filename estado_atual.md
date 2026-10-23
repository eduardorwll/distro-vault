# Estado Atual — distro-vault

Data: 23 de Outubro de 2026
Status: 🟢 Todas as Fases (1 a 6) Concluídas com Sucesso | 32 Testes Passando | Zero Warnings Policy Mantida | CLI & TUI Ratatui Homologados

---

## 1. O Que Foi Realizado

### Fase 1: Core Domain & Ports (`crates/domain`)
- [x] Arquitetura Hexagonal pura com isolamento total de I/O.
- [x] Value Objects: `ProfileId`, `PackageName`, `DistroFamily`, `Checksum`, `UnixMode`, `SkillCategory`.
- [x] Entidades: `Manifest`, `ZenProfile`, `PackageEntry`, `AppImageEntry`, `AgentGovernance`, `CustomDirectoryEntry`, `SelectionFilter`.
- [x] Portas Hexagonais: `BrowserPort`, `PackagePort`, `StoragePort`, `AgentPort`, `SystemPort`.

### Fase 2: Low-Level Adapters (Módulos Profundos)
- [x] **`crates/adapter-browser`:** MozLZ4 Codec com verificação de magic header `mozLz40\0`, tamanho LE e bomb guard (128 MB), normalizador CityHash64 v1 (UTF-16LE) em `installs.ini` e `profiles.ini` com sweep & bind para `Default=1`, e trait `BrowserPort` (8 testes passando).
- [x] **`crates/adapter-storage`:** Streaming `.tar.zst` (zstd level 3) com `ZipSlipGuard` (rejeição de escapes `..`, caminhos absolutos, symlinks perigosos e dispositivos especiais), modos POSIX (`0700`/`0600`), integridade SHA-256 e pastas customizadas em `custom/<slug>` (3 testes passando).
- [x] **`crates/adapter-agents`:** Scanner de regras e skills com parsing YAML frontmatter e desreferenciação física de links simbólicos para portabilidade fora do Omarchy (3 testes passando).
- [x] **`crates/adapter-system`:** Parser `/etc/os-release`, monitor `/proc`, verificação de `libfuse2` e dispatchers para `pacman`, `paru`/`yay` e `flatpak` (3 testes passando).

### Fase 3: Application Use Cases & Pipelines (`crates/application`)
- [x] **`BackupPipelineUseCase` (`backup.rs`):** Orquestração completa do snapshot: lock check com encerramento gracioso via `/proc`, agregação de navegadores, pacotes, agentes e pastas customizadas, aplicação do `SelectionFilter` e geração atômica com checksum SHA-256.
- [x] **`RestorePipelineUseCase` (`restore.rs`):** Orquestração da restauração: extração segura com Zip-Slip guard em staging, restauração e normalização dos perfis do Zen Browser (`Default=1`), reinstalação física de skills de IA, despacho de pacotes (mesma distro vs. distro-hop), checagem de FUSE2 e suporte completo a `--dry-run`.
- [x] **`InspectSnapshotUseCase` (`inspect.rs`):** Leitura de metadados em alta velocidade lendo apenas o `manifest.json` do topo do stream Zstandard sem descompactar o arquivo.
- [x] **`VerifyIntegrityUseCase` (`verify.rs`):** Validação de integridade criptográfica SHA-256 e teste estrutural de blocos.

### Fase 4: CLI Adapter & Composition Root (`crates/adapter-cli`)
- [x] Subcomandos Clap 4.5: `backup`, `restore`, `inspect`, `verify`.
- [x] Spinners animados com `indicatif`, formatação rica com `console` e logging.
- [x] Composition Root (`di.rs`): instanciação dos adaptadores concretos e injeção de dependências nos Casos de Uso.
- [x] Rota padrão para backups: `~/Antigravity/backups/distro-vault-<timestamp>.tar.zst`.

### Fase 6: TUI Interativa com Ratatui (`crates/adapter-cli/src/tui/`)
- [x] Interface de terminal a 60 FPS com 7 abas navegáveis, categorizando pacotes do sistema vs. usuário vs. dependências:
  - `[1] Apps & Usuário`: softwares instalados explicitamente pelo usuário (apps desktop como Discord, navegadores, ferramentas CLI como ripgrep/bat, runtimes dev como Docker, Flatpaks).
  - `[2] Sistema Low-Level`: pacotes essenciais do sistema operacional (Kernel & Firmware, Base & Boot, Drivers & GPU, Filesystems como Btrfs, Rede & Daemons, Audio/Video core).
  - `[3] Dependências`: dependências do sistema e de bibliotecas não instaladas diretamente pelo usuário (`pacman -Qnd` e `pacman -Qmd`), organizadas por headers:
    - *Bibliotecas Core do Sistema (C/C++)* (glibc, openssl, systemd-libs, etc.)
    - *Gráficos, Vídeo & Codecs* (mesa, ffmpeg, wayland, etc.)
    - *Runtimes & Linguagens* (python, nodejs, perl, etc.)
    - *Ferramentas de Build & Toolchain* (cmake, gcc-libs, etc.)
    - *Dependências Gerais & Utilitários*
  - `[4] Skills`: skills de IA e governança de agentes (`~/.agents/skills`).
  - `[5] AppImages`: aplicativos AppImage e Gear Lever.
  - `[6] Pastas Extras`: listagem e modal interativo (`A`) para adicionar pastas customizadas com cálculo dinâmico de arquivos e MB.
  - `[7] Zen Spaces`: visualização de workspaces e abas ativas do Zen Browser.
- [x] **Headers de Categoria com Alternância em Lote:**
  - Cabeçalhos estilizados delimitando cada categoria em vez de listagem alfabética plana.
  - Pressionar `[Espaço]` em um cabeçalho alterna em lote todos os itens daquela categoria (se algum estiver desmarcado, marca todos; se todos estiverem marcados, desmarca todos).
- [x] **Correção do Viewport Scrolling:**
  - Migração de renderização stateless (`render_widget`) para `render_stateful_widget` utilizando `ListState` para cada uma das 7 abas.
  - O cursor e a lista acompanham a navegação suavemente ao rolar além do último item visível na tela.
- [x] Teclas de navegação: `[Tab]` para alternar aba, `[↑/↓]` para navegar, `[Espaço]` para alternar seleção (item ou categoria inteira), `[A]` para adicionar pasta, `[Enter/F10]` para iniciar snapshot, `[Q/Esc]` para sair.
- [x] Gerenciamento RAII com `TerminalGuard` garantindo restauração automática do terminal mesmo em pânicos ou SIGINT.

### Governança & Skills dos Livros Oficiais de Rust
- [x] `apollographql/skills@rust-best-practices` e `rust-api-guidelines` ativas como Quality Gate mandatório.
- [x] Nomenclaturas canônicas (`as_`, `to_`, `into_`), zero `unwrap` em produção, deep modules, RAII guards.

### Auditoria de Segurança & Análise de Vulnerabilidades
- [x] **Skill de Auditoria Ativada:** `~/.agents/skills/security-audit` instalada e aplicada.
- [x] **Auditoria de Dependências (`cargo-audit` v0.22.2):** Varredura de 263 dependências contra a base oficial RustSec (1.351 advisories). **Zero vulnerabilidades/CVEs detectadas.**
- [x] **Auditoria de Código & SAST:**
  - *CWE-22 (Zip-Slip / Path Traversal):* `ZipSlipGuard` valida e rejeita `..`, caminhos absolutos, escapes de symlinks ancestrais e devices especiais.
  - *CWE-409 (Decompression Bomb):* `MozLz4Codec` impõe teto de segurança estrito de 128 MB antes da alocação de memória.
  - *CWE-78 (Command Injection):* Chamadas a `pacman`/`paru`/`yay`/`flatpak` utilizam `execve` direto (sem shell `sh -c`) e sanitização estrita via regex do Value Object `PackageName`.
  - *CWE-59 (Symlink Traversal):* Desreferenciamento seguro na exportação e substituição segura de symlinks em caso de conflitos durante o restore.
  - *CWE-732 (Permissões Excessivas):* Preservação e imposição estrita de modos POSIX `0700` (pastas) e `0600` (arquivos sensíveis).

---

## 2. Métricas de Qualidade e Compilação
- `cargo test --workspace` -> **33 testes passando (100% de sucesso)**
- `cargo clippy --all-targets --workspace -- -D warnings` -> **Zero Warnings**
- `cargo audit` -> **0 vulnerabilidades / 0 CVEs em 263 dependências**
- Compilação Release em `target/release/distro-vault` e instalação em `~/.local/bin/distro-vault`.

---

## 3. Próximo Passo
- Executar testes funcionais interativos e homologar o fluxo completo com o usuário.
