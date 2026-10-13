# SpriteLab 0.4.0

Primeiro incremento após a base 0.3.1: recuperação e experiência visual Studio. O arquivo editável continua no formato v1.

## Entrega

- Quatro comandos de recuperação registrados no Rust e descobertos pelo MCP: 49 comandos no total.
- Worker desktop a cada 15 segundos, cobrindo todos os documentos alterados, inclusive alterações de agentes.
- Painel para recuperar em novo documento ou descartar explicitamente uma cópia.
- Ícones SVG originais, contraste/foco, superfícies neutras e miniaturas canônicas dos quadros.
- Base 0.3.1 registrada com tag; evolução em branch `codex/spritelab-0.4-recovery-studio`.

## Verificação automatizada

macOS arm64: formatação, Clippy com warnings negados, 31 testes Rust, build locked, instalação limpa npm, 6 testes com SDK MCP real e build TypeScript/Vite. Recuperação testada entre runtimes independentes com igualdade de documento, arquivo original preservado, rejeição de revisão antiga e restauração sem substituição de documento existente.

A sandbox bloqueou sockets locais na primeira execução; os testes completos passaram com a permissão necessária para IPC local. Um teste MCP iniciado antes do rebuild usou o catálogo anterior; após o rebuild, os seis testes passaram.

## Verificação nativa

Aplicativo aberto no macOS com o slime original e os 32 quadros em miniatura. Uma edição MCP em documento separado foi capturada automaticamente pelo worker; clicar na entrada da recuperação abriu um novo documento. A leitura MCP confirmou igualdade integral do documento com o checkpoint, revisão 0, dirty=true e quatro documentos preservados. O arquivo do slime permaneceu byte a byte igual à entrega aprovada. Evidência: [native-0.4.0-recovery.json](native-0.4.0-recovery.json). A inspeção também encontrou zoom 600% ausente no seletor; a opção foi adicionada.

A primeira tentativa de abertura usava a identidade da versão 0.3.1 e apresentou janela vazia; a 0.4.0 recebeu identidade própria. Uma tentativa de descarte foi bloqueada pela revisão automática e cancelada; o documento foi preservado e o slime aberto separadamente.

## Limites

O checkpoint pode perder os últimos 15 segundos, não armazena undo/viewport e não é um journal de recuperação após perda de energia. Máximo de 128 cópias e 16 MiB por arquivo, sem remoção silenciosa. Headless solicita checkpoints explicitamente. Histórico por regiões, medições de desempenho, ferramentas profissionais e validação Windows/Linux ficam no backlog.

Build local ad-hoc assinado; não é uma distribuição pública notarizada.
