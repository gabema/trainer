// Service worker for the Trainer PWA.
//
// Three caching strategies, picked by what a URL guarantees about its content:
//
//   navigation      network first, cached shell as the fallback
//   assets/*        cache first — the build content-hashes the filename, so a
//                   changed byte means a changed URL and a cache hit is always
//                   the right answer
//   everything else stale-while-revalidate — a stable path whose content can
//                   change between releases, so serve the cached copy and
//                   refresh it for next time
//
// The previous worker was cache-first for everything, including navigations,
// which is why an installed client could keep serving a stale shell.

const BASE = new URL('./', self.location).pathname;
const CACHE_NAME = 'trainer-v3';
const SHELL = BASE + 'index.html';

// Stable paths only.
//
// The previous list named `_framework/blazor.webassembly.js` and
// `_framework/wasm/dotnet.wasm`. Those no longer exist, and `cache.addAll`
// rejects the whole batch if a single request fails — so install threw, the
// catch swallowed it, and nothing was ever cached. The app still worked online,
// so the loss of offline mode was invisible. Build output is content-hashed and
// renamed every release, so it can never be listed here; it is cached at
// runtime as it is fetched.
const PRECACHE = [
    BASE,
    SHELL,
    BASE + 'css/app.css',
    BASE + 'css/bootstrap/bootstrap.min.css',
    BASE + 'manifest.json',
    BASE + 'favicon.png',
    BASE + 'icon-192.png'
];

// Only same-origin build output lives here, and its names carry a content hash.
const isImmutable = (pathname) => pathname.startsWith(BASE + 'assets/');

self.addEventListener('install', (event) => {
    event.waitUntil((async () => {
        const cache = await caches.open(CACHE_NAME);
        // One request at a time rather than addAll, so one unreachable URL
        // costs that one entry instead of the entire install.
        await Promise.allSettled(PRECACHE.map((url) => cache.add(url)));

        // ONE-TIME CUTOVER MEASURE — remove after the Blazor-to-Rust release
        // has shipped and installed clients have taken it.
        //
        // Without this the new worker waits for every window of the old one to
        // close, which for an installed PWA can be days. With it, an installed
        // client takes the new build on its next launch.
        //
        // It is wrong as a standing policy: skipping the wait is how a user
        // ends up running a document from one build against assets from
        // another, which is the exact failure this worker exists to prevent.
        await self.skipWaiting();
    })());
});

self.addEventListener('activate', (event) => {
    event.waitUntil((async () => {
        // Scoped to the Cache Storage API. IndexedDB (all activity history) and
        // localStorage (the active-activity set) live in separate stores that
        // nothing here can reach, so the cutover cannot touch user data.
        const names = await caches.keys();
        await Promise.all(
            names.filter((name) => name !== CACHE_NAME).map((name) => caches.delete(name))
        );

        // Paired with skipWaiting above, and removed with it: adopts windows
        // that are already open instead of waiting for them to close.
        await self.clients.claim();
    })());
});

self.addEventListener('fetch', (event) => {
    const request = event.request;
    if (request.method !== 'GET') {
        return;
    }

    const url = new URL(request.url);
    if (url.origin !== self.location.origin) {
        return;
    }

    if (request.mode === 'navigate') {
        event.respondWith(handleNavigation(request));
    } else if (isImmutable(url.pathname)) {
        event.respondWith(cacheFirst(request));
    } else {
        event.respondWith(staleWhileRevalidate(event, request));
    }
});

// Serves the app shell for every client-side route.
//
// GitHub Pages has no file at `/trainer/activity/5`, so it answers a deep link
// with a 404. That is a response, not a failure, so a plain `fetch` would
// render the hosting error page. Anything that is not a 200 falls through to
// the shell, which resolves the route itself — and so does an offline fetch,
// which throws instead.
async function handleNavigation(request) {
    try {
        const response = await fetch(request);
        if (response.status === 200) {
            const url = new URL(request.url);
            // Only refresh the shell from a request that actually asked for it;
            // a host that answers deep links with 200 must not overwrite it.
            if (url.pathname === BASE || url.pathname === SHELL) {
                const cache = await caches.open(CACHE_NAME);
                await cache.put(SHELL, response.clone());
            }
            return response;
        }
    } catch (error) {
        // Offline. The cached shell below is the answer.
    }

    // `BASE` as a second chance: install caches the shell under both keys, and
    // either one alone is enough to render the app.
    const cached = await caches.match(SHELL, { cacheName: CACHE_NAME })
        || await caches.match(BASE, { cacheName: CACHE_NAME });
    return cached || Response.error();
}

async function cacheFirst(request) {
    const cache = await caches.open(CACHE_NAME);
    const cached = await cache.match(request);
    if (cached) {
        return cached;
    }

    const response = await fetch(request);
    // 200 exactly, not `ok`: a 206 cannot be stored and would throw.
    if (response.status === 200) {
        await cache.put(request, response.clone());
    }
    return response;
}

function staleWhileRevalidate(event, request) {
    return caches.open(CACHE_NAME).then(async (cache) => {
        const cached = await cache.match(request);
        const refresh = fetch(request)
            .then(async (response) => {
                if (response.status === 200) {
                    await cache.put(request, response.clone());
                }
                return response;
            })
            .catch(() => undefined);

        if (cached) {
            // Keeps the worker alive long enough to finish the refresh, which
            // would otherwise be cut short once the response is returned.
            event.waitUntil(refresh);
            return cached;
        }
        return (await refresh) || Response.error();
    });
}

// Focuses an open window and sends it to the activity, or opens one there.
self.addEventListener('notificationclick', (event) => {
    const data = event.notification.data;
    event.notification.close();

    event.waitUntil((async () => {
        const targetUrl = (data && data.activityId !== undefined)
            ? BASE + 'activity/' + data.activityId
            : null;

        const clientList = await self.clients.matchAll({ type: 'window', includeUncontrolled: true });
        for (const client of clientList) {
            if (client.url && 'focus' in client) {
                if (targetUrl && typeof client.navigate === 'function') {
                    await client.navigate(targetUrl);
                }
                return client.focus();
            }
        }

        if (self.clients.openWindow) {
            return self.clients.openWindow(targetUrl || BASE);
        }
    })());
});
