# Checklist de Tarefas e Roadmap — distro-vault

Status: 🟢 Todas as Fases (1 a 14) Concluídas com Sucesso (43 testes passando, 0 warnings no Clippy) | Release Oficial v1.0.0 Publicada | Fase 15 (Roadmap Futuro v1.1.0+) Mapeada

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
  - **Status:** Concluído. Interface de terminal interativa a 60 FPS com 7 abas organizadas com cabeçalhos de categoria e rolagem dinâmica via `ListState` (`[Tab]` para alternar, `[↑/↓]` para navegar, `[Espaço]` para alternar item ou categoria em lote):
    - Tab 1: 🖥️ Apps & Usuário (Softwares instalados explicitamente pelo usuário)
    - Tab 2: ⚙️ Sistema Low-Level (Kernel, Firmware, Drivers, Boot, Rede, Daemons, Filesystems)
    - Tab 3: 📦 Dependências (Bibliotecas C/C++, Codecs/Gráficos, Runtimes, Toolchains e libs auxiliares)
    - Tab 4: 🤖 Skills & Agentes (~/.agents/skills)
    - Tab 5: 🚀 AppImages (Binários e Gear Lever)
    - Tab 6: 📁 Pastas Extras do Usuário (Modal `A` para adicionar caminhos arbitrários)
    - Tab 7: 🌐 Zen Browser Spaces (Workspaces e abas ativas)
  - **Status Scroll & Navegação:** Viewport stateful com rastreamento de cursor ativo e seleção em lote via cabeçalhos.
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

---

