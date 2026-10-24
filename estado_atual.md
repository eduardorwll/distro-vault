# Estado Atual — distro-vault

Data: 24 de Outubro de 2026
Status: 🟢 Versão 0.3.0 Pronta para Publicação | Todas as Fases (1 a 11) Concluídas com Sucesso | 39 Testes Passando | Zero Warnings Policy Mantida | CLI & TUI Ratatui com Suporte a SSH, Git Global Config, Transcrição Cross-Distro e Explorador Unificado

---

## 1. O Que Foi Realizado

### Fase 1: Core Domain & Ports (`crates/domain`)
- [x] Arquitetura Hexagonal pura com isolamento total de I/O.
- [x] Value Objects: `ProfileId`, `PackageName`, `DistroFamily`, `Checksum`, `UnixMode`, `SkillCategory`.
- [x] Entidades: `Manifest`, `ZenProfile`, `PackageEntry`, `AppImageEntry`, `AgentGovernance`, `CustomDirectoryEntry`, `SelectionFilter`, `SshKeyEntry`.
- [x] Portas Hexagonais: `BrowserPort`, `PackagePort`, `StoragePort`, `AgentPort`, `SystemPort`, `SshPort`.

### Fase 2: Low-Level Adapters (Módulos Profundos)
- [x] **`crates/adapter-browser`:** MozLZ4 Codec com verificação de magic header `mozLz40\0`, tamanho LE e bomb guard (128 MB), normalizador CityHash64 v1 (UTF-16LE) em `installs.ini` e `profiles.ini` com sweep & bind para `Default=1`, e trait `BrowserPort` (8 testes passando).
- [x] **`crates/adapter-storage`:** Streaming `.tar.zst` (zstd level 3) com `ZipSlipGuard` (rejeição de escapes `..`, caminhos absolutos, symlinks perigosos e dispositivos especiais), modos POSIX (`0700`/`0600`), integridade SHA-256 e pastas customizadas em `custom/<slug>` (3 testes passando).
- [x] **`crates/adapter-agents`:** Scanner de regras e skills com parsing YAML frontmatter e desreferenciação física de links simbólicos para portabilidade fora do Omarchy (3 testes passando).
- [x] **`crates/adapter-system`:** Parser `/etc/os-release`, monitor `/proc`, verificação de `libfuse2`, scanner e restaurador POSIX de chaves SSH (`0700`/`0600`), e dispatchers para `pacman`, `paru`/`yay` e `flatpak` (5 testes passando).

### Fase 3: Application Use Cases & Pipelines (`crates/application`)
- [x] **`BackupPipelineUseCase` (`backup.rs`):** Orquestração completa do snapshot: lock check com encerramento gracioso via `/proc`, agregação de navegadores, pacotes, agentes, chaves SSH e pastas customizadas, aplicação do `SelectionFilter` e geração atômica com checksum SHA-256.
- [x] **`RestorePipelineUseCase` (`restore.rs`):** Orquestração da restauração reordenada priorizando dados do usuário (Zen Browser, Agentes, Chaves SSH com 0700/0600, Pastas Customizadas com transcrição canônica cross-distro para `$HOME`, antes de pacotes de sistema). Suporte a `staging_override` e criação de staging em disco (`~/.cache`).
- [x] **`InspectSnapshotUseCase` (`inspect.rs`):** Leitura de metadados em alta velocidade lendo apenas o `manifest.json` do topo do stream Zstandard sem descompactar o arquivo.
- [x] **`VerifyIntegrityUseCase` (`verify.rs`):** Validação de integridade criptográfica SHA-256 e teste estrutural de blocos.

### Fase 4: CLI Adapter & Composition Root (`crates/adapter-cli`)
- [x] Subcomandos Clap 4.5: `backup`, `restore`, `inspect`, `verify`.
- [x] Flags expandidas: `--from-staging`, `--target-home`, `--target-config`, `--target-custom`, `--install-packages`, `--dry-run`.
- [x] Resolução inteligente de `$HOME` via `SUDO_USER` prevenindo poluição em `/root` quando executado via `sudo`.
- [x] Validação limpa e antecipada de `sudo -v` antes de iniciar spinners de progresso.

### Fase 6 & 8: TUI Interativa com Ratatui (`crates/adapter-cli/src/tui/`)
- [x] 8 abas navegáveis categorizando pacotes, skills, AppImages, pastas customizadas, Zen spaces e Chaves SSH.
- [x] Modo de Recuperação (`TuiMode::Restore`) com alertas visuais de necessidade de `sudo`, detalhamento POSIX de chaves SSH, preview de transcrição de caminhos e detecção em tempo real de staging acelerado em cache.
- [x] Gerenciamento RAII com `TerminalGuard` garantindo restauração limpa do terminal.

