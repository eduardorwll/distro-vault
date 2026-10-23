# Checklist de Tarefas e Roadmap — distro-vault

Status: 🟢 Fase 3 (Application Use Cases) Concluída (30 testes passando, 0 warnings no Clippy) | Pronta para Fase 4 (CLI Adapter & Composition Root)

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
- [ ] **Task 4.1: Parser Clap CLI com Flags de Exclusão e Modos Não-Interativos**
  - **Descrição:** Subcomandos `backup`, `restore`, `inspect`, `verify` com spinners e flags (`--exclude-package`, `--add-dir`, etc.).

---

## Fase 5: Validação de Funcionalidades Base e Homologação
- [ ] **Task 5.1: Testes E2E de Backup e Restauração em Modo CLI**

---

## Fase 6: TUI Interativa com Ratatui (`crates/adapter-cli/src/tui/`)
- [ ] **Task 6.1: Layout de Abas Multi-Menu com Ratatui e Crossterm**
  - **Descrição:** Criar interface de terminal interativa com abas navegáveis via `Tab`:
    - Tab 1: 📦 Pacotes (Nativos, AUR, Flatpaks)
    - Tab 2: 🤖 Skills & Agentes (19 skills categorizadas)
    - Tab 3: 🚀 AppImages (Binários e Gear Lever)
    - Tab 4: 📁 Diretórios Customizados (Adicionar/Remover caminhos extras do usuário)
    - Tab 5: 🌐 Zen Browser Spaces (Visualização e seleção de workspaces)
  - **Critérios de Aceite:** Navegação fluida a 60 FPS no terminal; atalhos claros (`Tab` para trocar menu, `Espaço` para alternar inclusão/exclusão, `A` para adicionar diretório customizado, `Enter/F10` para iniciar snapshot).
  - **Verificação:** `cargo run -p adapter-cli -- backup --interactive`
  - **Arquivos:**
    - `crates/adapter-cli/src/tui/app.rs`
    - `crates/adapter-cli/src/tui/ui.rs`
    - `crates/adapter-cli/src/tui/events.rs`
    - `crates/adapter-cli/src/tui/tabs/*.rs`
- [ ] **Task 6.2: Sincronização da TUI com o `SelectionFilter` de Domínio**
  - **Descrição:** Ligar o estado visual da TUI ao `SelectionFilter`, garantindo que os itens desmarcados pelo usuário sejam excluídos deterministicamente antes do empacotamento.
  - **Critérios de Aceite:** O `Manifest` gerado reflete com precisão os itens excluídos e os diretórios customizados adicionados.
