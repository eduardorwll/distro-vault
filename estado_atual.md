# Estado Atual — distro-vault

Data: 23 de Outubro de 2026
Status: 🟢 Fase 3 (Application Use Cases) Concluída com Sucesso | 30 Testes Passando | Zero Warnings Policy Mantida | Rust Books Skills Integradas

---

## 1. O Que Foi Realizado

### Fase 1: Core Domain & Ports
- [x] Implementado o crate `crates/domain` com arquitetura hexagonal e isolamento estrito de I/O.
- [x] Value Objects: `ProfileId`, `PackageName`, `DistroFamily`, `Checksum`, `UnixMode`, `SkillCategory`.
- [x] Entidades: `Manifest`, `ZenProfile`, `PackageEntry`, `AppImageEntry`, `AgentGovernance`, `CustomDirectoryEntry`, `SelectionFilter`.
- [x] Portas Hexagonais: `BrowserPort`, `PackagePort`, `StoragePort`, `AgentPort`, `SystemPort`.

### Fase 2: Low-Level Adapters (Módulos Profundos)
- [x] **`crates/adapter-browser`:** MozLZ4 Codec com bomb guard (128 MB), normalizador CityHash64 v1 (UTF-16LE) em `installs.ini` e `profiles.ini`, e trait `BrowserPort` (8 testes passando).
- [x] **`crates/adapter-storage`:** Streaming `.tar.zst` com `ZipSlipGuard`, POSIX modes (`0700`/`0600`), integridade SHA-256 e pastas customizadas em `custom/<slug>` (3 testes passando).
- [x] **`crates/adapter-agents`:** Scanner de regras e skills com parsing YAML e desreferenciação física de links simbólicos para portabilidade fora do Omarchy (3 testes passando).
- [x] **`crates/adapter-system`:** Parser `/etc/os-release`, monitor `/proc`, verificação `libfuse2` e dispatchers para `pacman`, `paru`/`yay` e `flatpak` (3 testes passando).

### Fase 3: Application Use Cases & Pipelines (`crates/application`)
- [x] **`BackupPipelineUseCase` (`backup.rs`):** Orquestração completa do fluxo de backup: lock check com encerramento gracioso, agregação de navegadores, pacotes, agentes e pastas customizadas, aplicação do `SelectionFilter` e geração de snapshot com checksum SHA-256.
- [x] **`RestorePipelineUseCase` (`restore.rs`):** Orquestração da restauração: extração segura com Zip-Slip guard, restauração e normalização dos perfis do Zen Browser (`Default=1`), reinstalação física de skills de IA, despacho de pacotes (mesma distro vs. distro-hop) e checagem de FUSE2.
- [x] **`InspectSnapshotUseCase` (`inspect.rs`):** Leitura de metadados em alta velocidade lendo apenas o `manifest.json` do topo do stream Zstandard sem descompactar o arquivo.
- [x] **`VerifyIntegrityUseCase` (`verify.rs`):** Validação de integridade criptográfica SHA-256 e teste estrutural de blocos.
- [x] Testes de integração em `crates/application/tests/use_case_tests.rs` passando com 100% de sucesso.

### Governança & Skills dos Livros Oficiais de Rust
- [x] Instalada a skill comunitária `apollographql/skills@rust-best-practices` (20.9K installs).
- [x] Criada a skill nativa canônica `~/.agents/skills/rust-api-guidelines/` com base nos RFCs e livros oficiais do time do Rust (`C-NAMING`, `C-CONV`, `C-COMMON-TRAITS`, `C-ERRORS`, `C-BUILDER`).
- [x] Atualizados `AGENTS.md` e `GEMINI.md` definindo a conformidade com os livros oficiais do Rust como Quality Gate Mandatório.

---

## 2. Métricas de Qualidade e Compilação
- `cargo test --workspace` -> **30 testes passando (100% de sucesso)**
- `cargo clippy --all-targets --workspace -- -D warnings` -> **Zero Warnings**
- Dependências gerenciadas via Cargo Workspace Resolver 2.

---

## 3. Próximos Passos (Fase 4: CLI Adapter & Composition Root)
1. `crates/adapter-cli`:
   - Configuração do parser CLI com `clap` (subcomandos `backup`, `restore`, `inspect`, `verify`).
   - Conexão do Composition Root: instanciação de adapters reais e injeção nos Use Cases.
   - Spinners animados com `indicatif`.
2. Fase 6: TUI Interativa com Ratatui (`crates/adapter-cli/src/tui/`).