## Fase 8: Resiliência de Restauração, Transcrição Cross-Distro & TUI de Recuperação
- [x] **Task 8.1: Transcrição Canônica de Caminhos de Usuário (`RestorePipelineUseCase`)**
  - **Status:** Concluído. Função pura `transcribe_path_to_target_home` remapeia automaticamente qualquer `/home/<source_user>/...` para `target_home/...` preservando a estrutura exata do usuário. Testado com 4 asserções unitárias.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/restore.rs`, `crates/application/src/lib.rs`, `crates/application/tests/use_case_tests.rs`
- [x] **Task 8.2: Reordenação do Pipeline de Restauração e Prioridade de Dados do Usuário**
  - **Status:** Concluído. Restauração de configurações vitais (Perfis Zen, Agentes, Chaves SSH e Pastas Customizadas) movida para o início do pipeline, ocorrendo ANTES da instalação de pacotes de sistema, prevenindo que falhas de rede ou sudo bloqueiem a recuperação de dados.
  - **Arquivos:** `crates/application/src/restore.rs`
- [x] **Task 8.3: Acelerador de Staging em Cache (`--from-staging`)**
  - **Status:** Concluído. Suporte a `staging_override` em `RestoreRequest` e CLI flag `--from-staging`, permitindo reutilizar diretórios já descompactados em cache em segundos sem desgastar I/O de disco.
  - **Arquivos:** `crates/application/src/restore.rs`, `crates/adapter-cli/src/args.rs`, `crates/adapter-cli/src/main.rs`
- [x] **Task 8.4: Gestão de Privilégios `sudo` e Preservação de Usuário**
  - **Status:** Concluído. `execute_restore` valida previamente `sudo -v` de forma limpa no terminal antes de iniciar spinners quando pacotes estão ativados. `default_home()` inspeciona `SUDO_USER` para evitar poluição de `/root`.
  - **Arquivos:** `crates/adapter-cli/src/main.rs`
- [x] **Task 8.5: TUI Enriquecida para Modo de Recuperação (`crates/adapter-cli/src/tui/`)**
  - **Status:** Concluído. Exibição de alertas de `sudo`, detalhamento POSIX de chaves SSH (`0700`/`0600`), mapeamento de transcrição de pastas, detecção dinâmica de staging em cache e orientações claras de execução.
  - **Arquivos:** `crates/adapter-cli/src/tui/ui.rs`, `crates/adapter-cli/src/tui/app.rs`, `crates/adapter-cli/src/tui/runner.rs`

---

## Fase 9: Navegador Interativo Unificado de Arquivos e Diretórios (Explorer Modal)
- [x] **Task 9.1: Componente Unificado de Navegação no Filesystem (`FileExplorerState`)**
  - **Status:** Concluído. Estrutura unificada e desacoplada em `crates/adapter-cli/src/tui/explorer.rs` com suporte aos modos `CustomDir` (Backup) e `SnapshotOrFolder` (Restore). Testado com suíte de testes unitários dedicada.
  - **Verificação:** `cargo test -p adapter-cli`
  - **Arquivos:** `crates/adapter-cli/src/tui/explorer.rs`, `crates/adapter-cli/src/tui/mod.rs`
- [x] **Task 9.2: Integração de Eventos e Atalhos de Teclado no Explorer Modal**
  - **Status:** Concluído. Roteamento de eventos com navegação suave (`↑/↓/j/k`), expansão de diretório (`Enter`), confirmação de seleção de pasta (`Espaço/S`), subida de diretório (`Backspace`) e fechamento (`Esc/Q`).
  - **Arquivos:** `crates/adapter-cli/src/tui/events.rs`
- [x] **Task 9.3: Integração no Estado da Aplicação e Varredura de Backups (`TuiApp`)**
  - **Status:** Concluído. Integração do modal no `TuiApp` com função `scan_and_add_backup_folder` para importar instantaneamente snapshots `.tar.zst` descobertos em diretórios selecionados pelo usuário, e `add_custom_directory` para backup.
  - **Arquivos:** `crates/adapter-cli/src/tui/app.rs`
- [x] **Task 9.4: Renderização Visual do Modal de Exploração (`render_file_explorer`)**
  - **Status:** Concluído. Renderização limpa em Ratatui com cabeçalho de caminho atual, ícones contextuais (`📁`, `📦`), badges de tamanho em MB, highlight de cursor ativo e rodapé informativo de navegação.
  - **Arquivos:** `crates/adapter-cli/src/tui/ui.rs`

---

## Fase 10: Publicação da Release v0.3.0
- [x] **Task 10.1: Elevação de Versão no Workspace**
  - **Status:** Concluído. `version = "0.3.0"` definido no `Cargo.toml` raiz e sincronizado em `Cargo.lock` para todos os 7 crates internos.
  - **Verificação:** `cargo check --workspace`
  - **Arquivos:** `Cargo.toml`, `Cargo.lock`
- [x] **Task 10.2: Compilação de Release Otimizada e Sincronização de Binários**
  - **Status:** Concluído. Binários compilados via `cargo build --release` e sincronizados em `bin/distro-vault` e `~/distro-vault`. Verificado com `--version`.
  - **Arquivos:** `bin/distro-vault`, `~/distro-vault`
- [x] **Task 10.3: Documentação Viva e Registros de Arquitetura**
  - **Status:** Concluído. `tasks/todo.md` e `estado_atual.md` sincronizados.
- [x] **Task 10.4: Commit de Release e Tag Git v0.3.0**
  - **Status:** Concluído. Commit anotado e tag `v0.3.0` criados e enviados para o GitHub via SSH.

---

## Fase 11: Suporte a Backup e Restauração da Configuração Global do Git
- [x] **Task 11.1: Entidade e Contrato Hexagonal (`GitConfigEntry` & `GitConfigPort`)**
  - **Status:** Concluído. Entidade pura e trait definidas em `crates/domain/src/entities/git_config.rs` e `crates/domain/src/ports/git_config_port.rs`. `Manifest` expandido com `git_configs`.
  - **Verificação:** `cargo test -p domain`
  - **Arquivos:** `crates/domain/src/entities/git_config.rs`, `crates/domain/src/ports/git_config_port.rs`, `crates/domain/src/entities/manifest.rs`
- [x] **Task 11.2: Adaptador Concreto do Sistema (`LinuxGitConfigAdapter`)**
  - **Status:** Concluído. Implementação em `crates/adapter-system/src/git_config.rs` com varredura resiliente de `~/.gitconfig` e `~/.config/git/config`, extração de identidade `user.name` e `user.email`, e restauração com permissões POSIX `0644`.
  - **Verificação:** `cargo test -p adapter-system`
  - **Arquivos:** `crates/adapter-system/src/git_config.rs`, `crates/adapter-system/src/lib.rs`
- [x] **Task 11.3: Integração nos Pipelines de Aplicação (`backup.rs` & `restore.rs`)**
  - **Status:** Concluído. Backup captura arquivos sob prefixo `git/`. Restore implanta a configuração global no diretório alvo do usuário antes da instalação de pacotes de sistema.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/backup.rs`, `crates/application/src/restore.rs`
- [x] **Task 11.4: Apresentação na TUI e CLI (`adapter-cli`)**
  - **Status:** Concluído. Registro de identidade e arquivos Git exibidos no painel de inspeção da TUI de restauração e nos logs do CLI.
  - **Arquivos:** `crates/adapter-cli/src/di.rs`, `crates/adapter-cli/src/main.rs`, `crates/adapter-cli/src/tui/ui.rs`

