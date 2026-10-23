# Checklist de Tarefas e Roadmap — distro-vault

Status: 🟢 Fase 1 Concluída | Pronta para Fase 2 (Low-Level Adapters) | Planejamento Expandido com Ratatui TUI & Custom Dirs

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
- [ ] **Task 2.1: Codec MozLZ4 Nativo em Rust (`adapter-browser`)**
  - **Descrição:** Implementar compressor e descompressor para o formato `mozLz40\0` do Firefox/Zen usando `lz4_flex`.
  - **Critérios de Aceite:** Validação de magic header de 8 bytes; leitura de tamanho descompactado LE de 4 bytes; testes com payloads de sessão reais.
  - **Verificação:** `cargo test -p adapter-browser`
  - **Arquivos:** `crates/adapter-browser/src/mozlz4.rs`
- [ ] **Task 2.2: Normalizador de Hash e Perfis Zen (`adapter-browser`)**
  - **Descrição:** Parse e reescrita de `installs.ini` e `profiles.ini` para forçar o perfil restaurado como `Default=1`.
  - **Critérios de Aceite:** Eliminar a falha onde o Zen abre vazio na nova distro.
  - **Verificação:** Testes unitários com fixtures de `installs.ini`.
  - **Arquivos:** `crates/adapter-browser/src/profile_normalizer.rs`
- [ ] **Task 2.3: Scanner e Desreferenciador de Agentes (`adapter-agents`)**
  - **Descrição:** Implementar varredura de `~/.agents/skills/`, leitura de `SKILL.md` e conversão de symlinks em cópias reais.
  - **Critérios de Aceite:** Skills do Omarchy mantidas funcionais sem depender de `/usr/share/omarchy/`.
  - **Verificação:** `cargo test -p adapter-agents`
  - **Arquivos:** `crates/adapter-agents/src/scanner.rs`
- [ ] **Task 2.4: Adaptador do Sistema Operacional e Pacotes (`adapter-system`)**
  - **Descrição:** Leitura de `/etc/os-release` e dispatchers para `pacman`, `paru`, `flatpak`, `dnf` e `apt`.
  - **Critérios de Aceite:** Identificação correta de famílias de SO e mapeamento de comandos.
  - **Verificação:** `cargo test -p adapter-system`
  - **Arquivos:** `crates/adapter-system/src/lib.rs`
- [ ] **Task 2.5: Adaptador de Storage Atômico com Zip-Slip Guard (`adapter-storage`)**
  - **Descrição:** Criação e extração de snapshots em `.tar.zst` preservando permissões POSIX (`0700`/`0600`) com guardião contra path traversal.
  - **Critérios de Aceite:** Checksum SHA-256 verificado após compressão; bloqueio de paths `../`.
  - **Verificação:** `cargo test -p adapter-storage`
  - **Arquivos:** `crates/adapter-storage/src/lib.rs`
- [ ] **Task 2.6: Módulo de Diretórios Customizados no Storage (`adapter-storage`)**
  - **Descrição:** Suporte para inclusão, empacotamento e descompressão de diretórios arbitrários escolhidos pelo usuário (`CustomDirectoryEntry`), isolados sob o prefixo `custom/<nome>`.
  - **Critérios de Aceite:** Verificação de existência, permissões e isolamento seguro de paths.
  - **Verificação:** `cargo test -p adapter-storage`
  - **Arquivos:** `crates/adapter-storage/src/custom_dirs.rs`

---

## Fase 3: Application Use Cases & Pipelines (`application`)
- [ ] **Task 3.1: BackupPipelineUseCase com Suporte a Filtros e Custom Dirs**
- [ ] **Task 3.2: RestorePipelineUseCase**
- [ ] **Task 3.3: InspectSnapshotUseCase**
- [ ] **Task 3.4: VerifyIntegrityUseCase**

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
