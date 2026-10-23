# Estado Atual — distro-vault

Data: 23 de Outubro de 2026
Status: 🟢 Fase 2 (Low-Level Adapters) Concluída com Sucesso | 28 Testes Passando | Zero Warnings Policy Mantida

---

## 1. O Que Foi Realizado

### Fase 1: Core Domain & Ports
- [x] Implementado o crate `crates/domain` com arquitetura hexagonal e isolamento estrito de I/O.
- [x] Value Objects: `ProfileId`, `PackageName`, `DistroFamily`, `Checksum`, `UnixMode`, `SkillCategory`.
- [x] Entidades: `Manifest`, `ZenProfile`, `PackageEntry`, `AppImageEntry`, `AgentGovernance`, `CustomDirectoryEntry`, `SelectionFilter`.
- [x] Portas Hexagonais: `BrowserPort`, `PackagePort`, `StoragePort`, `AgentPort`, `SystemPort`.

### Fase 2: Low-Level Adapters (Módulos Profundos)
- [x] **`crates/adapter-browser`:**
  - Codec nativo MozLZ4 (`mozlz4.rs`) com validação de magic header `mozLz40\0`, tamanho LE e guardião contra Decompression Bombs (128 MB).
  - Normalizador de hashes de instalação do Mozilla (`profile_normalizer.rs`) com CityHash64 v1 (UTF-16LE) e sweep & bind automático de `installs.ini` e `profiles.ini`.
  - Implementação completa do trait `BrowserPort` em `ZenBrowserAdapter`.
  - 8 testes unitários passando, incluindo teste de descompressão contra sessão real do Zen Browser.
- [x] **`crates/adapter-storage`:**
  - Streaming `.tar.zst` (`archive.rs`) com nível de compressão zstd e retenção de permissões POSIX (`0700`, `0600`).
  - Guardião contra Zip-Slip (`ZipSlipGuard`) com rejeição imediata de `..`, caminhos absolutos, escapes de symlinks e dispositivos especiais.
  - Leitura rápida de `manifest.json` do topo do arquivo sem descompressão integral.
  - Manipulador de diretórios customizados (`custom_dirs.rs`) sob o namespace seguro `custom/<slug>`.
  - Implementação completa de `StoragePort` com cálculo SHA-256 via `sha2`.
  - 3 testes de integração passando.
- [x] **`crates/adapter-agents`:**
  - Scanner de regras e skills de IA (`scanner.rs`) com leitura de frontmatter YAML em `SKILL.md`.
  - Desreferenciação de symlinks (`copy_dereferenced`), garantindo que skills ligadas ao Omarchy sejam copiadas como diretórios físicos reais para operação autônoma em qualquer distribuição.
  - Implementação completa de `AgentPort` em `LocalAgentAdapter`.
  - 3 testes unitários passando, incluindo varredura real do ambiente.
- [x] **`crates/adapter-system`:**
  - Leitor e parser de `/etc/os-release` (`os_release.rs`) com normalização para `DistroFamily`.
  - Verificação de runtime de `libfuse2` para AppImages.
  - Gerenciador de processos e verificação de `.parentlock` (`process.rs`) com encerramento gracioso via SIGTERM.
  - Dispatchers de pacotes declarativos para `pacman` (`PacmanAdapter`), `paru`/`yay` (`AurAdapter`) e `flatpak` (`FlatpakAdapter`).
  - Implementação de `SystemPort` e `PackagePort`.
  - 3 testes unitários passando no host real Arch Linux.

---

## 2. Métricas de Qualidade e Compilação
- `cargo test --workspace` -> **28 testes passando (100% de sucesso)**
- `cargo clippy --all-targets --workspace -- -D warnings` -> **Zero Warnings**
- Dependências gerenciadas via Cargo Workspace Resolver 2.

---

## 3. Próximos Passos (Fase 3: Application Use Cases & Pipelines)
1. `crates/application`:
   - `BackupPipelineUseCase`: Orquestração completa do fluxo (Lock check -> Scan Browser -> Query Packages -> Scan Agents -> Inspect Custom Dirs -> Apply SelectionFilter -> Stream Tarball -> Compute Checksum).
   - `RestorePipelineUseCase`: Extração segura -> Restauração de perfis Zen e normalização de installs.ini -> Restauração de agentes -> Instalação de pacotes via PackagePort.
   - `InspectSnapshotUseCase` e `VerifyIntegrityUseCase`.
