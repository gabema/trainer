// Service Worker for Trainer PWA
// Detect base path from service worker location
const basePath = self.location.pathname.replace(/\/[^/]*$/, '') || '/';
const CACHE_NAME = 'trainer-v2';

const urlsToCache = [
  basePath + (basePath.endsWith('/') ? '' : '/'),
  basePath + (basePath.endsWith('/') ? '' : '/') + 'index.html',
  basePath + (basePath.endsWith('/') ? '' : '/') + 'css/bootstrap/bootstrap.min.css',
  basePath + (basePath.endsWith('/') ? '' : '/') + 'css/app.css',
  basePath + (basePath.endsWith('/') ? '' : '/') + '_framework/blazor.webassembly.js',
  basePath + (basePath.endsWith('/') ? '' : '/') + '_framework/wasm/dotnet.wasm',
  basePath + (basePath.endsWith('/') ? '' : '/') + 'manifest.json'
];

// Install event - cache resources
self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => cache.addAll(urlsToCache))
      .catch(err => console.log('Cache install failed:', err))
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return Promise.all(
        cacheNames.map(cacheName => {
          if (cacheName !== CACHE_NAME) {
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', event => {
  event.respondWith(
    caches.match(event.request)
      .then(response => {
        // Return cached version or fetch from network
        return response || fetch(event.request);
      })
      .catch(() => {
        // If both fail, return offline page if available
        if (event.request.destination === 'document') {
          const basePath = self.location.pathname.replace(/\/[^/]*$/, '') || '/';
          return caches.match(basePath + (basePath.endsWith('/') ? '' : '/') + 'index.html');
        }
      })
  );
});

// Notification click event - focus existing window; navigate to activity page if activityId is set
self.addEventListener('notificationclick', event => {
  const data = event.notification.data;
  event.notification.close();
  event.waitUntil(
    (async () => {
      const baseUrl = basePath + (basePath.endsWith('/') ? '' : '/');
      const targetUrl = (data && data.activityId !== undefined)
        ? baseUrl + 'activity/' + data.activityId
        : null;

      const clientList = await clients.matchAll({ type: 'window', includeUncontrolled: true });
      for (let i = 0; i < clientList.length; i++) {
        const client = clientList[i];
        if (client.url && 'focus' in client) {
          if (targetUrl && 'navigate' in client && typeof client.navigate === 'function') {
            await client.navigate(targetUrl);
          }
          return client.focus();
        }
      }

      if (clients.openWindow) {
        return clients.openWindow(targetUrl || baseUrl);
      }
    })()
  );
});

