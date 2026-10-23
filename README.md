# Distro-Vault 🦀📦

> Ferramenta de alta performance em Rust para backup declarativo, restauração e transição sem atrito entre distribuições Linux (Distro Hopping).

---

## Recursos Principais

- 🌐 **Zen Browser Preservation:** Extração e empacotamento atômico de Workspaces (Spaces), abas abertas e fixadas via codec nativo `mozLz40\0`, além de correção automática do hash de instalação (`installs.ini`/`profiles.ini`).
- 📦 **Multi-Distro Package Engine:** Gerenciamento declarativo para `pacman`, `aur` (`paru`/`yay`), `flatpak`, `dnf`, `apt` e `nix`.
- 🚀 **AppImage & Gear Lever Vault:** Preservação de binários portáteis em `~/AppImages` e garantia da biblioteca `FUSE 2` no host destino.
- 🤖 **Agent Ecosystem Sync:** Exportação declarativa de regras e das 67 skills de agentes de IA com desreferenciamento de symlinks para execução em qualquer SO.
- 🔒 **Secrets & POSIX Integrity:** Preservação estrita de modos `0700` e `0600` em chaves SSH/GPG e compactação Zstandard de alta velocidade.

---

## Arquitetura (Clean Architecture)

Construído como um **Cargo Workspace** modular:
* `crates/domain`: Entidades puras, value objects e portas (ports).
* `crates/application`: Casos de uso (`BackupPipeline`, `RestorePipeline`, `InspectSnapshot`).
* `crates/adapter-browser`: Codec Mozilla LZ4 nativo e normalizador de perfis.
* `crates/adapter-system`: Adaptador de gerenciadores de pacotes e `/etc/os-release`.
* `crates/adapter-agents`: Scanner de skills e regras de agentes.
* `crates/adapter-storage`: Snapshot engine com Tar e Zstandard.
* `crates/adapter-cli`: Interface CLI rica com `clap` e `indicatif`.

---

## Compilação e Testes

```bash
# Compilar todo o workspace
cargo check --workspace

# Executar testes unitários e de integração
cargo test --workspace

# Verificar conformidade de linter
cargo clippy --workspace -- -D warnings
```
