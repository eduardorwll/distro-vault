# Binário Pré-Compilado: Distro-Vault (x86_64 Linux)

Este diretório contém o executável pré-compilado em modo **Release** otimizado para distribuições Linux x86_64 (Arch, Omarchy, Fedora, Debian, Ubuntu, Pop!_OS, etc.).

## Instalação Rápida (Sem necessidade do Toolchain Rust)

Para instalar ou usar diretamente em qualquer máquina:

```bash
# Copiar para o diretório de binários locais do usuário:
cp bin/distro-vault ~/.local/bin/distro-vault
chmod +x ~/.local/bin/distro-vault

# Ou baixar diretamente do GitHub:
curl -fsSL https://raw.githubusercontent.com/eduardorwll/distro-vault/master/bin/distro-vault -o ~/.local/bin/distro-vault
chmod +x ~/.local/bin/distro-vault
```

## Como Usar

```bash
# Abrir TUI Interativa Ratatui:
distro-vault

# Gerar Snapshot Declarativo completo:
distro-vault backup

# Restaurar Snapshot em uma nova distribuição:
distro-vault restore
```
