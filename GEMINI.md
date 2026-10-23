# distro-vault — Diretrizes do Projeto para Agentes Gemini & Antigravity

Este arquivo define as regras mandatórias para agentes operando no repositório `distro-vault`.
Para especificações detalhadas de arquitetura, subagentes, guardrails e suíte de testes, consulte sempre [AGENTS.md](file:///home/eduardorwll/Projects/distro-vault/AGENTS.md).

---

## ⚡ Regra Mandatória: Eficiência Extrema de Tokens e Busca em Grafos (Graphify & Graft First)

Para acelerar o desenvolvimento, reduzir a latência e eliminar o consumo desnecessário de tokens de contexto, **TODO agente operando neste projeto DEVE seguir o protocolo de 2 níveis em grafos antes de ler arquivos inteiros ou executar buscas brutas:**

### 🚫 O QUE É ESTRITAMENTE PROIBIDO:
1. **Leitura Cega de Arquivos:** Nunca execute `read_file` em arquivos inteiros ou dezenas de arquivos sequencialmente para tentar entender fluxos ou encontrar definições.
2. **Grep Recursivo Descontrolado:** Nunca execute cascatas de `grep -rn` ou `find` na raiz do projeto para localizar funções, variáveis ou interfaces.

---

### 🎯 PROTOCOLO DE BUSCA EM 2 NÍVEIS (MACRO ➔ MICRO):

#### 1. Nível Macro — Arquitetura, Subsistemas e Relações ([Graphify](file:///home/eduardorwll/.gemini/config/skills/graphify/SKILL.md))
Para entender a floresta, o fluxo de dados entre camadas e as conexões entre subsistemas:
* **Consultar contexto relacional:**
  ```bash
  graphify query "<pergunta sobre fluxo/subsistema>"
  ```
* **Mapear menor caminho entre dois componentes:**
  ```bash
  graphify path "ComponenteA" "ComponenteB"
  ```
* **Explicar um conceito ou módulo:**
  ```bash
  graphify explain "<ModuloOuConceito>"
  ```

#### 2. Nível Micro — Localização Cirúrgica de Linha e Símbolo ([Graft](file:///home/eduardorwll/.agents/skills/graft/SKILL.md))
Para localizar a linha exata de código, interfaces, classes ou chamadores:
* **Entender ou localizar código rapidamente (com código inlined no resultado):**
  ```bash
  graft ask "<sua pergunta ou nome do símbolo>" --source
  ```
* **Busca exata de símbolos e usos (agrupada por função/módulo):**
  ```bash
  graft grep "<nome_do_simbolo>"
  ```
* **Inspecionar a API/assinaturas de um arquivo sem gastar tokens com o corpo:**
  ```bash
  graft skeleton <caminho/do/arquivo>
  ```
* **Mapear quem chama ou depende de uma função antes de alterar:**
  ```bash
  graft callers <nome_da_funcao>
  ```

Se for estritamente necessário abrir um arquivo com `read_file`, **use sempre o `StartLine` e `EndLine`** apontados pelo Graft para inspecionar apenas as linhas relevantes.

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
     cargo clippy --all-targets --workspace -- -D warnings
     ```
   - Nenhum `.unwrap()` em código de produção dos adapters ou domain. Sempre propague erros tipados via `Result<T, DomainError>`.
3. **Conformidade com os Livros Oficiais de Rust (`rust-api-guidelines`):**
   - Todas as estruturas públicas, métodos de conversão (`as_`, `to_`, `into_`), traits comuns (`Debug`, `Clone`, `Eq`, `Hash`, `Send`, `Sync`) e ausência de clones desnecessários devem seguir estritamente o `rust-api-guidelines` e o *The Rust Programming Language*.
