# SpriteLab 0.3.1 — editor e MCP

Dê dois cliques em `Abrir SpriteLab.command`, na pasta de entregas. Ele abre o aplicativo contido em `SpriteLab-0.3.1-macOS-arm64.zip`. Não precisa executar servidor nem comandos de desenvolvimento.

Os projetos ficam em `~/Documents/ChatGPT/SpriteLab/assets`. Clique em `exemplo-cogumelo.spritelab` na lateral para explorar uma animação original de quatro frames. Coloque outros projetos ou PNGs nessa pasta para abri-los/importá-los pela lateral.

- Novo cria um canvas quadrado de 1 a 512 pixels.
- B: lápis; E: borracha; L: linha; F: preencher; I: conta-gotas.
- Clique e arraste para desenhar. Cada traço gera uma única entrada no histórico. Esc cancela o traço em andamento.
- Command-Z desfaz; Command-Shift-Z refaz; Command-S salva o projeto editável.
- Escolha a camada na lateral. Os botões controlam visibilidade/bloqueio. Duplo clique renomeia a camada; o controle abaixo ajusta opacidade.
- Os frames ficam embaixo. Você pode duplicar/criar frames, ajustar duração e reproduzir com Espaço.
- PNG exporta o frame atual em tamanho original, com transparência. Spritesheet exporta todos os frames em uma linha.

Esta versão permite anexar CLI e MCP à sessão da janela. Tem seleção retangular, formas, transformações, onion skin, GIF e reorganização de frames/camadas. Consulte MCP-SESSAO.md para os modos live e headless e COBERTURA.md para as lacunas restantes.

O pacote entregue é uma compilação local de desenvolvimento para Mac com Apple Silicon; possui assinatura ad-hoc local; ainda não é uma distribuição pública com Developer ID/notarização. A pasta de trabalho é fixa nesta versão, mas o executável aceita `--workspace CAMINHO` ao ser iniciado pelo terminal.

No painel Comandos, escolha uma operação do catálogo Rust, veja o schema e envie os argumentos JSON. U seleciona retângulo, M seleção. Os botões da lateral movem/copiam/espelham/giram a região selecionada. Grade e onion são overlays visuais; não aparecem no PNG exportado. GIF reduz cores/alpha e arredonda tempos para unidades de 10 ms.

Se a janela 0.3 já estiver aberta, salve seu projeto e feche-a antes de abrir 0.3.1. A atualização foi criada em outra pasta para preservar sua sessão atual.

Dois servidores foram registrados no Codex: `spritelab_live` controla a sessão da janela aberta; `spritelab_headless` inicia uma sessão independente para scripts/lotes. Abra o SpriteLab antes de reconectar o servidor live. Recarregue os MCPs ou reinicie o Codex para carregar a nova configuração.

O seletor no topo permite trocar entre documentos da sessão, inclusive os abertos pelo MCP. Um ponto marca alterações não salvas; ao fechar, o editor verifica todos os documentos da sessão.
