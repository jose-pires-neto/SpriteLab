# MCP e sessão desktop

O processo Tauri possui um único `Runtime`, compartilhado com seus comandos nativos e com clientes por socket Unix privado (modo 0600). O MCP não possui implementação de edição. Ele pode iniciar uma sessão headless ou anexar-se ao runtime já aberto.

## Servidores registrados no Codex

- `spritelab_live`: usa `--attach-latest` na pasta `assets/`. Abra o SpriteLab antes de iniciar/recarregar o cliente MCP. Seleciona a sessão ativa mais recente dessa pasta; para múltiplas janelas, prefira `--attach SOCKET` explícito.
- `spritelab_headless`: inicia uma sessão isolada para scripts e geração em lote. Seus documentos não são os da janela.

O registro global foi feito pelo CLI e confirmado com `codex mcp get spritelab_live --json`. A configuração equivalente também está em `outputs/mcp-config.toml` e `.codex/config.toml`. Não há confirmação de recarga dos novos tools neste chat; reinicie/reconecte o cliente Codex quando necessário. Configurações locais dependem de confiança no projeto, conforme a [documentação oficial](https://learn.chatgpt.com/docs/config-file/config-basic); o formato stdio segue a [documentação MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).

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

## Instalação fixa para uso diário — 0.4.3

Aplicativo: `~/Applications/SpriteLab.app`, assinatura local verificada. MCP global e override do projeto apontam para `outputs/runtime-0.4.3/mcp/server.mjs` e `outputs/runtime-0.4.3/spritelab`; dependências Node foram copiadas junto. Essa versão permanece fixa durante novos builds de desenvolvimento. Não mova o repositório/pasta outputs sem atualizar os caminhos. Node continua em `/opt/homebrew/bin/node`.

Abra SpriteLab antes de conectar/recarregar os MCPs no Codex. No chat novo, use `spritelab_live` para ver e editar a janela; `spritelab_headless` é uma sessão independente. Para verificar, peça primeiro `editor.inspect`, catálogo e preview do documento selecionado. O MCP live anexa à sessão mais recente do workspace; com várias versões abertas, confirme documento/revisão antes de editar. A conexão não migra automaticamente para outra janela.

O registro global foi verificado com `codex mcp get`. O SDK oficial também verificou 50 tools nos dois modos, preview isolado de camada e leitura sem mutação da sessão live. Evidência: `installed-mcp-0.4.3.json`. A conexão de um novo chat no próprio Codex ainda precisa ser confirmada pelo usuário; o catálogo deste chat já aberto não foi recarregado. Se live não aparecer, mantenha SpriteLab aberto e reinicie o Codex. Configuração segue a [documentação oficial de MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).
