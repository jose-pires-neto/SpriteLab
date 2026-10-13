# ADR 0006: transferência de cels

Status: implementado na 0.4.2.

`cel.transfer` copia ou move um cel completo entre camada/quadro do mesmo documento. IDs de origem/destino são obrigatórios; `move_source` e `overwrite` são falsos por padrão. Origem ausente falha; destino presente exige overwrite explícito. Copiar de uma camada bloqueada é leitura permitida; mover exige origem desbloqueada. Destino sempre exige camada desbloqueada. Mesma origem/destino é no-op após validação da origem e dos bloqueios.

Efeito `edit`, undoable: true. Usa transação já existente, uma revisão e uma entrada de undo; falha de validação/orçamento descarta a operação inteira. Buffer compartilhado internamente por Arc, com detach ao escrever: não há vínculo semântico entre cels. O formato do projeto não muda. A UI não manipula buffers; MCP descobre o schema pelo catálogo Rust.

A área de transferência da GUI guarda IDs na sessão atual e é limpa ao trocar documento; colar lê o conteúdo atual da origem. Não é uma integração com o clipboard do sistema nem snapshot de pixels. Arrastar só usa IDs internos e nunca aceita arquivos externos como cels. A UI confirma destino ocupado; o engine exige overwrite também nos demais clientes. Guardas de revisão continuam a proteger todos os edits.
