const CACHE_NAME = "jmrl-v1";
const STATIC_CACHE = "jmrl-static-v1";
const DYNAMIC_CACHE = "jmrl-dynamic-v1";

// Files to cache immediately
const STATIC_FILES = [
	"/",
	"/static/css/style.css",
	"/static/js/app.js",
	"/static/manifest.json",
	"https://fonts.googleapis.com/css2?family=Prata&display=swap",
];

// Install event - cache static files
self.addEventListener("install", (event) => {
	console.log("Service Worker: Installing...");

	event.waitUntil(
		caches
			.open(STATIC_CACHE)
			.then((cache) => {
				console.log("Service Worker: Caching static files");
				return cache.addAll(STATIC_FILES);
			})
			.then(() => {
				console.log("Service Worker: Installed");
				return self.skipWaiting();
			})
			.catch((error) => {
				console.error("Service Worker: Install failed", error);
			}),
	);
});

// Activate event - clean up old caches
self.addEventListener("activate", (event) => {
	console.log("Service Worker: Activating...");

	event.waitUntil(
		caches
			.keys()
			.then((cacheNames) => {
				return Promise.all(
					cacheNames.map((cacheName) => {
						if (cacheName !== STATIC_CACHE && cacheName !== DYNAMIC_CACHE) {
							console.log("Service Worker: Deleting old cache", cacheName);
							return caches.delete(cacheName);
						}
					}),
				);
			})
			.then(() => {
				console.log("Service Worker: Activated");
				return self.clients.claim();
			}),
	);
});

// Fetch event - serve from cache with network fallback
self.addEventListener("fetch", (event) => {
	const { request } = event;
	const url = new URL(request.url);

	// Skip non-GET requests
	if (request.method !== "GET") {
		return;
	}

	// Skip external requests (except fonts)
	if (
		url.origin !== location.origin &&
		!url.hostname.includes("fonts.googleapis.com")
	) {
		return;
	}

	event.respondWith(
		caches.match(request).then((response) => {
			if (response) {
				console.log("Service Worker: Serving from cache", request.url);
				return response;
			}

			console.log("Service Worker: Fetching from network", request.url);
			return fetch(request)
				.then((response) => {
					// Don't cache non-successful responses
					if (
						!response ||
						response.status !== 200 ||
						response.type !== "basic"
					) {
						return response;
					}

					// Clone the response for caching
					const responseToCache = response.clone();

					// Cache dynamic content
					caches.open(DYNAMIC_CACHE).then((cache) => {
						cache.put(request, responseToCache);
					});

					return response;
				})
				.catch((error) => {
					console.error("Service Worker: Fetch failed", error);

					// Return offline page for navigation requests
					if (request.destination === "document") {
						return caches.match("/offline.html");
					}

					throw error;
				});
		}),
	);
});

// Background sync for form submissions
self.addEventListener("sync", (event) => {
	if (event.tag === "background-sync") {
		console.log("Service Worker: Background sync triggered");
		// Handle background sync tasks here
	}
});

// Push notifications
self.addEventListener("push", (event) => {
	if (event.data) {
		const data = event.data.json();
		console.log("Service Worker: Push notification received", data);

		const options = {
			body: data.body,
			icon: "/static/images/icon-192.png",
			badge: "/static/images/badge.png",
			vibrate: [100, 50, 100],
			data: {
				dateOfArrival: Date.now(),
				primaryKey: 1,
			},
			actions: [
				{
					action: "explore",
					title: "Explore",
					icon: "/static/images/checkmark.png",
				},
				{
					action: "close",
					title: "Close",
					icon: "/static/images/xmark.png",
				},
			],
		};

		event.waitUntil(self.registration.showNotification(data.title, options));
	}
});

// Notification click handler
self.addEventListener("notificationclick", (event) => {
	console.log("Service Worker: Notification clicked", event);
	event.notification.close();

	if (event.action === "explore") {
		// Open the app
		event.waitUntil(clients.openWindow("/"));
	}
});
