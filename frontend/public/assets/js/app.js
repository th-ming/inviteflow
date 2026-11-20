const KEYS = ['consent', 'freeDay', 'food', 'drink', 'place'];

export const byId = (id) => document.getElementById(id);

export function save(key, value) {
  if (KEYS.includes(key)) localStorage.setItem(key, value);
}

export function read(key) {
  return localStorage.getItem(key) || '';
}

export function go(page) {
  window.location.href = page;
}

export function formatDay(value) {
  if (!value) return 'Chưa chọn';
  const date = new Date(`${value}T00:00:00`);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat('vi-VN', {
    weekday: 'long', year: 'numeric', month: 'long', day: 'numeric',
  }).format(date);
}

export function setStatus(id, text) {
  const node = byId(id);
  if (node) node.textContent = text;
}

export function reveal(id) {
  const node = byId(id);
  if (node) node.hidden = false;
}

export function renderChoices(list, containerId, key, onPick) {
  const container = byId(containerId);
  if (!container) return;
  list.forEach((label) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'choice';
    button.textContent = label;
    button.setAttribute('aria-pressed', String(read(key) === label));
    button.addEventListener('click', () => {
      save(key, label);
      container.querySelectorAll('.choice').forEach((item) => {
        item.setAttribute('aria-pressed', String(item === button));
      });
      onPick(label);
    });
    container.appendChild(button);
  });
}