---

## Fase 12: Seleção Granular de Itens no Restore com Menus Drop / Colapsáveis na TUI
- [x] **Task 12.1: Expansão do `SelectionFilter` no Domínio (`crates/domain`)**
  - **Status:** Concluído. Adicionados campos `restore_zen`, `restore_git_config`, `excluded_custom_dirs: HashSet<String>` e métodos utilitários `is_custom_dir_included`, `toggle_custom_dir`, `set_restore_zen`, `set_restore_git_config`. Teste unitário dedicado `test_selection_filter_granular_restore_options`.
  - **Verificação:** `cargo test -p domain`
  - **Arquivos:** `crates/domain/src/entities/selection_filter.rs`, `crates/domain/tests/domain_tests.rs`
- [x] **Task 12.2: Filtragem Granular no Caso de Uso de Restauração (`crates/application`)**
  - **Status:** Concluído. `RestoreRequest` enriquecido com `selection_filter: Option<SelectionFilter>`. `RestorePipelineUseCase::execute` condiciona restauração de Zen Browser, chaves SSH, skills de agentes, pastas customizadas, pacotes de sistema, verificação de fuse2 de AppImages e Git global config de acordo com os filtros.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/restore.rs`, `crates/application/tests/use_case_tests.rs`
- [x] **Task 12.3: Árvore Hierárquica Interativa (`RestoreTreeState`) no Adapter CLI**
  - **Status:** Concluído. Implementado `crates/adapter-cli/src/tui/restore_tree.rs` com categorias (`Pacotes do Sistema`, `AppImages`, `Navegador Zen`, `Chaves SSH`, `AI Agent Skills`, `Configuração Global Git`, `Diretórios Customizados`), nós filhos recolhíveis/expansíveis (`Enter`, `→`, `←`, `e`), alternância de seleção em lote ou individual (`Espaço`, `x`) e geração automática de `SelectionFilter`.
  - **Verificação:** `cargo test -p adapter-cli`
  - **Arquivos:** `crates/adapter-cli/src/tui/restore_tree.rs`, `crates/adapter-cli/src/tui/mod.rs`
- [x] **Task 12.4: Alternância de Foco e Redesenho da TUI de Restauração**
  - **Status:** Concluído. Adicionado `RestoreFocus` (`SnapshotsList` vs `ItemTree`) alternável via `Tab`. TUI redesenhada em painel duplo com árvore hierárquica à direita exibindo ramificações (`├──`, `└──`), glifos colapsáveis (`▼`, `►`), caixas de seleção (`[x]`, `[ ]`) e cursor com scroll dinâmico.
  - **Arquivos:** `crates/adapter-cli/src/tui/app.rs`, `crates/adapter-cli/src/tui/events.rs`, `crates/adapter-cli/src/tui/ui.rs`, `crates/adapter-cli/src/tui/runner.rs`, `crates/adapter-cli/src/main.rs`

---

## Fase 13: Geração de Sub-Dumps Derivados a partir de Dump Existente (Slice / Filter Sub-Snapshot)
- [x] **Task 13.1: Caso de Uso `SliceSnapshotUseCase` em `crates/application`**
  - **Status:** Concluído. Extrai o snapshot de origem em staging isolado com RAII `StagingCleanupGuard`, poda o `Manifest` e os arquivos descartando itens não selecionados no `SelectionFilter`, re-empacota atômico via `StoragePort::create_snapshot` e calcula novo checksum SHA-256.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/src/slice.rs`, `crates/application/src/lib.rs`
- [x] **Task 13.2: Testes Automatizados de Slicing e Repacotamento**
  - **Status:** Concluído. Teste de integração `test_slice_snapshot_filters_and_repackages_correctly` cobrindo exclusões de chaves SSH, exclusão de configs Git, integridade do manifesto e rejeição defensiva de sobrescrita de origem.
  - **Verificação:** `cargo test -p application`
  - **Arquivos:** `crates/application/tests/use_case_tests.rs`
- [x] **Task 13.3: Subcomando CLI `slice` em `crates/adapter-cli`**
  - **Status:** Concluído. `distro-vault slice <SOURCE> -o <OUTPUT>` com opções de linha de comando (`--exclude-package`, `--exclude-skill`, `--no-ssh`, `--no-zen`, `--no-git-config`, `--exclude-custom-dir`, `--staging-dir`) e modo interativo.
  - **Verificação:** `cargo test -p adapter-cli`
  - **Arquivos:** `crates/adapter-cli/src/args.rs`, `crates/adapter-cli/src/di.rs`, `crates/adapter-cli/src/main.rs`, `crates/adapter-cli/tests/cli_tests.rs`
