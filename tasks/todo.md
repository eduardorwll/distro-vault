# Checklist de Tarefas e Roadmap — distro-vault

Status: 🟢 Fase 1 Concluída com Sucesso | Pronta para Fase 2 (Low-Level Adapters)

---

## Fase 1: Core Domain & Ports Definition (Crate `domain`)
- [x] **Task 1.1: Entidades de Domínio e Value Objects**
  - **Descrição:** Implementar estruturas de dados imutáveis para:
    - `Manifest` (metadados do snapshot, timestamp, host, checksum SHA-256)
    - `ZenProfile` & `Space` (sessão LZ4, abas, containers, caminhos)
    - `PackageEntry` & `PackageFamily` (Pacman, AUR, Flatpak, DNF, APT, Nix)
    - `AppImageEntry` (caminho, permissões, metadados Gear Lever)
    - `AgentGovernance` & `SkillEntry` (regras, skills categorizadas, quality gates ativos)
    - Value Objects: `ProfileId`, `PackageName`, `DistroFamily`, `Checksum`, `UnixMode`
  - **Status:** Concluído. 100% testado.
- [x] **Task 1.2: Definição de Portas (Traits do Hexágono) e Erros**
  - **Descrição:** Definir traits das portas: `PackagePort`, `BrowserPort`, `StoragePort`, `AgentPort`, `SystemPort`.
  - **Status:** Concluído. Zero vazamento de I/O no core domain.
- [x] **Task 1.3: Testes Unitários de Domínio**
  - **Descrição:** Testes unitários para criação de entidades, validação de invariantes e serialização de manifesto.
  - **Status:** Concluído. 8 testes unitários passando, 0 warnings no Clippy.

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

---

## Fase 3: Application Use Cases & Pipelines (`application`)
- [ ] **Task 3.1: BackupPipelineUseCase**
- [ ] **Task 3.2: RestorePipelineUseCase**
- [ ] **Task 3.3: InspectSnapshotUseCase**
- [ ] **Task 3.4: VerifyIntegrityUseCase**

---

## Fase 4: CLI Adapter & Composition Root (`adapter-cli`)
- [ ] **Task 4.1: Parser Clap e TUI Indicatif**

---

## Fase 5: Validação Final e Release
- [ ] **Task 5.1: Suíte de Testes E2E e Homologação**
