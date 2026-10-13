import { useEffect, useRef, useState } from "react";
import { FrameThumbnail } from "./studio";

type Cell = { layer_id: number; frame_id: number };
export type TimelineDocument = {
  layers: { id: number; name: string; visible: boolean; locked: boolean }[];
  frames: { id: number; duration_ms: number; cels: Record<string, unknown> }[];
  tags: { name: string; from_frame: number; to_frame: number }[];
};
export function CelTimeline({
  documentId,
  document,
  frameId,
  layerId,
  revision,
  select,
  transfer,
  clear,
}: {
  documentId: string;
  document: TimelineDocument;
  frameId: number;
  layerId: number;
  revision: number;
  select: (cell: Cell) => void;
  transfer: (source: Cell, target: Cell, move: boolean) => Promise<void>;
  clear: (cell: Cell) => void;
}) {
  const [clipboard, setClipboard] = useState<Cell>();
  const drag = useRef<Cell | undefined>(undefined);
  const [height, setHeight] = useState(160);
  const resize = useRef<{ y: number; height: number } | undefined>(undefined);
  useEffect(() => {
    setClipboard(undefined);
    drag.current = undefined;
  }, [documentId]);
  const selected = { layer_id: layerId, frame_id: frameId };
  const frame = document.frames.find((f) => f.id === frameId);
  const hasCel = (cell: Cell) =>
    !!document.frames.find((f) => f.id === cell.frame_id)?.cels[cell.layer_id];
  const occupied = hasCel(selected);
  const locked = document.layers.find((l) => l.id === layerId)?.locked;
  return (
    <>
      <div
        className="timeline-resizer"
        role="separator"
        aria-label="Altura da timeline"
        aria-orientation="horizontal"
        tabIndex={0}
        onKeyDown={(e) => {
          if (["ArrowUp", "ArrowDown"].includes(e.key)) {
            e.preventDefault();
            setHeight((h) =>
              Math.max(
                100,
                Math.min(360, h + (e.key === "ArrowUp" ? 20 : -20)),
              ),
            );
          }
        }}
        onPointerDown={(e) => {
          resize.current = { y: e.clientY, height };
          e.currentTarget.setPointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (resize.current)
            setHeight(
              Math.max(
                100,
                Math.min(
                  360,
                  resize.current.height + resize.current.y - e.clientY,
                ),
              ),
            );
        }}
        onPointerUp={() => {
          resize.current = undefined;
        }}
        onPointerCancel={() => {
          resize.current = undefined;
        }}
      />
      <div className="cel-toolbar">
        <span>
          Quadro {document.frames.findIndex((f) => f.id === frameId) + 1} ·{" "}
          {frame?.duration_ms} ms
        </span>
        <button disabled={!occupied} onClick={() => setClipboard(selected)}>
          Copiar cel
        </button>
        <button
          disabled={!clipboard || !hasCel(clipboard) || locked}
          onClick={() => {
            if (clipboard) void transfer(clipboard, selected, false);
          }}
        >
          Colar cel
        </button>
        <button disabled={!occupied || locked} onClick={() => clear(selected)}>
          Limpar cel
        </button>
        <small>Arraste para mover · Option/Alt para copiar</small>
      </div>
      <div className="cel-scroll" style={{ height }}>
        <table
          className="cel-matrix"
          aria-label="Camadas e quadros de animação"
        >
          <thead>
            <tr>
              <th className="cel-layer-name">Camadas / quadros</th>
              {document.frames.map((f, i) => (
                <th key={f.id}>
                  <button
                    className={f.id === frameId ? "active" : ""}
                    aria-label={`Selecionar quadro ${i + 1}`}
                    onClick={() =>
                      select({ layer_id: layerId, frame_id: f.id })
                    }
                  >
                    <FrameThumbnail
                      documentId={documentId}
                      frameId={f.id}
                      revision={revision}
                    />
                    <b>{i + 1}</b>
                    <small>{f.duration_ms} ms</small>
                  </button>
                </th>
              ))}
            </tr>
          </thead>
          {document.tags.length > 0 && (
            <tbody className="cel-tags">
              <tr>
                <th className="cel-layer-name">Animações</th>
                {document.frames.map((f, i) => (
                  <td key={f.id}>
                    {document.tags
                      .filter(
                        (t) =>
                          i >=
                            document.frames.findIndex(
                              (v) => v.id === t.from_frame,
                            ) &&
                          i <=
                            document.frames.findIndex(
                              (v) => v.id === t.to_frame,
                            ),
                      )
                      .map((t) => (
                        <span key={t.name} title={t.name}>
                          {t.name}
                        </span>
                      ))}
                  </td>
                ))}
              </tr>
            </tbody>
          )}
          <tbody>
            {[...document.layers].reverse().map((layer) => (
              <tr key={layer.id}>
                <th className="cel-layer-name">
                  <button
                    className={layer.id === layerId ? "active" : ""}
                    onClick={() =>
                      select({ layer_id: layer.id, frame_id: frameId })
                    }
                    title={layer.name}
                  >
                    {layer.locked ? "▣ " : ""}
                    {layer.visible ? "" : "○ "}
                    {layer.name}
                  </button>
                </th>
                {document.frames.map((f, i) => {
                  const cell = { layer_id: layer.id, frame_id: f.id };
                  const active = layer.id === layerId && f.id === frameId;
                  const exists = hasCel(cell);
                  return (
                    <td
                      key={f.id}
                      className={f.id === frameId ? "current-frame" : ""}
                    >
                      <button
                        className={`cel ${active ? "active" : ""}`}
                        aria-pressed={active}
                        aria-label={`${layer.name}, quadro ${i + 1}, ${exists ? "cel presente" : "vazio"}`}
                        draggable={exists && !layer.locked}
                        onDragStart={(e) => {
                          drag.current = cell;
                          e.dataTransfer.effectAllowed = "copyMove";
                          e.dataTransfer.setData(
                            "application/x-spritelab-cel",
                            documentId,
                          );
                        }}
                        onDragEnd={() => {
                          drag.current = undefined;
                        }}
                        onDragOver={(e) => {
                          if (drag.current && !layer.locked) {
                            e.preventDefault();
                            e.dataTransfer.dropEffect = e.altKey
                              ? "copy"
                              : "move";
                          }
                        }}
                        onDrop={(e) => {
                          e.preventDefault();
                          const source = drag.current;
                          drag.current = undefined;
                          if (source && !layer.locked)
                            void transfer(source, cell, !e.altKey);
                        }}
                        onClick={() => select(cell)}
                      >
                        {exists ? "●" : "·"}
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
