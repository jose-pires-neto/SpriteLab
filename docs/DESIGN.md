# SpriteLab Studio

Direção da interface 0.4: uma bancada de animação, com a arte no centro e ferramentas próximas. A referência funcional é um editor de sprites; os ícones e o desenho da interface são originais.

- Superfícies: #303640 nos painéis; #20252d no canvas; #2d333d na timeline.
- Texto: #e5e9f0 principal; #b1bac8 secundário. Fonte de sistema para controles nativos legíveis.
- Seleção: #45617d com contorno #a9c8e8. A cor sinaliza estado e não compete com a paleta do sprite.
- Layout: barra de ferramentas estreita à esquerda; canvas central; propriedades e biblioteca à direita; timeline horizontal embaixo.
- Detalhe principal: miniaturas reais dos quadros, produzidas pelo compositor Rust; nearest-neighbor preserva os pixels.
- Controles com foco visível; ícones SVG originais, nomes acessíveis e atalhos em tooltips. Movimento reduzido respeitado.

A próxima revisão deve melhorar agrupamento de opções, hierarquia da timeline e navegação. Não transformar todos os controles em cartões decorativos nem esconder funcionalidades para simplificar a aparência.

## Revisão 0.4.1: organização por tarefa

A revisão 0.4.0 manteve controles demais empilhados no mesmo painel. A 0.4.1 muda a estrutura: menu para operações de arquivo/sprite; abas para documentos; barra contextual para a ferramenta ativa; paleta e camadas fixas à direita, com rolagem independente; timeline compacta embaixo; zoom/grade no rodapé. Biblioteca e recuperação são diálogos próprios e não disputam espaço com a edição. Abrir/criar preserva documentos existentes; fechar uma aba alterada pede confirmação e usa revisão esperada.

Referências de fluxo: [workspace do Aseprite](https://www.aseprite.org/docs/workspace/) e [workspace do Photoshop](https://helpx.adobe.com/photoshop/desktop/get-started/learn-the-basics/workspace-overview.html). A implementação, ícones e estilos continuam próprios. Docking livre e layouts persistentes ainda não estão implementados.
