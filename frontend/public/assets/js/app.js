const KEYS = ['consent', 'freeDay', 'food', 'drink', 'place'];

const FALLBACK_CONTENT = {
  home: {
    eyebrow: 'Một lời mời',
    pageTitle: 'Một lời mời',
    title: 'Làm người yêu anh nhé?',
    lead: 'Mình cùng nhau đi chơi nha. Một buổi chiều, một ly nước, và một câu hỏi.',
    yes: 'Vâng',
    no: 'Không Nha',
    color: '#d48989',
    hearts: true,
    allowNote: true,
    images: [
      '/assets/img/content/54229922820d7c505a7ab53d783b310b.gif',
      '/assets/img/content/4406b868368695db27f93d9780e67f6b.gif',
      '/assets/img/content/a19c1a969a8a82f5cde6a7848feaa807.jpg',
      '/assets/img/content/b8d1ae6b9861b6e625b95e3b89bf9313.jpg'
    ]
  },
  thanks: {
    title: 'Cảm ơn bạn đã bấm nút',
    lead: 'Chọn một ngày mình cùng rảnh, rồi tính tiếp nhé.',
    spam: ['Bấm đi mà', 'Click đi', 'Thử đi', 'Đừng ngại', 'Bấm mạnh vào']
  },
  date: { title: 'Em rảnh vào hàm nè', lead: 'Chọn một ngày để cả hai cùng rảnh.' },
  food: {
    title: 'Em muốn ăn món gì?',
    items: [
      { name: 'Cơm', image: '/assets/img/content/acf21487e6638097f8e8922b153c1ffa.jpg', message: 'Một Cơm sẽ được phục vụ', special: false },
      { name: 'Bún Bò', image: '/assets/img/content/f441e4d163c5cc83ce64f29098ad34fa.jpg', message: 'Một Bún Bò sẽ được phục vụ', special: false },
      { name: 'Phở Cuốn', image: '/assets/img/content/6e06fe0a41b99505ebd6eec9175251f8.jpg', message: 'Một Phở Cuốn sẽ được phục vụ', special: false },
      { name: 'Nem Nướng', image: '/assets/img/content/dcc9c227c62d5146b7f4785d5788dc35.jpg', message: 'Một Nem Nướng sẽ được phục vụ', special: false },
      { name: 'Bún Cá', image: '/assets/img/content/b8ae1e54f6f0a340c9ab911c55983f07.jpg', message: 'Một Bún Cá sẽ được phục vụ', special: false },
      { name: 'Mì Cay', image: '/assets/img/content/4ea7c29e0afdbed1ac53d914a617a7fc.jpg', message: 'Một Mì Cay sẽ được phục vụ', special: false },
      { name: 'Ăn Gì Cũng Được', image: '/assets/img/content/15c536783014eb6fa4bbc953021b3eb6.jpg', message: 'Ăn gì cho em tự chọn nha. Không nữa thì ăn anh nè', special: true }
    ]
  },
  drink: {
    title: 'Em muốn uống gì?',
    items: [
      { name: 'Trà Sữa', image: '/assets/img/content/872a0953896df1c6e757fd54b845da23.jpg', message: 'Một ly Trà Sữa tuyệt vời sắp được phục vụ', special: false },
      { name: 'Cafe Sữa Đá', image: '/assets/img/content/1fc872840630055919672fce03723af9.jpg', message: 'Một ly Cafe Sữa Đá tuyệt vời sắp được phục vụ', special: false },
      { name: 'Nước Ép Cam', image: '/assets/img/content/3b5cfc41457081509f53f280020e6cd9.jpg', message: 'Một ly Nước Ép Cam tuyệt vời sắp được phục vụ', special: false },
      { name: 'Sinh Tố Bơ', image: '/assets/img/content/00d4296dc338a1b18c6d15ba68d9d0d3.jpg', message: 'Một ly Sinh Tố Bơ tuyệt vời sắp được phục vụ', special: false },
      { name: 'Nước Dừa', image: '/assets/img/content/f74676e64d0ccd46daf08d419c0fbe5d.jpg', message: 'Một ly Nước Dừa tuyệt vời sắp được phục vụ', special: false },
      { name: 'Trà Chanh', image: '/assets/img/content/c64d26e91c7c1a09916bcc3223da561b.jpg', message: 'Một ly Trà Chanh tuyệt vời sắp được phục vụ', special: false },
      { name: 'Uống Gì Cũng Được', image: '/assets/img/content/6f36c874861a46b8e4208454157a1d6b.jpg', message: 'Cho em chọn nhé :33', special: true }
    ]
  },
  place: {
    title: 'Xong em muốn đi chơi đâu nữa nà?',
    items: [
      { name: 'Công Viên', image: '/assets/img/content/c9645891fc4c84d400a93c00c8dc42a2.jpg', message: 'Một buổi đi chơi Công Viên thật tuyệt vời nhé', special: false },
      { name: 'Rạp Phim', image: '/assets/img/content/1f5ecca4b80819393a320a95d14c51d0.jpg', message: 'Một buổi đi chơi Rạp Phim thật tuyệt vời nhé', special: false },
      { name: 'Quán Cafe', image: '/assets/img/content/c7b8fcfc09f0c709458ba1e657405c15.jpg', message: 'Một buổi đi chơi Quán Cafe thật tuyệt vời nhé', special: false },
      { name: 'Nhà Ma', image: '/assets/img/content/dia-chi-cac-nha-ma-o-ha-noi-03.webp', message: 'Một buổi đi chơi Nhà Ma thật tuyệt vời nhé', special: false },
      { name: 'Dạo Quanh VNUA', image: '/assets/img/content/2017-2707hocvien004.jpg', message: 'Một buổi đi chơi Dạo Quanh VNUA thật tuyệt vời nhé', special: false },
      { name: 'Nơi Có Chúng Ta', image: '/assets/img/content/b6083cc8a66ab145ec71efce64ad96b2.jpg', message: 'Đi đâu miễn đi cùng em là được', special: true }
    ]
  },
  done: { title: 'Hẹn gặp bạn', closing: 'Đi đâu miễn đi cùng em là được' }
};

