// Chart.js helper functions for Blazor
window.chartHelper = {
    _charts: {}, // Store chart instances in JavaScript to avoid circular reference serialization
    
    // Get current theme
    getTheme: function() {
        return document.documentElement.getAttribute('data-bs-theme') || 'light';
    },
    
    // Get chart text color based on theme
    getTextColor: function() {
        return this.getTheme() === 'dark' ? '#fff' : '#212529';
    },
    
    // Get chart grid color based on theme
    getGridColor: function() {
        return this.getTheme() === 'dark' ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)';
    },
    
    createChart: function (canvasId, chartType, labels, data, backgroundColor) {
        const ctx = document.getElementById(canvasId);
        if (!ctx) return null;

        // Destroy existing chart if it exists
        if (window.chartHelper._charts[canvasId]) {
            window.chartHelper._charts[canvasId].destroy();
        }

        const textColor = window.chartHelper.getTextColor();
        const gridColor = window.chartHelper.getGridColor();

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
                plugins: {
                    legend: {
                        labels: {
                            color: textColor
                        }
                    }
                },
                scales: {
                    x: {
                        ticks: {
                            color: textColor
                        },
                        grid: {
                            color: gridColor
                        }
                    },
                    y: {
                        beginAtZero: true,
                        ticks: {
                            stepSize: 1,
                            color: textColor
                        },
                        grid: {
                            color: gridColor
                        }
                    }
                }
            }
        });

        // Store chart instance in JavaScript object, return canvasId as identifier
        window.chartHelper._charts[canvasId] = chart;
        return canvasId;
    },
    
    // Update chart theme colors
    updateChartTheme: function(canvasId) {
        const chart = window.chartHelper._charts[canvasId];
        if (!chart) return;
        
        const textColor = window.chartHelper.getTextColor();
        const gridColor = window.chartHelper.getGridColor();
        
        // Update legend colors
        if (chart.options.plugins && chart.options.plugins.legend) {
            chart.options.plugins.legend.labels.color = textColor;
        }
        
        // Update scale colors
        if (chart.options.scales) {
            Object.keys(chart.options.scales).forEach(scaleId => {
                const scale = chart.options.scales[scaleId];
                if (scale.ticks) {
                    scale.ticks.color = textColor;
                }
                if (scale.grid) {
                    scale.grid.color = gridColor;
                }
            });
        }
        
        chart.update();
    },
    
    // Update all charts when theme changes
    updateAllCharts: function() {
        Object.keys(window.chartHelper._charts).forEach(canvasId => {
            window.chartHelper.updateChartTheme(canvasId);
        });
    },
    
    destroyChart: function (canvasId) {
        if (canvasId && window.chartHelper._charts[canvasId]) {
            window.chartHelper._charts[canvasId].destroy();
            delete window.chartHelper._charts[canvasId];
        }
    }
};

// Listen for theme changes and update all charts
window.addEventListener('themechange', function(event) {
    if (window.chartHelper && typeof window.chartHelper.updateAllCharts === 'function') {
        window.chartHelper.updateAllCharts();
    }
});

// Also listen for attribute changes on html element (as a fallback)
if (window.MutationObserver) {
    const observer = new MutationObserver(function(mutations) {
        mutations.forEach(function(mutation) {
            if (mutation.type === 'attributes' && mutation.attributeName === 'data-bs-theme') {
                if (window.chartHelper && typeof window.chartHelper.updateAllCharts === 'function') {
                    setTimeout(() => {
                        window.chartHelper.updateAllCharts();
                    }, 0);
                }
            }
        });
    });
    
    // Start observing when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            observer.observe(document.documentElement, {
                attributes: true,
                attributeFilter: ['data-bs-theme']
            });
        });
    } else {
        observer.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ['data-bs-theme']
        });
    }
}

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

