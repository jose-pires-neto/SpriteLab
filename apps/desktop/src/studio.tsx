import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const paths: Record<string, string> = {
  stroke: "m4 17 1-4L15 3l4 4L9 17l-5 1Z M12 6l4 4 M4 20h16",
  eraser: "m3 14 9-11 8 7-8 11H9L3 14Z M7 10l8 7 M12 21h9",
  line: "M4 19 20 3 M3 17v4h4 M17 3h4v4",
  fill: "m3 12 9-9 8 8-9 9-8-8Z M6 9l9 9 M18 16c-3 4-2 6 0 6s3-2 0-6Z",
  pick: "m5 15 9-9 4 4-9 9H5v-4Z M12 8l4 4 M15 5l3-3 4 4-3 3",
  rectangle: "M4 5h16v14H4V5Z",
  ellipse: "M21 12a9 7 0 1 1-18 0 9 7 0 1 1 18 0Z",
  select:
    "M3 7V3h4 M10 3h4 M17 3h4v4 M21 10v4 M21 17v4h-4 M14 21h-4 M7 21H3v-4 M3 14v-4",
};
export function ToolIcon({ tool }: { tool: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="21"
      height="21"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[tool]} />
    </svg>
  );
}
export function FrameThumbnail({
  documentId,
  frameId,
  revision,
  layerId,
}: {
  documentId: string;
  frameId: number;
  revision: number;
  layerId?: number;
}) {
  const [source, Source] = useState<{ key: string; url: string }>();
  const key = `${documentId}:${frameId}:${layerId ?? "all"}`;
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => {
      void invoke<any>("execute", {
        request: {
          document_id: documentId,
          command: "document.preview",
          args: {
            frame_id: frameId,
            ...(layerId !== undefined ? { layer_id: layerId } : {}),
          },
        },
      })
        .then((response) => {
          if (active && response.ok)
            Source({
              key,
              url: "data:image/png;base64," + response.result.png_base64,
            });
        })
        .catch(() => {
          if (active) Source(undefined);
        });
    }, layerId === undefined ? 150 : 0);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [documentId, frameId, revision, layerId, key]);
  return source?.key === key ? (
    <img className="frame-thumbnail" src={source.url} alt="" />
  ) : (
    <span className="frame-placeholder" aria-hidden="true" />
  );
}

export function StudioMenu({
  label,
  children,
}: {
  label: string;
  children: import("react").ReactNode;
}) {
  const ref = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    const close = (event: PointerEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node))
        ref.current.open = false;
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && ref.current) ref.current.open = false;
    };
    document.addEventListener("pointerdown", close);
    document.addEventListener("keydown", escape);
    return () => {
      document.removeEventListener("pointerdown", close);
      document.removeEventListener("keydown", escape);
    };
  }, []);
  return (
    <details ref={ref} className="studio-menu">
      <summary>{label}</summary>
      <div
        className="menu-items"
        onClick={(event) => {
          if ((event.target as HTMLElement).closest("button") && ref.current)
            ref.current.open = false;
        }}
      >
        {children}
      </div>
    </details>
  );
}
export function ResourceDialog({
  title,
  onClose,
  children,
}: {
  title: string;
  onClose: () => void;
  children: import("react").ReactNode;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const dialog = ref.current;
    dialog?.showModal();
    return () => dialog?.close();
  }, []);
  return (
    <dialog
      ref={ref}
      className="resource-dialog"
      aria-labelledby="resource-title"
      onCancel={onClose}
    >
      <div className="resource-title">
        <h2 id="resource-title">{title}</h2>
        <button autoFocus aria-label="Fechar painel" onClick={onClose}>
          ×
        </button>
      </div>
      {children}
    </dialog>
  );
}
