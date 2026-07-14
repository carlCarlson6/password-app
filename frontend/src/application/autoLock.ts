/**
 * Idle countdown for auto-lock: pure timer logic, no DOM. The UI layer
 * (`ui/hooks/useAutoLock`) feeds it activity events; tests drive it with
 * vitest fake timers.
 */

export const DEFAULT_IDLE_LOCK_MS = 5 * 60 * 1000;

export interface IdleAutoLock {
  /** Arm (or re-arm) the countdown. */
  start(): void;
  /** User activity: restart the countdown from zero. No-op while stopped. */
  recordActivity(): void;
  /** Disarm without firing (e.g. already locked, or component unmounted). */
  stop(): void;
}

export function makeIdleAutoLock(
  onIdle: () => void,
  idleMs: number = DEFAULT_IDLE_LOCK_MS,
): IdleAutoLock {
  let handle: ReturnType<typeof setTimeout> | null = null;

  const stop = () => {
    if (handle !== null) {
      clearTimeout(handle);
      handle = null;
    }
  };

  const start = () => {
    stop();
    handle = setTimeout(() => {
      handle = null;
      onIdle();
    }, idleMs);
  };

  return {
    start,
    recordActivity() {
      // Only reset a running countdown — activity must not re-arm a lock
      // that was explicitly stopped.
      if (handle !== null) start();
    },
    stop,
  };
}
