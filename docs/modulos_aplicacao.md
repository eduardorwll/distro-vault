# Arquitetura e Mapeamento Modular — distro-vault

Este documento descreve detalhadamente cada módulo, camada e adaptador do projeto **distro-vault**, organizado no padrão de **Clean Architecture Hexagonal** em Rust 2021 Workspace.

---

## 1. Camada de Interface & Apresentação (`crates/adapter-cli`)

Ponto de entrada do usuário. Captura comandos, gerencia o terminal interativo e emite os filtros de seleção e exclusão para a camada de aplicação.

### 1.1. CLI Parser (`clap 4.5`)
- **Subcomandos Disponíveis:**
  - `backup`: Inicia a rotina de criação de snapshot.
  - `restore`: Executa a extração, normalização do navegador e reinstalação declarativa.
  - `inspect`: Analisa o conteúdo de um arquivo `.tar.zst` existente sem descompactá-lo.
  - `verify`: Checa o hash SHA-256 e a integridade estrutural do arquivo.
- **Automação Declarativa:** Suporta flags de linha de comando (`--exclude-pkg`, `--add-dir`, `--json`, `--output`, `--non-interactive`) para uso em automações e cron.
- **Feedback Visual:** Integração com o crate `indicatif` para spinners animados e barras de progresso durante compressão e I/O intensivo.
- **Modo Headless / Não-interativo:** Permite agendamentos noturnos e migrações totalmente automatizadas via scripts bash.

### 1.2. TUI Interativa (`ratatui 0.30` + `crossterm`)
- **Navegação em Abas via `Tab`:**
  - `[1] Pacotes`: Lista pacotes nativos do Pacman, foreign do AUR e Flatpaks.
  - `[2] Skills`: Lista as 19 skills de governança de IA categorizadas.
  - `[3] Apps`: Lista binários e metadados gerenciados pelo Gear Lever em `~/AppImages`.
  - `[4] Pastas Extras`: Permite ao usuário incluir e excluir diretórios arbitrários.
  - `[5] Zen Spaces`: Visualização de workspaces, abas e containers abertos no Zen Browser.
- **Seleção Dinâmica com `Espaço`:** Marca e desmarca itens instantaneamente com renderização suave a 60 FPS no terminal.
- **Adição de Diretórios Customizados com `A`:** Abre um popup de input para o usuário digitar qualquer caminho (ex: `~/Documentos/estudos`) a ser empacotado.
- **Emissão do `SelectionFilter`:** Traduz o estado visual da tela no objeto de domínio que dita o que será empacotado e o que será descartado.

---

## 2. Camada de Aplicação & Casos de Uso (`crates/application`)

Coordena os fluxos de negócio da aplicação. Não possui dependências de bibliotecas de baixo nível de compressão ou detalhes de SO; comunica-se estritamente através das Portas Hexagonais.

### 2.1. `BackupPipelineUseCase`
- **Verificação de Trava:** Consulta `BrowserPort::is_locked` para checar `.parentlock` e aciona `SystemPort` para término gracioso via SIGTERM se necessário.
- **Agregação do Inventário:** Varre o Zen Browser (`BrowserPort`), pacotes instalados (`PackagePort`), governança de agentes (`AgentPort`) e pastas customizadas.
- **Filtragem Declarativa:** Aplica o `SelectionFilter`, expurgando itens que o usuário optou por desmarcar na CLI ou TUI.
- **Gravação Atômica:** Monta o `Manifest`, aciona `StoragePort::create_snapshot` e calcula o checksum SHA-256 do arquivo `.tar.zst` final.

### 2.2. `RestorePipelineUseCase`
- **Validação e Extração:** Lê o manifesto e descompacta os arquivos utilizando `StoragePort::extract_snapshot` com proteção ativa contra Zip-Slip.
- **Normalização do Zen Browser:** Restaura a árvore de perfis e invoca `BrowserPort::normalize_installation_hash`, reescrevendo `installs.ini` e `profiles.ini` com CityHash64 para garantir que o Zen abra o perfil restaurado imediatamente como `Default=1`.
- **Restauração de Agentes:** Copia as skills e regras de volta para `~/.agents/`, já desreferenciadas fisicamente.
- **Despacho Inteligente de Pacotes:**
  - *Mesma Distribuição (Arch -> Arch):* Reinstala pacotes nativos, AUR e Flatpaks em lote.
  - *Distro-Hopping (Arch -> Fedora/Debian):* Instala Flatpaks diretamente e emite um relatório com a lista mapeada de pacotes equivalentes.
