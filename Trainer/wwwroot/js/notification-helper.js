window.notificationHelper = {
    // Request notification permission
    requestPermission: async function() {
        if (!('Notification' in window)) {
            return { granted: false, error: 'Notifications not supported' };
        }
        
        if (Notification.permission === 'granted') {
            return { granted: true };
        }
        
        if (Notification.permission === 'denied') {
            return { granted: false, error: 'Permission denied' };
        }
        
        try {
            const permission = await Notification.requestPermission();
            return { granted: permission === 'granted' };
        } catch (error) {
            return { granted: false, error: error.message };
        }
    },
    
    // Get service worker registration
    _getRegistration: async function() {
        if (!('serviceWorker' in navigator)) {
            throw new Error('Service workers not supported');
        }
        
        const registration = await navigator.serviceWorker.ready;
        return registration;
    },

    // Resolve icon URL with base path for subpath deployment
    _getIconUrl: function() {
        const base = document.querySelector('base');
        const href = base && base.getAttribute('href');
        if (href) {
            try {
                const path = new URL(href, window.location.origin).pathname;
                const basePath = path.endsWith('/') ? path.slice(0, -1) : path || '';
                return (basePath ? basePath + '/' : '/') + 'favicon.png';
            } catch (_) {}
        }
        return '/favicon.png';
    },
    
    // Show a browser notification for a started active activity.
    // Safe to call even if permission is not granted — exits silently.
    startActiveNotification: async function(activityId, name, elapsed) {
        if (!('Notification' in window) || Notification.permission !== 'granted') return;
        try {
            const registration = await this._getRegistration();
            const iconUrl = this._getIconUrl();
            await registration.showNotification(name, {
                tag: `active-${activityId}`,
                body: `Active — ${elapsed}`,
                icon: iconUrl,
                badge: iconUrl,
                renotify: false,
                silent: true
            });
        } catch (e) {
            console.warn('startActiveNotification failed:', e);
        }
    },

    // Update an existing active notification with the latest elapsed time.
    // Uses the same tag so the notification is replaced silently.
    updateActiveNotification: async function(activityId, name, elapsed) {
        if (!('Notification' in window) || Notification.permission !== 'granted') return;
        try {
            const registration = await this._getRegistration();
            const iconUrl = this._getIconUrl();
            await registration.showNotification(name, {
                tag: `active-${activityId}`,
                body: `Active — ${elapsed}`,
                icon: iconUrl,
                badge: iconUrl,
                renotify: false,
                silent: true
            });
        } catch (e) {
            console.warn('updateActiveNotification failed:', e);
        }
    },

    // Close the browser notification for a finished active activity.
    closeActiveNotification: async function(activityId) {
        try {
            const registration = await this._getRegistration();
            const tag = `active-${activityId}`;
            const notifications = await registration.getNotifications({ tag });
            for (const n of notifications) n.close();
        } catch (e) {
            console.warn('closeActiveNotification failed:', e);
        }
    }
};
