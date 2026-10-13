# SpriteLab Studio

Direção da interface 0.4: uma bancada de animação, com a arte no centro e ferramentas próximas. A referência funcional é um editor de sprites; os ícones e o desenho da interface são originais.

- Superfícies: #303640 nos painéis; #20252d no canvas; #2d333d na timeline.
- Texto: #e5e9f0 principal; #b1bac8 secundário. Fonte de sistema para controles nativos legíveis.
- Seleção: #45617d com contorno #a9c8e8. A cor sinaliza estado e não compete com a paleta do sprite.
- Layout: barra de ferramentas estreita à esquerda; canvas central; propriedades e biblioteca à direita; timeline horizontal embaixo.
- Detalhe principal: miniaturas reais dos quadros, produzidas pelo compositor Rust; nearest-neighbor preserva os pixels.
- Controles com foco visível; ícones SVG originais, nomes acessíveis e atalhos em tooltips. Movimento reduzido respeitado.

A próxima revisão deve melhorar agrupamento de opções, hierarquia da timeline e navegação. Não transformar todos os controles em cartões decorativos nem esconder funcionalidades para simplificar a aparência.
