# MCP e sessão desktop

O processo Tauri possui um único `Runtime`, compartilhado com seus comandos nativos e com clientes por socket Unix privado (modo 0600). O MCP não possui implementação de edição. Ele pode iniciar uma sessão headless ou anexar-se ao runtime já aberto.

## Registro no Codex e outros hosts

Rode `node scripts/configure-mcp.mjs --workspace /caminho/absoluto/assets` a partir do checkout ou pacote instalado. O script descobre Node, engine e adapter no computador do usuário e registra globalmente `spritelab_live` e `spritelab_headless`. `--print` gera JSON sem modificar Codex. Veja [Quick start](QUICKSTART.md). Configurações locais `.codex/config.toml` podem sobrescrever registros globais; remova ou atualize overrides antigos do SpriteLab.

Abra SpriteLab antes de conectar/recarregar o MCP live. Este seleciona a sessão ativa mais recente da pasta; para múltiplas janelas, `--attach SOCKET` permite escolher explicitamente. Headless inicia sessão isolada; não compartilha os documentos da janela. Se tools não aparecerem num chat novo, recarregue Codex com a aplicação aberta. Registro e handshake não comprovam o fluxo do novo chat: confirme inspeção, preview e uma operação real.

## Uso

1. `spritelab_editor_inspect`: obter documento/frame/camada/ferramenta selecionados, revisão, nome salvo e estado dirty.
2. `spritelab_document_list` / `spritelab_document_inspect`: consultar objetos e IDs; nunca assumir que todos os sprites usam os mesmos IDs.
3. Aplicar comandos com `document_id`, `expected_revision` e `args`. Revisões antigas falham com `revision_conflict`, sem sobrescrever outra edição.
4. `spritelab_editor_configure` pode selecionar o documento e frame que a janela apresenta. Todos os comandos do catálogo Rust são tools MCP, incluindo transforms, GIF, onion e queries.
5. `spritelab_document_preview` retorna imagem PNG nativa; `spritelab_document_save` preserva o projeto editável.

A janela sincroniza contexto/revisão a cada 750 ms, pausando a sincronização durante o gesto local. Um traço iniciado numa revisão antiga é rejeitado se outro cliente editar no meio; o documento é recarregado. Não há merge colaborativo automático. File writes e contexto visual não entram no undo de assets.

## CLI conectada

O editor grava `assets/.spritelab-session-PID.json`, contendo o socket e a identidade da sessão. Há uma sessão por processo; não use a metadata de outra pasta.

```sh
./target/debug/spritelab --attach /caminho/ao/socket
node adapters/mcp/server.mjs --engine "$PWD/target/debug/spritelab" --workspace "$PWD/assets" --attach-latest
node adapters/mcp/server.mjs --engine "$PWD/target/debug/spritelab" --workspace "$PWD/assets"
```

O último comando é headless. `spritelab --workspace assets --host` também pode hospedar uma sessão sem GUI. A demonstração `node adapters/mcp/demo-live.mjs assets/.spritelab-session-PID.json` usa o SDK MCP oficial e cria um documento de validação separado na janela.

## Limites

Transporte local Unix validado no macOS; named pipes do Windows ainda não existem. Até 16 conexões, 16 MiB por mensagem e timeout de resposta/escrita; a conexão do servidor pode permanecer ociosa. Encerrar o MCP não fecha a sessão do editor. Encerrar o editor fecha o host; resultados de requests interrompidos não devem ser tratados como rollback. Não há retry automático de mutações, recuperação pós-crash ou reconexão transparente. Metadata/socket obsoletos podem persistir após saída abrupta; descoberta ignora PIDs encerrados.
