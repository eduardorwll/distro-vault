# Estado Atual — distro-vault

Data: 25 de Outubro de 2026
Status: 🟢 Versão 1.1.1 Oficial Publicada | Todas as Fases (1 a 16) Concluídas com Sucesso | 43 Testes Passando | Zero Warnings Policy Rigorosamente Mantida | Persistência Declarativa TOML (~/.config/distro-vault/config.toml) | TUI Ratatui Desinchaçada com Modais Flutuantes (Opções [O] e Gerenciador de Diretórios [C]), Alternância de Densidade [V], Árvore Hierárquica em Altura Total e Slicing de Snapshots | Documentação Completa e Cenários Práticos Entregues

---

## 1. O Que Foi Realizado

### Fase 1: Core Domain & Ports (`crates/domain`)
- [x] Arquitetura Hexagonal pura com isolamento total de I/O.
- [x] Value Objects: `ProfileId`, `PackageName`, `DistroFamily`, `Checksum`, `UnixMode`, `SkillCategory`.
- [x] Entidades: `Manifest`, `ZenProfile`, `PackageEntry`, `AppImageEntry`, `AgentGovernance`, `CustomDirectoryEntry`, `SelectionFilter`, `SshKeyEntry`, `GitConfigEntry`, `VaultConfig`.
- [x] Portas Hexagonais: `BrowserPort`, `PackagePort`, `StoragePort`, `AgentPort`, `SystemPort`, `SshPort`, `GitConfigPort`, `ConfigPort`.

### Fase 2: Low-Level Adapters (Módulos Profundos)
- [x] **`crates/adapter-browser`:** MozLZ4 Codec com verificação de magic header `mozLz40\0`, tamanho LE e bomb guard (128 MB), normalizador CityHash64 v1 (UTF-16LE) em `installs.ini` e `profiles.ini` com sweep & bind para `Default=1`, e trait `BrowserPort` (8 testes passando).
- [x] **`crates/adapter-storage`:** Streaming `.tar.zst` (zstd level 3) com `ZipSlipGuard` (rejeição de escapes `..`, caminhos absolutos, symlinks perigosos e dispositivos especiais), modos POSIX (`0700`/`0600`), integridade SHA-256 e pastas customizadas em `custom/<slug>` (3 testes passando).
- [x] **`crates/adapter-agents`:** Scanner de regras e skills com parsing YAML frontmatter e desreferenciação física de links simbólicos para portabilidade fora do Omarchy (3 testes passando).
- [x] **`crates/adapter-system`:** Parser `/etc/os-release`, monitor `/proc`, verificação de `libfuse2`, scanner e restaurador POSIX de chaves SSH (`0700`/`0600`), scanner de Git config global, adaptador de configuração TOML `TomlConfigAdapter` com fallback resiliente e dispatchers para `pacman`, `aur`, `dnf`, `apt` e `flatpak` (7 testes passando).

### Fase 3: Application Use Cases & Pipelines (`crates/application`)
- [x] **`BackupPipelineUseCase` (`backup.rs`):** Orquestração completa do snapshot com suporte a Git config global, chaves SSH, agentes, Zen Browser e pacotes de sistema.
- [x] **`RestorePipelineUseCase` (`restore.rs`):** Restauração hierárquica granular com suporte a filtros detalhados (`SelectionFilter`).
- [x] **`SliceSnapshotUseCase` (`slice.rs`):** Geração de sub-dumps a partir de snapshots existentes com poda atômica do manifesto e re-empacotamento com novo checksum SHA-256.
- [x] **`InspectSnapshotUseCase` (`inspect.rs`):** Leitura de metadados em alta velocidade lendo apenas o `manifest.json` do topo do stream Zstandard.
- [x] **`VerifyIntegrityUseCase` (`verify.rs`):** Validação de integridade criptográfica SHA-256 e teste estrutural de blocos.

### Fase 15: Personalização de I/O, Persistência TOML e Desinchaço da TUI (v1.1.0)
- [x] **`VaultConfig` & `ConfigPort` no Domínio:** Entidade com `input_dirs`, `output_dir`, `auto_install_packages`, `compact_view` e `staging_root`.
- [x] **`TomlConfigAdapter`:** Persistência transparente e atômica em `~/.config/distro-vault/config.toml` com permissões POSIX `0644`.
- [x] **TUI Desinchaçada (Anti-Bloat):** Eliminação de painéis estáticos secundários na tela de Restore, dedicando 100% da coluna à árvore hierárquica interativa.
- [x] **Modal de Parâmetros de Execução `[O]`:** Submenu flutuante com toggles diretos para instalação de pacotes nativos (`[P]`), simulação Dry-Run (`[D]`) e densidade da árvore (`[V]`).
- [x] **Modal de Gerenciador de Diretórios `[C]`:** Gerenciamento visual do diretório padrão de saída (`[O]`), inclusão (`[+]`/`[A]`) e remoção (`[D]`/`[Del]`) de diretórios de entrada, alternância de densidade e persistência com `[S]` / `Enter`.
- [x] **Modo Compacto de Alta Densidade `[V]`:** Ocultação dinâmica de subtítulos/caminhos secundários na árvore, permitindo visualizar o triplo de nós simultaneamente sem necessidade de scroll.

