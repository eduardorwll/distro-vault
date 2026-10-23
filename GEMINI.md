# distro-vault — Diretrizes do Projeto para Agentes Gemini & Antigravity

Este arquivo define as regras mandatórias para agentes operando no repositório `distro-vault`.
Para especificações detalhadas de arquitetura, subagentes, guardrails e suíte de testes, consulte sempre [AGENTS.md](file:///home/eduardorwll/Projects/distro-vault/AGENTS.md).

---

## ⚡ Regra Mandatória: Eficiência Extrema de Tokens e Busca Estruturada

Para acelerar o desenvolvimento, reduzir a latência e eliminar o consumo desnecessário de tokens de contexto:

### 🚫 O QUE É ESTRITAMENTE PROIBIDO:
1. **Leitura Cega de Arquivos:** Nunca execute `read_file` em arquivos inteiros ou dezenas de arquivos sequencialmente para tentar entender fluxos ou encontrar definições.
2. **Grep Recursivo Descontrolado:** Nunca execute cascatas de `grep -rn` ou `find` na raiz do projeto para localizar funções, variáveis ou interfaces.

### 🎯 PROTOCOLO DE TRABALHO:
1. **Nível Macro:** Consulte a documentação viva:
   * `estado_atual.md`
   * `estado_arquitetural_atual.md`
   * `tasks/todo.md`
2. **Nível Micro:** Use leituras cirúrgicas com `StartLine` e `EndLine` para inspecionar apenas as assinaturas e linhas de código estritamente necessárias.

---

## 🛡️ Regras de Engenharia Rust (Zero Warning Policy)

1. **Clean Architecture Obrigatória:**
   - `crates/domain`: Somente regras de negócio puras, entidades, value objects e traits (ports). Zero dependência de I/O, SQLite ou CLI.
   - `crates/application`: Casos de uso e orquestração.
   - `crates/adapter-*`: Implementações concretas de portas.
2. **Qualidade e Linters:**
   - Todo código adicionado deve passar limpo em:
     ```bash
     cargo check --workspace
     cargo test --workspace
     cargo clippy --workspace -- -D warnings
     ```
   - Nenhum `.unwrap()` em código de produção dos adapters ou domain. Sempre propague erros tipados via `Result<T, DomainError>`.
