# SpriteLab: investigação e direção arquitetural

13 de outubro de 2026. Entrega: pesquisa com REA, mapa de arquitetura do Aseprite e primeira base executável original do SpriteLab. O editor desktop completo ainda não foi implementado.

## Decisão principal

O **Command Engine em Rust é o coração**. MCP, CLI, Python e a futura interface são meios de executar operações desse engine. O código entregue já permite remover o adaptador MCP e continuar criando, desenhando, animando, salvando e exportando pela CLI.

A expansão para AssetForge deve compartilhar contratos, registro de comandos, serviços de sessão/transação/histórico, jobs, persistência e catálogo. Pixel art e 3D terão modelos e kernels próprios. Uma malha 3D não deve ser encaixada numa estrutura de cels. Nesta entrega, os contratos já estão separados do domínio raster; host genérico de múltiplos domínios, plugins e catálogo ainda são arquitetura futura.

## O que foi feito com o REA

O REA foi realmente instalado e executado pela CLI, dentro de `work/rea-runtime`, sem alterar a configuração global do Codex. Versão npm utilizada: **6.4.0**. O checkout de referência do REA ficou fixado em `43738bec579fc69bab084bffa964916d43d43ed1`. O próprio [workflow do REA](https://github.com/morluto/rea/blob/43738bec579fc69bab084bffa964916d43d43ed1/.agents/skills/reverse-engineer-anything/SKILL.md) recomenda análise normal de repositório para código-fonte completo; REA foi aplicado ao pacote e à observação do aplicativo distribuído.

Foi baixada a avaliação oficial **Aseprite 1.3.18.6 para macOS**, montada somente para leitura. A [página oficial de avaliação](https://www.aseprite.org/trial/) informa a limitação de salvamento. Não houve descompilação, disassembly, contorno de licença ou cópia da implementação. A escolha preserva a implementação independente exigida pela proposta e considera a [EULA publicada](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/EULA.txt).

| Investigação | Evidência | Conclusão e limite |
|---|---|---|
| Inventário do pacote `.app` | `ev_334e08a4a2bb10872fffc86421d06040059b2b8043e4169a60c1a9cf0efec337`; inventário interno `ev_1c2effebafb1285d4e3dc28cb958a345880a25d5acd6205414543ce1685de346` | 254 nós e 255 ocorrências, com hashes. Inventário estático não revela execução nem prova toda a arquitetura. |
| Metadados Info.plist | `ev_2c3af6a9171c18c7726f96634a05fe595f264929b72f5eaff4406fbc3cabecb5` | Identidade `org.aseprite.Aseprite`, executável `aseprite`, versão declarada 1.3.18.6 e associações de formatos. Declaração não prova fidelidade de importação. |
| Execução de `--version` | `ev_d21525e9acf7dda09e0fff355da383db2e467c58c6c47dfa46c5908115f80f6b` | Execução retornou 0 e identificou `1.3.18.6-arm64-trial`; a saída também confirmou restrição de salvamento. Nenhum arquivo foi selecionado para observação; efeitos de filesystem permanecem desconhecidos. |
| Execução de `--help` | `rea-aseprite-help.json`, resposta de erro com observação parcial preservada | O processo observado retornou 0 e a saída listou CLI headless/atlas/padding/metadados. O REA terminou com `cleanup_incomplete`, pois não conseguiu confirmar ownership de processos no macOS. Esta captura **não é um registro completo bem-sucedido** e não recebeu Evidence ID de sucesso. |

Os registros originais e um índice estão em [evidence/](evidence/index.json). O executável observado foi vinculado ao SHA-256 `7d427b9022f9934df7f02e4af62daff7ee8893960e5af0fb43ce23fe5416b440`. Os cenários e as fontes permanecem na pasta de trabalho; o Aseprite não é dependência do SpriteLab.

A tentativa inicial de inspecionar o DMG diretamente encontrou limites de montagem da sandbox e o diálogo de licença. A montagem explícita somente para leitura permitiu concluir inventário e leitura de plist. A captura de `--version` ficou completa. O erro de limpeza de `--help` foi preservado e declarado, sem convertê-lo em prova de sucesso ou repetir indefinidamente a mesma operação.

## O que aprendemos com o Aseprite

A referência de código foi fixada em **`14fe117e853bd642672e72018166239506060cd4`**. O [mapa detalhado](aseprite-source-map.md) registra caminhos e linhas para comandos, transações, documento, cels, ferramentas, renderização, CLI e Lua.

1. **Comando da aplicação e alteração reversível são abstrações diferentes.** Aseprite separa `Command` de `Cmd` e agrupa alterações em `Transaction`. O SpriteLab deve expor intenção estável e guardar alterações reversíveis internamente. [Command](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/app/commands/command.h), [Transaction](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/app/transaction.cpp).
2. **Um traço é uma ação, não milhares de ações de histórico.** O pipeline de ferramentas usa canvases temporários e publica regiões alteradas ao terminar. Isso informa a próxima etapa de stroke sessions e patches por região no SpriteLab. [Tool loop](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/app/ui/editor/tool_loop_impl.cpp), [ExpandCelCanvas](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/app/util/expand_cel_canvas.cpp).
3. **Cel e imagem precisam de identidades distintas.** Uma imagem pode ser compartilhada; posição, frame, layer e semântica de edição pertencem às estruturas apropriadas. A primeira base usa buffers compartilhados com copy-on-write; linked cels explícitos ainda não existem. [Cel](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/doc/cel.h), [CelData](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/doc/cel_data.h).
4. **Reuso não prova paridade universal.** CLI e Lua reutilizam várias operações, mas há caminhos Lua que manipulam pixels diretamente. Não podemos afirmar que o Aseprite já satisfaz o contrato rigoroso desejado para SpriteLab. [Lua Image](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/app/script/image_class.cpp), [CLI processor](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/app/cli/cli_processor.cpp).
5. **Preview e exportação precisam do mesmo compositor canônico.** O Aseprite compartilha renderização em diversos fluxos, mas também tem variantes de renderer de interface. Paridade entre variantes exige teste; para SpriteLab, preview/export já usam a mesma função. [Renderer](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/src/render/render.h).

Essas conclusões estruturais vêm de análise estática do código. Não foram medidos desempenho, UX, fidelidade de todos os formatos, undo de cada API Lua ou equivalência visual CPU/GPU do Aseprite. A observação de CLI não foi usada para afirmar comportamento não observado do editor.

## Base executável entregue

O workspace Rust contém `assetforge-command` (contratos sem pixels/MCP), `spritelab-core` (engine/modelo/compositor) e `spritelab-cli` (runtime/arquivos e transporte JSONL). O adaptador MCP descobre os schemas no catálogo Rust e usa o SDK oficial. Python envia os mesmos envelopes pela CLI.

São 26 operações: documentos e sessão, inspeção/render/preview, layers, frames/duração, palette, stroke/line/fill/patch, tag, batch, undo/redo, projeto editável, import PNG e export PNG/atlas. Os comandos realmente implementados e seus schemas estão em [commands.json](../commands.json); recursos ainda planejados não aparecem como funcionalidades prontas.

O lote é atômico no documento e um único item de undo. Revisões são monotônicas; a revisão esperada pode rejeitar uma edição concorrente/desatualizada. I/O fica fora dos lotes de edição. O exemplo Python gera sprites 32×32 com oito cores, duas layers, quatro frames e tag idle, salva os projetos e exporta PNG/atlas/metadados.

Foi comprovada a igualdade de resultados/estado/erros entre CLI direta, replay Python e MCP para a sequência de integração. Foram gerados **200 sprites de teste, totalizando 800 frames**, numa única sessão CLI, com liberação dos documentos após salvar/exportar. Isso é prova funcional de processamento em lote; não é benchmark profissional nem prova de diversidade artística.

## Ajustes recomendados à proposta

- Antecipar CLI e adaptador MCP fino para a fundação. A paridade deve ser verificável antes de acumular regras na interface.
- Manter Tauri + React como cliente do runtime Rust. As ferramentas de desenho persistente e regras de animação ficam no engine.
- Implementar a sessão local compartilhada antes de prometer que um agente e o artista editam o mesmo documento aberto. Hoje cada processo é isolado.
- Evoluir history para patches de região/tile e o documento para cels/imagens separados sem alterar os contratos dos clientes.
- Projetar jobs, commit points, cancelamento e idempotência antes de liberar automações longas ou remotas.
- Adicionar compatibilidade `.aseprite` por especificação pública e fixtures, com relatório de perdas. Ela ainda não está implementada. [Especificação do formato](https://github.com/aseprite/aseprite/blob/14fe117e853bd642672e72018166239506060cd4/docs/ase-file-specs.md).
- Preservar SpriteLab como produto independente. O próximo domínio 3D compartilhará infraestrutura, não o modelo de pixels.

A próxima entrega deve ser o primeiro fluxo manual real na interface Tauri: criar documento, desenhar, editar layers, desfazer, salvar e reabrir, comprovando paridade com este engine. A [arquitetura](../ARCHITECTURE.md), os ADRs e o [roadmap](../ROADMAP.md) detalham esses limites e critérios.
