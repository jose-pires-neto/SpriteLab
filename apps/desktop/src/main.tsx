import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import "./style.css";
import { ToolIcon, FrameThumbnail, StudioMenu, ResourceDialog } from "./studio";
import { FrameTimeline } from "./timeline";
type Doc = {
  width: number;
  height: number;
  layers: {
    id: number;
    name: string;
    visible: boolean;
    locked: boolean;
    opacity: number;
  }[];
  frames: { id: number; duration_ms: number; cels: Record<string, unknown> }[];
  tags: { name: string; from_frame: number; to_frame: number }[];
  palette: number[][];
};
function App() {
  const [dialog, Dialog] = useState<{
    title: string;
    value?: string;
    resolve: (v: any) => void;
  }>();
  const ask = (title: string, value?: string): Promise<any> =>
    new Promise((resolve) => Dialog({ title, value, resolve }));
  const answer = (v: any) => {
    dialog?.resolve(v);
    Dialog(undefined);
  };
  const documentId = useRef("editor");
  const ready = useRef(false);
  const incoming = useRef(0);
  const [d, D] = useState<Doc>();
  const [documents, Documents] = useState<
    { document_id: string; dirty: boolean; path?: string }[]
  >([]);
  const [f, F] = useState(2);
  const [l, L] = useState(1);
  const [tool, T] = useState("stroke");
  const [color, C] = useState("#e3a75c");
  const [zoom, Z] = useState(12);
  const [status, S] = useState("Iniciando…");
  const [resource, Resource] = useState<"files" | "recovery">();
  const [recovery, Recovery] = useState<
    {
      entry: string;
      source_path?: string;
      document_id: string;
      error?: string;
      updated_ms: number;
    }[]
  >([]);
  const [recoveryError, RecoveryError] = useState<string>();
  const [files, Files] = useState<string[]>([]);
  const [root, Root] = useState("");
  const [name, N] = useState("sem-titulo");
  const [dirty, Dirty] = useState(false);
  const [play, Play] = useState(false);
  const [grid, Grid] = useState(false);
  const [onion, Onion] = useState(false);
  const [brush, Brush] = useState(1);
  const [alpha, Alpha] = useState(255);
  const [mx, MX] = useState(false);
  const [my, MY] = useState(false);
  const [perfect, Perfect] = useState(false);
  const [selection, Selection] = useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  }>();
  const [filled, Filled] = useState(false);
  const [socket, Socket] = useState("");
  const [commandResult, CommandResult] = useState("");
  const [catalog, Catalog] = useState<any[]>([]);
  const [panel, Panel] = useState(false);
  const [operation, Operation] = useState("canvas.rectangle");
  const [jsonArgs, JsonArgs] = useState("{}");
  const oc = useRef<HTMLCanvasElement>(null);
  const gesture = useRef<number | undefined>(undefined);
  const configure = async (args: any) => {
    const r: any = await invoke("execute", {
      request: { command: "editor.configure", args },
    });
    if (!r.ok) S(r.error.message);
  };
  const view = (args: any) => {
    void configure(args);
  };
  const c = useRef<HTMLCanvasElement>(null);
  const rev = useRef(0);
  const queue = useRef(Promise.resolve());
  const pts = useRef<number[][]>([]);
  const drawing = useRef(false);
  const generation = useRef(0);
  const waiting = useRef<any>(undefined);
  const previewing = useRef(false);
  const saved = useRef("");
  const current = useRef<Doc>(undefined);
  const pixels = useRef<number[]>([]);
  const cmd = (command: string, args: any = {}, guard = true): Promise<any> => {
    let yes!: (a: any) => void, no!: (a: any) => void;
    const p = new Promise((a, b) => {
      yes = a;
      no = b;
    });
    queue.current = queue.current.then(async () => {
      try {
        const r: any = await invoke("execute", {
          request: {
            command,
            args,
            document_id: documentId.current,
            ...(guard
              ? {
                  expected_revision:
                    command.startsWith("canvas.") &&
                    gesture.current !== undefined
                      ? gesture.current
                      : rev.current,
                }
              : {}),
          },
        });
        if (!r.ok) throw Error(r.error.message);
        rev.current = r.revision ?? rev.current;
        yes(r.result);
      } catch (e) {
        no(e);
      }
    });
    return p;
  };
  const paint = (r: any) => {
    if (!c.current) return;
    c.current.width = r.width;
    c.current.height = r.height;
    c.current
      .getContext("2d")!
      .putImageData(
        new ImageData(new Uint8ClampedArray(r.rgba), r.width, r.height),
        0,
        0,
      );
    pixels.current = r.rgba;
  };
  const refresh = async (frame = f) => {
    const r = await cmd("document.snapshot", { frame_id: frame }, false);
    const a = r.document as Doc;
    current.current = a;
    D(a);
    F(r.frame_id);
    L((old) => (a.layers.some((v) => v.id === old) ? old : a.layers[0].id));
    Dirty(JSON.stringify(a) !== saved.current);
    paint(r);
    if (onion && oc.current) {
      const o = await cmd("document.onion", { frame_id: r.frame_id }, false);
      oc.current.width = o.width;
      oc.current.height = o.height;
      oc.current
        .getContext("2d")!
        .putImageData(
          new ImageData(new Uint8ClampedArray(o.rgba), o.width, o.height),
          0,
          0,
        );
    }
  };
  const workspace = async () => {
    const a: any = await invoke("workspace");
    Root(a.path);
    Files(a.files);
    Socket(a.socket);
  };
  const clearDocument = () => {
    documentId.current = "";
    rev.current = 0;
    current.current = undefined;
    pixels.current = [];
    D(undefined);
    Play(false);
    Selection(undefined);
    Dirty(false);
    if (c.current) c.current.width = 1;
    if (oc.current) oc.current.width = 1;
  };
  const run = async (command: string, args: any = {}) => {
    if (drawing.current) return;
    generation.current++;
    Play(false);
    try {
      const result = await cmd(command, args);
      CommandResult(JSON.stringify(result, null, 2).slice(0, 4000));
      if (command === "document.close") {
        clearDocument();
        S("Documento fechado");
        return;
      }
      gesture.current = undefined;
      const item = result.results?.[0];
      if (item?.layer_id && command === "layer.create") L(item.layer_id);
      await refresh(
        item?.frame_id && command === "frame.add" ? item.frame_id : f,
      );
      await configure({
        document_id: documentId.current,
        playback: false,
        frame_id:
          command === "frame.add"
            ? item.frame_id
            : current.current!.frames.some((v) => v.id === f)
              ? f
              : current.current!.frames[0].id,
        layer_id: ["layer.create", "layer.duplicate"].includes(command)
          ? item.layer_id
          : current.current!.layers.some((v) => v.id === l)
            ? l
            : current.current!.layers[0].id,
      });
      S(command + " / revisão " + rev.current);
    } catch (e) {
      gesture.current = undefined;
      S(String(e));
      void refresh();
    }
  };
  useEffect(() => {
    (async () => {
      try {
        await workspace();
        await cmd("document.create", { width: 32, height: 32 }, false);
        await configure({
          document_id: documentId.current,
          frame_id: 2,
          layer_id: 1,
        });
        Catalog((await cmd("command.list", {}, false)).commands);
        await refresh();
        saved.current = JSON.stringify(current.current);
        Dirty(false);
        ready.current = true;
        S("Pronto para desenhar");
      } catch (e) {
        S(String(e));
      }
    })();
  }, []);
  useEffect(() => {
    const update = async () => {
      try {
        const result: any = await invoke("execute", {
          request: { command: "recovery.list", args: {} },
        });
        if (!result.ok) throw Error(result.error.message);
        Recovery(result.result.entries);
        const info: any = await invoke("workspace");
        RecoveryError(info.recovery_error ?? undefined);
      } catch (e) {
        RecoveryError(String(e));
      }
    };
    void update();
    const timer = setInterval(() => void update(), 15000);
    return () => clearInterval(timer);
  }, []);
  const restore = async (entry: string) => {
    try {
      const next = "recovered-" + Date.now();
      const response: any = await invoke("execute", {
        request: {
          document_id: next,
          command: "recovery.restore",
          args: { entry },
        },
      });
      if (!response.ok) throw Error(response.error.message);
      documentId.current = next;
      rev.current = 0;
      await configure({ document_id: next, playback: false });
      await refresh();
      Resource(undefined);
      N("recuperado");
      Dirty(true);
      S(
        "Trabalho recuperado em um novo documento. Salve para escolher o arquivo.",
      );
    } catch (e) {
      S(String(e));
    }
  };
  useEffect(() => {
    if (!play || !d) return;
    const timer = setTimeout(
      () => {
        const next =
          d.frames[
            (d.frames.findIndex((v) => v.id === f) + 1) % d.frames.length
          ].id;
        F(next);
        view({ frame_id: next });
        cmd("document.render", { frame_id: next })
          .then(paint)
          .catch((e) => S(String(e)));
      },
      d.frames.find((v) => v.id === f)!.duration_ms,
    );
    return () => clearTimeout(timer);
  }, [play, f, d]);
  useEffect(() => {
    let busy = false;
    const timer = setInterval(async () => {
      if (!ready.current || busy || drawing.current || dialog) return;
      busy = true;
      try {
        const r: any = await invoke("execute", {
          request: { command: "editor.inspect", args: {} },
        });
        const v = r.result.view;
        Documents(r.result.documents);
        Dirty(r.result.dirty);
        N(
          r.result.document_path
            ? r.result.document_path.replace(/\.spritelab$/, "")
            : (v.document_id ?? "sem-titulo"),
        );
        if (!r.result.dirty) saved.current = JSON.stringify(current.current);
        if (v.document_id && r.result.document_revision !== null) {
          const changed =
            v.document_id !== documentId.current ||
            r.result.document_revision !== rev.current ||
            v.frame_id !== f ||
            v.onion_skin !== onion;
          documentId.current = v.document_id;
          T(v.tool);
          C(
            "#" +
              v.color
                .slice(0, 3)
                .map((n: number) => n.toString(16).padStart(2, "0"))
                .join(""),
          );
          Filled(v.filled);
          Z(v.zoom);
          Grid(v.grid);
          Onion(v.onion_skin);
          Play(v.playback);
          Brush(v.brush_size);
          Alpha(v.alpha);
          MX(v.mirror_x);
          MY(v.mirror_y);
          Perfect(v.pixel_perfect);
          Selection(v.selection ?? undefined);
          if (v.layer_id) L(v.layer_id);
          if (changed) {
            await refresh(v.frame_id);
            S("Sessão sincronizada / revisão " + rev.current);
          }
        } else if (documentId.current) {
          clearDocument();
          S("Nenhum documento selecionado / crie ou abra um projeto");
        }
      } catch (e) {
        S(String(e));
      } finally {
        busy = false;
      }
    }, 750);
    return () => clearInterval(timer);
  }, [f, onion, dialog]);
  useEffect(() => {
    if (current.current && onion) void refresh(f);
  }, [onion, f]);
  useEffect(() => {
    const timer = setInterval(
      () => void workspace().catch((e) => S(String(e))),
      5000,
    );
    return () => clearInterval(timer);
  }, []);
  const save = async () => {
    const n = await ask("Nome do projeto (sem extensão)", name);
    if (!n) return;
    if (!/^[A-Za-z0-9_-]+$/.test(n)) {
      S("Use letras, números, - ou _");
      return;
    }
    const path = n + ".spritelab";
    if (files.includes(path) && !(await ask("Substituir " + path + "?")))
      return;
    try {
      await cmd("document.save", { path, overwrite: files.includes(path) });
      saved.current = JSON.stringify(current.current);
      Dirty(false);
      N(n);
      await workspace();
      S("Salvo: " + path);
    } catch (e) {
      S(String(e));
    }
  };
  const open = async (path: string) => {
    Play(false);
    try {
      const next = "editor-" + ++incoming.current;
      const r: any = await invoke("execute", {
        request: {
          document_id: next,
          command: path.endsWith(".png") ? "import.png" : "document.open",
          args: { path },
        },
      });
      if (!r.ok) throw Error(r.error.message);
      documentId.current = next;
      await configure({ document_id: next });
      rev.current = 0;
      await refresh();
      saved.current = JSON.stringify(current.current);
      Dirty(false);
      N(path.replace(/\.(spritelab|png)$/, ""));
      Resource(undefined);
      S("Aberto: " + path);
    } catch (e) {
      S(String(e));
    }
  };
  const closeTab = async (id: string) => {
    if (drawing.current) return;
    try {
      const inspection: any = await invoke("execute", {
        request: { command: "document.inspect", document_id: id, args: {} },
      });
      if (!inspection.ok) throw Error(inspection.error.message);
      const listing: any = await invoke("execute", {
        request: { command: "document.list", args: {} },
      });
      if (!listing.ok) throw Error(listing.error.message);
      const entry = listing.result.documents.find(
        (item: any) => item.document_id === id,
      );
      if (
        entry?.dirty &&
        !(await ask("Fechar este documento e descartar alterações não salvas?"))
      )
        return;
      const response: any = await invoke("execute", {
        request: {
          command: "document.close",
          document_id: id,
          expected_revision: inspection.revision,
          args: {},
        },
      });
      if (!response.ok) throw Error(response.error.message);
      const state: any = await invoke("execute", {
        request: { command: "editor.inspect", args: {} },
      });
      documentId.current = state.result.view.document_id ?? "";
      if (documentId.current) {
        rev.current = state.result.document_revision;
        await refresh(state.result.view.frame_id);
      } else clearDocument();
      Documents(state.result.documents);
    } catch (e) {
      S(String(e));
    }
  };
  const fresh = async () => {
    const size = Number(
      await ask("Tamanho do canvas quadrado (1 a 512)", "32"),
    );
    if (!Number.isInteger(size) || size < 1 || size > 512) return;
    const next = "editor-" + ++incoming.current;
    try {
      const result: any = await invoke("execute", {
        request: {
          document_id: next,
          command: "document.create",
          args: { width: size, height: size },
        },
      });
      if (!result.ok) throw Error(result.error.message);
    } catch (e) {
      S(String(e));
      return;
    }
    documentId.current = next;
    rev.current = 0;
    saved.current = "";
    N("sem-titulo");
    await configure({ document_id: documentId.current });
    await refresh();
  };
  const exportFile = async (sheet = false, gif = false) => {
    const path =
      ((await ask(
        "Nome do PNG (sem extensão)",
        name + (sheet ? "-atlas" : ""),
      )) ?? "") + ".png";
    const target = gif ? path.replace(/\.png$/, ".gif") : path;
    if (path === ".png") return;
    if (files.includes(target) && !(await ask("Substituir " + target + "?")))
      return;
    try {
      await cmd(
        gif ? "export.gif" : sheet ? "export.spritesheet" : "export.png",
        {
          path: target,
          overwrite: files.includes(target),
          ...(gif
            ? {}
            : sheet
              ? { columns: d!.frames.length }
              : { frame_id: f }),
        },
      );
      await workspace();
      S("Exportado: " + target);
    } catch (e) {
      S(String(e));
    }
  };
  const bounds = () => {
    const a = pts.current[0],
      b = pts.current.at(-1)!;
    return {
      x: Math.min(a[0], b[0]),
      y: Math.min(a[1], b[1]),
      width: Math.abs(a[0] - b[0]) + 1,
      height: Math.abs(a[1] - b[1]) + 1,
    };
  };
  const edit = () => ({
    command: ["line", "rectangle", "ellipse"].includes(tool)
      ? "canvas." + tool
      : "canvas.stroke",
    args: {
      layer_id: l,
      frame_id: f,
      color:
        tool === "eraser"
          ? [0, 0, 0, 0]
          : [
              parseInt(color.slice(1, 3), 16),
              parseInt(color.slice(3, 5), 16),
              parseInt(color.slice(5, 7), 16),
              alpha,
            ],
      ...(tool === "line"
        ? { from: pts.current[0], to: pts.current.at(-1) }
        : ["rectangle", "ellipse"].includes(tool)
          ? { rect: bounds(), filled }
          : {
              points: [...pts.current],
              size: brush,
              mirror_x: mx,
              mirror_y: my,
              pixel_perfect: perfect,
            }),
    },
  });
  const preview = async () => {
    if (tool === "select") {
      Selection(bounds());
      return;
    }
    waiting.current = edit();
    if (previewing.current) return;
    previewing.current = true;
    try {
      while (waiting.current) {
        const e = waiting.current;
        waiting.current = undefined;
        const token = generation.current;
        const r = await cmd("command.preview", { edit: e, frame_id: f });
        if (token === generation.current && drawing.current) paint(r);
      }
    } catch (e) {
      S(String(e));
      drawing.current = false;
      waiting.current = undefined;
      generation.current++;
      void refresh();
    } finally {
      previewing.current = false;
    }
  };
  const point = (e: React.PointerEvent) => {
    const r = c.current!.getBoundingClientRect();
    return [
      Math.max(
        0,
        Math.min(
          d!.width - 1,
          Math.floor(((e.clientX - r.left) / r.width) * d!.width),
        ),
      ),
      Math.max(
        0,
        Math.min(
          d!.height - 1,
          Math.floor(((e.clientY - r.top) / r.height) * d!.height),
        ),
      ),
    ];
  };
  const down = (e: React.PointerEvent) => {
    if (!d || play || dialog || e.button !== 0) return;
    gesture.current = rev.current;
    const p = point(e);
    if (tool === "pick") {
      const i = (p[1] * d.width + p[0]) * 4;
      C(
        "#" +
          pixels.current
            .slice(i, i + 3)
            .map((v) => v.toString(16).padStart(2, "0"))
            .join(""),
      );
      view({ color: pixels.current.slice(i, i + 4) });
      return;
    }
    pts.current = [p];
    if (tool === "fill") {
      void run("canvas.fill", {
        layer_id: l,
        frame_id: f,
        color: edit().args.color,
        at: p,
      });
      return;
    }
    e.currentTarget.setPointerCapture(e.pointerId);
    drawing.current = true;
    generation.current++;
    void preview();
  };
  const cancel = () => {
    drawing.current = false;
    waiting.current = undefined;
    generation.current++;
    pts.current = [];
    gesture.current = undefined;
    Selection(undefined);
    view({ clear_selection: true });
    void refresh();
  };
  const up = () => {
    if (!drawing.current) return;
    if (tool === "select") {
      const r = bounds();
      drawing.current = false;
      pts.current = [];
      Selection(r);
      view({ selection: r });
      gesture.current = undefined;
      return;
    }
    const e = edit();
    drawing.current = false;
    waiting.current = undefined;
    generation.current++;
    pts.current = [];
    void run(e.command, e.args);
  };
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (dialog || (e.target as HTMLElement).closest("dialog,.menu-items"))
        return;
      if (drawing.current) {
        if (e.key === "Escape") cancel();
        return;
      }
      if ((e.target as HTMLElement).matches("input,select")) return;
      if (e.metaKey || e.ctrlKey) {
        if (e.key === "s") {
          e.preventDefault();
          void save();
        }
        if (e.key === "z") {
          e.preventDefault();
          void run(e.shiftKey ? "history.redo" : "history.undo");
        }
      } else {
        const t: Record<string, string> = {
          b: "stroke",
          e: "eraser",
          l: "line",
          f: "fill",
          i: "pick",
          m: "select",
          u: "rectangle",
        };
        if (t[e.key]) {
          T(t[e.key]);
          view({ tool: t[e.key] });
        }
        if (e.key === "Escape") cancel();
        if (e.code === "Space") {
          e.preventDefault();
          Play((v) => {
            view({ playback: !v });
            return !v;
          });
        }
      }
    };
    window.addEventListener("keydown", key);
    const unload = (e: BeforeUnloadEvent) => {
      if (dirty) {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", unload);
    return () => {
      window.removeEventListener("keydown", key);
      window.removeEventListener("beforeunload", unload);
    };
  }, [d, f, l, dirty, name, files, dialog]);
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    getCurrentWindow()
      .onCloseRequested(async (e) => {
        e.preventDefault();
        try {
          const r: any = await invoke("execute", {
            request: { command: "document.list", args: {} },
          });
          if (!r.ok) throw Error(r.error.message);
          const unsaved = r.result.documents.filter((v: any) => v.dirty);
          if (
            !unsaved.length ||
            (await ask(
              "Fechar e descartar alterações em " +
                unsaved.length +
                " documento(s) não salvo(s): " +
                unsaved.map((v: any) => v.path ?? v.document_id).join(", ") +
                "?",
            ))
          ) {
            await getCurrentWindow().destroy();
          }
        } catch (error) {
          S(String(error));
        }
      })
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [dirty]);
  return (
    <div className="app">
      <header className="menu-bar">
        <strong className="brand">
          <span className="brand-mark" aria-hidden="true">
            ▟
          </span>{" "}
          SpriteLab
        </strong>
        <StudioMenu label="Arquivo">
          <button onClick={() => void fresh()}>Novo documento…</button>
          <button onClick={() => Resource("files")}>Abrir / importar…</button>
          <button disabled={!d} onClick={() => void save()}>
            Salvar… <kbd>⌘S</kbd>
          </button>
          <hr />
          <button disabled={!d} onClick={() => void exportFile()}>
            Exportar PNG…
          </button>
          <button disabled={!d} onClick={() => void exportFile(true)}>
            Exportar spritesheet…
          </button>
          <button disabled={!d} onClick={() => void exportFile(false, true)}>
            Exportar GIF…
          </button>
          <hr />
          <button onClick={() => Resource("recovery")}>
            Recuperar trabalho…
          </button>
        </StudioMenu>
        <StudioMenu label="Editar">
          <button disabled={!d} onClick={() => void run("history.undo")}>
            Desfazer <kbd>⌘Z</kbd>
          </button>
          <button disabled={!d} onClick={() => void run("history.redo")}>
            Refazer <kbd>⇧⌘Z</kbd>
          </button>
        </StudioMenu>
        <StudioMenu label="Sprite">
          <fieldset disabled={!d}>
            {" "}
            <button
              onClick={async () => {
                const text = await ask(
                  "Redimensionar largura,altura",
                  `${d!.width},${d!.height}`,
                );
                if (!text) return;
                const [width, height] = text.split(",").map(Number);
                void run("document.resize", { width, height });
              }}
            >
              Redimensionar sprite
            </button>
            <button
              onClick={async () => {
                const text = await ask(
                  "Canvas largura,altura,offsetX,offsetY",
                  `${d!.width},${d!.height},0,0`,
                );
                if (!text) return;
                const [width, height, offset_x, offset_y] = text
                  .split(",")
                  .map(Number);
                void run("document.canvas_size", {
                  width,
                  height,
                  offset_x,
                  offset_y,
                });
              }}
            >
              Tamanho do canvas
            </button>
          </fieldset>
        </StudioMenu>
        <StudioMenu label="Automação">
          <button onClick={() => Panel(true)}>Inspecionar comandos…</button>
        </StudioMenu>
        <span className="document-meta">
          {d ? `${d.width} × ${d.height} px / RGBA` : "Nenhum documento"}
        </span>
      </header>
      <div
        className="document-tabs"
        role="tablist"
        aria-label="Documentos abertos"
      >
        {documents.map((doc) => (
          <div className="document-tab" key={doc.document_id}>
            <button
              role="tab"
              aria-selected={doc.document_id === documentId.current}
              className={doc.document_id === documentId.current ? "active" : ""}
              onClick={async () => {
                if (drawing.current) return;
                await configure({
                  document_id: doc.document_id,
                  playback: false,
                });
                documentId.current = doc.document_id;
                await refresh();
              }}
            >
              {doc.path?.replace(/\.spritelab$/, "") ?? doc.document_id}
              {doc.dirty ? " •" : ""}
            </button>
            <button
              className="close-tab"
              aria-label={"Fechar documento " + (doc.path ?? doc.document_id)}
              onClick={() => void closeTab(doc.document_id)}
            >
              ×
            </button>
          </div>
        ))}
        <button
          className="new-tab"
          aria-label="Novo documento"
          onClick={() => void fresh()}
        >
          +
        </button>
      </div>
      <div className="options" aria-label="Opções da ferramenta">
        <span className="tool-caption">
          <ToolIcon tool={tool} />
          {
            {
              stroke: "Lápis",
              eraser: "Borracha",
              line: "Linha",
              fill: "Preenchimento",
              pick: "Conta-gotas",
              rectangle: "Retângulo",
              ellipse: "Elipse",
              select: "Seleção",
            }[tool]
          }
        </span>
        <fieldset disabled={!d} className="tool-options">
          {["stroke", "eraser", "line", "rectangle", "ellipse"].includes(
            tool,
          ) && (
            <>
              <label>
                Pincel{" "}
                <input
                  type="number"
                  min="1"
                  max="32"
                  value={brush}
                  onChange={(e) => {
                    Brush(+e.target.value);
                    view({ brush_size: +e.target.value });
                  }}
                />
              </label>
              <label>
                Alpha{" "}
                <input
                  type="number"
                  min="0"
                  max="255"
                  value={alpha}
                  onChange={(e) => {
                    Alpha(+e.target.value);
                    view({ alpha: +e.target.value });
                  }}
                />
              </label>
            </>
          )}
          {["stroke", "eraser", "line"].includes(tool) && (
            <>
              <label>
                Espelho X{" "}
                <input
                  type="checkbox"
                  checked={mx}
                  onChange={(e) => {
                    MX(e.target.checked);
                    view({ mirror_x: e.target.checked });
                  }}
                />
              </label>
              <label>
                Y{" "}
                <input
                  type="checkbox"
                  checked={my}
                  onChange={(e) => {
                    MY(e.target.checked);
                    view({ mirror_y: e.target.checked });
                  }}
                />
              </label>
              <label>
                Pixel perfect{" "}
                <input
                  type="checkbox"
                  checked={perfect}
                  onChange={(e) => {
                    Perfect(e.target.checked);
                    view({ pixel_perfect: e.target.checked });
                  }}
                />
              </label>
            </>
          )}
          {["rectangle", "ellipse"].includes(tool) && (
            <>
              <label>
                <input
                  type="checkbox"
                  checked={filled}
                  onChange={(e) => {
                    Filled(e.target.checked);
                    view({ filled: e.target.checked });
                  }}
                />{" "}
                Formas preenchidas
              </label>
            </>
          )}
          {tool === "fill" && (
            <>
              <label>
                Alpha{" "}
                <input
                  type="number"
                  min="0"
                  max="255"
                  value={alpha}
                  onChange={(e) => {
                    Alpha(+e.target.value);
                    view({ alpha: +e.target.value });
                  }}
                />
              </label>
            </>
          )}
          {tool === "pick" && (
            <span className="hint">
              Clique no canvas para escolher uma cor.
            </span>
          )}
          {tool === "select" && (
            <fieldset disabled={!selection} className="selection-options">
              {" "}
              <p>
                {selection
                  ? `${selection.width} × ${selection.height} em ${selection.x}, ${selection.y}`
                  : "Selecione uma região com M"}
              </p>
              <div className="row">
                {[
                  ["flip_x", "↔"],
                  ["flip_y", "↕"],
                  ["rotate90", "↻"],
                  ["clear", "Limpar"],
                ].map(([operation, label]) => (
                  <button
                    key={operation}
                    onClick={() =>
                      void run("canvas.transform", {
                        layer_id: l,
                        frame_id: f,
                        rect: selection ?? {
                          x: 0,
                          y: 0,
                          width: d!.width,
                          height: d!.height,
                        },
                        operation,
                      })
                    }
                  >
                    {label}
                  </button>
                ))}
              </div>
              <div className="row">
                {["move", "copy"].map((operation) => (
                  <button
                    key={operation}
                    onClick={async () => {
                      if (!selection) return;
                      const text = await ask("Deslocamento dx,dy", "1,0");
                      if (!text) return;
                      const [dx, dy] = text.split(",").map(Number);
                      void run("canvas.transform", {
                        layer_id: l,
                        frame_id: f,
                        rect: selection,
                        operation,
                        dx,
                        dy,
                      });
                    }}
                  >
                    {operation === "move" ? "Mover" : "Copiar"}
                  </button>
                ))}
              </div>
            </fieldset>
          )}
        </fieldset>
      </div>
      <main>
        <nav>
          {[
            ["stroke", "✎", "Lápis (B)"],
            ["eraser", "▱", "Borracha (E)"],
            ["line", "╱", "Linha (L)"],
            ["fill", "◩", "Preencher (F)"],
            ["pick", "⌖", "Conta-gotas (I)"],
            ["rectangle", "□", "Retângulo (U)"],
            ["ellipse", "○", "Elipse"],
            ["select", "⌗", "Seleção (M)"],
          ].map(([t, , label]) => (
            <button
              title={label}
              aria-label={label}
              className={tool === t ? "active" : ""}
              key={t}
              onClick={() => {
                T(t);
                view({ tool: t });
              }}
            >
              <ToolIcon tool={t} />
            </button>
          ))}
          <input
            type="color"
            aria-label="Cor"
            value={color}
            onChange={(e) => {
              C(e.target.value);
              view({
                color: [
                  parseInt(e.target.value.slice(1, 3), 16),
                  parseInt(e.target.value.slice(3, 5), 16),
                  parseInt(e.target.value.slice(5, 7), 16),
                  alpha,
                ],
              });
            }}
          />
        </nav>
        <div className="stage">
          <div
            className="canvas-wrap"
            style={{
              width: (d?.width ?? 32) * zoom,
              height: (d?.height ?? 32) * zoom,
            }}
          >
            <canvas
              ref={oc}
              className="onion"
              style={{ display: onion ? "block" : "none" }}
            />
            <canvas
              ref={c}
              style={{
                width: (d?.width ?? 32) * zoom,
                height: (d?.height ?? 32) * zoom,
              }}
              onPointerDown={down}
              onPointerMove={(e) => {
                if (!drawing.current) return;
                const p = point(e);
                if (
                  String(p) === String(pts.current.at(-1)) ||
                  pts.current.length >= 4096
                )
                  return;
                pts.current.push(p);
                void preview();
              }}
              onPointerUp={up}
              onPointerCancel={cancel}
            />
            {grid && zoom >= 4 && (
              <div
                className="grid-overlay"
                style={{ backgroundSize: `${zoom}px ${zoom}px` }}
              />
            )}
            {selection && (
              <div
                className="selection"
                style={{
                  left: selection.x * zoom,
                  top: selection.y * zoom,
                  width: selection.width * zoom,
                  height: selection.height * zoom,
                }}
              />
            )}
          </div>
        </div>
        <aside>
          <fieldset disabled={!d}>
            <section className="color-panel">
              <h2>
                Paleta <small>Cor & luz</small>
              </h2>
              <div className="palette">
                {(d?.palette.length
                  ? d.palette.map(
                      (p) =>
                        "#" +
                        p
                          .slice(0, 3)
                          .map((v) => v.toString(16).padStart(2, "0"))
                          .join(""),
                    )
                  : [
                      "#182434",
                      "#374e62",
                      "#678296",
                      "#a2bdc4",
                      "#e9dfbc",
                      "#e3a75c",
                      "#c55c45",
                      "#713f53",
                      "#725f95",
                      "#4f8561",
                      "#9fc66c",
                      "#f3f1e8",
                    ]
                ).map((p, i) => (
                  <button
                    title={p}
                    aria-label={p}
                    key={i}
                    style={{ background: p }}
                    onClick={() => {
                      C(p);
                      view({
                        color: [
                          parseInt(p.slice(1, 3), 16),
                          parseInt(p.slice(3, 5), 16),
                          parseInt(p.slice(5, 7), 16),
                          alpha,
                        ],
                      });
                    }}
                  />
                ))}
              </div>
              <p>{color.toUpperCase()}</p>
            </section>
            <section className="layers-panel">
              <h2>
                Camadas{" "}
                <button
                  onClick={() =>
                    void run("layer.create", {
                      name: "Camada " + ((d?.layers.length ?? 0) + 1),
                    })
                  }
                >
                  +
                </button>
              </h2>
              <div className="layer-list">
                {[...(d?.layers ?? [])].reverse().map((v) => (
                  <div
                    key={v.id}
                    className={"layer " + (l === v.id ? "selected" : "")}
                  >
                    <button
                      title="Visibilidade"
                      onClick={() =>
                        void run("layer.update", {
                          layer_id: v.id,
                          visible: !v.visible,
                        })
                      }
                    >
                      {v.visible ? "●" : "○"}
                    </button>
                    <button
                      className="layer-name"
                      onClick={() => {
                        L(v.id);
                        view({ layer_id: v.id });
                      }}
                      onDoubleClick={async () => {
                        const n = await ask("Nome da camada", v.name);
                        if (n)
                          void run("layer.update", { layer_id: v.id, name: n });
                      }}
                    >
                      <FrameThumbnail
                        documentId={documentId.current}
                        frameId={f}
                        layerId={v.id}
                        revision={rev.current}
                      />
                      <span>{v.name}</span>
                    </button>
                    <button
                      title="Bloquear"
                      onClick={() =>
                        void run("layer.update", {
                          layer_id: v.id,
                          locked: !v.locked,
                        })
                      }
                    >
                      {v.locked ? "▣" : "□"}
                    </button>
                  </div>
                ))}
              </div>
              <label>
                Opacidade{" "}
                <input
                  type="range"
                  min="0"
                  max="255"
                  value={d?.layers.find((v) => v.id === l)?.opacity ?? 255}
                  onChange={(e) =>
                    void run("layer.update", {
                      layer_id: l,
                      opacity: +e.target.value,
                    })
                  }
                />
              </label>
              <div className="row">
                <button
                  onClick={() =>
                    void run("layer.duplicate", {
                      layer_id: l,
                      name:
                        (d?.layers.find((v) => v.id === l)?.name ?? "Camada") +
                        " cópia",
                    })
                  }
                >
                  Duplicar
                </button>
                <button
                  onClick={() =>
                    void run("layer.reorder", {
                      layer_id: l,
                      index: Math.min(
                        d!.layers.length - 1,
                        d!.layers.findIndex((v) => v.id === l) + 1,
                      ),
                    })
                  }
                >
                  ↑
                </button>
                <button
                  onClick={() =>
                    void run("layer.reorder", {
                      layer_id: l,
                      index: Math.max(
                        0,
                        d!.layers.findIndex((v) => v.id === l) - 1,
                      ),
                    })
                  }
                >
                  ↓
                </button>
              </div>
              <button
                disabled={(d?.layers.length ?? 0) < 2}
                onClick={() => void run("layer.remove", { layer_id: l })}
              >
                Excluir camada
              </button>
            </section>
          </fieldset>
        </aside>
      </main>
      <div className="timeline">
        <fieldset disabled={!d} className="timeline-header">
          <strong>
            Animação <small>{d?.frames.length ?? 0} quadros</small>
          </strong>
          <button
            onClick={() =>
              Play((v) => {
                view({ playback: !v });
                return !v;
              })
            }
          >
            {play ? "Pausar" : "Reproduzir"}
          </button>
          <button
            onClick={() =>
              void run("frame.add", { source_frame_id: f, duration_ms: 100 })
            }
          >
            + Duplicar frame
          </button>
          <button onClick={() => void run("frame.add", { duration_ms: 100 })}>
            + Frame vazio
          </button>
          <button
            disabled={(d?.frames.length ?? 0) <= 1}
            onClick={() => void run("frame.remove", { frame_id: f })}
          >
            Excluir
          </button>
          <button
            onClick={() =>
              void run("frame.reorder", {
                frame_id: f,
                index: Math.max(0, d!.frames.findIndex((v) => v.id === f) - 1),
              })
            }
          >
            ←
          </button>
          <button
            onClick={() =>
              void run("frame.reorder", {
                frame_id: f,
                index: Math.min(
                  d!.frames.length - 1,
                  d!.frames.findIndex((v) => v.id === f) + 1,
                ),
              })
            }
          >
            →
          </button>
          <button
            onClick={async () => {
              const tag = await ask("Nome da animação", "idle");
              if (tag)
                void run("animation.tag", {
                  name: tag,
                  from_frame: d!.frames[0].id,
                  to_frame: d!.frames.at(-1)!.id,
                });
            }}
          >
            Tag
          </button>
          <label>
            Onion{" "}
            <input
              type="checkbox"
              checked={onion}
              onChange={(e) => {
                Onion(e.target.checked);
                view({ onion_skin: e.target.checked });
              }}
            />
          </label>
          <label>
            Duração{" "}
            <input
              key={f + "-" + rev.current}
              type="number"
              min="10"
              max="60000"
              defaultValue={
                d?.frames.find((v) => v.id === f)?.duration_ms ?? 100
              }
              onBlur={(e) =>
                void run("frame.duration", {
                  frame_id: f,
                  duration_ms: +e.target.value,
                })
              }
            />{" "}
            ms
          </label>
        </fieldset>
        {d && (
          <FrameTimeline
            documentId={documentId.current}
            frames={d.frames}
            frameId={f}
            revision={rev.current}
            select={(frame) => {
              if (drawing.current) return;
              Play(false);
              F(frame);
              view({ frame_id: frame, playback: false });
              void refresh(frame).catch((e) => S(String(e)));
            }}
          />
        )}
      </div>
      <footer>
        <span>{status}</span>
        <div className="viewport-controls">
          <label>
            Grade{" "}
            <input
              type="checkbox"
              checked={grid}
              onChange={(e) => {
                Grid(e.target.checked);
                view({ grid: e.target.checked });
              }}
            />
          </label>
          <label>
            Zoom{" "}
            <select
              value={zoom}
              onChange={(e) => {
                Z(+e.target.value);
                view({ zoom: +e.target.value });
              }}
            >
              {[1, 2, 4, 6, 8, 12, 16, 24].map((z) => (
                <option key={z} value={z}>
                  {z * 100}%
                </option>
              ))}
            </select>
          </label>
          <span>v0.5.0</span>
        </div>
      </footer>
      {resource && (
        <ResourceDialog
          title={
            resource === "files" ? "Abrir documento" : "Recuperar trabalho"
          }
          onClose={() => Resource(undefined)}
        >
          {resource === "files" ? (
            <>
              {" "}
              <section>
                <h2>Biblioteca</h2>
                <p className="path">{root}</p>
                <div className="files">
                  {files
                    .filter((v) => v.endsWith(".spritelab"))
                    .map((v) => (
                      <button key={v} onClick={() => void open(v)}>
                        {v}
                      </button>
                    ))}
                </div>
                <label>
                  Importar PNG{" "}
                  <select
                    value=""
                    onChange={(e) => {
                      if (e.target.value) void open(e.target.value);
                    }}
                  >
                    <option value="">Selecionar arquivo…</option>
                    {files
                      .filter((v) => v.endsWith(".png"))
                      .map((v) => (
                        <option key={v}>{v}</option>
                      ))}
                  </select>
                </label>
                <p className="hint">
                  Esc cancela o traço.
                  <br />
                  Espaço reproduz a animação.
                  <br />
                  Duplo clique renomeia a camada.
                </p>
              </section>
            </>
          ) : (
            <>
              {" "}
              <section className="recovery-panel">
                <h2>
                  Recuperação <small>Automática • 15 s</small>
                </h2>
                <p className="hint">
                  Cópias independentes do arquivo salvo. Restaurar abre um novo
                  documento.
                </p>
                {recoveryError && (
                  <p role="alert" className="recovery-error">
                    {recoveryError}
                  </p>
                )}
                {!recovery.length && (
                  <p className="hint">
                    As cópias do trabalho em andamento aparecerão aqui.
                  </p>
                )}
                <div className="files">
                  {recovery.map((item) => (
                    <div className="recovery-entry" key={item.entry}>
                      <button
                        disabled={!!item.error}
                        onClick={() => void restore(item.entry)}
                        title={item.error ?? item.entry}
                      >
                        {item.source_path ?? item.document_id}
                        <small>
                          {item.error ??
                            new Date(item.updated_ms).toLocaleString("pt-BR")}
                        </small>
                      </button>
                      <button
                        aria-label="Descartar cópia de recuperação"
                        onClick={async () => {
                          if (
                            !(await ask(
                              "Descartar esta cópia de recuperação? O documento aberto será preservado.",
                            ))
                          )
                            return;
                          try {
                            const result: any = await invoke("execute", {
                              request: {
                                command: "recovery.delete",
                                args: { entry: item.entry },
                              },
                            });
                            if (!result.ok) throw Error(result.error.message);
                            Recovery((items) =>
                              items.filter((v) => v.entry !== item.entry),
                            );
                          } catch (e) {
                            S(String(e));
                          }
                        }}
                      >
                        ×
                      </button>
                    </div>
                  ))}
                </div>
              </section>
            </>
          )}
        </ResourceDialog>
      )}
      {panel && (
        <div className="command-panel">
          <h2>
            COMMAND ENGINE <button onClick={() => Panel(false)}>×</button>
          </h2>
          <p>
            Documento: {documentId.current} / revisão {rev.current}
          </p>
          <select value={operation} onChange={(e) => Operation(e.target.value)}>
            {catalog
              .filter((v) => v.effect !== "lifecycle")
              .map((v) => (
                <option key={v.id}>{v.id}</option>
              ))}
          </select>
          <p>{catalog.find((v) => v.id === operation)?.summary}</p>
          <textarea
            aria-label="Argumentos JSON"
            value={jsonArgs}
            onChange={(e) => JsonArgs(e.target.value)}
          />
          <button
            onClick={async () => {
              try {
                const args = JSON.parse(jsonArgs);
                if (operation === "editor.configure") await configure(args);
                else await run(operation, args);
              } catch (e) {
                S(String(e));
              }
            }}
          >
            Executar
          </button>
          <details>
            <summary>Resultado</summary>
            <pre>{commandResult}</pre>
          </details>
          <details>
            <summary>Schema dos argumentos</summary>
            <pre>
              {JSON.stringify(
                catalog.find((v) => v.id === operation)?.arguments_schema,
                null,
                2,
              )}
            </pre>
          </details>
          <p className="path">Socket MCP: {socket}</p>
        </div>
      )}
      {dialog && (
        <div className="backdrop">
          <form
            className="dialog"
            onSubmit={(e) => {
              e.preventDefault();
              answer(dialog.value === undefined ? true : dialog.value);
            }}
          >
            <h2>{dialog.title}</h2>
            {dialog.value !== undefined && (
              <input
                autoFocus
                value={dialog.value}
                onChange={(e) => Dialog({ ...dialog, value: e.target.value })}
              />
            )}
            <div>
              <button type="button" onClick={() => answer(null)}>
                Cancelar
              </button>
              <button type="submit">Confirmar</button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
createRoot(document.getElementById("root")!).render(<App />);
