// Chart.js helper functions for Blazor
window.chartHelper = {
    _charts: {}, // Store chart instances in JavaScript to avoid circular reference serialization
    
    createChart: function (canvasId, chartType, labels, data, backgroundColor) {
        const ctx = document.getElementById(canvasId);
        if (!ctx) return null;

        // Destroy existing chart if it exists
        if (window.chartHelper._charts[canvasId]) {
            window.chartHelper._charts[canvasId].destroy();
        }

        const chart = new Chart(ctx, {
            type: chartType,
            data: {
                labels: labels,
                datasets: [{
                    label: 'Activities',
                    data: data,
                    backgroundColor: backgroundColor || 'rgba(54, 162, 235, 0.2)',
                    borderColor: backgroundColor || 'rgba(54, 162, 235, 1)',
                    borderWidth: 1
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: {
                            stepSize: 1
                        }
                    }
                }
            }
        });

        // Store chart instance in JavaScript object, return canvasId as identifier
        window.chartHelper._charts[canvasId] = chart;
        return canvasId;
    },
    destroyChart: function (canvasId) {
        if (canvasId && window.chartHelper._charts[canvasId]) {
            window.chartHelper._charts[canvasId].destroy();
            delete window.chartHelper._charts[canvasId];
        }
    }
};

// File download helper
window.downloadFile = function (filename, text) {
    const element = document.createElement('a');
    element.setAttribute('href', 'data:text/json;charset=utf-8,' + encodeURIComponent(text));
    element.setAttribute('download', filename);
    element.style.display = 'none';
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
};

// Trigger file input
window.triggerFileInput = function (elementId) {
    const element = document.getElementById(elementId);
    if (element) {
        element.click();
    }
};

// Debug logging helper
window.debugLog = function (data) {
    fetch('http://127.0.0.1:7242/ingest/6fb7d9d3-de00-4a0c-862d-2c22552903af', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...data, timestamp: Date.now(), sessionId: 'debug-session' })
    }).catch(() => {});
};

