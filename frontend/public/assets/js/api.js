const base = () => String(window.API_BASE || '').replace(/\/$/, '');

function visitorId() {
  let id = localStorage.getItem('inviteflow-visitor-id');
  if (!id) {
    id = crypto.randomUUID ? crypto.randomUUID() : `v-${Date.now()}-${Math.random()}`;
    localStorage.setItem('inviteflow-visitor-id', id);
  }
  return id;
}

export async function postResponse(payload) {
  const meta = {
    visitor_id: visitorId(),
    user_agent: navigator.userAgent || '',
    language: navigator.language || '',
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || '',
  };
  const response = await fetch(`${base()}/api/responses`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ...payload, ...meta }),
  });
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error(body.error || `Lỗi ${response.status}`);
  }
  return response.json();
}

export async function adminRequest(path, options = {}) {
  const token = sessionStorage.getItem('adminToken') || '';
  const response = await fetch(`${base()}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
      ...(options.headers || {}),
    },
  });
  if (response.status === 401) {
    sessionStorage.removeItem('adminToken');
    throw new Error('Token không đúng');
  }
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error(body.error || `Lỗi ${response.status}`);
  }
  return response.json();
}
