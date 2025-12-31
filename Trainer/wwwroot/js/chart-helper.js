// Chart.js helper functions for Blazor
window.chartHelper = {
    createChart: function (canvasId, chartType, labels, data, backgroundColor) {
        const ctx = document.getElementById(canvasId);
        if (!ctx) return null;

        return new Chart(ctx, {
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
    },
    destroyChart: function (chart) {
        if (chart) {
            chart.destroy();
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