export async function loadContent() {
  const base = String(window.API_BASE || '').replace(/\/$/, '');
  try {
    const response = await fetch(`${base}/api/content`);
    if (!response.ok) throw new Error(String(response.status));
    const data = await response.json();
    const merged = { ...FALLBACK_CONTENT };
    Object.keys(FALLBACK_CONTENT).forEach((page) => {
      merged[page] = { ...FALLBACK_CONTENT[page], ...(data[page] || {}) };
    });
    return merged;
  } catch {
    return FALLBACK_CONTENT;
  }
}

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
  if (!node) return;
  node.hidden = false;
  requestAnimationFrame(() => {
    const rect = node.getBoundingClientRect();
    if (rect.top >= 0 && rect.bottom <= window.innerHeight) return;
    const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    node.scrollIntoView({ behavior: reduce ? 'auto' : 'smooth', block: 'center' });
  });
}

export function toast(message) {
  const node = document.createElement('div');
  node.className = 'toast';
  node.textContent = message;
  document.body.appendChild(node);
  setTimeout(() => node.remove(), 2400);
}

export function hearts(count = 54) {
  const field = document.createElement('div');
  field.className = 'heart-field';
  document.body.appendChild(field);
  for (let i = 0; i < count; i += 1) {
    const heart = document.createElement('div');
    heart.className = 'heart';
    heart.style.left = `${Math.random() * 100}%`;
    const size = 14 + Math.random() * 16;
    heart.style.width = `${size}px`;
    heart.style.height = `${size}px`;
    heart.style.animationDuration = `${2800 + Math.random() * 1400}ms`;
    heart.style.animationDelay = `${(i / count) * 2400 + Math.random() * 240}ms`;
    heart.style.setProperty('--drift', `${(Math.random() - 0.5) * 120}px`);
    heart.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 21s-9-5.7-9-11a5 5 0 0 1 9-3 5 5 0 0 1 9 3c0 5.3-9 11-9 11z"/></svg>';
    field.appendChild(heart);
  }
  setTimeout(() => field.remove(), 6600);
}

export function renderChoices(items, containerId, key, onPick) {
  const container = byId(containerId);
  if (!container) return;

  items.forEach((entry) => {
    const item = typeof entry === 'string' ? { name: entry } : entry;
    const label = item.name || '';
    const image = item.image || '';
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
      onPick({ ...item, name: label });
    });
    container.appendChild(button);
  });
}
