const CACHE = 'ter-shell-v4'
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
  const url = new URL(event.request.url)
  if (event.request.method !== 'GET' || url.origin !== self.location.origin || url.pathname.startsWith('/api/')) return
  // A license is a bearer token. Never retain a returned checkout URL as a
  // navigation cache key; activation also removes the old v3 cache wholesale.
  if (url.searchParams.has('license')) {
    event.respondWith(fetch(event.request).catch(() => caches.match(url.pathname).then(hit => hit || caches.match('/'))))
    return
  }
  event.respondWith(fetch(event.request).then(response => {
    const copy = response.clone()
    caches.open(CACHE).then(cache => cache.put(url.pathname, copy))
    return response
  }).catch(() => caches.match(url.pathname).then(hit => hit || caches.match('/'))))
})
