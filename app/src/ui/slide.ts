export type SlideState = "idle" | "drag" | "over";

const MOVE_PX = 8;

/**
 * Lets `btn` be dragged sideways onto `target`. Releasing over the target calls
 * `onDrop`. Releasing elsewhere snaps back. A tap stays a normal click: after a
 * drag, `btn.dataset.dragged` is set until the click event has passed.
 */
export function enableSlide(
  btn: HTMLElement,
  target: HTMLElement,
  onState: (s: SlideState) => void,
  onDrop: () => void,
  signal: AbortSignal,
) {
  let startX = 0;
  let pointer = -1;
  let range = 0;
  let moved = false;
  const opts = { signal };
  const isOver = (e: PointerEvent) => {
    const r = target.getBoundingClientRect();
    return e.clientX >= r.left && e.clientX <= r.right && Math.abs(e.clientY - (r.top + r.height / 2)) < r.height;
  };
  const reset = () => {
    btn.style.transform = "";
    btn.classList.remove("dragging");
    onState("idle");
  };

  btn.addEventListener("pointerdown", (e) => {
    pointer = e.pointerId;
    startX = e.clientX;
    range = target.getBoundingClientRect().right - btn.getBoundingClientRect().right;
    moved = false;
    btn.setPointerCapture(pointer);
  }, opts);

  btn.addEventListener("pointermove", (e) => {
    if (e.pointerId !== pointer) return;
    const dx = Math.max(0, Math.min(e.clientX - startX, Math.max(range, 0)));
    if (!moved && Math.abs(e.clientX - startX) < MOVE_PX) return;
    moved = true;
    btn.classList.add("dragging");
    btn.style.transform = `translateX(${dx}px)`;
    onState(isOver(e) ? "over" : "drag");
  }, opts);

  const end = (e: PointerEvent) => {
    if (e.pointerId !== pointer) return;
    pointer = -1;
    const drop = moved && e.type === "pointerup" && isOver(e);
    if (moved) {
      btn.dataset.dragged = "1";
      window.setTimeout(() => delete btn.dataset.dragged, 0);
    }
    reset();
    if (drop) onDrop();
  };
  btn.addEventListener("pointerup", end, opts);
  btn.addEventListener("pointercancel", end, opts);
}
