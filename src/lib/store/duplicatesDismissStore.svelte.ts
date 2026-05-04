const STORAGE_KEY = "enso:dismissed-dupe-names";

function load(): Set<string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return new Set(JSON.parse(raw));
  } catch {}
  return new Set();
}

function save() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify([..._dismissed]));
}

let _dismissed = $state<Set<string>>(load());

export const isDismissedName = (name: string) => _dismissed.has(name);

export function dismissName(name: string) {
  _dismissed = new Set([..._dismissed, name]);
  save();
}

export function restoreName(name: string) {
  const s = new Set(_dismissed);
  s.delete(name);
  _dismissed = s;
  save();
}

export const dismissedCount = () => _dismissed.size;
