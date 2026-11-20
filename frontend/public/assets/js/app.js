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

export function toast(message) {
  const node = document.createElement('div');
  node.className = 'toast';
  node.textContent = message;
  document.body.appendChild(node);
  setTimeout(() => node.remove(), 2400);
}

export function hearts(count = 15) {
  const field = document.createElement('div');
  field.className = 'heart-field';
  document.body.appendChild(field);
  for (let i = 0; i < count; i += 1) {
    const heart = document.createElement('div');
    heart.className = 'heart';
    heart.style.left = `${Math.random() * 100}%`;
    heart.style.top = `${60 + Math.random() * 35}%`;
    heart.style.animationDelay = `${Math.random() * 400}ms`;
    heart.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 21s-9-5.7-9-11a5 5 0 0 1 9-3 5 5 0 0 1 9 3c0 5.3-9 11-9 11z"/></svg>';
    field.appendChild(heart);
  }
  setTimeout(() => field.remove(), 2600);
}

export function renderChoices(items, containerId, key, onPick) {
  const container = byId(containerId);
  if (!container) return;

  items.forEach((item) => {
    const label = typeof item === 'string' ? item : item.name;
    const image = typeof item === 'string' ? '' : item.image;
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'choice';
    button.setAttribute('aria-pressed', String(read(key) === label));

    if (image) {
      const img = document.createElement('img');
      img.src = image;
      img.alt = label;
      img.loading = 'lazy';
      img.addEventListener('error', () => { img.hidden = true; });
      button.appendChild(img);
    }

    const text = document.createElement('span');
    text.className = 'label';
    text.textContent = label;
    button.appendChild(text);

    button.addEventListener('click', () => {
      save(key, label);
      container.querySelectorAll('.choice').forEach((node) => {
        node.setAttribute('aria-pressed', String(node === button));
      });
      onPick(label);
    });
    container.appendChild(button);
  });
}