---

## 2. Métricas de Qualidade e Compilação
- `cargo test --workspace` -> **43 testes passando (100% de sucesso)**
- `cargo clippy --workspace --all-targets -- -D warnings` -> **Zero Warnings Policy rigorosamente mantida**
- Binário de release compilado e sincronizado em `bin/distro-vault` e `/home/eduardorwll/distro-vault`.stema `LinuxGitConfigAdapter` varrendo `~/.gitconfig` e `~/.config/git/config`, extraindo metadados informativos de identidade do autor (`user.name`, `user.email`) e restaurando com permissão `0644`.
- [x] Implantação automática no `$HOME` da máquina alvo executada na fase de recuperação prioritária de dados do usuário (junto com SSH e perfis).
- [x] Exibição de identidade Git no painel de inspeção de snapshots da TUI e logs detalhados no CLI.

### Fase 12: Seleção Granular de Itens no Restore com Menus Drop / Colapsáveis na TUI
- [x] `SelectionFilter` expandido no Core Domain com flags (`restore_zen`, `restore_git_config`) e exclusão granular de diretórios customizados.
- [x] `RestorePipelineUseCase` condicionando a extração e restauração de cada subsistema com base no filtro selecionado.
- [x] Árvore hierárquica interativa (`RestoreTreeState`) com nós de categoria e nós folhas de itens individuais (Pacotes, AppImages, Perfis Zen, Chaves SSH, Skills de IA, Configuração Git e Diretórios Customizados).
- [x] Menus drop com expansão/recolhimento dinâmico (`Enter`, `→`, `←`, `e`), alternância em lote e individual de checkboxes (`Espaço`, `x`).
- [x] Navegação por foco duplo (`Tab`) entre a lista de snapshots disponíveis e a árvore de seleção hierárquica à direita.

### Fase 13: Geração de Sub-Dumps Derivados a partir de Dump Existente (Slice / Filter Sub-Snapshot)
- [x] Caso de uso `SliceSnapshotUseCase` em `crates/application` extraindo snapshot fonte em staging isolado com proteção RAII `StagingCleanupGuard`.
- [x] Poda do `Manifest` e filtragem atômica de arquivos para empacotar um novo snapshot `.tar.zst` autocontido com novo checksum SHA-256.
- [x] Subcomando CLI `distro-vault slice <ORIGEM> -o <DESTINO>` suportando filtros de linha de comando (`--exclude-package`, `--no-ssh`, etc.) e modo interativo.
- [x] Atalho `[S]` / `[s]` na TUI de restauração para gerar o sub-dump derivado instantaneamente a partir das marcações da árvore.

### Fase 14: Marco 1.0.0 de Lançamento e Documentação Completa
- [x] Versão `1.0.0` elevada no workspace Cargo e sincronizada em todos os crates internos.
- [x] `README.md` reescrito integralmente com guia visual da TUI, tabela completa de atalhos, referência de todos os subcomandos CLI (`backup`, `restore`, `slice`, `inspect`, `verify`), arquitetura hexagonal e garantias de segurança.
- [x] Auditoria de segurança confirmada (zero credenciais ou segredos em versionamento, 43 testes passando e zero warnings no Clippy).
- [x] Publicação da tag e release oficial `v1.0.0` no GitHub.

### Fase 15: Roadmap Futuro (v1.1.0+) — Personalização de I/O e Refinamento Ergonômico da TUI
- [ ] Personalização dinâmica de diretórios de entrada e saída na TUI com persistência em `~/.config/distro-vault/config.toml`.
- [ ] Refinamento e desinchaço ergonômico dos menus na TUI, com modal dedicado para configurações de execução e visualização compacta/expandida.

---

## 2. Métricas de Qualidade e Compilação
- `cargo test --workspace` -> **43 testes passando (100% de sucesso)**
- `cargo clippy --workspace --all-targets -- -D warnings` -> **Zero Warnings Policy rigorosamente mantida**
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
