// IndexedDB storage implementation for Trainer app
(function () {
    const DB_NAME = 'Trainer';
    const DB_VERSION = 1;
    const STORE_NAME = 'activities';

    let dbPromise = null;

    function openDatabase() {
        if (dbPromise) {
            return dbPromise;
        }

        dbPromise = new Promise((resolve, reject) => {
            const request = indexedDB.open(DB_NAME, DB_VERSION);

            request.onerror = () => {
                reject(new Error('Failed to open IndexedDB: ' + request.error));
            };

            request.onsuccess = () => {
                resolve(request.result);
            };

            request.onupgradeneeded = (event) => {
                const db = event.target.result;
                if (!db.objectStoreNames.contains(STORE_NAME)) {
                    db.createObjectStore(STORE_NAME);
                }
            };
        });

        return dbPromise;
    }

    window.indexedDbStorage = {
        // Get a single item by key
        async getItem(key) {
            try {
                const db = await openDatabase();
                return new Promise((resolve, reject) => {
                    const transaction = db.transaction([STORE_NAME], 'readonly');
                    const store = transaction.objectStore(STORE_NAME);
                    const request = store.get(key);

                    request.onerror = () => reject(request.error);
                    request.onsuccess = () => {
                        const result = request.result;
                        resolve(result !== undefined ? JSON.stringify(result) : null);
                    };
                });
            } catch (error) {
                console.error('IndexedDB getItem error:', error);
                return null;
            }
        },

        // Set a single item by key
        async setItem(key, value) {
            try {
                const db = await openDatabase();
                return new Promise((resolve, reject) => {
                    const transaction = db.transaction([STORE_NAME], 'readwrite');
                    const store = transaction.objectStore(STORE_NAME);
                    const parsedValue = value ? JSON.parse(value) : null;
                    const request = store.put(parsedValue, key);

                    request.onerror = () => reject(request.error);
                    request.onsuccess = () => resolve();
                });
            } catch (error) {
                console.error('IndexedDB setItem error:', error);
                throw error;
            }
        },

        // Remove a single item by key
        async removeItem(key) {
            try {
                const db = await openDatabase();
                return new Promise((resolve, reject) => {
                    const transaction = db.transaction([STORE_NAME], 'readwrite');
                    const store = transaction.objectStore(STORE_NAME);
                    const request = store.delete(key);

                    request.onerror = () => reject(request.error);
                    request.onsuccess = () => resolve();
                });
            } catch (error) {
                console.error('IndexedDB removeItem error:', error);
                throw error;
            }
        },

        // Get all keys matching a prefix
        async getAllKeysWithPrefix(prefix) {
            try {
                const db = await openDatabase();
                return new Promise((resolve, reject) => {
                    const transaction = db.transaction([STORE_NAME], 'readonly');
                    const store = transaction.objectStore(STORE_NAME);
                    const request = store.getAllKeys();

                    request.onerror = () => reject(request.error);
                    request.onsuccess = () => {
                        const allKeys = request.result;
                        const matchingKeys = allKeys
                            .filter(key => {
                                const keyStr = typeof key === 'string' ? key : key.toString();
                                return keyStr.startsWith(prefix);
                            })
                            .map(key => typeof key === 'string' ? key : key.toString());
                        resolve(matchingKeys);
                    };
                });
            } catch (error) {
                console.error('IndexedDB getAllKeysWithPrefix error:', error);
                return [];
            }
        },

        // Get multiple items by keys
        async getItems(keys) {
            try {
                const db = await openDatabase();
                return new Promise((resolve, reject) => {
                    // Convert keys to array - Blazor may serialize arrays as JSON strings
                    let keysArray = [];
                    if (Array.isArray(keys)) {
                        keysArray = keys;
                    } else if (typeof keys === 'string') {
                        // Blazor serialized the array as JSON
                        try {
                            keysArray = JSON.parse(keys);
                        } catch (e) {
                            console.error('IndexedDB getItems: Failed to parse keys as JSON:', e);
                            resolve(JSON.stringify({}));
                            return;
                        }
                    } else if (keys && typeof keys.length === 'number') {
                        // Array-like object - convert to array
                        keysArray = Array.prototype.slice.call(keys);
                    }

                    if (!Array.isArray(keysArray) || keysArray.length === 0) {
                        resolve(JSON.stringify({}));
                        return;
                    }

                    const transaction = db.transaction([STORE_NAME], 'readonly');
                    const store = transaction.objectStore(STORE_NAME);
                    const results = {};
                    let completed = 0;
                    const total = keysArray.length;

                    // Handle transaction error
                    transaction.onerror = () => {
                        reject(transaction.error);
                    };

                    // Issue all get requests
                    keysArray.forEach(key => {
                        const request = store.get(key);
                        request.onerror = () => {
                            console.error(`IndexedDB getItems: Error getting key ${key}:`, request.error);
                            completed++;
                            if (completed === total) {
                                resolve(JSON.stringify(results));
                            }
                        };
                        request.onsuccess = () => {
                            const result = request.result;
                            if (result !== undefined && result !== null) {
                                results[key] = result;
                            }
                            completed++;
                            if (completed === total) {
                                resolve(JSON.stringify(results));
                            }
                        };
                    });
                });
            } catch (error) {
                console.error('IndexedDB getItems error:', error);
                return JSON.stringify({});
            }
        },

        // Clear all items
        async clear() {
            try {
                const db = await openDatabase();
                return new Promise((resolve, reject) => {
                    const transaction = db.transaction([STORE_NAME], 'readwrite');
                    const store = transaction.objectStore(STORE_NAME);
                    const request = store.clear();

                    request.onerror = () => reject(request.error);
                    request.onsuccess = () => resolve();
                });
            } catch (error) {
                console.error('IndexedDB clear error:', error);
                throw error;
            }
        }
    };
})();

