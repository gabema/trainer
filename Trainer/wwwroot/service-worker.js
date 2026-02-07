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

// Notification click event - handle guided notification navigation
self.addEventListener('notificationclick', event => {
  event.notification.close();
  
  const data = event.notification.data;
  const action = event.action;
  
  // If notification body was clicked (no action), navigate to activity edit page
  if (!action) {
    event.waitUntil(
      (async () => {
        // Determine the URL to navigate to (use basePath for subpath deployment)
        const baseUrl = basePath + (basePath.endsWith('/') ? '' : '/');
        let targetUrl = baseUrl;
        if (data && data.activityId !== undefined) {
          targetUrl = baseUrl + 'activity/' + data.activityId;
        }
        
        // Try to find and navigate an existing window
        const clientList = await clients.matchAll({ type: 'window', includeUncontrolled: true });
        
        for (let i = 0; i < clientList.length; i++) {
          const client = clientList[i];
          if (client.url && 'focus' in client) {
            // Try to navigate the existing window if supported
            if ('navigate' in client && typeof client.navigate === 'function') {
              await client.navigate(targetUrl);
              return client.focus();
            }
            // If navigate is not available, focus the existing window
            // and open the activity page in a new tab
            await client.focus();
            if (clients.openWindow) {
              return clients.openWindow(targetUrl);
            }
            return;
          }
        }
        
        // If no window is open, open a new one
        if (clients.openWindow) {
          return clients.openWindow(targetUrl);
        }
      })()
    );
    return;
  }
  
  // Handle action buttons (previous/next)
  if (data && data.activityId !== undefined) {
    const activityId = data.activityId;
    const currentLineIndex = data.currentLineIndex || 0;
    const totalLines = data.totalLines || 0;
    
    let newLineIndex = currentLineIndex;
    
    if (action === 'previous') {
      newLineIndex = Math.max(0, currentLineIndex - 1);
    } else if (action === 'next') {
      newLineIndex = Math.min(totalLines - 1, currentLineIndex + 1);
    }
    
    // Get state from IndexedDB
    event.waitUntil(
      (async () => {
        try {
          const dbName = 'TrainerDB';
          const storeName = 'guidedNotifications';
          
          // Open IndexedDB
          const db = await new Promise((resolve, reject) => {
            const request = indexedDB.open(dbName, 1);
            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result);
            request.onupgradeneeded = (e) => {
              const db = e.target.result;
              if (!db.objectStoreNames.contains(storeName)) {
                db.createObjectStore(storeName, { keyPath: 'activityId' });
              }
            };
          });
          
          // Get current state
          const transaction = db.transaction([storeName], 'readonly');
          const store = transaction.objectStore(storeName);
          const stateRequest = store.get(activityId);
          
          const state = await new Promise((resolve, reject) => {
            stateRequest.onsuccess = () => resolve(stateRequest.result);
            stateRequest.onerror = () => reject(stateRequest.error);
          });
          
          if (!state || !state.notesLines) {
            return;
          }
          
          const notesLines = state.notesLines;
          const newLine = notesLines[newLineIndex];
          
          // Update state
          const writeTransaction = db.transaction([storeName], 'readwrite');
          const writeStore = writeTransaction.objectStore(storeName);
          await new Promise((resolve, reject) => {
            const putRequest = writeStore.put({
              activityId: activityId,
              currentLineIndex: newLineIndex,
              notesLines: notesLines,
              timestamp: Date.now()
            });
            putRequest.onsuccess = () => resolve();
            putRequest.onerror = () => reject(putRequest.error);
          });
          
          // Show updated notification (use basePath for subpath deployment)
          const baseUrl = basePath + (basePath.endsWith('/') ? '' : '/');
          const iconUrl = baseUrl + 'favicon.png';
          const tag = `guided-${activityId}`;
          const options = {
            body: newLine,
            tag: tag,
            icon: iconUrl,
            badge: iconUrl,
            requireInteraction: true,
            actions: [
              {
                action: 'previous',
                title: 'Previous',
                icon: iconUrl
              },
              {
                action: 'next',
                title: 'Next',
                icon: iconUrl
              }
            ],
            data: {
              activityId: activityId,
              currentLineIndex: newLineIndex,
              totalLines: notesLines.length
            }
          };
          
          await self.registration.showNotification('Activity Notes', options);
        } catch (error) {
          console.error('Error handling notification action:', error);
        }
      })()
    );
  }
});

// When a guided notification is dismissed, remove its state from IndexedDB to avoid indefinite accumulation
self.addEventListener('notificationclose', event => {
  const data = event.notification && event.notification.data;
  if (!data || data.activityId === undefined) {
    return;
  }
  const activityId = data.activityId;
  const dbName = 'TrainerDB';
  const storeName = 'guidedNotifications';
  event.waitUntil(
    (async () => {
      try {
        const db = await new Promise((resolve, reject) => {
          const request = indexedDB.open(dbName, 1);
          request.onerror = () => reject(request.error);
          request.onsuccess = () => resolve(request.result);
          request.onupgradeneeded = (e) => {
            const db = e.target.result;
            if (!db.objectStoreNames.contains(storeName)) {
              db.createObjectStore(storeName, { keyPath: 'activityId' });
            }
          };
        });
        const transaction = db.transaction([storeName], 'readwrite');
        const store = transaction.objectStore(storeName);
        await new Promise((resolve, reject) => {
          const request = store.delete(activityId);
          request.onsuccess = () => resolve();
          request.onerror = () => reject(request.error);
        });
      } catch (error) {
        console.error('Error removing guided notification state:', error);
      }
    })()
  );
});
