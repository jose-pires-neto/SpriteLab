import { useEffect, useState } from "react";
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
}: {
  documentId: string;
  frameId: number;
  revision: number;
}) {
  const [source, Source] = useState<string>();
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => {
      void invoke<any>("execute", {
        request: {
          document_id: documentId,
          command: "document.preview",
          args: { frame_id: frameId },
        },
      })
        .then((response) => {
          if (active && response.ok)
            Source("data:image/png;base64," + response.result.png_base64);
        })
        .catch(() => {
          if (active) Source(undefined);
        });
    }, 150);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [documentId, frameId, revision]);
  return source ? (
    <img className="frame-thumbnail" src={source} alt="" />
  ) : (
    <span className="frame-placeholder" aria-hidden="true" />
  );
}
