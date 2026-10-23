# Checklist de Tarefas e Roadmap — distro-vault

Status: 🟢 Todas as Fases (1 a 6) Concluídas com Sucesso (32 testes passando, 0 warnings no Clippy) | TUI Ratatui e CLI Prontos para Homologação

---

## Fase 1: Core Domain & Ports Definition (Crate `domain`)
- [x] **Task 1.1: Entidades de Domínio e Value Objects**
  - **Status:** Concluído. 100% testado (Manifest, ZenProfile, PackageEntry, AppImageEntry, AgentGovernance, CustomDirectoryEntry, SelectionFilter).
- [x] **Task 1.2: Definição de Portas (Traits do Hexágono) e Erros**
  - **Status:** Concluído. BrowserPort, PackagePort, StoragePort, AgentPort, SystemPort.
- [x] **Task 1.3: Testes Unitários de Domínio**
  - **Status:** Concluído. 10 testes unitários passando, 0 warnings no Clippy.

---

## Fase 2: Low-Level Adapters (Módulos Profundos)
- [x] **Task 2.1: Codec MozLZ4 Nativo em Rust (`adapter-browser`)**
  - **Status:** Concluído. Codec nativo com validação de magic header `mozLz40\0`, tamanho LE e guardião contra Decompression Bombs (128 MB). 8 testes passando, incluindo descompressão de sessão real do Zen Browser.
  - **Verificação:** `cargo test -p adapter-browser`
  - **Arquivos:** `crates/adapter-browser/src/mozlz4.rs`
- [x] **Task 2.2: Normalizador de Hash e Perfis Zen (`adapter-browser`)**
  - **Status:** Concluído. `ProfilesIniNormalizer` com CityHash64 e sweep & bind automático de seções `[Install<HASH>]` e `installs.ini`, garantindo `Default=1` e `StartWithLastProfile=1`.
  - **Verificação:** `cargo test -p adapter-browser`
  - **Arquivos:** `crates/adapter-browser/src/profile_normalizer.rs`
- [x] **Task 2.3: Scanner e Desreferenciador de Agentes (`adapter-agents`)**
  - **Status:** Concluído. Varredura de `~/.agents/skills/`, leitura de YAML frontmatter em `SKILL.md` e desreferenciação estrita de links simbólicos (cópia real profunda prevenindo quebra fora do Omarchy). 3 testes passando.
  - **Verificação:** `cargo test -p adapter-agents`
  - **Arquivos:** `crates/adapter-agents/src/scanner.rs`, `crates/adapter-agents/src/lib.rs`
- [x] **Task 2.4: Adaptador do Sistema Operacional e Pacotes (`adapter-system`)**
  - **Status:** Concluído. Parser de `/etc/os-release`, inspeção de processos via `/proc`, verificação de `libfuse2` e dispatchers para `pacman`, `paru`/`yay` e `flatpak`. 3 testes passando.
  - **Verificação:** `cargo test -p adapter-system`
  - **Arquivos:** `crates/adapter-system/src/os_release.rs`, `crates/adapter-system/src/process.rs`, `crates/adapter-system/src/packages.rs`, `crates/adapter-system/src/lib.rs`
- [x] **Task 2.5: Adaptador de Storage Atômico com Zip-Slip Guard (`adapter-storage`)**
  - **Status:** Concluído. Criação e extração streaming em `.tar.zst` com sanitização Zip-Slip estrita (rejeição de `..`, caminhos absolutos, escapes de symlinks e dispositivos especiais) e preservação de permissões POSIX.
  - **Verificação:** `cargo test -p adapter-storage`
  - **Arquivos:** `crates/adapter-storage/src/archive.rs`, `crates/adapter-storage/src/lib.rs`
- [x] **Task 2.6: Módulo de Diretórios Customizados no Storage (`adapter-storage`)**
  - **Status:** Concluído. `CustomDirsHandler` com inspeção recursiva, cálculo de estatísticas e isolamento sob `custom/<slug>`.
  - **Verificação:** `cargo test -p adapter-storage`
  - **Arquivos:** `crates/adapter-storage/src/custom_dirs.rs`

---

