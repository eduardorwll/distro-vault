# Diretrizes de Engenharia e Regras de Agentes — distro-vault

Este repositório é uma aplicação em **Rust 2021** (Cargo Workspace) para backup e restauração declarativa de sistemas Linux, navegadores (Zen Browser com parsing nativo de `mozLz4`), AppImages, pacotes de distribuição e agentes de IA.

---

## Regra Obrigatória: Aprovação Prévia de Modificações (Mandatory Pre-Approval)

**ATENÇÃO (Mandatório para todos os agentes de IA no projeto):**
Antes de executar qualquer prompt, rodar comandos que modifiquem o repositório ou fazer qualquer alteração no projeto (criar, editar ou apagar arquivos de código, configurações, crates, schemas ou documentação), o agente DEVE obrigatoriamente:
1. Limitar-se a operações de leitura e análise prévia (inspeção de arquivos, status, contexto).
2. Escrever e apresentar ao usuário uma **lista clara, concisa e estruturada** exatamente com o que será feito e quais arquivos/estruturas serão alterados.
3. **Aguardar a aprovação explícita do usuário** antes de aplicar qualquer alteração ou executar qualquer ação que modifique o estado do projeto.

---

## Regra Obrigatória: Protocolo de Alterações de Código e Rotina de Engenharia (Code Change & Engineering Gate)

**ATENÇÃO (Mandatório e Global para todos os agentes em chamadas com alteração de código):**
Em qualquer interação onde o usuário solicitar ou mencionar alterações em código (criar, editar, refatorar, apagar arquivos, crates, dependências, testes ou documentação), o agente DEVE seguir estritamente o seguinte ciclo de 5 fases:

### 1. Fase 1: Levantamento e Diagnóstico Inicial (Apenas Leitura)
- O agente analisa arquivos, logs de compilação, tipos, traits e dependências.
- Apresenta um resumo técnico conciso do diagnóstico e a abordagem preliminar.

### 2. Fase 2: Pergunta Obrigatória de Gatilho de Engenharia
- Antes de propor ou redigir qualquer diff ou código, o agente DEVE OBRIGATORIAMENTE perguntar:
  > *"Deseja rodar a rotina completa de engenharia com skills para escalabilidade, refatoração e checklist estruturado de pré-lançamento antes de prosseguir?"*

### 3. Fase 3: Execução da Rotina de Engenharia (quando confirmada)
- Submeter o plano às skills de engenharia do projeto:
  - `clean-architecture` (separação estrita: Core Domain -> Use Cases -> Adapters; dependências apontando para dentro)
  - `a-philosophy-of-software-design` (módulos profundos, interfaces enxutas, "define errors away")
  - `clean-code` (nomes reveladores de intenção, responsabilidade única, Newtype pattern)
  - `doubt-driven-development` (escrutínio adversarial, modos de falha ocultos em I/O e compressão)
  - `security-and-hardening` (permissões POSIX 0700/0600 em chaves, integridade SHA-256)
  - `release-it` (resiliência a corrupção de sessão, timeouts e recuperação)
  - `test-driven-development` (testes de unidade de domínio com mocks antes da infraestrutura real)
  - `code-review-and-quality` (inspeção de padrões, clippy, ausência de unwrap em código de produção)
  - `planning-and-task-breakdown` (elaboração ou atualização do `tasks/todo.md`)
- **Obrigatoriedade de Checklist de Tarefas (`planning-and-task-breakdown`):**
  Formular e manter sincronizado o checklist granular em `tasks/todo.md` contendo descrição, critérios de aceitação específicos, comandos de verificação, dependências e arquivos tocados.

### 4. Fase 4: Aprovação Prévia Obrigatória
- Apresentar a lista exata e atômica de arquivos a serem modificados e o checklist de tarefas a ser executado (`tasks/todo.md`).
- Aguardar o "sim" / aprovação explícita do usuário antes de tocar em qualquer arquivo do repositório.

### 5. Fase 5: Conclusão, Validação e Roteiro de Testes Obrigatório
- Executar a suíte de validação:
  ```bash
  cargo check --workspace
  cargo test --workspace
  cargo clippy --workspace -- -D warnings
  ```
- Atualizar a documentação viva (`estado_atual.md`, `estado_arquitetural_atual.md`, `README.md`, `tasks/todo.md`).
- Apresentar obrigatoriamente na resposta final o **Roteiro Estruturado de Testes Funcionais** para homologação pelo usuário.

---

## Regra Obrigatória: Paradigma de Orquestração com Subagentes Especializados

Para preservar a integridade da janela de contexto do agente principal e garantir máximo rigor técnico em tarefas complexas, o projeto adota formalmente a topologia multiagente com segregação de responsabilidades:

1. **Agente Orquestrador (Lead / Parent Agent):**
   - **Papel:** Atua como o Arquiteto Principal e interlocutor direto do usuário.
   - **Responsabilidades:** Gerencia a conversa, conduz com rigor o protocolo das 5 Fases de Engenharia, apresenta diagnósticos preliminares e checklists estruturados, recolhe a aprovação explícita e entrega ao final o Roteiro Estruturado de Testes Funcionais.
   - **Preservação de Contexto:** Nunca deve saturar seu contexto com leituras exaustivas de arquivos ou compilações extensas; delega a pesquisa ao `DeepInvestigator` e a codificação ao `DeepCoder`.

2. **Subagente Investigador (`DeepInvestigator`):**
   - **Invocação:** Invocado via `invoke_subagent` com tipo `research`.
   - **Responsabilidades:** Mapeia código, busca referências, pesquisa padrões, crates ou documentação, diagnostica falhas de tipos ou testes e sintetiza relatórios concisos para o Orquestrador.

3. **Subagente Implementador (`DeepCoder`):**
   - **Invocação:** Invocado com tipo `self` ou com permissões completas de escrita e compilação.
   - **Responsabilidades:** Implementa código cirúrgico respeitando os padrões de Clean Architecture, cria testes unitários, roda `cargo check` e `cargo test`, corrige avisos do `clippy` e reporta o diff exato.

---

## Regra Obrigatória: Roteiro Estruturado de Testes Funcionais

Ao concluir qualquer implementação ou refatoração, o agente DEVE OBRIGATORIAMENTE fornecer um roteiro de testes no seguinte formato:

```markdown
### 🧪 Roteiro Estruturado de Testes Funcionais

#### Pré-requisitos:
- [Comandos de build ou dependências necessárias]

#### Cenário de Teste 1: [Nome do Caso]
1. **Ação:** [Passo a passo com comando exato]
2. **Resultado Esperado:** [Saída esperada ou mudança no filesystem]
3. **Critério de Aceite:** [Condição estrita de validação]
```

---

## Regra Obrigatória: Manutenção da Documentação Viva (Mandatory Upkeep)

- Documentos de estado:
  - `estado_atual.md`: Snapshot rápido do estado atual do repositório, crates e tarefas imediatas.
  - `estado_arquitetural_atual.md`: Arquitetura completa, diagramas, entidades, contratos e decisões de design.
  - `tasks/todo.md`: Checklist granular rastreável.
  - `README.md`: Documentação de instalação e uso da ferramenta.
- Antes de iniciar qualquer tarefa, leia `estado_atual.md` e `estado_arquitetural_atual.md`.
- Após QUALQUER modificação que altere comportamento, crates, dependências ou tarefas, atualize esses arquivos no mesmo commit/etapa.