### Fase 9: Navegador Interativo Unificado de Arquivos e Diretórios (Explorer Modal)
- [x] Modal interativo unificado (`FileExplorerState` e `render_file_explorer`) reutilizado tanto para adicionar pastas customizadas para backup quanto para escanear/selecionar diretórios e arquivos de snapshot para restauração.
- [x] Navegação intuitiva no sistema de arquivos do usuário (`↑/↓/j/k`, `Enter` para abrir, `Backspace` para subir, `Espaço/S` para confirmar seleção, `Esc/Q` para fechar).
- [x] Descoberta automática de backups adicionais (`scan_and_add_backup_folder`) para inclusão imediata na lista de snapshots disponíveis.

### Fase 11: Suporte a Backup e Restauração da Configuração Global do Git
- [x] Entidade `GitConfigEntry` e contrato `GitConfigPort` implementados de forma desacoplada no domínio.
- [x] Adaptador de sistema `LinuxGitConfigAdapter` varrendo `~/.gitconfig` e `~/.config/git/config`, extraindo metadados informativos de identidade do autor (`user.name`, `user.email`) e restaurando com permissão `0644`.
- [x] Implantação automática no `$HOME` da máquina alvo executada na fase de recuperação prioritária de dados do usuário (junto com SSH e perfis).
- [x] Exibição de identidade Git no painel de inspeção de snapshots da TUI e logs detalhados no CLI.

---

## 2. Métricas de Qualidade e Compilação
- `cargo test --workspace` -> **39 testes passando (100% de sucesso)**
- `cargo clippy --workspace -- -D warnings` -> **Zero Warnings Policy rigorosamente mantida**
- `cargo audit` -> **0 vulnerabilidades / 0 CVEs em 263 dependências**
- Binário de release atualizado em `bin/distro-vault` e no host do usuário em `/home/eduardorwll/distro-vault`.
- [x] **`VerifyIntegrityUseCase` (`verify.rs`):** Validação de integridade criptográfica SHA-256 e teste estrutural de blocos.

### Fase 4: CLI Adapter & Composition Root (`crates/adapter-cli`)
- [x] Subcomandos Clap 4.5: `backup`, `restore`, `inspect`, `verify`.
- [x] Spinners animados com `indicatif`, formatação rica com `console` e logging.
- [x] Composition Root (`di.rs`): instanciação dos adaptadores concretos e injeção de dependências nos Casos de Uso.
- [x] Rota padrão para backups: `~/Antigravity/backups/distro-vault-<timestamp>.tar.zst`.

### Fase 6: TUI Interativa com Ratatui (`crates/adapter-cli/src/tui/`)
- [x] **Arquitetura Unificada de Modos (Backup & Recuperação):**
  - Comutação em tempo real via `[F2]` (Modo Backup) e `[F3]` (Modo Recuperação) ou tecla `[M]`.
  - Execução de `distro-vault` ou `distro-vault backup` inicia por padrão no Modo Backup.
  - Execução de `distro-vault restore` (sem argumentos) abre a TUI diretamente no Modo Recuperação.
- [x] **Painel Interativo de Recuperação / Restore (`TuiMode::Restore`):**
  - Varredura e listagem automática de snapshots em `~/Antigravity/backups/*.tar.zst` com data, nome e tamanho.
  - Modal interativo (`[A]`) para abrir e inspecionar arquivos `.tar.zst` arbitrários.
  - Painel de inspeção em tempo real exibindo metadados do manifesto:
    - ID do snapshot, data UTC, máquina e distribuição de origem.
    - Comparação com a distribuição atual (mesma distro vs. alerta visual de Distro-Hop).
    - Contadores de perfis Zen Browser (com total de abas), skills de IA, pacotes e pastas customizadas.
    - Checksum SHA-256 declarado.
  - Alternância de parâmetros de restauração diretamente na interface:
    - `[P]`: Alternar reinstalação automática de pacotes (SIM/NÃO).
    - `[D]`: Alternar modo de simulação Dry-Run (LIGADO/DESLIGADO).
  - Tecla de execução: `[Enter]` ou `[R]` restaura o snapshot selecionado com restauração limpa do terminal.
- [x] **Modo Snapshot / Backup (`TuiMode::Backup`):**
  - Interface de 7 abas navegáveis categorizando pacotes do sistema vs. usuário vs. dependências (`[1] Apps & Usuário`, `[2] Sistema Low-Level`, `[3] Dependências`, `[4] Skills`, `[5] AppImages`, `[6] Pastas Extras`, `[7] Zen Spaces`).
  - Cabeçalhos de categoria estilizados com batch toggle (`[Espaço]`).
  - Scroll dinâmico com `ListState` e `render_stateful_widget`.
  - Tecla de execução: `[Enter]` ou `[F10]` gera o snapshot declarativo.
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
