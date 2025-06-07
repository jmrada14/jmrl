// Enhanced theme management with improved performance and accessibility
class ThemeManager {
	constructor() {
		this.themeToggle = document.getElementById("theme-toggle");
		this.body = document.body;
		this.storageKey = "theme-preference";
		this.mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

		this.init();
	}

	init() {
		// Initialize theme based on user preference or system preference
		this.initializeTheme();

		// Set up event listeners
		this.setupEventListeners();

		// Handle system theme changes
		this.mediaQuery.addEventListener("change", () => {
			if (!this.hasUserPreference()) {
				this.updateTheme(this.mediaQuery.matches);
			}
		});
	}

	setupEventListeners() {
		if (this.themeToggle) {
			this.themeToggle.addEventListener("click", () => {
				const currentTheme =
					document.documentElement.getAttribute("data-theme");
				const isDarkMode = currentTheme !== "dark";
				this.updateTheme(isDarkMode);
				this.savePreference(isDarkMode);

				// Announce theme change for screen readers
				this.announceThemeChange(isDarkMode);
			});
		}

		// Handle visibility changes (for mobile browsers that suspend tabs)
		document.addEventListener("visibilitychange", () => {
			if (document.visibilityState === "visible") {
				this.initializeTheme();
			}
		});

		// Handle page show (for bfcache)
		window.addEventListener("pageshow", (event) => {
			if (event.persisted) {
				document.body.style.visibility = "visible";
				this.initializeTheme();
			}
		});
	}

	initializeTheme() {
		const savedPreference = this.getSavedPreference();
		const shouldUseDarkMode =
			savedPreference !== null
				? savedPreference === "dark"
				: this.mediaQuery.matches;

		this.updateTheme(shouldUseDarkMode);
	}

	updateTheme(isDarkMode) {
		document.documentElement.setAttribute(
			"data-theme",
			isDarkMode ? "dark" : "light",
		);
		this.body.classList.toggle("dark-mode", isDarkMode);

		if (this.themeToggle) {
			this.themeToggle.setAttribute(
				"aria-label",
				isDarkMode ? "Switch to light mode" : "Switch to dark mode",
			);
		}

		// Update meta theme-color for mobile browsers
		this.updateThemeColor(isDarkMode);
	}

	updateThemeColor(isDarkMode) {
		let themeColorMeta = document.querySelector('meta[name="theme-color"]');
		if (!themeColorMeta) {
			themeColorMeta = document.createElement("meta");
			themeColorMeta.name = "theme-color";
			document.head.appendChild(themeColorMeta);
		}
		themeColorMeta.content = isDarkMode ? "#0d1117" : "#ffffff";
	}

	savePreference(isDarkMode) {
		try {
			localStorage.setItem(this.storageKey, isDarkMode ? "dark" : "light");
		} catch (e) {
			console.warn("Failed to save theme preference:", e);
		}
	}

	getSavedPreference() {
		try {
			return localStorage.getItem(this.storageKey);
		} catch (e) {
			console.warn("Failed to load theme preference:", e);
			return null;
		}
	}

	hasUserPreference() {
		return this.getSavedPreference() !== null;
	}

	announceThemeChange(isDarkMode) {
		// Create a live region for screen reader announcements
		let announcement = document.getElementById("theme-announcement");
		if (!announcement) {
			announcement = document.createElement("div");
			announcement.id = "theme-announcement";
			announcement.setAttribute("aria-live", "polite");
			announcement.setAttribute("aria-atomic", "true");
			announcement.style.position = "absolute";
			announcement.style.left = "-10000px";
			announcement.style.width = "1px";
			announcement.style.height = "1px";
			announcement.style.overflow = "hidden";
			document.body.appendChild(announcement);
		}

		announcement.textContent = `Switched to ${isDarkMode ? "dark" : "light"} mode`;
	}
}

// Performance optimization: Use passive listeners for scroll events
class PerformanceOptimizer {
	constructor() {
		this.setupPassiveListeners();
		this.optimizeImages();
	}

	setupPassiveListeners() {
		// Add passive scroll listeners to improve performance
		document.addEventListener("scroll", this.handleScroll.bind(this), {
			passive: true,
		});
		document.addEventListener("touchstart", this.handleTouch.bind(this), {
			passive: true,
		});
	}

	handleScroll() {
		// Throttled scroll handling can be added here if needed
	}

	handleTouch() {
		// Touch event handling for mobile optimization
	}

	optimizeImages() {
		// Add loading="lazy" to images that don't have it
		const images = document.querySelectorAll("img:not([loading])");
		for (const img of images) {
			img.loading = "lazy";
		}
	}
}

// Enhanced analytics and error reporting
class Analytics {
	constructor() {
		this.setupErrorReporting();
		this.trackPerformance();
	}