- [x] **Task 13.4: Gatilho TUI `[S]` "Salvar Sub-Dump" no Modo Restore**
  - **Status:** Concluído. Tecla `[S]` / `[s]` mapeada no modo restore da TUI gerando automaticamente `<origem>-subset-<timestamp>.tar.zst` na pasta de backups a partir das caixas marcadas na árvore hierárquica.
  - **Arquivos:** `crates/adapter-cli/src/tui/app.rs`, `crates/adapter-cli/src/tui/events.rs`, `crates/adapter-cli/src/tui/ui.rs`, `crates/adapter-cli/src/tui/runner.rs`

---

## Fase 14: Marco 1.0.0 de Lançamento e Documentação Completa
- [x] **Task 14.1: Elevação de Versão no Workspace para 1.0.0**
  - **Status:** Concluído. `version = "1.0.0"` atualizado no `Cargo.toml` raiz e propagado em `Cargo.lock` para os 7 crates internos.
  - **Verificação:** `cargo check --workspace`
  - **Arquivos:** `Cargo.toml`, `Cargo.lock`
- [x] **Task 14.2: Reescrita e Expansão Completa do `README.md`**
  - **Status:** Concluído. Documentação abrangente cobrindo visão geral, guia visual da TUI, tabela completa de teclas de atalho, referência de todos os subcomandos CLI (`backup`, `restore`, `slice`, `inspect`, `verify`), arquitetura hexagonal e garantias de segurança.
  - **Arquivos:** `README.md`
- [x] **Task 14.3: Atualização dos Registros Vivos de Arquitetura**
  - **Status:** Concluído. `tasks/todo.md`, `estado_atual.md` e `estado_arquitetural_atual.md` sincronizados para a versão estável 1.0.0.
  - **Arquivos:** `tasks/todo.md`, `estado_atual.md`, `estado_arquitetural_atual.md`
- [x] **Task 14.4: Auditoria de Segurança Pré-Commit e Validação Global**
  - **Status:** Concluído. 43 testes de unidade e integração passando, Zero Warnings no Clippy (`-D warnings`) e zero credenciais/tokens rastreados.
  - **Verificação:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- [x] **Task 14.5: Build de Release Otimizada e Empacotamento de Assets**
  - **Status:** Concluído. Binários compilados via `cargo build --release` e sincronizados em `bin/distro-vault` e `~/distro-vault`.
  - **Arquivos:** `bin/distro-vault`, `~/distro-vault`
- [x] **Task 14.6: Push para Branch Remota e Publicação da Release v1.0.0**
  - **Status:** Concluído. Push em `master` e `main`, criação da tag `v1.0.0` e release oficial no GitHub.

---

## Fase 15: Personalização de Diretórios de I/O, Persistência TOML e Refinamento Ergonômico da TUI (v1.1.0)
- [x] **Task 15.1: Entidade `VaultConfig` e Porta `ConfigPort` no Domínio (`crates/domain`)**
  - **Status:** Concluído. Entidade pura `VaultConfig` (`output_dir`, `input_dirs`, `auto_install_packages`, `compact_view`, `staging_root`) com métodos construtores, saneamento de caminhos e trait `ConfigPort` em `crates/domain/src/ports/config_port.rs`.
  - **Verificação:** `cargo test -p domain` (15 testes passando).
  - **Arquivos:** `crates/domain/src/entities/vault_config.rs`, `crates/domain/src/ports/config_port.rs`, `crates/domain/tests/domain_tests.rs`.
- [x] **Task 15.2: Adaptador de Persistência `TomlConfigAdapter` em `adapter-system`**
  - **Status:** Concluído. Implementado em `crates/adapter-system/src/config.rs` persistindo em `~/.config/distro-vault/config.toml` com permissões POSIX `0644`, criação defensiva de diretórios pais e fallback gracioso em caso de arquivo corrompido.
  - **Verificação:** `cargo test -p adapter-system` (7 testes passando).
  - **Arquivos:** `crates/adapter-system/src/config.rs`, `crates/adapter-system/src/lib.rs`, `crates/adapter-system/tests/system_tests.rs`.
