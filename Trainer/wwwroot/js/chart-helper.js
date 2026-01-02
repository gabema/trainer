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
    },
    
    // Create goal duration chart with dynamic y-axis scaling
    createGoalDurationChart: function (canvasId, labels, percentages, netBenefits, maxPercentage) {
        const ctx = document.getElementById(canvasId);
        if (!ctx) {
            return null;
        }

        // Destroy existing chart if it exists and clear the reference
        if (window.chartHelper._charts[canvasId]) {
            try {
                window.chartHelper._charts[canvasId].destroy();
            } catch (e) {
                // Silently handle errors
            }
            delete window.chartHelper._charts[canvasId];
        }
        
        // Set fixed dimensions on canvas to prevent resizing
        // Get container width once and use it, but keep height fixed
        const container = ctx.parentElement;
        const containerWidth = container ? container.clientWidth : 800;
        const fixedHeight = 400;
        
        // Set explicit dimensions on canvas (set once, won't change)
        ctx.style.width = containerWidth + 'px';
        ctx.style.height = fixedHeight + 'px';
        ctx.width = containerWidth;
        ctx.height = fixedHeight;
        
        // Clear the canvas to ensure no residual rendering
        const context = ctx.getContext('2d');
        if (context) {
            context.clearRect(0, 0, ctx.width, ctx.height);
        }

        const textColor = window.chartHelper.getTextColor();
        const gridColor = window.chartHelper.getGridColor();

        // Calculate y-axis range (value axis for vertical bars) with padding (10% of max)
        const padding = Math.max(maxPercentage * 0.1, 10);
        const yMin = -Math.max(maxPercentage, 100) - padding;
        const yMax = Math.max(maxPercentage, 100) + padding;

        // Prepare data arrays - Chart.js floating bars use [min, max] format
        const chartLabels = [];
        const chartData = [];
        const chartColors = [];
        const chartBorderColors = [];

        for (let i = 0; i < labels.length; i++) {
            const percentage = percentages[i];
            const netBenefit = netBenefits[i];
            
            if (netBenefit === 'Positive') {
                // Positive: bar bottom = percentage - 100, bar top = percentage
                chartLabels.push(labels[i]);
                chartData.push([percentage - 100, percentage]);
                chartColors.push('rgba(40, 167, 69, 0.8)');
                chartBorderColors.push('rgba(40, 167, 69, 1)');
            } else if (netBenefit === 'Negative') {
                // Negative: bar bottom = -percentage, bar top = -percentage + 100
                chartLabels.push(labels[i]);
                chartData.push([-percentage, -percentage + 100]);
                chartColors.push('rgba(220, 53, 69, 0.8)');
                chartBorderColors.push('rgba(220, 53, 69, 1)');
            }
        }

        // Ensure we have valid data
        if (chartLabels.length === 0 || chartData.length === 0) {
            return null;
        }

        const chart = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: chartLabels,
                datasets: [{
                    label: 'Goal Progress',
                    data: chartData,
                    backgroundColor: chartColors,
                    borderColor: chartBorderColors,
                    borderWidth: 1
                }]
            },
            options: {
                // Vertical bars (default) - no indexAxis needed
                responsive: false, // Disable responsive to maintain fixed size
                maintainAspectRatio: false,
                animation: false, // Disable animations to prevent resize loops
                interaction: {
                    intersect: false,
                    mode: 'index'
                },
                layout: {
                    padding: {
                        left: 10,
                        right: 10,
                        top: 10,
                        bottom: 10
                    }
                },
                plugins: {
                    legend: {
                        display: false
                    },
                    tooltip: {
                        callbacks: {
                            label: function(context) {
                                const range = context.raw;
                                // For positive: top of bar is the percentage
                                // For negative: bottom of bar (abs value) is the percentage
                                const percentage = range[1] >= 0 ? range[1] : Math.abs(range[0]);
                                return `${percentage.toFixed(1)}% of goal`;
                            }
                        },
                        color: textColor
                    }
                },
                scales: {
                    x: {
                        // X-axis is the category axis (activity type names) for vertical bars
                        position: 'bottom',
                        ticks: {
                            color: textColor
                        },
                        grid: {
                            color: gridColor,
                            drawOnChartArea: true
                        },
                        title: {
                            display: false
                        }
                    },
                    y: {
                        // Y-axis is the value axis (percentage) for vertical bars
                        position: 'left',
                        min: yMin,
                        max: yMax,
                        ticks: {
                            display: false // Hide y-axis labels and increment text
                        },
                        grid: {
                            color: function(context) {
                                // Make the zero line bold
                                if (context.tick && Math.abs(context.tick.value) < 0.01) {
                                    return textColor; // Use text color for zero line (more visible)
                                }
                                return gridColor;
                            },
                            lineWidth: function(context) {
                                // Make the zero line bold
                                if (context.tick && Math.abs(context.tick.value) < 0.01) {
                                    return 2; // Bold line for zero
                                }
                                return 1; // Normal line for other grid lines
                            }
                        },
                        title: {
                            display: false // Hide y-axis title
                        }
                    }
                }
            }
        });

        // Store chart instance
        window.chartHelper._charts[canvasId] = chart;
        
        return canvasId;
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