	setupErrorReporting() {
		window.addEventListener("error", (event) => {
			this.reportError({
				message: event.message,
				filename: event.filename,
				lineno: event.lineno,
				colno: event.colno,
				error: event.error?.stack,
			});
		});

		window.addEventListener("unhandledrejection", (event) => {
			this.reportError({
				message: "Unhandled promise rejection",
				error: event.reason,
			});
		});
	}

	reportError(errorInfo) {
		// In a production environment, send this to your error reporting service
		console.error("Application error:", errorInfo);
	}

	trackPerformance() {
		// Track Core Web Vitals
		if ("PerformanceObserver" in window) {
			try {
				// Largest Contentful Paint
				new PerformanceObserver((list) => {
					const entries = list.getEntries();
					const lastEntry = entries[entries.length - 1];
					console.log("LCP:", lastEntry.startTime);
				}).observe({ entryTypes: ["largest-contentful-paint"] });

				// First Input Delay
				new PerformanceObserver((list) => {
					const entries = list.getEntries();
					for (const entry of entries) {
						console.log("FID:", entry.processingStart - entry.startTime);
					}
				}).observe({ entryTypes: ["first-input"] });

				// Cumulative Layout Shift
				new PerformanceObserver((list) => {
					let cumulativeScore = 0;
					const entries = list.getEntries();
					for (const entry of entries) {
						if (!entry.hadRecentInput) {
							cumulativeScore += entry.value;
						}
					}
					console.log("CLS:", cumulativeScore);
				}).observe({ entryTypes: ["layout-shift"] });
			} catch (e) {
				console.warn("Performance monitoring not available:", e);
			}
		}
	}
}

// Accessibility enhancements
class AccessibilityEnhancer {
	constructor() {
		this.enhanceKeyboardNavigation();
		this.addSkipLinks();
		this.improveFormAccessibility();
	}

	enhanceKeyboardNavigation() {
		// Improve focus management
		document.addEventListener("keydown", (event) => {
			// Add custom keyboard shortcuts
			if (event.key === "/" && !event.ctrlKey && !event.metaKey) {
				const searchInput = document.querySelector('input[type="search"]');
				if (searchInput) {
					event.preventDefault();
					searchInput.focus();
				}
			}
		});

		// Add focus indicators for better visibility
		document.addEventListener("keydown", (event) => {
			if (event.key === "Tab") {
				document.body.classList.add("keyboard-navigation");
			}
		});

		document.addEventListener("mousedown", () => {
			document.body.classList.remove("keyboard-navigation");
		});
	}

	improveFormAccessibility() {
		// Add proper labels and descriptions to form elements
		const inputs = document.querySelectorAll("input, textarea, select");
		for (const input of inputs) {
			if (
				!input.getAttribute("aria-label") &&
				!input.getAttribute("aria-labelledby")
			) {
				const label = document.querySelector(`label[for="${input.id}"]`);
				if (!label && input.placeholder) {
					input.setAttribute("aria-label", input.placeholder);
				}
			}
		}
	}
}

// Service Worker registration for offline support
class ServiceWorkerManager {
	constructor() {
		this.registerServiceWorker();
	}

	async registerServiceWorker() {
		if ("serviceWorker" in navigator) {
			try {
				// Only register in production
				if (
					location.hostname !== "localhost" &&
					location.hostname !== "127.0.0.1"
				) {
					const registration = await navigator.serviceWorker.register("/sw.js");
					console.log("Service Worker registered:", registration);

					// Check for updates
					registration.addEventListener("updatefound", () => {
						const newWorker = registration.installing;
						newWorker.addEventListener("statechange", () => {
							if (
								newWorker.state === "installed" &&
								navigator.serviceWorker.controller
							) {
								// New content is available
								this.showUpdateNotification();
							}
						});
					});
				}
			} catch (error) {
				console.log("Service Worker registration failed:", error);
			}
		}
	}

	showUpdateNotification() {
		// Show a notification that new content is available
		const notification = document.createElement("div");
		notification.className = "update-notification";
		notification.innerHTML = `
      <p>New content is available!</p>
      <button onclick="window.location.reload()">Refresh</button>
      <button onclick="this.parentElement.remove()">Dismiss</button>
    `;
		notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: var(--bg-color-light);
      border: 1px solid var(--border-color-light);
      border-radius: 8px;
      padding: 16px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
      z-index: 1000;
    `;
		document.body.appendChild(notification);
	}
}

// Initialize all enhancements when DOM is ready
function initializeApp() {
	// Ensure body is visible (handles bfcache and fast page loads)
	document.body.style.visibility = "visible";

	// Initialize all managers
	new ThemeManager();
	new PerformanceOptimizer();
	new Analytics();
	new AccessibilityEnhancer();
	new ServiceWorkerManager();

	// Mark app as initialized
	document.body.classList.add("app-initialized");

	console.log("✅ Personal website initialized successfully");
}

// Initialize immediately if DOM is ready, otherwise wait
if (document.readyState === "loading") {
	document.addEventListener("DOMContentLoaded", initializeApp);
} else {
	initializeApp();
}
