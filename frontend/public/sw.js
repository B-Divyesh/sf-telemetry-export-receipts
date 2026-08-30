const CACHE = 'ter-shell-v3'
const SHELL = ['/', '/demo', '/privacy', '/terms', '/favicon.svg', '/assets/receipt-gate.webp', '/assets/receipt-gate-mobile.webp', '/assets/receipt-gate.jpg']
self.addEventListener('install', event => event.waitUntil((async () => {
  const cache = await caches.open(CACHE)
  await Promise.all(SHELL.map(async url => {
    try {
      const response = await fetch(url, { cache: 'reload' })
      if (response.ok) await cache.put(url, response)
    } catch { /* A later visit can fill a missing optional asset. */ }
  }))
  const root = await cache.match('/')
  if (root) {
    const html = await root.text()
    const builtAssets = [...html.matchAll(/(?:src|href)="(\/assets\/[^"]+)"/g)].map(match => match[1])
    await Promise.all(builtAssets.map(async url => {
      try {
        const response = await fetch(url, { cache: 'reload' })
        if (response.ok) await cache.put(url, response)
      } catch { /* The current page still works online. */ }
    }))
  }
  await self.skipWaiting()
})()))
self.addEventListener('activate', event => event.waitUntil((async () => {
  const keys = await caches.keys()
  await Promise.all(keys.filter(key => key !== CACHE).map(key => caches.delete(key)))
  await self.clients.claim()
})()))
self.addEventListener('fetch', event => {
  if (event.request.method !== 'GET' || new URL(event.request.url).pathname.startsWith('/api/')) return
  event.respondWith(fetch(event.request).then(response => {
    const copy = response.clone()
    caches.open(CACHE).then(cache => cache.put(event.request, copy))
    return response
  }).catch(() => caches.match(event.request).then(hit => hit || caches.match('/'))))
})
