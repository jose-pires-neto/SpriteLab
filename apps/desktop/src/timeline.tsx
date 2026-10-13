import { useRef, useState } from "react";
import { FrameThumbnail } from "./studio";

export function FrameTimeline({
  documentId,
  frames,
  frameId,
  revision,
  select,
}: {
  documentId: string;
  frames: { id: number; duration_ms: number }[];
  frameId: number;
  revision: number;
  select: (frame: number) => void;
}) {
  const [height, setHeight] = useState(90);
  const resize = useRef<{ y: number; height: number } | undefined>(undefined);
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
              Math.max(80, Math.min(200, h + (e.key === "ArrowUp" ? 20 : -20))),
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
                80,
                Math.min(
                  200,
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
      <div
        className="frame-strip"
        style={{ height }}
        aria-label="Quadros de animação"
      >
        {frames.map((frame, i) => (
          <button
            key={frame.id}
            className={frame.id === frameId ? "active" : ""}
            aria-pressed={frame.id === frameId}
            aria-label={`Selecionar quadro ${i + 1}`}
            onClick={() => select(frame.id)}
          >
            <FrameThumbnail
              documentId={documentId}
              frameId={frame.id}
              revision={revision}
            />
            <b>{i + 1}</b>
            <small>{frame.duration_ms} ms</small>
          </button>
        ))}
      </div>
    </>
  );
}