- [x] **Task 15.3: Desinchaço da TUI (Anti-Bloat) e Modais Ergonômicos Flutuantes**
  - **Status:** Concluído. Removida caixa estática inferior de parâmetros da coluna direita do Restore mode, liberando 100% da altura para a árvore hierárquica. Implementado modal flutuante de opções `[O]` (`OptionsModalState`), modal completo de gerenciamento de diretórios de entrada/saída `[C]` (`ConfigModalState`), e alternância de densidade compacta vs detalhada via `[V]`.
  - **Verificação:** `cargo test -p adapter-cli` (3 testes passando).
  - **Arquivos:** `crates/adapter-cli/src/tui/config_modal.rs`, `crates/adapter-cli/src/tui/ui.rs`, `crates/adapter-cli/src/tui/events.rs`, `crates/adapter-cli/src/tui/app.rs`.
- [x] **Task 15.4: Injeção no Composition Root e Ciclo de Vida da Aplicação**
  - **Status:** Concluído. Injetado `config_adapter` em `AppContainer` (`di.rs`), carregando as preferências do usuário no startup interativo (`main.rs`), alimentando múltiplos `input_dirs` em `populate_snapshots`, persistindo alterações defensivamente ao sair e aplicando nos destinos de backups e sub-dumps.
  - **Verificação:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`.
  - **Arquivos:** `crates/adapter-cli/src/di.rs`, `crates/adapter-cli/src/main.rs`, `crates/adapter-cli/src/tui/runner.rs`.

---

## Fase 16: Lançamento Oficial v1.1.1, Expansão Exaustiva da Documentação e Auditoria de Segurança
- [x] **Task 16.1: Elevação de Versão no Workspace para 1.1.1**
  - **Status:** Concluído. `Cargo.toml` elevado para `1.1.1` e propagado para os 7 crates internos em `Cargo.lock`.
  - **Verificação:** `cargo check --workspace`
  - **Arquivos:** `Cargo.toml`, `Cargo.lock`
- [x] **Task 16.2: Reescrita e Expansão Completa do `README.md`**
  - **Status:** Concluído. Documentação aprofundada com arquitetura hexagonal atualizada no Mermaid, mockups ASCII dos novos modais da TUI (`[O]` e `[C]`), tabela completa de atalhos, guia declarativo de `~/.config/distro-vault/config.toml` e 3 cenários práticos passo a passo.
  - **Arquivos:** `README.md`
- [x] **Task 16.3: Sincronização dos Registros Vivos de Arquitetura**
  - **Status:** Concluído. `tasks/todo.md`, `estado_atual.md` e `estado_arquitetural_atual.md` sincronizados para a versão 1.1.1.
  - **Arquivos:** `tasks/todo.md`, `estado_atual.md`, `estado_arquitetural_atual.md`
- [x] **Task 16.4: Auditoria de Segurança Pré-Commit e Validação Global**
  - **Status:** Concluído. 43 testes de unidade e integração aprovados, Zero Warnings no Clippy (`--all-targets -- -D warnings`) e validação de ausência total de credenciais e tokens no repositório.
  - **Verificação:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- [x] **Task 16.5: Compilação de Release Otimizada e Deploy Local**
  - **Status:** Concluído. Binários standalone compilados com `cargo build --release` e sincronizados em `bin/distro-vault` e `~/distro-vault`.
  - **Arquivos:** `bin/distro-vault`, `~/distro-vault`
- [x] **Task 16.6: Sincronização Git e Publicação da Release v1.1.1 no GitHub**
  - **Status:** Concluído. Push em `master` e `main`, e criação da release oficial `v1.1.1` com binário compilado anexado.

---

## Fase 17: Conformidade de Licença de Software Aberto (Licença MIT)
- [x] **Task 17.1: Criação do Arquivo Canônico `LICENSE`**
  - **Status:** Concluído. Criado na raiz do repositório contendo os termos da licença MIT padrão da OSI com Copyright 2026 Eduardo.
  - **Arquivos:** `LICENSE`
- [x] **Task 17.2: Atualização dos Metadados em `Cargo.toml` e `Cargo.lock`**
  - **Status:** Concluído. Campo `license = "MIT"` configurado no workspace e sincronizado no lockfile.
  - **Verificação:** `cargo check --workspace`
  - **Arquivos:** `Cargo.toml`, `Cargo.lock`
- [x] **Task 17.3: Sincronização da Documentação e Badges no `README.md`**
  - **Status:** Concluído. Badge e rodapé ajustados para a licença MIT.
  - **Arquivos:** `README.md`
- [x] **Task 17.4: Validação Global e Push para Branches Principais**
  - **Status:** Concluído. Suíte com 43 testes aprovados, Zero Warnings mantido e branches `master` e `main` sincronizadas.
  - **Verificação:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
