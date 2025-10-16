// Service Worker for SolCron - Enhanced Caching and Performance

const CACHE_NAME = 'solcron-v1.0.0';
const RUNTIME_CACHE = 'solcron-runtime';

// Assets to cache immediately
const PRECACHE_URLS = [
  '/',
  '/manifest.json',
  // Add critical CSS/JS files here
];

// Cache strategies
const CACHE_STRATEGIES = {
  // Cache first, then network for static assets
  CACHE_FIRST: 'cache-first',
  // Network first, then cache for dynamic data
  NETWORK_FIRST: 'network-first',
  // Stale while revalidate for balanced performance
  STALE_WHILE_REVALIDATE: 'stale-while-revalidate'
};

// Install event - precache critical resources
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_URLS))
      .then(() => self.skipWaiting())
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME && cacheName !== RUNTIME_CACHE) {
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => self.clients.claim())
  );
});

// Fetch event - implement caching strategies
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Handle different request types
  if (request.method !== 'GET') return;

  // API requests - network first with cache fallback
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(networkFirst(request));
    return;
  }

  // Static assets - cache first
  if (isStaticAsset(url)) {
    event.respondWith(cacheFirst(request));
    return;
  }

  // HTML pages - stale while revalidate
  if (request.headers.get('accept')?.includes('text/html')) {
    event.respondWith(staleWhileRevalidate(request));
    return;
  }

  // Default to network first
  event.respondWith(networkFirst(request));
});

// Cache-first strategy
async function cacheFirst(request) {
  const cache = await caches.open(RUNTIME_CACHE);
  const cachedResponse = await cache.match(request);

  if (cachedResponse) {
    return cachedResponse;
  }

  try {
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.log('Cache first failed:', error);
    return new Response('Offline', { status: 503 });
  }
}

// Network-first strategy
async function networkFirst(request) {
  const cache = await caches.open(RUNTIME_CACHE);

  try {
    const networkResponse = await fetch(request);
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  } catch (error) {
    console.log('Network first fallback to cache:', error);
    const cachedResponse = await cache.match(request);
    return cachedResponse || new Response('Offline', { status: 503 });
  }
}

// Stale-while-revalidate strategy
async function staleWhileRevalidate(request) {
  const cache = await caches.open(RUNTIME_CACHE);
  const cachedResponse = await cache.match(request);

  const fetchPromise = fetch(request).then((networkResponse) => {
    if (networkResponse.ok) {
      cache.put(request, networkResponse.clone());
    }
    return networkResponse;
  });

  return cachedResponse || fetchPromise;
}

// Helper function to identify static assets
function isStaticAsset(url) {
  return /\.(js|css|png|jpg|jpeg|gif|svg|woff|woff2|ttf|eot)$/i.test(url.pathname) ||
         url.pathname.startsWith('/_next/static/');
}

// Background sync for offline actions
self.addEventListener('sync', (event) => {
  if (event.tag === 'background-sync') {
    event.waitUntil(doBackgroundSync());
  }
});

// Background sync implementation
async function doBackgroundSync() {
  // Handle offline actions when connection is restored
  console.log('Background sync triggered');
}

// Push notification handling
self.addEventListener('push', (event) => {
  const options = {
    body: event.data ? event.data.text() : 'New update available',
    icon: '/icon-192x192.png',
    badge: '/badge-72x72.png',
    actions: [
      {
        action: 'view',
        title: 'View',
        icon: '/icon-view.png'
      },
      {
        action: 'dismiss',
        title: 'Dismiss',
        icon: '/icon-dismiss.png'
      }
    ]
  };

  event.waitUntil(
    self.registration.showNotification('SolCron', options)
  );
});

// Notification click handling
self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  if (event.action === 'view') {
    event.waitUntil(
      clients.openWindow('/')
    );
  }
});

// Performance monitoring
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'PERFORMANCE_MEASURE') {
    // Log performance measurements
    console.log('Performance measure:', event.data.measurement);
  }
});