# Cobertura funcional — SpriteLab 0.4.1

Referência de produto: [recursos publicados do Aseprite](https://www.aseprite.org/) e [documentação](https://www.aseprite.org/docs/), consultados em 13/10/2026. Esta matriz registra o estado real do SpriteLab; não declara equivalência integral, compatibilidade de arquivos ou superioridade de desempenho.

| Área | Implementado nesta versão | Lacunas para cobertura profissional |
|---|---|---|
| Desenho | Lápis, borracha, linha, fill, retângulo/elipse preenchidos ou contornados, tamanho do pincel, alpha, simetria X/Y, remoção de cantos de staircase | Pincéis personalizados, dithering, gradientes, shading, controle por pressão |
| Seleção e transformação | Seleção retangular, mover/copiar/limpar regiões, flip X/Y, rotação exata de 90° | Lasso, seleção por cor/máscaras arbitrárias, clipboard do sistema, rotação livre/RotSprite, escala de seleção |
| Camadas | Criar, nomear, duplicar, ordenar, excluir, opacidade, visibilidade e bloqueio de pixels | Grupos, blend modes, merge/flatten, camadas referência |
| Animação | Criar/duplicar/ordenar/excluir frames, duração, reprodução, tags, onion skin anterior/posterior com cores distintas | Cels ligados como recurso de edição, faixas de tags na UI, reverse/ping-pong, preview separado, drag/drop |
| Cor | RGBA, alpha, conta-gotas, metadados de paleta editáveis pelo catálogo/MCP | Modo indexed/grayscale, perfis de cor, editor avançado de paleta |
| Arquivos | Projeto editável JSON, PNG import/export, atlas PNG e metadados via comando, GIF animado | Importação/exportação .aseprite, APNG, sequências PNG, atlas empacotado/trim, templates de engines |
| Navegação | Zoom, scroll do canvas, grade, ferramentas por teclado, painel de comandos/schemas, abas de documentos e opções contextuais | Pan com ferramenta dedicada, atalhos customizáveis, layouts e abas com previews |
| Automação | 49 comandos descobertos em Rust; CLI JSONL, Python e MCP; sessão desktop compartilhada; view/contexto também expostos | Jobs com progresso/cancelamento, eventos/subscriptions, retries com idempotência durável |
| Robustez | Revisões, operações atômicas, undo/redo, limites de recursos, arquivos escritos atomicamente, estado salvo por documento, checkpoints de recuperação a cada 15 s, proteção no fechamento, paridade testada | Histórico por tiles, documentos muito grandes, testes Windows/Linux e distribuição assinada |
| Tilemaps e slices | Não implementado | Tilemap/tilesets, slice/nine-patch, tiled preview |
| Extensões | Contratos e catálogo preparados | SDK público versionado, plugins isolados, catálogo/gerenciamento de assets e domínio 3D |

Todas as operações implementadas de asset têm entrada no Command Engine e no MCP. A interface usa os mesmos comandos. Recursos apenas visuais (ferramenta ativa, cor, alpha, pincel, zoom, grade, onion, seleção e playback) são expostos por `editor.configure`; não entram no undo de pixels.

Limites atuais: 512×512, 32 camadas, 64 frames, 4 documentos por sessão, 524.288 pixels de cels alocados por documento e 32 entradas de histórico com estimativa conservadora de 32 MiB. Esses limites continuam sendo de protótipo.

## Próxima sequência de implementação

1. Ampliar testes de recuperação e implementar histórico de regiões; garantir orçamento de memória em projetos representativos.
2. Blend modes, merge/grupos, cels explícitos e semântica de vinculação.
3. Seleções por máscaras, clipboard, transformações adicionais e pincéis personalizados.
4. Modo indexed/perfis, tilemap, slices e tiled preview.
5. Interoperabilidade .aseprite, sequências/formatos adicionais e presets Godot/Unity.
6. Recursos completos de automação: jobs, notificações, cancelamento e idempotência; QA cruzado entre plataformas.

Cada item só muda para implementado depois de código, teste do comportamento, prova de exposição MCP e validação do fluxo de uso. O objetivo continua sendo um editor completo, com cobertura rastreável, e não uma afirmação prematura de que todas as funções do Aseprite já existem.

### Incremento 0.4.2

Grade de cels, seleção conjunta de camada/quadro, transferência (copiar/mover) e limpeza, miniaturas canônicas, indicação de tags e ajuste de altura implementados. O engine valida transferências e fornece undo/redo, efeito `edit`, schemas e MCP. Ainda não há cels vinculados, seleção múltipla, timeline com docking, edição visual de intervalos de tags ou reprodução reversa/ping-pong.

## Correção 0.4.3

A lista de camadas e a grade de cels da timeline foram removidas da GUI. Miniaturas de camada estão implementadas no painel direito. O comando cel.transfer permanece disponível por engine/CLI/MCP; document.preview aceita layer_id opcional como consulta de conteúdo isolado sem alterar visibilidade, opacidade ou histórico.