- **Checagem de FUSE2:** Invoca `SystemPort::is_fuse2_available` para alertar ou orientar a instalação da biblioteca `libfuse2` para AppImages na nova distro.
- **Restauração de Pastas Customizadas:** Restaura os diretórios armazenados sob `custom/<slug>` nos seus locais de origem.

### 2.3. `InspectSnapshotUseCase`
- **Leitura em Streaming:** Descompacta apenas o início do stream Zstandard para extrair o `manifest.json`.
- **Relatório Estruturado:** Exibe contagem de pacotes, espaços do Zen Browser, abas, skills de IA e pastas extras sem precisar descompactar gigabytes de dados.

### 2.4. `VerifyIntegrityUseCase`
- **Validação Criptográfica:** Calcula o hash SHA-256 de todo o arquivo `.tar.zst` e compara com o valor registrado no cabeçalho ou metadados.
- **Teste de Blocos:** Valida se a stream zstd e a estrutura tar não sofreram corrupção física ou truncamento de bytes.

---

## 3. Camada de Núcleo de Domínio (`crates/domain`)

O centro do sistema. Não possui dependências de I/O, rede, filesystem ou sistema operacional. Contém entidades puras, Value Objects e os traits que definem as portas do hexágono.

### 3.1. Entidades de Domínio
- **`Manifest`:** Registro declarativo completo do snapshot (versão, timestamp UTC, host de origem, distro de origem, checksum, inventário).
- **`ZenProfile`:** Representação detalhada de perfis do Zen Browser com coleções de `Space`, `Container`, abas e flags de dados sensíveis (senhas e favoritos).
- **`PackageEntry`:** Entidade unificada para softwares do Pacman, AUR, Flatpak, DNF e APT.
- **`AppImageEntry`:** Metadados de executáveis AppImage e dependência de runtime FUSE2.
- **`AgentGovernance`:** Entidade que modela skills, regras ([`GEMINI.md`](/home/eduardorwll/GEMINI.md), [`AGENTS.md`](/home/eduardorwll/AGENTS.md)) e os 5 Quality Gates de IA.
- **`CustomDirectoryEntry`:** Metadados de pastas arbitrárias do usuário com contagem de arquivos e volume em bytes.
- **`SelectionFilter`:** Conjunto de regras de inclusão/exclusão gerado pela TUI ou argumentos de CLI.

### 3.2. Value Objects Fortemente Tipados
- **`ProfileId`:** Identificador tipado e não-vazio de perfil do navegador (ex: `"7awtlvx8.Default (release)"`).
- **`PackageName`:** Identificador único de pacote sanitizado.
- **`DistroFamily`:** Enum com inferência inteligente (`Arch`, `Fedora`, `Debian`, `OpenSuse`, `Unknown`).
- **`Checksum`:** String hexadecimal estrita de 64 caracteres representando um hash SHA-256.
- **`UnixMode`:** Permissões POSIX com helpers de segurança (`SECURE_DIR = 0o700`, `SECURE_PRIVATE_KEY = 0o600`).
- **`SkillCategory`:** Categorização semântica das skills (`DesignPattern`, `Architecture`, `Security`, `Testing`, `Process`, `General`).

### 3.3. Portas Hexagonais (Interfaces / Traits)
- **`BrowserPort`:** `is_locked`, `scan_profiles`, `decompress_session`, `compress_session`, `normalize_installation_hash`, `export_profile_state`, `restore_profile_state`.
- **`StoragePort`:** `create_snapshot`, `read_manifest`, `extract_snapshot`, `compute_checksum`.
- **`AgentPort`:** `scan_governance`, `export_governance`, `restore_governance`.
- **`PackagePort`:** `source`, `query_installed`, `install_batch`, `is_available`.
- **`SystemPort`:** `detect_distro`, `hostname`, `is_process_running`, `terminate_process_gracefully`, `is_fuse2_available`.
- **`DomainError`:** Enum central de erros tipados sem `panic!` ou `.unwrap()` em runtime.

---

## 4. Camada de Adaptadores de Infraestrutura (`crates/adapter-*`)

Implementações concretas profundas (*Deep Modules*) que realizam o trabalho pesado de I/O, manipulação de arquivos e chamadas ao sistema operacional.

