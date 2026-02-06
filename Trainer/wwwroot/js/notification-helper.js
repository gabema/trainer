// Notification helper for guided activity notes
window.notificationHelper = {
    _dbName: 'TrainerDB',
    _dbVersion: 1,
    _storeName: 'guidedNotifications',
    
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
    
    // Initialize IndexedDB for storing notification state
    _initDB: function() {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open(this._dbName, this._dbVersion);
            
            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve(request.result);
            
            request.onupgradeneeded = (event) => {
                const db = event.target.result;
                if (!db.objectStoreNames.contains(this._storeName)) {
                    db.createObjectStore(this._storeName, { keyPath: 'activityId' });
                }
            };
        });
    },
    
    // Store notification state
    _storeState: async function(activityId, currentLineIndex, notesLines) {
        try {
            const db = await this._initDB();
            const transaction = db.transaction([this._storeName], 'readwrite');
            const store = transaction.objectStore(this._storeName);
            
            await new Promise((resolve, reject) => {
                const request = store.put({
                    activityId: activityId,
                    currentLineIndex: currentLineIndex,
                    notesLines: notesLines,
                    timestamp: Date.now()
                });
                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            });
        } catch (error) {
            console.error('Error storing notification state:', error);
        }
    },
    
    // Get notification state
    _getState: async function(activityId) {
        try {
            const db = await this._initDB();
            const transaction = db.transaction([this._storeName], 'readonly');
            const store = transaction.objectStore(this._storeName);
            
            return new Promise((resolve, reject) => {
                const request = store.get(activityId);
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error);
            });
        } catch (error) {
            console.error('Error getting notification state:', error);
            return null;
        }
    },
    
    // Remove notification state
    _removeState: async function(activityId) {
        try {
            const db = await this._initDB();
            const transaction = db.transaction([this._storeName], 'readwrite');
            const store = transaction.objectStore(this._storeName);
            await new Promise((resolve, reject) => {
                const request = store.delete(activityId);
                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error);
            });
        } catch (error) {
            console.error('Error removing notification state:', error);
        }
    },
    
    // Split notes into lines, filtering empty lines
    _splitNotes: function(notes) {
        if (!notes || typeof notes !== 'string') {
            return [];
        }
        
        return notes
            .split(/\r?\n/)
            .map(line => line.trim())
            .filter(line => line.length > 0);
    },
    
    // Get service worker registration
    _getRegistration: async function() {
        if (!('serviceWorker' in navigator)) {
            throw new Error('Service workers not supported');
        }
        
        const registration = await navigator.serviceWorker.ready;
        return registration;
    },
    
    // Start guided notification
    startGuidedNotification: async function(activityId, notes) {
        // Check permission first
        const permissionResult = await this.requestPermission();
        if (!permissionResult.granted) {
            throw new Error(permissionResult.error || 'Notification permission not granted');
        }
        
        // Split notes into lines
        const notesLines = this._splitNotes(notes);
        if (notesLines.length === 0) {
            throw new Error('No notes to display');
        }
        
        // Store state
        await this._storeState(activityId, 0, notesLines);
        
        // Get service worker registration
        const registration = await this._getRegistration();
        
        // Show first line notification
        const tag = `guided-${activityId}`;
        const options = {
            body: notesLines[0],
            tag: tag,
            icon: '/favicon.png',
            badge: '/favicon.png',
            requireInteraction: false,
            actions: [
                {
                    action: 'previous',
                    title: 'Previous',
                    icon: '/favicon.png'
                },
                {
                    action: 'next',
                    title: 'Next',
                    icon: '/favicon.png'
                }
            ],
            data: {
                activityId: activityId,
                currentLineIndex: 0,
                totalLines: notesLines.length
            }
        };
        
        await registration.showNotification('Activity Notes', options);
    }
};
