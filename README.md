# Distro-Vault 🦀📦

> **Sistema de Alta Performance em Rust para Backup Declarativo, Restauração Cirúrgica, Slicing de Snapshots e Transição sem Atrito entre Distribuições Linux (Distro Hopping).**

[![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Zero Clippy Warnings](https://img.shields.io/badge/clippy-zero%20warnings-brightgreen.svg?style=flat-square)](Cargo.toml)
[![Release v1.1.1](https://img.shields.io/badge/release-v1.1.1-success.svg?style=flat-square)](https://github.com/eduardorwll/distro-vault/releases/tag/v1.1.1)

![Arquitetura da Aplicação](docs/architecture.svg)

---

## 📖 Visão Geral

O **distro-vault** é uma ferramenta de infraestrutura pessoal projetada para resolver a dor de reinstalar e reconfigurar sistemas Linux durante migrações ou recuperação de desastres (*Disaster Recovery*). Construído sobre os pilares da **Clean Architecture** (Arquitetura Hexagonal) e **Deep Modules**, ele trata o estado do usuário de forma declarativa e atômica.

Seja saltando de um Arch Linux / Omarchy para um Fedora, Debian ou Ubuntu, o `distro-vault` garante que suas sessões do Zen Browser, chaves SSH, identidades Git, regras de agentes de IA, AppImages e pastas de trabalho sejam reconstituídas de maneira imediata, segura e consistente.

---

## 🌟 Principais Recursos

- 🌐 **Preservação Nativa do Zen Browser:**
  - Extração e empacotamento atômico de Workspaces (Spaces), abas fixadas e abas abertas através de codec nativo Rust para `mozLz40\0` (`lz4_flex`).
  - Normalizador automático de hashes de instalação via CityHash64 v1 (UTF-16LE) com injeção automática em `profiles.ini` e `installs.ini`, garantindo `Default=1` e inicialização instantânea no host de destino.
- 📦 **Gerenciamento Declarativo Multi-Distro:**
  - Descoberta inteligente e segregação de pacotes entre Usuário (`DesktopApp`, `CliTool`, `Development`) e Sistema (`Kernel`, `Drivers`, `Audio`, `Network`).
  - Suporte agnóstico para `pacman`, `paru`/`yay` (AUR), `flatpak`, `dnf`, `apt` e `nix`.
- 🤖 **Sincronização de Ecossistema de IA & Governança:**
  - Detecção e catalogação declarativa das 19 skills essenciais e regras do ecossistema de agentes (`AGENTS.md`, `GEMINI.md`).
  - Desreferenciamento físico de links simbólicos: resolve arquivos reais prevenindo quebras em sistemas sem as árvores de symlinks originais.
- ⚙️ **Auto-Deployment da Configuração Global do Git:**
  - Captura transparente de `~/.gitconfig` e `~/.config/git/config`.
  - Preservação da identidade do autor (`user.name`, `user.email`), aliases e configurações com permissão POSIX `0644`.
- 🌲 **Restauração Granular com Menus Drop (TUI):**
  - Árvore interativa no painel de restauração da TUI com foco duplo (`Tab`).
  - Menus drop colapsáveis por categoria e nós folhas individuais para alternar via caixas de seleção (`[x]`/`[ ]`) exatamente o que restaurar.
- 💾 **Geração de Sub-Dumps Derivados (Snapshot Slicing):**
  - Capacidade de selecionar um dump existente e extrair cirurgicamente apenas os componentes desejados para gerar um novo arquivo `.tar.zst` autocontido (via CLI `distro-vault slice` ou tecla `[S]` na TUI).
- 🔁 **Transcrição Canônica Cross-Distro:**
  - O algoritmo `transcribe_path_to_target_home` mapeia automaticamente caminhos de arquivos de origem (`/home/<user_origem>/...`) para o `$HOME` do usuário no destino, eliminando incompatibilidades de nomes de usuário.
- ⚡ **Staging Acelerado em Cache (`--from-staging`):**
  - Detecção automática e reaproveitamento de diretórios já descompactados em cache em `~/.cache/distro-vault/staging/`, permitindo restaurar sistemas pesados em segundos sem overhead de descompressão.
- 📁 **Explorador Unificado de Arquivos (`FileExplorerState`):**
  - Modal interativo (`[A]`) reutilizado para selecionar pastas extras de backup e explorar snapshots no sistema de arquivos.
- 🔒 **Segurança POSIX & Guardiões de I/O:**
  - Imposição rigorosa de modos `0700` (diretórios confidenciais) e `0600` (chaves privadas SSH).
  - Defesa contra **Zip-Slip (CWE-22)** via `ZipSlipGuard` e teto de segurança contra **Decompression Bombs (CWE-409)** com limite de 128 MB.

---

## 🖥️ Guia da Interface Interativa (TUI)

A TUI interativa é desenvolvida com **Ratatui 0.30** e **Crossterm 0.29**, operando em modo raw com proteção RAII (`TerminalGuard`) contra quebra de terminal.

Na versão 1.1+, a interface foi completamente **desinchaçada (anti-bloat)**: as caixas estáticas redundantes foram eliminadas da coluna direita, dedicando 100% da altura para a árvore hierárquica interativa, com novos submenus modais flutuantes (`[O]` Opções e `[C]` Pastas) e alternância de densidade compacta (`[V]`).

```text
┌──────────────────────── distro-vault :: [F2] 📦 NOVO BACKUP  •  [F3] 🔄 RECUPERAÇÃO ────────────────────────┐
│ ┌─ 📁 Snapshots Locais ──────────────┐ ┌─ 🗂️ Itens do Snapshot [V: Modo Compacto] ──────────────────────────┐ │
│ │ ▶ distro-vault-20261025.tar.zst    │ │   [x] ▼ Pacotes do Sistema (152 selecionados)                    │ │
│ │   distro-vault-subset.tar.zst      │ │       ├── [x] alacritty • Terminal Emulator                      │ │
│ │                                    │ │       └── [x] ripgrep • Fast search CLI                          │ │
│ │                                    │ │   [x] ▼ Chaves SSH (2 chaves)                                    │ │
│ │                                    │ │       ├── [x] 🔑 id_ed25519 (0600)                               │ │
│ │                                    │ │       └── [x] 📜 id_ed25519.pub (0644)                           │ │
│ │                                    │ │   [x] ▼ Configuração Global Git (1 arquivo)                      │ │
│ │                                    │ │   [x] ▼ Navegador Zen (3 spaces, 18 abas)                         │ │
│ │                                    │ │   [x] ▼ Diretórios Customizados (3 pastas)                       │ │
│ └────────────────────────────────────┘ └──────────────────────────────────────────────────────────────────┘ │
└────────── [↑/↓] Navegar Árvore  [Enter/→] Drop  [Espaço] Marcar  [Enter/R] RESTAURAR  [S] SUB-DUMP  [O] Opções  [C] Pastas  [V] Densidade ──┘
```

### Submenus Modais Flutuantes (Anti-Bloat Architecture)

Para evitar poluição visual e perda de área de visualização em terminais compactos, todos os seletores secundários foram desacoplados em janelas modais flutuantes centralizadas que utilizam o widget `Clear` do Ratatui para evitar vazamento de caracteres em background:

#### 1. Modal de Opções de Execução (`[O]`)
Pressione `[O]` no modo Restore para abrir o painel de parâmetros operacionais:
```text
┌────────────────────────────── ⚙️ Opções & Parâmetros de Execução ──────────────────────────────┐
│ Defina os parâmetros operacionais antes de iniciar a restauração ou sub-dump:                 │
│                                                                                               │
│   [P] Instalar Pacotes Nativos: [X] ATIVADO (pacman / flatpak / apt / dnf)                    │
│   [D] Simulação Dry-Run:       [ ] RESTAURAÇÃO REAL NO SISTEMA                                │
│   [V] Densidade da Árvore:    [ ] MODO EXPANDIDO (Detalhes & Sublabels)                       │
│                                                                                               │
│                     [P/D/V] Alternar Opções  •  [Esc/Enter/Q] Fechar Janela                   │
└───────────────────────────────────────────────────────────────────────────────────────────────┘
```

#### 2. Modal de Gerenciador de Diretórios & Configuração (`[C]`)
Pressione `[C]` no modo Restore para gerenciar caminhos persistidos em `~/.config/distro-vault/config.toml`:
```text
┌────────────────── 📁 Gerenciador de Diretórios (~/.config/distro-vault/config.toml) ──────────────────┐
│ ┌─ [1] Diretório Padrão de Saída (Dumps & Slices) [Tecla O p/ Explorador] ──────────────────────────┐ │
│ │   📁 /home/usuario/Antigravity/backups                                                             │ │
│ └────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌─ [2] Diretórios de Entrada para Varredura (2) [+/A: Adicionar | D/Del: Remover] ──────────────────┐ │
│ │  ▶ 📂 /home/usuario/Antigravity/backups                                                            │ │
│ │    📂 /mnt/storage/vault-dumps                                                                     │ │
│ └────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌─ [3] Densidade Padrão da Interface [Espaço para Alternar] ─────────────────────────────────────────┐ │
│ │    [X] MODO COMPACTO (Alta Densidade de Itens)                                                     │ │
│ └────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│ │                     ▶ [S / Enter] SALVAR E APLICAR PREFERÊNCIAS NO DISCO ◀                         │ │
│ └────────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│           [Tab/↑/↓] Campo  •  [O] Mudar Saída  •  [+/A] Add Entrada  •  [D/Del] Remover  •  [S] Salvar         │
└───────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Controles Universais da TUI

| Tecla | Contexto | Ação |
| :--- | :--- | :--- |
| `F2` / `F3` / `M` | Global | Alterna em tempo real entre o **Modo Backup** e o **Modo Recuperação**. |
| `Tab` | Modo Backup | Alterna entre as 8 abas de componentes do sistema. |
| `Tab` | Modo Recuperação | Alterna o foco entre a **Lista de Snapshots** (esquerda) e a **Árvore de Seleção** (direita). |
| `↑` / `↓` / `j` / `k` | Listas | Navegação vertical entre linhas, itens ou nós da árvore. |
| `Enter` / `→` / `←` / `e` | Árvore | Expande ou recolhe o menu drop da categoria selecionada. |
| `Espaço` / `x` | Listas / Árvore | Alterna o estado de seleção (`[x]` ↔ `[ ]`) do item ou da categoria em lote. |
| `O` | Modo Recuperação | Abre o **Modal de Parâmetros de Execução** (`[P]` Pacotes, `[D]` Dry-Run, `[V]` Densidade). |
| `C` | Modo Recuperação | Abre o **Modal de Gerenciamento de Pastas** (configuração de diretórios de entrada e saída). |
| `V` | Modo Recuperação | Alterna instantaneamente a **Densidade Visual** (Modo Compacto de Alta Densidade vs Expandido). |
| `A` | Backup / Restore | Abre o **Explorador Unificado de Arquivos** para adicionar pastas ou carregar dumps. |
| `R` / `F10` | Modo Recuperação | Executa a restauração completa dos componentes selecionados no sistema. |
| `S` | Modo Recuperação | **Salva um novo Sub-Dump (`.tar.zst`)** contendo exclusivamente os itens marcados na pasta de saída. |
| `Enter` / `F10` | Modo Backup | Inicia a criação do snapshot declarativo no diretório de saída configurado. |
| `Esc` / `Q` | Global | Fecha modais abertos ou encerra a aplicação com restauração limpa do terminal. |

---

## ⚙️ Arquivo de Configuração Declarativa (`config.toml`)

O `distro-vault` persiste automaticamente as preferências do usuário em `~/.config/distro-vault/config.toml` com permissões seguras POSIX `0644`.

Exemplo de configuração gerada:

```toml
# Diretório padrão onde novos snapshots e sub-dumps derivados serão gravados
output_dir = "/home/usuario/Antigravity/backups"

# Lista de diretórios locais varridos automaticamente em busca de snapshots existentes
input_dirs = [
    "/home/usuario/Antigravity/backups",
    "/home/usuario/backups",
    "/mnt/storage/vault-dumps"
]

# Reinstalação de pacotes ativada por padrão no modo de restauração
auto_install_packages = true

# Densidade da árvore na interface TUI (true = compacto sem subtítulos, false = detalhado)
compact_view = false

# Raiz opcional para staging acelerado
staging_root = "/home/usuario/.cache/distro-vault/staging"
```

---

## ⌨️ Referência Completa da Linha de Comando (CLI)

O `distro-vault` pode ser operado de forma totalmente não-interativa e automatizada em scripts via Clap 4.5.

### 1. Criar Snapshot (`backup`)

```bash
# Iniciar TUI interativa para seleção e personalização
distro-vault backup -i

# Snapshot padrão não-interativo salvo em ~/Antigravity/backups/
distro-vault backup

# Definir caminho do arquivo e adicionar diretórios extras
distro-vault backup -o /mnt/external/meu-backup.tar.zst \
  --add-dir ~/Projetos/distro-vault \
  --add-dir ~/.config/nvim

# Excluir pacotes, chaves SSH ou skills específicas
distro-vault backup \
  --exclude-package steam \
  --exclude-package discord \
  --no-ssh \
  --dry-run
```

### 2. Restaurar Sistema (`restore`)

```bash
# Iniciar diretamente no Modo Recuperação da TUI
distro-vault restore

# Restaurar arquivo específico instalando pacotes de sistema
distro-vault restore /caminho/do/snapshot.tar.zst --install-packages

# Aceleração instantânea via staging em cache
distro-vault restore /caminho/do/snapshot.tar.zst \
  --from-staging ~/.cache/distro-vault/staging/meu-snapshot/

# Simulação (Dry-Run) sem modificar o disco ou pacotes
distro-vault restore /caminho/do/snapshot.tar.zst --dry-run
```

### 3. Fatiar e Gerar Sub-Dumps (`slice`)

```bash
# Extrair somente o que for necessário para gerar um novo snapshot derivado
distro-vault slice /backups/snapshot-completo.tar.zst \
  -o /tmp/snapshot-leve.tar.zst \
  --no-ssh \
  --no-zen \
  --exclude-package libreoffice

# Abrir o dump na TUI para seleção visual e exportação
distro-vault slice /backups/snapshot-completo.tar.zst -i
```

### 4. Inspecionar Snapshot (`inspect`)

```bash
# Inspeção rápida em milissegundos sem descompactar
distro-vault inspect /backups/distro-vault-20261024.tar.zst

# Saída estruturada em JSON para pipelines e scripts
distro-vault inspect /backups/distro-vault-20261024.tar.zst --json
```

### 5. Validar Integridade (`verify`)

```bash
# Validação criptográfica do hash SHA-256 e teste de blocos Zstd
distro-vault verify /backups/distro-vault-20261024.tar.zst
```

---

## 🏗️ Arquitetura e Estrutura de Crates

O projeto adota rigorosamente a **Clean Architecture (Hexagonal)** com dependências apontando estritamente para dentro:

```mermaid
graph TD
    subgraph Driving["Driving Adapters (Apresentação)"]
        CLI["adapter-cli (Clap 4.5, Indicatif, Console)"]
        TUI["TUI Interativa (Ratatui 0.30, Crossterm 0.29)"]
        TREE["RestoreTreeState (Menus Drop & Checkboxes)"]
        EXPL["FileExplorerState (Navegador Modal)"]
        MODALS["OptionsModalState & ConfigModalState"]
        CLI --- TUI
        TUI --- TREE
        TUI --- EXPL
        TUI --- MODALS
    end

    subgraph App["Application Layer (Casos de Uso)"]
        BACKUP["BackupPipelineUseCase"]
        RESTORE["RestorePipelineUseCase"]
        SLICE["SliceSnapshotUseCase"]
        INSPECT["InspectSnapshotUseCase"]
        VERIFY["VerifyIntegrityUseCase"]
    end

    subgraph Core["Core Domain Layer (Regras Puras - Zero I/O)"]
        ENTITIES["Entities (Manifest, ZenProfile, AgentGovernance, SshKeyEntry, GitConfigEntry, VaultConfig)"]
        VO["Value Objects (Checksum, DistroFamily, UnixMode, PackageName, SkillCategory)"]
        PORTS["Ports Traits (BrowserPort, StoragePort, SystemPort, SshPort, GitConfigPort, ConfigPort, PackagePort)"]
    end

    subgraph Driven["Driven Adapters (Infraestrutura)"]
        SYS["adapter-system (Pacman, Flatpak, DNF, APT, SSH, Git, TomlConfigAdapter)"]
        BRW["adapter-browser (Zen mozLz4 codec, CityHash64 normalizer)"]
        AGT["adapter-agents (Agent Governance, symlink dereferencer)"]
        STR["adapter-storage (Tar, Zstd, ZipSlipGuard, CustomDirsHandler)"]
    end

    CLI --> App
    App --> Core
    SYS --> Core
    BRW --> Core
    AGT --> Core
    STR --> Core
```

| Crate | Responsabilidade |
| :--- | :--- |
| `crates/domain` | Entidades puras (`VaultConfig`, `Manifest`, `SelectionFilter`), Value Objects, contratos de Portas (`ConfigPort`, `StoragePort`, `BrowserPort`) e erros tipados. **Zero I/O e zero dependências de infraestrutura.** |
| `crates/application` | Casos de uso e orquestração dos fluxos de Backup, Restore, Slice, Inspect e Verify. |
| `crates/adapter-browser` | Codec Mozilla LZ4 nativo (`mozLz40\0`) e normalizador CityHash64 de `profiles.ini` e `installs.ini`. |
| `crates/adapter-system` | Integração com o sistema operacional, `/etc/os-release`, monitor `/proc`, chaves SSH, identidade Git global, gerenciadores de pacotes e persistência declarativa TOML (`TomlConfigAdapter`). |
| `crates/adapter-agents` | Scanner de regras de IA (`AGENTS.md`) e 19 skills com desreferenciamento de symlinks para independência de ambiente. |
| `crates/adapter-storage` | Motor de compactação e streaming `.tar.zst` com sanitização Zip-Slip e preservação de permissões POSIX. |
| `crates/adapter-cli` | Composition Root, CLI com Clap 4.5 e interface TUI rica em Ratatui com arquitetura desinchaçada e modais flutuantes. |

---

## 🛡️ Segurança e Hardening

| Vetor de Risco | Mitigação Implementada |
| :--- | :--- |
| **Zip-Slip / Path Traversal (CWE-22)** | O `ZipSlipGuard` valida e rejeita caminhos com `..`, prefixos absolutos, escapes para fora da raiz canônica de extração e symlinks ancestrais. |
| **Decompression Bomb (CWE-409)** | O codec `MozLz4Codec` impõe teto de segurança rigoroso de 128 MB antes da alocação de memória na descompactação de sessões. |
| **Command Injection (CWE-78)** | Execução de gerenciadores de pacotes via `execve` direto (sem subshell `sh -c`) com validação de regex nos nomes via Value Object `PackageName`. |
| **Exposição de Credenciais (CWE-732)** | Imposição de permissões POSIX estritas `0700` para diretórios sensíveis (`~/.ssh`) e `0600` para chaves privadas SSH. |
| **Symlink Traversal (CWE-59)** | Desreferenciamento seguro na exportação de skills e verificação defensiva de alvos de symlink na restauração. |

---

## 💡 Cenários Práticos de Uso

### 1. Migração Cross-Distro Completa (Arch Linux ➔ Fedora / Ubuntu)
```bash
# Na máquina de origem (Arch Linux): crie um snapshot completo
distro-vault backup -o ~/meu-sistema.tar.zst

# Na nova máquina (Fedora): copie o arquivo e inicie a restauração
distro-vault restore ~/meu-sistema.tar.zst --install-packages
```
*O algoritmo interno remapeia `/home/usuario_antigo` para `/home/novo_usuario`, regenera os hashes CityHash64 do Zen Browser e reinstala os pacotes desktop equivalentes via Flatpak / DNF.*

### 2. Slicing e Compartilhamento Seguro (Remoção de Segredos)
Precisa compartilhar suas configurações de desenvolvimento ou dotfiles sem expor chaves privadas SSH e sessões bancárias do navegador?
```bash
# Na TUI: Abra o dump com `distro-vault restore`, desmarque Chaves SSH e Zen Browser e aperte [S]
# Ou via CLI de forma atômica e automatizada:
distro-vault slice /backups/snapshot-completo.tar.zst \
  -o ~/dotfiles-compartilhaveis.tar.zst \
  --no-ssh \
  --no-zen
```

### 3. Armazenamento em Múltiplos Discos / SSD Externo
Configure seu SSD externo ou partição de dados para ser detectado automaticamente:
1. Abra a TUI com `distro-vault` e pressione `[C]` (Pastas).
2. Pressione `[O]` para selecionar seu drive externo como `output_dir`.
3. Pressione `[+]` e aponte para pastas antigas de backup para adicionar em `input_dirs`.
4. Pressione `[S]` para salvar em `~/.config/distro-vault/config.toml`. Pronto!

---

## 🗺️ Roadmap de Evolução

- [x] **v1.0.0 (Lançamento Oficial):**
  - Pipelines de backup/restore declarativo, codec nativo mozLz4, normalizador CityHash64 do Zen Browser, governança de agentes, subcomando `slice`, árvore hierárquica e explorador unificado de arquivos.
- [x] **v1.1.0 (Ergonomia & Personalização de I/O):**
  - Persistência em `~/.config/distro-vault/config.toml` via `TomlConfigAdapter`.
  - Desinchaço da TUI (anti-bloat) com 100% da altura da coluna dedicada à árvore hierárquica.
  - Submenu modal de parâmetros `[O]`, gerenciador modal de diretórios `[C]` e alternância de densidade `[V]`.
- [x] **v1.1.1 (Expansão de Documentação & Hardening):**
  - Reescrita completa da documentação com diagramas de arquitetura, mockups visuais da TUI e cenários passo a passo.
  - Auditoria de segurança pré-commit e validação global estrita.
- [ ] **v1.2.0 (Próximos Passos):**
  - Sincronização remota automatizada com backends SSH/SFTP e S3.
  - Agendador integrado via timers Systemd (`distro-vault schedule`).

---

## 🔨 Compilação e Testes

### Pré-requisitos
- Rust 1.75+ (Edição 2021)
- Cargo

```bash
# Compilar todo o workspace em modo debug
cargo check --workspace

# Executar a suíte completa de testes (43 testes unitários e de integração)
cargo test --workspace

# Validar conformidade estrita com Zero Warnings no Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Compilar binário standalone de alta performance otimizado
cargo build --release
```

---

## 📄 Licença

Distribuído sob licença dupla **MIT** ou **Apache-2.0**. Consulte [LICENSE](LICENSE) para mais detalhes.