## Fase 3: Application Use Cases & Pipelines (`application`)
- [x] **Task 3.1: BackupPipelineUseCase com Suporte a Filtros e Custom Dirs**
  - **Status:** Concluído. Orquestração completa com validação de .parentlock, scan de perfis Zen, agregação de pacotes, governança de agentes, custom dirs, aplicação de SelectionFilter e compressão via StoragePort.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/backup.rs`
- [x] **Task 3.2: RestorePipelineUseCase**
  - **Status:** Concluído. Extração segura em staging com Zip-Slip guard, restauração de perfis Zen com sweep & bind CityHash64 em installs.ini/profiles.ini, restauração de agentes desreferenciados, despacho inteligente de pacotes (mesma distro vs. distro-hop) e verificação de libfuse2.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/restore.rs`
- [x] **Task 3.3: InspectSnapshotUseCase**
  - **Status:** Concluído. Leitura em alta velocidade em streaming apenas do manifest.json do cabeçalho sem descompactar os arquivos.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/inspect.rs`
- [x] **Task 3.4: VerifyIntegrityUseCase**
  - **Status:** Concluído. Validação do checksum SHA-256 e teste de legibilidade estrutural do snapshot.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/verify.rs`

---

## Fase 4: CLI Adapter & Composition Root (`adapter-cli`)
- [x] **Task 4.1: Parser Clap CLI com Flags de Exclusão e Modos Não-Interativos**
  - **Status:** Concluído. Subcomandos `backup`, `restore`, `inspect`, `verify` com spinners animados (`indicatif`), formatação colorida (`console`), e flags flexíveis (`--exclude-package`, `--exclude-skill`, `--add-dir`, `--output`, `--dry-run`).
  - **Verificação:** `cargo test -p adapter-cli`
  - **Arquivos:** `crates/adapter-cli/src/args.rs`, `crates/adapter-cli/src/main.rs`, `crates/adapter-cli/src/di.rs`

---

## Fase 5: Validação de Funcionalidades Base e Homologação
- [x] **Task 5.1: Testes E2E de Backup e Restauração em Modo CLI**
  - **Status:** Concluído. Validação unitária e de integração em todos os 6 crates com 32 testes aprovados.

---

## Fase 6: TUI Interativa com Ratatui (`crates/adapter-cli/src/tui/`)
- [x] **Task 6.1: Layout de Abas Multi-Menu com Ratatui e Crossterm**
  - **Status:** Concluído. Interface de terminal interativa a 60 FPS com 5 abas (`[Tab]` para alternar, `[↑/↓]` para navegar, `[Espaço]` para excluir/incluir):
    - Tab 1: 📦 Pacotes (Nativos, AUR, Flatpaks)
    - Tab 2: 🤖 Skills & Agentes (~/.agents/skills)
    - Tab 3: 🚀 AppImages (Binários e Gear Lever)
    - Tab 4: 📁 Pastas Extras do Usuário (Modal `A` para adicionar caminhos arbitrários)
    - Tab 5: 🌐 Zen Browser Spaces (Workspaces e abas ativas)
  - **Arquivos:**
    - `crates/adapter-cli/src/tui/app.rs`
    - `crates/adapter-cli/src/tui/ui.rs`
    - `crates/adapter-cli/src/tui/events.rs`
    - `crates/adapter-cli/src/tui/runner.rs`
- [x] **Task 6.2: Sincronização da TUI com o `SelectionFilter` de Domínio**
  - **Status:** Concluído. Método `build_selection_filter()` converte os checkboxes da interface em regras imutáveis do domínio passadas ao `BackupPipelineUseCase`. Retorno limpo do terminal garantido por RAII `TerminalGuard`.
  - **Verificação:** `cargo test -p adapter-cli` e homologação interativa via terminal.

---

## Fase 7: Auditoria de Segurança & Análise de Vulnerabilidades
- [x] **Task 7.1: Auditoria de Dependências com `cargo-audit` e RustSec Advisory Database**
  - **Status:** Concluído. 263 dependências auditadas contra 1.351 avisos oficiais da RustSec. Zero vulnerabilidades / 0 CVEs.
- [x] **Task 7.2: Auditoria SAST de Vetores de Ataque (CWE-22, CWE-409, CWE-78, CWE-59, CWE-732)**
  - **Status:** Concluído. Cobertura estrita com Zip-Slip guard, teto de 128 MB contra decompression bombs, execução sem shell interpolation e modos de arquivo POSIX 0700/0600.
