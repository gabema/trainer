window.geolocationHelper = {
    getLocation: function () {
        return new Promise((resolve) => {
            if (!navigator.geolocation) {
                resolve({ error: 'unavailable' });
                return;
            }
            navigator.geolocation.getCurrentPosition(
                (position) => {
                    resolve({
                        latitude: position.coords.latitude,
                        longitude: position.coords.longitude,
                        accuracy: position.coords.accuracy
                    });
                },
                (error) => {
                    if (error.code === error.PERMISSION_DENIED) {
                        resolve({ error: 'denied' });
                    } else {
                        resolve({ error: 'unavailable' });
                    }
                },
                { enableHighAccuracy: true, timeout: 10000, maximumAge: 0 }
            );
        });
    }
};
