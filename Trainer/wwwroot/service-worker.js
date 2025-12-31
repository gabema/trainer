// Service Worker for Trainer PWA
// Detect base path from service worker location
const basePath = self.location.pathname.replace(/\/[^/]*$/, '') || '/';
const CACHE_NAME = 'trainer-v1';
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