### 4.1. `adapter-browser` (Zen Browser / Mozilla Engine)
- **`MozLz4Codec`:**
  - Processa o formato binário proprietário da Mozilla: magic header de 8 bytes `b"mozLz40\0"` seguido de tamanho descomprimido Little-Endian em `u32` (4 bytes).
  - Descompressão e compressão em blocos LZ4 raw via `lz4_flex`.
  - **Bomb Guard:** Rejeita preventivamente qualquer payload que declare tamanho descomprimido superior a 128 MB.
- **`ProfilesIniNormalizer`:**
  - Implementação nativa e pura de **Google CityHash64 v1** sobre UTF-16LE para geração precisa dos hashes de diretórios de instalação do Mozilla.
  - Mecanismo *Sweep & Bind*: localiza todas as seções `[Install<HASH>]` em `profiles.ini` e `[<HASH>]` em `installs.ini`, apontando-as para `Default=<perfil_restaurado>` e `Locked=1`.
  - Garante `[General] StartWithLastProfile=1` e `[ProfileX] Default=1`.
- **`ZenBrowserAdapter`:**
  - Extração de metadados de Workspaces/Spaces, Containers e abas diretamente do JSON descomprimido.
  - Detecção da trava `.parentlock`.
  - Cópia recursiva do perfil ignorando caches efêmeros (`cache2`, `startupCache`, `thumbnails`).

### 4.2. `adapter-storage` (Armazenamento, Zstandard & Tarball)
- **`TarZstdPacker`:**
  - Criação streaming de arquivos `.tar.zst` com compressão Zstandard nível 3 (equilíbrio ideal entre taxa de compressão e velocidade).
  - Posicionamento obrigatório do `manifest.json` como primeira entrada do arquivo para leitura sem overhead.
- **`ZipSlipGuard`:**
  - Mecanismo rigoroso de sanitização que impede ataques de directory traversal.
  - Rejeição categórica de componentes `..`, caminhos com raízes absolutas e symlinks maliciosos que apontem para fora da raiz alvo.
  - Bloqueio de arquivos de dispositivos de bloco (`EntryType::Block`), caracteres e FIFOs.
- **`CustomDirsHandler`:**
  - Varredura e cálculo estatístico de pastas customizadas do usuário.
  - Mapeamento e isolamento dentro do tarball sob o namespace seguro `custom/<slug>`.
- **`ZstdStorageAdapter`:**
  - Implementação completa do trait `StoragePort` com cálculo SHA-256 via `sha2`.

### 4.3. `adapter-agents` (Governança e Skills de IA)
- **`AgentScanner`:**
  - Varredura de [`~/.agents/skills/`](/home/eduardorwll/.agents/skills) e leitura de arquivos de regras ([`GEMINI.md`](/home/eduardorwll/GEMINI.md), [`AGENTS.md`](/home/eduardorwll/AGENTS.md)).
  - Parsing de frontmatter YAML contido nos arquivos `SKILL.md` para extração de descrições e categorização automática.
- **Desreferenciador de Links Simbólicos (`copy_dereferenced`):**
  - Detecta se uma skill é um link simbólico apontando para `/usr/share/omarchy/` (como as skills `omarchy` e `diagnose-crash`).
  - Resolve o link e realiza uma cópia física profunda de todos os arquivos, garantindo que o backup seja 100% autônomo e não quebre caso o usuário restaure o sistema em distribuições não-Omarchy (Fedora, Debian, Ubuntu, openSUSE).
- **`LocalAgentAdapter`:**
  - Implementação de `AgentPort` para exportação e restauração dos diretórios de agentes.

### 4.4. `adapter-system` (Sistema Operacional e Gerenciador de Pacotes)
- **`OsReleaseParser`:**
  - Leitura e parsing estrito de `/etc/os-release`, extraindo campos `ID` e `ID_LIKE` para normalização em `DistroFamily`.
  - Leitura de hostname do sistema e detecção da presença física de `libfuse.so.2` em múltiplos diretórios de biblioteca.
- **`ProcessManager`:**
  - Varredura em `/proc/*/comm` para detecção de processos do Zen Browser em execução.
  - Sinalização graciosa via `SIGTERM` e verificação de encerramento em loop com timeout configurável.
- **Dispatchers de Pacotes:**
  - **`PacmanAdapter`:** Consulta pacotes explícitos nativos com `pacman -Qne` e instala em lote via `pacman -S --needed --noconfirm`.
  - **`AurAdapter`:** Identifica automaticamente se o sistema possui `paru` ou `yay`, consulta pacotes foreign com `<helper> -Qme` e instala em lote.
  - **`FlatpakAdapter`:** Consulta aplicações instaladas via `flatpak list --app` e instala pacotes faltantes via `flatpak install -y flathub`.
