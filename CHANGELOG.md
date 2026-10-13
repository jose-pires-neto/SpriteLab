# Histórico de versões

## 0.4.0 — 13/10/2026

- Recuperação automática a cada 15 segundos de todos os documentos alterados na sessão desktop, incluindo alterações via MCP.
- Comandos Rust `recovery.checkpoint`, `recovery.list`, `recovery.restore` e `recovery.delete`, disponíveis também na CLI e no MCP. Restauração em novo documento; arquivos originais preservados.
- Painel de recuperação com erros explícitos e descarte confirmado; cópias independentes por runtime e sem remoção automática.
- Primeira revisão visual Studio: superfícies neutras, legibilidade, foco de teclado, ícones SVG próprios e miniaturas reais dos quadros.
- Catálogo expandido de 45 para 49 comandos.

Limites: recuperação cobre conteúdo, não histórico/viewport; o intervalo pode perder até os últimos 15 segundos. Histórico por regiões, medições de desempenho, ferramentas avançadas e validação Windows/Linux continuam pendentes.

## 0.3.1

Base desktop/Command Engine/MCP registrada em Git antes da evolução. Desenho, animação básica, sessão compartilhada, PNG/GIF/atlas e projeto editável.
